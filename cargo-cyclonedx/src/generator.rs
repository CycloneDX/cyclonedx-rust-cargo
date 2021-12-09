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
use cargo::core::dependency::DepKind;
use cargo::core::Package;
use cargo::core::PackageSet;
use cargo::core::Resolve;
use cargo::core::Workspace;
use cargo::ops;
use cargo::Config;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub trait Generator {
    fn create_sboms(&self, manifest_path: PathBuf)
        -> Result<HashMap<PathBuf, Bom>, GeneratorError>;
}

pub struct SbomGenerator {
    pub all: bool,
}

impl Generator for SbomGenerator {
    fn create_sboms(
        &self,
        manifest_path: PathBuf,
    ) -> Result<HashMap<PathBuf, Bom>, GeneratorError> {
        let config_filepath = manifest_path.to_string_lossy().to_string();
        let config = Config::default().map_err(|error| GeneratorError::CargoConfigError {
            config_filepath: config_filepath.clone(),
            error,
        })?;

        let ws = Workspace::new(&manifest_path, &config).map_err(|error| {
            GeneratorError::CargoConfigError {
                config_filepath: config_filepath.clone(),
                error,
            }
        })?;
        let members: Vec<Package> = ws.members().cloned().collect();
        let (package_ids, resolve) =
            ops::resolve_ws(&ws).map_err(|error| GeneratorError::CargoConfigError {
                config_filepath: config_filepath.clone(),
                error,
            })?;

        let mut result = HashMap::with_capacity(members.len());
        for member in members.iter() {
            let dependencies = if self.all {
                all_dependencies(&members, &package_ids, &resolve)?
            } else {
                top_level_dependencies(&members, &package_ids)?
            };

            let mut bom: Bom = dependencies.iter().collect();

            bom.metadata = get_metadata(member.manifest_path());
            result.insert(member.manifest_path().to_path_buf(), bom);
        }

        Ok(result)
    }
}

/// attempt to treat the Cargo.toml as a simple project to get the metadata
/// for now, do not attempt to generate metadata about a workspace
fn get_metadata(toml_file_path: &Path) -> Option<Metadata> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&toml_file_path)
        .features(cargo_metadata::CargoOpt::AllFeatures)
        .exec();

    match metadata {
        Ok(metadata) => metadata
            .root_package()
            .map(Metadata::from)
            .or_else(|| Some(Metadata::default())),
        Err(e) => {
            log::error!(
                "Attempted to get metadata from the cargo config: {}",
                toml_file_path.to_string_lossy(),
            );
            log::debug!("Got error: {}", e);
            Some(Metadata::default())
        }
    }
}

fn top_level_dependencies(
    members: &[Package],
    package_ids: &PackageSet<'_>,
) -> Result<BTreeSet<Package>, GeneratorError> {
    let mut dependencies = BTreeSet::new();

    let all_dependencies = members
        .iter()
        .flat_map(|m| m.dependencies().iter())
        .filter(|d| d.kind() == DepKind::Normal);
    for dependency in all_dependencies {
        if let Some(package_id) = package_ids
            .package_ids()
            .find(|id| dependency.matches_id(*id))
        {
            let package = package_ids
                .get_one(package_id)
                .map_err(|error| GeneratorError::PackageError { package_id, error })?;
            dependencies.insert(package.to_owned());
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
    package_ids: &PackageSet<'_>,
    resolve: &Resolve,
) -> Result<BTreeSet<Package>, GeneratorError> {
    let mut dependencies = BTreeSet::new();

    for package_id in resolve.iter() {
        let package = package_ids
            .get_one(package_id)
            .map_err(|error| GeneratorError::PackageError { package_id, error })?;
        if members.contains(package) {
            // Skip listing our own packages in our workspace
            continue;
        }
        dependencies.insert(package.to_owned());
    }

    Ok(dependencies)
}

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Expected a root package in the cargo config: {config_filepath}")]
    RootPackageMissingError { config_filepath: String },

    #[error("Could not process the cargo config: {config_filepath}")]
    CargoConfigError {
        config_filepath: String,
        #[source]
        error: anyhow::Error,
    },

    #[error("Error with Cargo metadata")]
    CargoMetaDataError(#[from] cargo_metadata::Error),

    #[error("Error retrieving package information: {package_id}")]
    PackageError {
        package_id: cargo::core::package_id::PackageId,
        #[source]
        error: anyhow::Error,
    },
}
