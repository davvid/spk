// Copyright (c) 2021 Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk
use std::collections::HashSet;

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Ident, OptionMap, Request};
use crate::{Error, Result};

#[cfg(test)]
#[path = "./requirements_list_test.rs"]
mod requirements_list_test;

/// A set of installation requirements.
#[derive(Debug, Default, Hash, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct RequirementsList(Vec<Request>);

impl std::ops::Deref for RequirementsList {
    type Target = Vec<Request>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RequirementsList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RequirementsList {
    /// Add or update a requirement in this list.
    ///
    /// If a request exists for the same name, it is replaced with the given
    /// one. Otherwise the new request is appended to the list.
    pub fn upsert(&mut self, request: Request) {
        let name = request.name();
        for other in self.iter_mut() {
            if other.name() == name {
                let _ = std::mem::replace(other, request);
                return;
            }
        }
        self.push(request);
    }

    /// Render all requests with a package pin using the given resolved packages.
    pub fn render_all_pins<'a>(
        &mut self,
        options: &OptionMap,
        resolved: impl Iterator<Item = &'a Ident>,
    ) -> Result<()> {
        let mut by_name = std::collections::HashMap::new();
        for pkg in resolved {
            by_name.insert(pkg.name(), pkg);
        }
        for request in self.iter_mut() {
            match request {
                Request::Pkg(request) => {
                    if request.pin.is_none() {
                        continue;
                    }
                    match by_name.get(&request.pkg.name()) {
                        None => {
                            return Err(Error::String(
                                format!("Cannot resolve fromBuildEnv, package not present: {}\nIs it missing from your package build options?", request.pkg.name())
                            ));
                        }
                        Some(resolved) => {
                            let rendered = request.render_pin(resolved)?;
                            let _ = std::mem::replace(request, rendered);
                        }
                    }
                }
                Request::Var(request) => {
                    if !request.pin {
                        continue;
                    }
                    let mut split = request.var.splitn(2, '.');
                    let (var, opts) = match (split.next().unwrap(), split.next()) {
                        (package, Some(var)) => (var, options.package_options(package)),
                        (var, None) => (var, options.clone()),
                    };
                    match opts.get(var) {
                        None => {
                            return Err(Error::String(
                                format!("Cannot resolve fromBuildEnv, variable not set: {}\nIs it missing from the package build options?", request.var)
                            ));
                        }
                        Some(opt) => {
                            let rendered = request.render_pin(opt)?;
                            let _ = std::mem::replace(request, rendered);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for RequirementsList {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let unchecked = Vec::<Request>::deserialize(deserializer)?;

        let mut requirement_names = HashSet::with_capacity(unchecked.len());
        for name in unchecked.iter().map(Request::name) {
            if requirement_names.contains(&name) {
                return Err(serde::de::Error::custom(format!(
                    "found multiple install requirements for '{}'",
                    name
                )));
            }
            requirement_names.insert(name);
        }

        Ok(RequirementsList(unchecked))
    }
}

impl IntoPy<Py<pyo3::types::PyAny>> for super::RequirementsList {
    fn into_py(self, py: Python) -> Py<pyo3::types::PyAny> {
        self.0.into_py(py)
    }
}

impl<'source> FromPyObject<'source> for RequirementsList {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(RequirementsList(Vec::<Request>::extract(ob)?))
    }
}
