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
use crate::config::IncludedDependencies;
use crate::config::Pattern;
use crate::config::Prefix;
use crate::config::SbomConfig;
use crate::format::Format;
use crate::toml::config_from_toml;
use crate::toml::ConfigError;
use crate::traits::ToXml;
use crate::Bom;
use crate::Metadata;
use cargo::core::dependency::DepKind;
use cargo::core::Package;
use cargo::core::PackageSet;
use cargo::core::Resolve;
use cargo::core::Workspace;
use cargo::ops;
use std::{
    collections::BTreeSet,
    fs::File,
    io::LineWriter,
    path::{Path, PathBuf},
};
use thiserror::Error;
use xml_writer::XmlWriter;

pub struct SbomGenerator {}

impl SbomGenerator {
    pub fn create_sboms(
        ws: Workspace,
        config_override: &SbomConfig,
    ) -> Result<Vec<GeneratedSbom>, GeneratorError> {
        log::trace!(
            "Processing the workspace {} configuration",
            ws.root_manifest().to_string_lossy()
        );
        let workspace_config = config_from_toml(ws.custom_metadata())?;
        let members: Vec<Package> = ws.members().cloned().collect();

        let (package_ids, resolve) =
            ops::resolve_ws(&ws).map_err(|error| GeneratorError::CargoConfigError {
                config_filepath: ws.root_manifest().to_string_lossy().to_string(),
                error,
            })?;

        let mut result = Vec::with_capacity(members.len());
        for member in members.iter() {
            log::trace!(
                "Processing the package {} configuration",
                member.manifest_path().to_string_lossy()
            );
            let package_config = config_from_toml(member.manifest().custom_metadata())?;
            let config = workspace_config
                .merge(&package_config)
                .merge(config_override);

            log::trace!("Config from workspace metadata: {:?}", workspace_config);
            log::trace!("Config from package metadata: {:?}", package_config);
            log::trace!("Config from config override: {:?}", config_override);
            log::debug!("Config from merged config: {:?}", config);

            let dependencies =
                if config.included_dependencies() == IncludedDependencies::AllDependencies {
                    all_dependencies(&members, &package_ids, &resolve)?
                } else {
                    top_level_dependencies(&members, &package_ids)?
                };

            let mut bom: Bom = dependencies.iter().collect();

            bom.metadata = get_metadata(member.manifest_path());

            let generated = GeneratedSbom {
                bom,
                manifest_path: member.manifest_path().to_path_buf(),
                package_name: member.name().to_string(),
                sbom_config: config,
            };

            result.push(generated);
        }

        Ok(result)
    }
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

    #[error("Error with Cargo custom_metadata")]
    CustomMetadataTomlError(#[from] ConfigError),
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
    log::trace!("Adding top-level dependencies to SBOM");
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
    log::trace!("Adding all dependencies to SBOM");
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

/// Contains a generated SBOM and context used in its generation
///
/// * `bom` - Generated SBOM
/// * `manifest_path` - Folder containing the `Cargo.toml` manifest
/// * `package_name` - Package from which this SBOM was generated
/// * `sbom_config` - Configuration options used during generation
pub struct GeneratedSbom {
    pub bom: Bom,
    pub manifest_path: PathBuf,
    pub package_name: String,
    pub sbom_config: SbomConfig,
}

impl GeneratedSbom {
    /// Writes SBOM to either a JSON or XML file in the same folder as `Cargo.toml` manifest
    pub fn write_to_file(&self) -> Result<(), SbomWriterError> {
        let path = self.manifest_path.with_file_name(self.filename());
        log::info!("Outputting {}", path.display());
        let file = File::create(path).map_err(SbomWriterError::FileCreateError)?;
        match self.sbom_config.format() {
            Format::Json => {
                serde_json::to_writer_pretty(file, &self.bom)?;
            }
            Format::Xml => {
                let file = LineWriter::new(file);
                let mut xml = XmlWriter::new(file);

                self.bom
                    .to_xml(&mut xml)
                    .map_err(SbomWriterError::SerializeXmlError)?;
                xml.close().map_err(SbomWriterError::SerializeXmlError)?;
                xml.flush().map_err(SbomWriterError::SerializeXmlError)?;
                let _actual = xml.into_inner();
            }
        }

        Ok(())
    }

    fn filename(&self) -> String {
        let output_options = self.sbom_config.output_options();
        let prefix = match output_options.prefix {
            Prefix::Pattern(Pattern::Bom) => "bom".to_string(),
            Prefix::Pattern(Pattern::Package) => self.package_name.clone(),
            Prefix::Custom(c) => c.to_string(),
        };

        format!(
            "{}{}.{}",
            prefix,
            output_options.cdx_extension.extension(),
            self.sbom_config.format()
        )
    }
}

#[derive(Error, Debug)]
pub enum SbomWriterError {
    #[error("Error creating file")]
    FileCreateError(#[source] std::io::Error),

    #[error("Error serializing to JSON")]
    SerializeJsonError(#[from] serde_json::Error),

    #[error("Error serializing to XML")]
    SerializeXmlError(#[source] std::io::Error),
}
