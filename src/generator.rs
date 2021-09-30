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
use crate::Bom;
use crate::Metadata;
use anyhow::anyhow;
use cargo::core::dependency::DepKind;
use cargo::core::Package;
use cargo::core::PackageSet;
use cargo::core::Resolve;
use cargo::core::Workspace;
use cargo::ops;
use cargo::CargoResult;
use cargo::Config;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

pub trait Generator {
    fn create_sbom<'a>(&self, manifest_path: PathBuf) -> Result<Bom, GeneratorError>;
}

pub struct SbomGenerator {
    pub all: bool,
}

impl Generator for SbomGenerator {
    fn create_sbom<'a>(&self, manifest_path: PathBuf) -> Result<Bom, GeneratorError> {
        let config = Config::default()?;

        let ws = Workspace::new(&manifest_path, &config)?;
        let members: Vec<Package> = ws.members().cloned().collect();
        let (package_ids, resolve) = ops::resolve_ws(&ws)?;

        let root = get_root_package(manifest_path)?;

        let dependencies = if self.all {
            all_dependencies(&members, package_ids, resolve)?
        } else {
            top_level_dependencies(&members, package_ids)?
        };

        let mut bom: Bom = dependencies.iter().collect();

        let metadata = Metadata::from(&root);

        bom.metadata = Some(metadata);

        Ok(bom)
    }
}

fn get_root_package(toml_file_path: PathBuf) -> anyhow::Result<cargo_metadata::Package> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(toml_file_path)
        .features(cargo_metadata::CargoOpt::AllFeatures)
        .exec()?;

    if let Some(root) = metadata.clone().root_package() {
        return Ok(root.to_owned());
    }

    Err(anyhow!("Could not get root package"))
}

fn top_level_dependencies(
    members: &[Package],
    package_ids: PackageSet<'_>,
) -> CargoResult<BTreeSet<Package>> {
    let mut dependencies = BTreeSet::new();

    for member in members {
        for dependency in member.dependencies() {
            // Filter out Build and Development dependencies
            match dependency.kind() {
                DepKind::Normal => (),
                DepKind::Build | DepKind::Development => continue,
            }
            if let Some(dep) = package_ids
                .package_ids()
                .find(|id| dependency.matches_id(*id))
            {
                let package = package_ids.get_one(dep)?;
                dependencies.insert(package.to_owned());
            }
        }
    }

    // Filter out our own workspace crates from dependency list
    for member in members {
        dependencies.remove(member);
    }

    Ok(dependencies)
}

fn all_dependencies(
    members: &[Package],
    package_ids: PackageSet<'_>,
    resolve: Resolve,
) -> CargoResult<BTreeSet<Package>> {
    let mut dependencies = BTreeSet::new();

    for package_id in resolve.iter() {
        let package = package_ids.get_one(package_id)?;
        if members.contains(&package) {
            // Skip listing our own packages in our workspace
            continue;
        }
        dependencies.insert(package.to_owned());
    }

    Ok(dependencies)
}

#[derive(Debug)]
pub struct GeneratorError {
    details: String,
}

impl From<anyhow::Error> for GeneratorError {
    fn from(_: anyhow::Error) -> Self {
        Self {
            details: "An error occurred returning an anyhow error".to_string(),
        }
    }
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occurred generating a Bom")
    }
}

impl Error for GeneratorError {}
