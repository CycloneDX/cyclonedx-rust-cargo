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
use cargo::core::dependency::DepKind;
use cargo::core::Package;
use cargo::core::PackageSet;
use cargo::core::Resolve;
use cargo::core::Workspace;
use cargo::ops;

use cyclonedx_bom::external_models::normalized_string::NormalizedString;
use cyclonedx_bom::external_models::spdx::SpdxExpression;
use cyclonedx_bom::external_models::uri::{Purl, Uri};
use cyclonedx_bom::models::bom::Bom;
use cyclonedx_bom::models::component::{Classification, Component, Components, Scope};
use cyclonedx_bom::models::external_reference::{
    ExternalReference, ExternalReferenceType, ExternalReferences,
};
use cyclonedx_bom::models::license::{License, LicenseChoice, Licenses};
use cyclonedx_bom::models::metadata::Metadata;
use cyclonedx_bom::models::metadata::MetadataError;
use cyclonedx_bom::models::organization::OrganizationalContact;
use cyclonedx_bom::models::tool::{Tool, Tools};
use cyclonedx_bom::validation::Validate;
use once_cell::sync::Lazy;
use regex::Regex;

use std::convert::TryFrom;
use std::{collections::BTreeSet, fs::File, path::PathBuf};
use thiserror::Error;
use validator::validate_email;

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
                    top_level_dependencies(member, &package_ids, &resolve)?
                };

            let bom = create_bom(member, dependencies)?;

            log::debug!("Bom validation: {:?}", &bom.validate());

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

fn create_bom(package: &Package, dependencies: BTreeSet<Package>) -> Result<Bom, GeneratorError> {
    let mut bom = Bom::default();

    let components: Vec<_> = dependencies
        .into_iter()
        .map(|package| create_component(&package))
        .collect();

    bom.components = Some(Components(components));

    let metadata = create_metadata(package)?;

    bom.metadata = Some(metadata);

    Ok(bom)
}

fn create_component(package: &Package) -> Component {
    let name = package.name().to_owned().trim().to_string();
    let version = package.version().to_string();

    let purl = match Purl::new("cargo", &name, &version) {
        Ok(purl) => Some(purl),
        Err(e) => {
            log::error!("Package {} has an invalid Purl: {} ", package.name(), e);
            None
        }
    };

    let mut component = Component::new(
        Classification::Library,
        &name,
        &version,
        purl.clone().map(|p| p.to_string()),
    );

    component.purl = purl;
    component.scope = Some(Scope::Required);
    component.external_references = get_external_references(package);
    component.licenses = get_licenses(package);

    component.description = package
        .manifest()
        .metadata()
        .description
        .as_ref()
        .map(|s| NormalizedString::new(s));

    component
}

fn get_classification(pkg: &Package) -> Classification {
    if pkg.targets().iter().any(|tgt| tgt.is_bin()) {
        return Classification::Application;
    }

    Classification::Library
}

fn get_external_references(package: &Package) -> Option<ExternalReferences> {
    let mut references = Vec::new();

    let metadata = package.manifest().metadata();

    if let Some(documentation) = &metadata.documentation {
        match Uri::try_from(documentation.to_string()) {
            Ok(uri) => references.push(ExternalReference::new(
                ExternalReferenceType::Documentation,
                uri,
            )),
            Err(e) => log::error!(
                "Package {} has an invalid documentation URI ({}): {} ",
                package.name(),
                documentation,
                e
            ),
        }
    }

    if let Some(website) = &metadata.homepage {
        match Uri::try_from(website.to_string()) {
            Ok(uri) => references.push(ExternalReference::new(ExternalReferenceType::Website, uri)),
            Err(e) => log::error!(
                "Package {} has an invalid homepage URI ({}): {} ",
                package.name(),
                website,
                e
            ),
        }
    }

    if let Some(other) = &metadata.links {
        match Uri::try_from(other.to_string()) {
            Ok(uri) => references.push(ExternalReference::new(ExternalReferenceType::Other, uri)),
            Err(e) => log::error!(
                "Package {} has an invalid links URI ({}): {} ",
                package.name(),
                other,
                e
            ),
        }
    }

    if let Some(vcs) = &metadata.repository {
        match Uri::try_from(vcs.to_string()) {
            Ok(uri) => references.push(ExternalReference::new(ExternalReferenceType::Vcs, uri)),
            Err(e) => log::error!(
                "Package {} has an invalid repository URI ({}): {} ",
                package.name(),
                vcs,
                e
            ),
        }
    }

    if !references.is_empty() {
        return Some(ExternalReferences(references));
    }

    None
}

fn get_licenses(package: &Package) -> Option<Licenses> {
    let mut licenses = vec![];

    if let Some(license) = package.manifest().metadata().license.as_ref() {
        match SpdxExpression::try_from(license.to_string()) {
            Ok(expression) => licenses.push(LicenseChoice::Expression(expression)),
            Err(err) => {
                log::error!(
                    "Package {} has an invalid license expression, trying lax parsing ({}): {}",
                    package.name(),
                    license,
                    err
                );

                match SpdxExpression::parse_lax(license.to_string()) {
                    Ok(expression) => licenses.push(LicenseChoice::Expression(expression)),
                    Err(err) => {
                        log::error!(
                        "Package {} has an invalid license expression that could not be converted to a valid expression, using named license ({}): {}",
                        package.name(),
                        license,
                        err
                    );

                        licenses.push(LicenseChoice::License(License::named_license(license)))
                    }
                }
            }
        }
    }

    if licenses.is_empty() {
        log::trace!("Package {} has no licenses", package.name());
        return None;
    }

    Some(Licenses(licenses))
}

fn create_metadata(package: &Package) -> Result<Metadata, GeneratorError> {
    let authors = create_authors(package);

    let mut metadata = Metadata::new()?;
    if !authors.is_empty() {
        metadata.authors = Some(authors);
    }

    let mut component = create_component(package);

    component.component_type = get_classification(package);

    metadata.component = Some(component);

    let tool = Tool::new("CycloneDX", "cargo-cyclonedx", env!("CARGO_PKG_VERSION"));

    metadata.tools = Some(Tools(vec![tool]));

    Ok(metadata)
}

fn create_authors(package: &Package) -> Vec<OrganizationalContact> {
    let mut authors = vec![];
    let mut invalid_authors = vec![];

    for author in &package.manifest().metadata().authors {
        match parse_author(author) {
            Ok(author) => authors.push(author),
            Err(e) => invalid_authors.push((author, e)),
        }
    }

    invalid_authors
        .into_iter()
        .for_each(|(author, error)| log::error!("Invalid author {}: {:?}", author, error));

    authors
}

fn parse_author(author: &str) -> Result<OrganizationalContact, GeneratorError> {
    static AUTHORS_REGEX: Lazy<Result<Regex, regex::Error>> =
        Lazy::new(|| Regex::new(r"^(?P<author>[^<]+)\s*(<(?P<email>[^>]+)>)?$"));

    match AUTHORS_REGEX
        .as_ref()
        .map_err(|e| GeneratorError::InvalidRegexError(e.to_owned()))?
        .captures(author)
    {
        Some(captures) => {
            let name = captures.name("author").map_or("", |m| m.as_str().trim());
            let email = captures.name("email").map(|m| m.as_str());

            if let Some(email) = email {
                if !validate_email(email) {
                    return Err(GeneratorError::AuthorParseError(
                        "Invalid email, does not conform to HTML5 spec".to_string(),
                    ));
                }
            }

            Ok(OrganizationalContact::new(name, email))
        }
        None => Ok(OrganizationalContact::new(author, None)),
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

    #[error("Error retrieving package information: {package_id}")]
    PackageError {
        package_id: cargo::core::package_id::PackageId,
        #[source]
        error: anyhow::Error,
    },

    #[error("Error with Cargo custom_metadata")]
    CustomMetadataTomlError(#[from] ConfigError),

    #[error("Error creating Metadata")]
    MetadataError(#[from] MetadataError),

    #[error("Could not parse author string: {}", .0)]
    AuthorParseError(String),

    #[error("Invalid regular expression")]
    InvalidRegexError(#[source] regex::Error),
}

fn top_level_dependencies(
    member: &Package,
    package_ids: &PackageSet<'_>,
    resolve: &Resolve,
) -> Result<BTreeSet<Package>, GeneratorError> {
    log::trace!("Adding top-level dependencies to SBOM");
    let mut dependencies = BTreeSet::new();

    let all_dependencies = resolve
        .deps(member.package_id())
        .filter(move |r| r.0 != member.package_id())
        .flat_map(|(_, dependency)| dependency)
        .filter(|d| d.kind() == DepKind::Normal);

    for dependency in all_dependencies {
        log::trace!("Dependency: {dependency:?}");
        match package_ids
            .package_ids()
            .find(|id| dependency.matches_id(*id))
        {
            Some(package_id) => {
                let package = package_ids
                    .get_one(package_id)
                    .map_err(|error| GeneratorError::PackageError { package_id, error })?;
                dependencies.insert(package.to_owned());
            }
            None => {
                log::warn!(
                    "Unable to find package for dependency (name: {}, req: {}, source_id: {})",
                    dependency.package_name(),
                    dependency.version_req(),
                    dependency.source_id(),
                );
            }
        }
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
    pub fn write_to_file(self) -> Result<(), SbomWriterError> {
        let path = self.manifest_path.with_file_name(self.filename());
        log::info!("Outputting {}", path.display());
        let mut file = File::create(path).map_err(SbomWriterError::FileCreateError)?;
        match self.sbom_config.format() {
            Format::Json => {
                self.bom
                    .output_as_json_v1_3(&mut file)
                    .map_err(SbomWriterError::JsonWriteError)?;
            }
            Format::Xml => {
                self.bom
                    .output_as_xml_v1_3(&mut file)
                    .map_err(SbomWriterError::XmlWriteError)?;
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

    #[error("Error writing JSON file")]
    JsonWriteError(#[source] cyclonedx_bom::errors::JsonWriteError),

    #[error("Error writing XML file")]
    XmlWriteError(#[source] cyclonedx_bom::errors::XmlWriteError),

    #[error("Error serializing to XML")]
    SerializeXmlError(#[source] std::io::Error),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_parse_author_and_email() {
        let actual = parse_author("First Last <user@domain.tld>").expect("Failed to parse author");
        let expected = OrganizationalContact::new("First Last", Some("user@domain.tld"));

        assert_eq!(actual, expected);
    }
    #[test]
    fn it_should_parse_author_only() {
        let actual = parse_author("First Last").expect("Failed to parse author");
        let expected = OrganizationalContact::new("First Last", None);

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_fail_to_parse_invalid_email() {
        let actual =
            parse_author("First Last <userdomain.tld>").expect_err("Failed to throw error");

        match actual {
            GeneratorError::AuthorParseError(e) => assert_eq!(
                e,
                "Invalid email, does not conform to HTML5 spec".to_string()
            ),
            e => panic!("Expected AuthorParse error got: {:?}", e),
        }
    }

    #[test]
    fn it_should_parse_author_inside_brackets() {
        let actual = parse_author("<First Last user@domain.tld>").expect("Failed to parse author");
        let expected = OrganizationalContact::new("<First Last user@domain.tld>", None);

        assert_eq!(actual, expected);
    }
}
