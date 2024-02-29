// Copyright (c) Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk

use std::convert::TryFrom;
use std::str::FromStr;

use rstest::rstest;
use spfs::prelude::*;
use spk_schema::foundation::fixtures::*;
use spk_schema::foundation::version::Version;
use spk_schema::ident_ops::NormalizedTagStrategy;
use spk_schema::BuildIdent;

use super::SpfsRepository;
use crate::storage::{CachePolicy, Repository};
use crate::NameAndRepositoryWithTagStrategy;

#[rstest]
fn test_repo_meta_tag_is_valid() {
    spfs::tracking::TagSpec::parse(super::REPO_METADATA_TAG)
        .expect("repo metadata tag must be a valid spfs tag");
}

#[rstest]
fn test_repo_version_is_valid() {
    Version::from_str(super::REPO_VERSION)
        .expect("repo current version must be a valid spk version string");
}

#[rstest]
#[tokio::test]
async fn test_metadata_io(tmpdir: tempfile::TempDir) {
    init_logging();
    let repo_root = tmpdir.path();
    let repo = SpfsRepository::try_from(NameAndRepositoryWithTagStrategy::<
        _,
        _,
        NormalizedTagStrategy,
    >::new(
        "test-repo",
        spfs::storage::fs::FsRepository::create(repo_root)
            .await
            .unwrap(),
    ))
    .unwrap();

    let meta = super::RepositoryMetadata::default();
    repo.write_metadata(&meta).await.unwrap();
    let actual = repo.read_metadata().await.unwrap();
    assert_eq!(actual, meta, "should return metadata as it was stored");
}

#[rstest]
#[tokio::test]
async fn test_upgrade_sets_version(tmpdir: tempfile::TempDir) {
    init_logging();
    let current_version = Version::from_str(super::REPO_VERSION).unwrap();
    let repo_root = tmpdir.path();
    let repo = SpfsRepository::try_from(NameAndRepositoryWithTagStrategy::<
        _,
        _,
        NormalizedTagStrategy,
    >::new(
        "test-repo",
        spfs::storage::fs::FsRepository::create(repo_root)
            .await
            .unwrap(),
    ))
    .unwrap();

    assert_eq!(
        repo.read_metadata().await.unwrap().version,
        Version::default()
    );
    repo.upgrade()
        .await
        .expect("upgrading an empty repo should succeed");
    assert_eq!(repo.read_metadata().await.unwrap().version, current_version);
}

#[rstest]
#[tokio::test]
async fn test_upgrade_changes_tags(tmpdir: tempfile::TempDir) {
    init_logging();
    let repo_root = tmpdir.path();
    let spfs_repo = spfs::storage::fs::FsRepository::create(repo_root)
        .await
        .unwrap();
    let repo = SpfsRepository::<NormalizedTagStrategy>::new(
        "test-repo",
        &format!("file://{}", repo_root.display()),
    )
    .await
    .unwrap();

    let ident = BuildIdent::from_str("mypkg/1.0.0/src").unwrap();

    // publish an "old style" package spec and build
    let mut old_path = spfs::tracking::TagSpec::from_str(
        repo.build_package_tag::<NormalizedTagStrategy, _>(&ident)
            .as_str(),
    )
    .unwrap();
    spfs_repo
        .push_tag(&old_path, &spfs::encoding::EMPTY_DIGEST.into())
        .await
        .unwrap();
    old_path = spfs::tracking::TagSpec::from_str(
        repo.build_spec_tag::<NormalizedTagStrategy, _>(&ident)
            .as_str(),
    )
    .unwrap();
    spfs_repo
        .push_tag(&old_path, &spfs::encoding::EMPTY_DIGEST.into())
        .await
        .unwrap();

    let pkg = repo.lookup_package(&ident).await.unwrap();
    assert!(matches!(pkg, super::StoredPackage::WithoutComponents(_)));

    repo.upgrade()
        .await
        .expect("upgrading a simple repo should succeed");

    let pkg = crate::with_cache_policy!(repo, CachePolicy::BypassCache, {
        repo.lookup_package(&ident)
    })
    .await
    .unwrap();
    assert!(matches!(pkg, super::StoredPackage::WithComponents(_)));
}
