// Copyright (c) 2021 Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk
use pyo3::prelude::*;

use crate::api::{self, Build};

use super::graph;

#[derive(Clone, Copy)]
pub enum Validators {
    Deprecation(DeprecationValidator),
    BinaryOnly(BinaryOnlyValidator),
}

pub trait ValidatorT {
    /// Check if the given package is appropriate for the provided state.
    fn validate(&self, state: &graph::State, spec: &api::Spec)
        -> crate::Result<api::Compatibility>;
}

impl ValidatorT for Validators {
    fn validate(
        &self,
        state: &graph::State,
        spec: &api::Spec,
    ) -> crate::Result<api::Compatibility> {
        match self {
            Validators::Deprecation(v) => v.validate(state, spec),
            Validators::BinaryOnly(v) => v.validate(state, spec),
        }
    }
}

#[pyclass(subclass)]
pub struct Validator {}

/// Ensures that deprecated packages are not included unless specifically requested.
#[pyclass(extends=Validator)]
#[derive(Clone, Copy)]
pub struct DeprecationValidator {}

impl ValidatorT for DeprecationValidator {
    fn validate(
        &self,
        state: &graph::State,
        spec: &api::Spec,
    ) -> crate::Result<api::Compatibility> {
        if !spec.deprecated {
            return Ok(api::Compatibility::Compatible);
        }
        if spec.pkg.build.is_none() {
            return Ok(api::Compatibility::Incompatible(
                "package version is deprecated".to_owned(),
            ));
        }
        let request = state.get_merged_request(spec.pkg.name())?;
        if request.pkg.build == spec.pkg.build {
            return Ok(api::Compatibility::Compatible);
        }
        Ok(api::Compatibility::Incompatible(
            "build is deprecated (and not requested exactly)".to_owned(),
        ))
    }
}

/// Enforces the resolution of binary packages only, denying new builds from source.
#[pyclass(extends=Validator)]
#[derive(Clone, Copy)]
pub struct BinaryOnlyValidator {}

const ONLY_BINARY_PACKAGES_ALLOWED: &str = "only binary packages are allowed";

impl ValidatorT for BinaryOnlyValidator {
    fn validate(
        &self,
        state: &graph::State,
        spec: &api::Spec,
    ) -> crate::Result<api::Compatibility> {
        if spec.pkg.build.is_none() {
            return Ok(api::Compatibility::Incompatible(
                ONLY_BINARY_PACKAGES_ALLOWED.to_owned(),
            ));
        }
        let request = state.get_merged_request(spec.pkg.name())?;
        if spec.pkg.build == Some(Build::Source) && request.pkg.build != spec.pkg.build {
            return Ok(api::Compatibility::Incompatible(
                ONLY_BINARY_PACKAGES_ALLOWED.to_owned(),
            ));
        }
        Ok(api::Compatibility::Compatible)
    }
}

pub fn default_validators() -> Vec<Validators> {
    vec![Validators::Deprecation(DeprecationValidator {})]
}
