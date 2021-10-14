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
use std::collections::BTreeSet;
use std::path::PathBuf;
use thiserror::Error;

pub trait Generator {
    fn create_sbom<'a>(&self, manifest_path: PathBuf) -> Result<Bom, GeneratorError>;
}

pub struct SbomGenerator {
    pub all: bool,
}

impl Generator for SbomGenerator {
    fn create_sbom<'a>(&self, manifest_path: PathBuf) -> Result<Bom, GeneratorError> {
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

fn get_root_package(toml_file_path: PathBuf) -> Result<cargo_metadata::Package, GeneratorError> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(toml_file_path)
        .features(cargo_metadata::CargoOpt::AllFeatures)
        .exec()?;

    if let Some(root) = metadata.clone().root_package() {
        return Ok(root.to_owned());
    }

    Err(GeneratorError::RootPackageMissingError)
}

fn top_level_dependencies(
    members: &[Package],
    package_ids: PackageSet<'_>,
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
    package_ids: PackageSet<'_>,
    resolve: Resolve,
) -> Result<BTreeSet<Package>, GeneratorError> {
    let mut dependencies = BTreeSet::new();

    for package_id in resolve.iter() {
        let package = package_ids
            .get_one(package_id)
            .map_err(|error| GeneratorError::PackageError { package_id, error })?;
        if members.contains(&package) {
            // Skip listing our own packages in our workspace
            continue;
        }
        dependencies.insert(package.to_owned());
    }

    Ok(dependencies)
}

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Could not get root package")]
    RootPackageMissingError,

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
