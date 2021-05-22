// Copyright (c) 2021 Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk
use serde::{Deserialize, Serialize};

use super::{OptionMap, Request};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStage {
    Sources,
    Build,
    Install,
}

impl Serialize for TestStage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            TestStage::Sources => "sources".serialize(serializer),
            TestStage::Build => "build".serialize(serializer),
            TestStage::Install => "install".serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for TestStage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "sources" => Ok(Self::Sources),
            "build" => Ok(Self::Build),
            "install" => Ok(Self::Install),
            other => Err(serde::de::Error::custom(format!(
                "Invalid test stage '{}', must be one of: source, build, install",
                other
            ))),
        }
    }
}

/// A set of structured inputs used to build a package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSpec {
    stage: TestStage,
    script: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    selectors: Vec<OptionMap>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    requirements: Vec<Request>,
}
