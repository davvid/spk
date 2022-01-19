// Copyright (c) 2021 Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk

use std::{
    ffi::OsStr,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    pin::Pin,
    task::Poll,
};

use futures::Stream;
use relative_path::RelativePath;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

use super::FSRepository;
use crate::{
    encoding,
    storage::{tag::TagSpecAndTagIter, TagStorage},
    tracking, Error, Result,
};
use encoding::{Decodable, Encodable};

#[cfg(test)]
#[path = "./tag_test.rs"]
mod tag_test;

const TAG_EXT: &str = "tag";

impl FSRepository {
    fn tags_root(&self) -> PathBuf {
        self.root().join("tags")
    }

    async fn push_raw_tag_without_lock(&self, tag: &tracking::Tag) -> Result<()> {
        let tag_spec = tracking::build_tag_spec(tag.org(), tag.name(), 0)?;
        let filepath = tag_spec.to_path(self.tags_root());

        let mut buf = Vec::new();
        tag.encode(&mut buf)?;
        let size = buf.len();

        crate::runtime::makedirs_with_perms(filepath.parent().unwrap(), 0o777)?;

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&filepath)
            .await?;
        file.write_i64(size as i64).await?;
        tokio::io::copy(&mut buf.as_slice(), &mut file).await?;
        if let Err(err) = file.sync_all().await {
            return Err(Error::wrap_io(err, "Failed to finalize tag data file"));
        }

        let perms = std::fs::Permissions::from_mode(0o777);
        if let Err(err) = tokio::fs::set_permissions(&filepath, perms).await {
            tracing::warn!(?err, ?filepath, "Failed to set tag permissions");
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl TagStorage for FSRepository {
    fn ls_tags(&self, path: &RelativePath) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
        let filepath = path.to_path(self.tags_root());
        let read_dir = match std::fs::read_dir(&filepath) {
            Ok(r) => r,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => return Box::pin(futures::stream::empty()),
                _ => return Box::pin(futures::stream::once(async { Err(err.into()) })),
            },
        };

        let mut entries = std::collections::HashSet::new();
        let iter = read_dir.filter_map(move |entry| {
            let entry = match entry {
                Err(err) => return Some(Err(err.into())),
                Ok(entry) => entry,
            };
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new(TAG_EXT)) {
                match path.file_stem().map(|s| s.to_string_lossy().to_string()) {
                    None => None,
                    Some(tag_name) => {
                        if entries.insert(tag_name.clone()) {
                            Some(Ok(tag_name))
                        } else {
                            None
                        }
                    }
                }
            } else {
                match path
                    .file_name()
                    .map(|s| s.to_string_lossy() + "/")
                    .map(|s| s.to_string())
                {
                    None => None,
                    Some(tag_dir) => {
                        if entries.insert(tag_dir.clone()) {
                            Some(Ok(tag_dir))
                        } else {
                            None
                        }
                    }
                }
            }
        });
        Box::pin(futures::stream::iter(iter))
    }

    /// Find tags that point to the given digest.
    ///
    /// This is an O(n) operation based on the number of all
    /// tag versions in each tag stream.
    fn find_tags(
        &self,
        digest: &encoding::Digest,
    ) -> Pin<Box<dyn Stream<Item = Result<tracking::TagSpec>> + Send>> {
        let digest = *digest;
        Box::pin(self.iter_tag_streams().filter_map(move |res| {
            let (spec, stream) = match res {
                Ok(res) => res,
                Err(err) => return Some(Err(err)),
            };
            for (i, tag) in stream.into_iter().enumerate() {
                if tag.target == digest {
                    return Some(Ok(spec.with_version(i as u64)));
                }
            }
            None
        }))
    }

    /// Iterate through the available tags in this storage.
    fn iter_tag_streams(&self) -> Pin<Box<dyn Stream<Item = Result<TagSpecAndTagIter>> + Send>> {
        Box::pin(TagStreamIter::new(&self.tags_root()))
    }

    async fn read_tag(
        &self,
        tag: &tracking::TagSpec,
    ) -> Result<Pin<Box<dyn Stream<Item = tracking::Tag> + Send>>> {
        let path = tag.to_path(self.tags_root());
        match read_tag_file(path) {
            Err(err) => match err.raw_os_error() {
                Some(libc::ENOENT) => Err(Error::UnknownReference(tag.to_string())),
                _ => Err(err),
            },
            Ok(iter) => {
                let tags: Result<Vec<_>> = iter.into_iter().collect();
                Ok(Box::pin(futures::stream::iter(tags?.into_iter().rev())))
            }
        }
    }

    async fn push_raw_tag(&mut self, tag: &tracking::Tag) -> Result<()> {
        let tag_spec = tracking::build_tag_spec(tag.org(), tag.name(), 0)?;
        let filepath = tag_spec.to_path(self.tags_root());
        crate::runtime::makedirs_with_perms(filepath.parent().unwrap(), 0o777)?;
        let _lock = TagLock::new(&filepath).await?;
        self.push_raw_tag_without_lock(tag).await
    }

    async fn remove_tag_stream(&mut self, tag: &tracking::TagSpec) -> Result<()> {
        let tag_spec = tracking::build_tag_spec(tag.org(), tag.name(), 0)?;
        let filepath = tag_spec.to_path(self.tags_root());
        let lock = match TagLock::new(&filepath).await {
            Ok(lock) => lock,
            Err(err) => match err.raw_os_error() {
                Some(libc::ENOENT) | Some(libc::ENOTDIR) => return Ok(()),
                _ => return Err(err),
            },
        };
        match tokio::fs::remove_file(&filepath).await {
            Ok(_) => (),
            Err(err) => {
                return match err.raw_os_error() {
                    Some(libc::ENOENT) => Err(Error::UnknownReference(tag.to_string())),
                    _ => Err(err.into()),
                }
            }
        }
        // the lock file needs to be removed if the directory has any hope of being empty
        drop(lock);

        let mut filepath = filepath.as_path();
        while filepath.starts_with(self.tags_root()) {
            if let Some(parent) = filepath.parent() {
                tracing::trace!(?parent, "seeing if parent needs removing");
                match tokio::fs::remove_dir(self.tags_root().join(parent)).await {
                    Ok(_) => {
                        tracing::debug!(path = ?parent, "removed tag parent dir");
                        filepath = parent;
                    }
                    Err(err) => match err.raw_os_error() {
                        Some(libc::ENOTEMPTY) => return Ok(()),
                        Some(libc::ENOENT) => return Ok(()),
                        _ => return Err(err.into()),
                    },
                }
            }
        }
        Ok(())
    }

    async fn remove_tag(&mut self, tag: &tracking::Tag) -> Result<()> {
        let tag_spec = tracking::build_tag_spec(tag.org(), tag.name(), 0)?;
        let filepath = tag_spec.to_path(self.tags_root());
        let _lock = TagLock::new(&filepath).await?;
        let tags: Vec<_> = self
            .read_tag(&tag_spec)
            .await?
            .filter(|version| version != tag)
            .collect()
            .await;
        let backup_path = &filepath.with_extension("tag.backup");
        tokio::fs::rename(&filepath, &backup_path).await?;
        let mut res = Ok(());
        for version in tags.iter().rev() {
            // we are already holding the lock for this operation
            if let Err(err) = self.push_raw_tag_without_lock(version).await {
                res = Err(err);
                break;
            }
        }
        if let Err(err) = res {
            tokio::fs::rename(&backup_path, &filepath).await?;
            Err(err)
        } else if let Err(err) = tokio::fs::remove_file(&backup_path).await {
            tracing::warn!(?err, "failed to cleanup tag backup file");
            Ok(())
        } else {
            Ok(())
        }
    }
}

struct TagStreamIter {
    root: PathBuf,
    inner: walkdir::IntoIter,
}

impl TagStreamIter {
    fn new<P: AsRef<std::path::Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            inner: walkdir::WalkDir::new(root).into_iter(),
        }
    }
}

impl Stream for TagStreamIter {
    type Item = Result<(tracking::TagSpec, Box<dyn Iterator<Item = tracking::Tag>>)>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // TODO: this is not actually async and should be fixed
        Poll::Ready(loop {
            let entry = self.inner.next();
            match entry {
                None => break None,
                Some(Err(err)) => break Some(Err(err.into())),
                Some(Ok(entry)) => {
                    if !entry.file_type().is_file() {
                        continue;
                    }
                    let path = entry.path();
                    if path.extension() != Some(OsStr::new(TAG_EXT)) {
                        continue;
                    }
                    let spec = match tag_from_path(&path, &self.root) {
                        Err(err) => break Some(Err(err)),
                        Ok(spec) => spec,
                    };
                    let tags: Result<Vec<_>> = match read_tag_file(&path) {
                        Err(err) => break Some(Err(err)),
                        Ok(stream) => stream.into_iter().collect(),
                    };
                    break match tags {
                        Err(err) => Some(Err(err)),
                        Ok(tags) => Some(Ok((spec, Box::new(tags.into_iter().rev())))),
                    };
                }
            }
        })
    }
}

/// Return an iterator over all tags in the identified tag file
///
/// This iterator outputs tags from earliest to latest, as stored
/// in the file starting at the beginning
fn read_tag_file<P: AsRef<Path>>(path: P) -> Result<TagIter<std::fs::File>> {
    let reader = std::fs::File::open(path.as_ref())?;
    Ok(TagIter::new(reader))
}

struct TagIter<R: std::io::Read + std::io::Seek>(R);

impl<R: std::io::Read + std::io::Seek> TagIter<R> {
    fn new(reader: R) -> Self {
        Self(reader)
    }
}

impl<R: std::io::Read + std::io::Seek> Iterator for TagIter<R> {
    type Item = Result<tracking::Tag>;

    fn next(&mut self) -> Option<Self::Item> {
        let _size = match encoding::read_uint(&mut self.0) {
            Ok(size) => size,
            Err(err) => match err.raw_os_error() {
                Some(libc::EOF) => return None,
                _ => return Some(Err(err)),
            },
        };
        match tracking::Tag::decode(&mut self.0) {
            Err(err) => Some(Err(err)),
            Ok(tag) => Some(Ok(tag)),
        }
    }
}

fn tag_from_path<P: AsRef<Path>, R: AsRef<Path>>(path: P, root: R) -> Result<tracking::TagSpec> {
    let mut path = path.as_ref().to_path_buf();
    let filename = match path.file_stem() {
        Some(stem) => stem.to_owned(),
        None => {
            return Err(format!("Path must end with '.{}' to be considered a tag", TAG_EXT).into())
        }
    };
    path.set_file_name(filename);
    let path = path.strip_prefix(root)?;
    tracking::TagSpec::parse(path.to_string_lossy())
}
pub trait TagExt {
    fn to_path<P: AsRef<Path>>(&self, root: P) -> PathBuf;
}

impl TagExt for tracking::TagSpec {
    fn to_path<P: AsRef<Path>>(&self, root: P) -> PathBuf {
        let mut filepath = root.as_ref().join(self.path());
        let new_name = self.name() + "." + TAG_EXT;
        filepath.set_file_name(new_name);
        filepath
    }
}

struct TagLock(PathBuf);

impl TagLock {
    pub async fn new<P: AsRef<Path>>(tag_file: P) -> Result<TagLock> {
        let mut lock_file = tag_file.as_ref().to_path_buf();
        lock_file.set_extension("tag.lock");

        let timeout = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match tokio::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&lock_file)
                .await
            {
                Ok(_file) => {
                    break Ok(TagLock(lock_file));
                }
                Err(err) => {
                    if std::time::Instant::now() < timeout {
                        continue;
                    }
                    break match err.raw_os_error() {
                        Some(libc::EEXIST) => Err("Tag already locked, cannot edit".into()),
                        _ => Err(err.into()),
                    };
                }
            }
        }
    }
}

impl Drop for TagLock {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_file(&self.0) {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(?err, path = ?self.0, "Failed to remove tag lock file");
            }
        }
    }
}
