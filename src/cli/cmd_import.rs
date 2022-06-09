// Copyright (c) 2022 Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk

use anyhow::{Context, Result};
use clap::Args;

use super::Run;

/// Import a previously exported package/archive
#[derive(Args)]
pub struct Import {
    /// The archive to import from
    #[clap(name = "FILE")]
    pub files: Vec<std::path::PathBuf>,
}

impl Run for Import {
    fn run(&mut self) -> Result<i32> {
        for filename in self.files.iter() {
            spk::HANDLE
                .block_on(spk::storage::import_package(filename))
                .context("Import failed")?;
        }
        Ok(0)
    }
}