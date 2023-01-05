// Copyright (c) Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk

use clap::Args;
use futures::{StreamExt, TryStreamExt};
use spfs::prelude::*;

/// Check a repositories internal integrity
#[derive(Debug, Args)]
pub struct CmdCheck {
    /// Trigger the check operation on a remote repository instead of the local one
    #[clap(short, long)]
    remote: Option<String>,

    /// Attempt to fix problems by pulling from another repository. Defaults to "origin".
    #[clap(long)]
    pull: Option<Option<String>>,

    /// Objects to recursively check, defaults to everything
    #[clap(name = "REF")]
    reference: Vec<String>,
}

impl CmdCheck {
    pub async fn run(&mut self, config: &spfs::Config) -> spfs::Result<i32> {
        let repo = spfs::config::open_repository_from_string(config, self.remote.as_ref()).await?;

        let pull_from = match self.pull.take() {
            Some(name @ Some(_)) if name == self.remote => {
                return Err("Cannot --pull from same repo as --remote".into());
            }
            Some(None)
                if self
                    .remote
                    .as_ref()
                    .map(|r| r == "origin")
                    .unwrap_or_default() =>
            {
                return Err("Cannot --pull from same repo as --remote".into());
            }
            Some(mut repo) => Some(
                spfs::config::open_repository_from_string(
                    config,
                    repo.take().or_else(|| Some("origin".to_owned())),
                )
                .await?,
            ),
            None => None,
        };

        let digests = futures::stream::iter(&self.reference)
            .then(|reference| repo.resolve_ref(reference))
            .try_collect()
            .await?;

        tracing::info!("walking repository...");

        let errors = match &repo {
            RepositoryHandle::FS(repo) => {
                spfs::graph::check_database_integrity(repo, digests).await
            }
            RepositoryHandle::Tar(repo) => {
                spfs::graph::check_database_integrity(repo, digests).await
            }
            RepositoryHandle::Rpc(repo) => {
                spfs::graph::check_database_integrity(repo, digests).await
            }
            RepositoryHandle::PayloadFallback(repo) => {
                spfs::graph::check_database_integrity(&**repo, digests).await
            }
            RepositoryHandle::Proxy(repo) => {
                spfs::graph::check_database_integrity(&**repo, digests).await
            }
        };
        let mut repair_count = 0;
        for error in errors.iter() {
            tracing::error!("{error}");

            if let Some(pull_from) = pull_from.as_ref() {
                let syncer = spfs::Syncer::new(pull_from, &repo)
                    .with_policy(spfs::sync::SyncPolicy::ResyncEverything)
                    .with_reporter(spfs::sync::ConsoleSyncReporter::default());
                match error {
                    spfs::Error::UnknownObject(digest)
                    | spfs::Error::ObjectMissingPayload(_, digest) => {
                        match syncer.sync_digest(*digest).await {
                            Ok(_) => {
                                // Drop syncer to be able to see tracing message.
                                drop(syncer);
                                tracing::info!("Successfully repaired!");
                                repair_count += 1;
                            }
                            Err(err) => {
                                // Drop syncer to be able to see tracing message.
                                drop(syncer);
                                tracing::warn!("Could not repair: {err}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if !errors.is_empty() && repair_count < errors.len() {
            if pull_from.is_none() {
                tracing::info!("running with `--pull` may be able to resolve these issues")
            }
            return Ok(1);
        }
        tracing::info!("repository OK");
        Ok(0)
    }
}
