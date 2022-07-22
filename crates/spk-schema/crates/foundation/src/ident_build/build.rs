// Copyright (c) Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk
use std::str::FromStr;

use thiserror::Error;

#[cfg(test)]
#[path = "./build_test.rs"]
mod build_test;

pub const SRC: &str = "src";
pub const EMBEDDED: &str = "embedded";

/// Denotes that an invalid build digest was given.
#[derive(Debug, Error)]
#[error("Invalid build: {message}")]
pub struct InvalidBuildError {
    pub message: String,
}

impl InvalidBuildError {
    pub fn new_error(msg: String) -> super::Error {
        super::Error::InvalidBuildError(Self { message: msg })
    }
}

/// An embedded package's source (if known).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EmbeddedSource {
    Ident(String),
    Unknown,
}

impl std::fmt::Display for EmbeddedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{EMBEDDED}")?;
        match self {
            EmbeddedSource::Ident(ident) => write!(f, "[{ident}]"),
            EmbeddedSource::Unknown => Ok(()),
        }
    }
}

/// Build represents a package build identifier.
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Build {
    Source,
    Embedded(EmbeddedSource),
    Digest([char; crate::option_map::DIGEST_SIZE]),
}

impl Build {
    /// The name or digest of this build as shown in a version
    pub fn digest(&self) -> String {
        match self {
            Build::Source => SRC.to_string(),
            Build::Embedded(by) => by.to_string(),
            Build::Digest(d) => d.iter().collect(),
        }
    }

    pub fn is_source(&self) -> bool {
        matches!(self, Build::Source)
    }

    pub fn is_embedded(&self) -> bool {
        matches!(self, Build::Embedded(_))
    }
}

impl std::fmt::Debug for Build {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.digest().as_str())
    }
}

impl std::fmt::Display for Build {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.digest().as_str())
    }
}

impl FromStr for Build {
    type Err = super::Error;

    fn from_str(source: &str) -> super::Result<Self> {
        use nom::combinator::all_consuming;

        all_consuming(super::parsing::build::<nom_supreme::error::ErrorTree<_>>)(source)
            .map(|(_, build)| build)
            .map_err(|err| match err {
                nom::Err::Error(e) | nom::Err::Failure(e) => {
                    InvalidBuildError::new_error(e.to_string())
                }
                nom::Err::Incomplete(_) => unreachable!(),
            })
    }
}

/// Parse the given string as a build identifier
pub fn parse_build<S: AsRef<str>>(digest: S) -> super::Result<Build> {
    Build::from_str(digest.as_ref())
}
