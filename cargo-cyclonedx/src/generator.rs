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
use crate::config::Pattern;
use crate::config::PlatformSuffix;
use crate::config::Prefix;
use crate::config::SbomConfig;
use crate::config::{IncludedDependencies, ParseMode};
use crate::format::Format;
use crate::purl::get_purl;

use cargo_metadata;
use cargo_metadata::DependencyKind;
use cargo_metadata::Metadata as CargoMetadata;
use cargo_metadata::Node;
use cargo_metadata::NodeDep;
use cargo_metadata::Package;
use cargo_metadata::PackageId;

use cargo_metadata::camino::Utf8PathBuf;
use cyclonedx_bom::external_models::normalized_string::NormalizedString;
use cyclonedx_bom::external_models::spdx::SpdxExpression;
use cyclonedx_bom::external_models::uri::Uri;
use cyclonedx_bom::models::bom::Bom;
use cyclonedx_bom::models::component::{Classification, Component, Components, Scope};
use cyclonedx_bom::models::dependency::{Dependencies, Dependency};
use cyclonedx_bom::models::external_reference::{
    ExternalReference, ExternalReferenceType, ExternalReferences,
};
use cyclonedx_bom::models::license::{License, LicenseChoice, Licenses};
use cyclonedx_bom::models::metadata::Metadata;
use cyclonedx_bom::models::metadata::MetadataError;
use cyclonedx_bom::models::organization::OrganizationalContact;
use cyclonedx_bom::models::tool::{Tool, Tools};
use cyclonedx_bom::validation::Validate;
use cyclonedx_bom::validation::ValidationResult;
use once_cell::sync::Lazy;
use regex::Regex;

use log::Level;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;
use validator::validate_email;

// Maps from PackageId to Package for efficiency - faster lookups than in a Vec
type PackageMap = BTreeMap<PackageId, Package>;
type ResolveMap = BTreeMap<PackageId, Node>;

pub struct SbomGenerator {
    config: SbomConfig,
    workspace_root: Utf8PathBuf,
}

impl SbomGenerator {
    pub fn create_sboms(
        meta: CargoMetadata,
        config: &SbomConfig,
    ) -> Result<Vec<GeneratedSbom>, GeneratorError> {
        log::trace!("Processing the workspace {}", meta.workspace_root);
        let members: Vec<PackageId> = meta.workspace_members;
        let packages = index_packages(meta.packages);
        let resolve = index_resolve(meta.resolve.unwrap().nodes);

        let mut result = Vec::with_capacity(members.len());
        for member in members.iter() {
            log::trace!("Processing the package {}", member);

            let (dependencies, pruned_resolve) =
                if config.included_dependencies() == IncludedDependencies::AllDependencies {
                    all_dependencies(member, &packages, &resolve)
                } else {
                    top_level_dependencies(member, &packages, &resolve)
                };

            let generator = SbomGenerator {
                config: config.clone(),
                workspace_root: meta.workspace_root.to_owned(),
            };
            let bom = generator.create_bom(member, &dependencies, &pruned_resolve)?;

            if cfg!(debug_assertions) {
                let result = bom.validate().unwrap();
                if let ValidationResult::Failed { reasons } = result {
                    panic!("The generated SBOM failed validation: {:?}", &reasons);
                }
            }

            let generated = GeneratedSbom {
                bom,
                manifest_path: packages[member].manifest_path.clone().into_std_path_buf(),
                package_name: packages[member].name.clone(),
                sbom_config: generator.config,
            };

            result.push(generated);
        }

        Ok(result)
    }

    fn create_bom(
        &self,
        package: &PackageId,
        packages: &PackageMap,
        resolve: &ResolveMap,
    ) -> Result<Bom, GeneratorError> {
        let mut bom = Bom::default();
        let root_package = &packages[package];

        let components: Vec<_> = packages
            .values()
            .filter(|p| &p.id != package)
            .map(|component| self.create_component(component, root_package))
            .collect();

        bom.components = Some(Components(components));

        let metadata = self.create_metadata(&packages[package])?;

        bom.metadata = Some(metadata);

        bom.dependencies = Some(create_dependencies(resolve));

        Ok(bom)
    }

    fn create_component(&self, package: &Package, root_package: &Package) -> Component {
        let name = package.name.to_owned().trim().to_string();
        let version = package.version.to_string();

        let purl = match get_purl(package, root_package, &self.workspace_root, None) {
            Ok(purl) => Some(purl),
            Err(e) => {
                log::warn!("Package {} has an invalid Purl: {} ", package.name, e);
                None
            }
        };

        let mut component = Component::new(
            Classification::Library,
            &name,
            &version,
            Some(package.id.to_string()),
        );

        component.purl = purl;
        component.scope = Some(Scope::Required);
        component.external_references = Self::get_external_references(package);
        component.licenses = self.get_licenses(package);

        component.description = package
            .description
            .as_ref()
            .map(|s| NormalizedString::new(s));

        component
    }

    /// Same as [Self::create_component] but also includes information
    /// on binaries and libraries comprising it as subcomponents
    fn create_toplevel_component(&self, package: &Package) -> Component {
        let mut top_component = self.create_component(package, package);
        let mut subcomponents: Vec<Component> = Vec::new();
        let mut subcomp_count: u32 = 0;
        for tgt in &package.targets {
            // Ignore tests, benches, examples and build scripts.
            // They are not part of the final build artifacts, which is what we are after.
            if !(tgt.is_bench() || tgt.is_example() || tgt.is_test() || tgt.is_custom_build()) {
                // classification
                #[allow(clippy::if_same_then_else)]
                let cdx_type = if tgt.is_bin() {
                    Classification::Application
                // sadly no .is_proc_macro() yet
                } else if tgt.kind.iter().any(|kind| kind == "proc-macro") {
                    // There isn't a better way to express it with CycloneDX types
                    Classification::Library
                } else if tgt.kind.iter().any(|kind| kind.contains("lib")) {
                    Classification::Library
                } else {
                    log::warn!(
                        "Target {} is neither a binary nor a library! Kinds: {}",
                        tgt.name,
                        tgt.kind.join(", ")
                    );
                    continue;
                };

                // bom_ref
                let bom_ref = format!(
                    "{} bin-target-{}",
                    top_component.bom_ref.as_ref().unwrap(),
                    subcomp_count
                );
                subcomp_count += 1;

                // create the subcomponent
                let mut subcomponent = Component::new(
                    cdx_type,
                    &tgt.name,
                    &package.version.to_string(),
                    Some(bom_ref),
                );

                // PURL subpaths are computed relative to the directory with the `Cargo.toml`
                // *for this specific package*, not the workspace root.
                // This is done because the tarball uploaded to crates.io only contains the package,
                // not the workspace, so paths resolved relatively to the workspace root would not be valid.
                //
                // When using a git repo that contains a workspace, Cargo will automatically select
                // the right package out of the workspace. Paths can then be resolved relatively to it.
                // So the information we encode here is sufficient to idenfity the file in git too.
                let package_dir = package
                    .manifest_path
                    .parent()
                    .expect("manifest_path in `cargo metadata` output is not a file!");
                if let Ok(relative_path) = tgt.src_path.strip_prefix(package_dir) {
                    subcomponent.purl =
                        get_purl(package, package, &self.workspace_root, Some(relative_path)).ok();
                } else {
                    log::warn!(
                        "Source path \"{}\" is not a subpath of workspace root \"{}\"",
                        tgt.src_path,
                        self.workspace_root
                    );
                }

                subcomponents.push(subcomponent);
            }
        }
        top_component.components = Some(Components(subcomponents));
        top_component
    }

    fn get_classification(pkg: &Package) -> Classification {
        // Transitive dependencies that contain both libraries and binaries
        // get surfaces only as a library by `cargo metadata`.
        //
        // Both "bin" and "lib" can only occur together in the toplevel package,
        // and we record its constituent parts in detail.
        //
        // We have to make a judgement call how to summarise having both bin and lib targets,
        // and that call is "consider it a binary".
        if pkg.targets.iter().any(|tgt| tgt.is_bin()) {
            return Classification::Application;
        }

        Classification::Library
    }

    fn get_external_references(package: &Package) -> Option<ExternalReferences> {
        let mut references = Vec::new();

        if let Some(documentation) = &package.documentation {
            match Uri::try_from(documentation.to_string()) {
                Ok(uri) => references.push(ExternalReference::new(
                    ExternalReferenceType::Documentation,
                    uri,
                )),
                Err(e) => log::warn!(
                    "Package {} has an invalid documentation URI ({}): {} ",
                    package.name,
                    documentation,
                    e
                ),
            }
        }

        if let Some(website) = &package.homepage {
            match Uri::try_from(website.to_string()) {
                Ok(uri) => {
                    references.push(ExternalReference::new(ExternalReferenceType::Website, uri))
                }
                Err(e) => log::warn!(
                    "Package {} has an invalid homepage URI ({}): {} ",
                    package.name,
                    website,
                    e
                ),
            }
        }

        if let Some(other) = &package.links {
            match Uri::try_from(other.to_string()) {
                Ok(uri) => {
                    references.push(ExternalReference::new(ExternalReferenceType::Other, uri))
                }
                Err(e) => log::warn!(
                    "Package {} has an invalid links URI ({}): {} ",
                    package.name,
                    other,
                    e
                ),
            }
        }

        if let Some(vcs) = &package.repository {
            match Uri::try_from(vcs.to_string()) {
                Ok(uri) => references.push(ExternalReference::new(ExternalReferenceType::Vcs, uri)),
                Err(e) => log::warn!(
                    "Package {} has an invalid repository URI ({}): {} ",
                    package.name,
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

    fn get_licenses(&self, package: &Package) -> Option<Licenses> {
        let mut licenses: Option<LicenseChoice> = None;

        if let Some(license) = &package.license {
            let parse_mode = self
                .config
                .license_parser
                .as_ref()
                .map(|opts| opts.mode)
                .unwrap_or_default();

            log::trace!(
                "Using license parser mode [{:?}] for package [{}@{}]",
                parse_mode,
                package.name,
                package.version
            );

            let result = match parse_mode {
                ParseMode::Strict => SpdxExpression::try_from(license.to_string()),
                ParseMode::Lax => SpdxExpression::parse_lax(license.to_string()),
            };

            match result {
                Ok(expression) => licenses = Some(LicenseChoice::Expressions(vec![expression])),
                Err(err) => {
                    let level = match &self.config.license_parser {
                        Some(opts) if opts.accept_named.contains(license) => Level::Info,
                        _ => Level::Warn,
                    };
                    log::log!(
                        level,
                        "Package {} has an invalid license expression ({}), using as named license: {}",
                        package.name,
                        license,
                        err,
                    );
                    licenses = Some(LicenseChoice::Licenses(vec![License::named_license(
                        license,
                    )]));
                }
            }
        }

        if let Some(licenses) = licenses {
            Some(Licenses(licenses))
        } else {
            log::trace!("Package {} has no licenses", package.name);
            None
        }
    }

    fn create_metadata(&self, package: &Package) -> Result<Metadata, GeneratorError> {
        let authors = Self::create_authors(package);

        let mut metadata = Metadata::new()?;
        if !authors.is_empty() {
            metadata.authors = Some(authors);
        }

        let mut component = self.create_toplevel_component(package);

        component.component_type = Self::get_classification(package);

        metadata.component = Some(component);

        let tool = Tool::new("CycloneDX", "cargo-cyclonedx", env!("CARGO_PKG_VERSION"));

        metadata.tools = Some(Tools(vec![tool]));

        Ok(metadata)
    }

    fn create_authors(package: &Package) -> Vec<OrganizationalContact> {
        let mut authors = vec![];
        let mut invalid_authors = vec![];

        for author in &package.authors {
            match Self::parse_author(author) {
                Ok(author) => authors.push(author),
                Err(e) => invalid_authors.push((author, e)),
            }
        }

        invalid_authors
            .into_iter()
            .for_each(|(author, error)| log::warn!("Invalid author {}: {:?}", author, error));

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
}

fn index_packages(packages: Vec<Package>) -> PackageMap {
    packages
        .into_iter()
        .map(|pkg| (pkg.id.clone(), pkg))
        .collect()
}

fn index_resolve(packages: Vec<Node>) -> ResolveMap {
    packages
        .into_iter()
        .map(|pkg| (pkg.id.clone(), pkg))
        .collect()
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
        package_id: cargo_metadata::PackageId,
        #[source]
        error: anyhow::Error,
    },

    #[error("Error creating Metadata")]
    MetadataError(#[from] MetadataError),

    #[error("Could not parse author string: {}", .0)]
    AuthorParseError(String),

    #[error("Invalid regular expression")]
    InvalidRegexError(#[source] regex::Error),
}

/// Generates the `Dependencies` field in the final SBOM
fn create_dependencies(resolve: &ResolveMap) -> Dependencies {
    let deps = resolve
        .values()
        .map(|node| Dependency {
            dependency_ref: node.id.to_string(),
            dependencies: node.dependencies.iter().map(|d| d.to_string()).collect(),
        })
        .collect();
    Dependencies(deps)
}

fn top_level_dependencies(
    root: &PackageId,
    packages: &PackageMap,
    resolve: &ResolveMap,
) -> (PackageMap, ResolveMap) {
    log::trace!("Adding top-level dependencies to SBOM");

    // Only include packages that have dependency kinds other than "Development"
    let root_node = strip_dev_dependencies(&resolve[root]);

    let mut pkg_result = PackageMap::new();
    // Record the root package, then its direct non-dev dependencies
    pkg_result.insert(root.to_owned(), packages[root].to_owned());
    for id in &root_node.dependencies {
        pkg_result.insert((*id).to_owned(), packages[id].to_owned());
    }

    let mut resolve_result = ResolveMap::new();
    for id in &root_node.dependencies {
        // Clear all dependencies, pretend there is only one level
        let mut node = resolve[id].clone();
        node.deps = Vec::new();
        node.dependencies = Vec::new();
        resolve_result.insert((*id).to_owned(), node);
    }
    // Insert the root node at the end now that we're done iterating over it
    resolve_result.insert(root.to_owned(), root_node);

    (pkg_result, resolve_result)
}

fn all_dependencies(
    root: &PackageId,
    packages: &PackageMap,
    resolve: &ResolveMap,
) -> (PackageMap, ResolveMap) {
    log::trace!("Adding all dependencies to SBOM");

    // Note: using Vec (without deduplication) can theoretically cause quadratic memory usage,
    // but since `Node` does not implement `Ord` or `Hash` it's hard to deduplicate them.
    // These are all pointers and there's not a lot of them, it's highly unlikely to be an issue in practice.
    // We can work around this by using a map instead of a set if need be.
    let mut current_queue: Vec<&Node> = vec![&resolve[root]];
    let mut next_queue: Vec<&Node> = Vec::new();

    let mut out_resolve = ResolveMap::new();

    // Run breadth-first search (BFS) over the dependency graph
    // to determine which nodes are actually depended on by our package
    // (not other packages) and to remove dev-dependencies
    while !current_queue.is_empty() {
        for node in current_queue.drain(..) {
            // If we haven't processed this node yet...
            if !out_resolve.contains_key(&node.id) {
                // Add the node to the output
                out_resolve.insert(node.id.to_owned(), strip_dev_dependencies(node));
                // Queue its dependencies for the next BFS loop iteration
                next_queue.extend(non_dev_dependencies(&node.deps).map(|dep| &resolve[&dep.pkg]));
            }
        }
        std::mem::swap(&mut current_queue, &mut next_queue);
    }

    // Remove everything from `packages` that doesn't appear in the `resolve` we've built
    let out_packages = packages
        .iter()
        .filter(|(id, _pkg)| out_resolve.contains_key(id))
        .map(|(id, pkg)| (id.to_owned(), pkg.to_owned()))
        .collect();

    (out_packages, out_resolve)
}

fn strip_dev_dependencies(node: &Node) -> Node {
    let mut node = node.clone();
    node.deps = non_dev_dependencies(&node.deps).cloned().collect();
    node.dependencies = node.deps.iter().map(|d| d.pkg.to_owned()).collect();
    node
}

/// Filters out dependencies only used for development, and not affecting the final binary.
/// These are specified under `[dev-dependencies]` in Cargo.toml.
fn non_dev_dependencies(input: &[NodeDep]) -> impl Iterator<Item = &NodeDep> {
    input.iter().filter(|p| {
        p.dep_kinds
            .iter()
            .any(|dep| dep.kind != DependencyKind::Development)
    })
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
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        match self.sbom_config.format() {
            Format::Json => {
                self.bom
                    .output_as_json_v1_3(&mut writer)
                    .map_err(SbomWriterError::JsonWriteError)?;
            }
            Format::Xml => {
                self.bom
                    .output_as_xml_v1_3(&mut writer)
                    .map_err(SbomWriterError::XmlWriteError)?;
            }
        }

        // Flush the writer explicitly to catch and report any I/O errors
        writer.flush()?;

        Ok(())
    }

    fn filename(&self) -> String {
        let output_options = self.sbom_config.output_options();
        let prefix = match output_options.prefix {
            Prefix::Pattern(Pattern::Bom) => "bom".to_string(),
            Prefix::Pattern(Pattern::Package) => self.package_name.clone(),
            Prefix::Custom(c) => c.to_string(),
        };

        let platform_suffix = match output_options.platform_suffix {
            PlatformSuffix::NotIncluded => "".to_owned(),
            PlatformSuffix::Included => {
                let target_string = self.sbom_config.target.as_ref().unwrap();
                format!("_{}", target_string.as_str())
            }
        };

        format!(
            "{}{}{}.{}",
            prefix,
            platform_suffix,
            output_options.cdx_extension.extension(),
            self.sbom_config.format()
        )
    }
}

#[derive(Error, Debug)]
pub enum SbomWriterError {
    #[error("I/O error")]
    IoError(#[source] std::io::Error),

    #[error("Error writing JSON file")]
    JsonWriteError(#[source] cyclonedx_bom::errors::JsonWriteError),

    #[error("Error writing XML file")]
    XmlWriteError(#[source] cyclonedx_bom::errors::XmlWriteError),

    #[error("Error serializing to XML")]
    SerializeXmlError(#[source] std::io::Error),
}

impl From<std::io::Error> for SbomWriterError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_parse_author_and_email() {
        let actual = SbomGenerator::parse_author("First Last <user@domain.tld>")
            .expect("Failed to parse author");
        let expected = OrganizationalContact::new("First Last", Some("user@domain.tld"));

        assert_eq!(actual, expected);
    }
    #[test]
    fn it_should_parse_author_only() {
        let actual = SbomGenerator::parse_author("First Last").expect("Failed to parse author");
        let expected = OrganizationalContact::new("First Last", None);

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_fail_to_parse_invalid_email() {
        let actual = SbomGenerator::parse_author("First Last <userdomain.tld>")
            .expect_err("Failed to throw error");

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
        let actual = SbomGenerator::parse_author("<First Last user@domain.tld>")
            .expect("Failed to parse author");
        let expected = OrganizationalContact::new("<First Last user@domain.tld>", None);

        assert_eq!(actual, expected);
    }
}
