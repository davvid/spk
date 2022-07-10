// Copyright (c) 2022 Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use spk::storage::Repository;

use super::Run;

/// Perform repository-level actions and maintenance
#[derive(Args)]
pub struct Repo {
    #[clap(subcommand)]
    command: RepoCommand,
}

#[async_trait::async_trait]
impl Run for Repo {
    async fn run(&mut self) -> Result<i32> {
        self.command.run().await
    }
}

#[derive(Subcommand)]
pub enum RepoCommand {
    /// Perform any pending upgrades to a package repository.
    ///
    /// This will bring the repository up-to-date for the current
    /// spk library version, but may also make it incompatible with
    /// older ones. Upgrades can also take time depending on their
    /// nature and the size of the repository so. Please, take time to
    /// read any release and upgrade notes before invoking this.
    Upgrade {
        /// The repository to upgrade (name or path or url)
        #[clap(name = "REPO")]
        repo: String,
    },
}

impl RepoCommand {
    pub async fn run(&mut self) -> Result<i32> {
        let repo = match &self {
            Self::Upgrade { repo } => repo,
        };
        let repo = spk::storage::remote_repository(repo).await?;
        let status = repo.upgrade().await.context("Upgrade failed")?;
        tracing::info!("{}", status);
        Ok(1)
    }
}
