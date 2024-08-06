/*
 * This file is part of CycloneDX Rust Cargo.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::prelude::NormalizedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lifecycles(pub Vec<Lifecycle>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Phase(Phase),
    Description(Description),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    Design,
    PreBuild,
    Build,
    PostBuild,
    Operations,
    Discovery,
    Decommission,
    #[doc(hidden)]
    Unknown(String),
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Phase::Design => "design",
            Phase::PreBuild => "pre-build",
            Phase::Build => "build",
            Phase::PostBuild => "post-build",
            Phase::Operations => "operations",
            Phase::Discovery => "discovery",
            Phase::Decommission => "decommission",
            Phase::Unknown(unknown) => unknown,
        };
        write!(f, "{}", s)
    }
}

impl Phase {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "design" => Self::Design,
            "pre-build" => Self::PreBuild,
            "build" => Self::Build,
            "post-build" => Self::PostBuild,
            "operations" => Self::Operations,
            "discovery" => Self::Discovery,
            "decommission" => Self::Decommission,
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

/// A description of a `Lifecycle`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    pub name: NormalizedString,
    pub description: Option<NormalizedString>,
}
