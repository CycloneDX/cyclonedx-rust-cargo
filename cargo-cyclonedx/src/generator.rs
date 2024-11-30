use crate::config::Describe;
use std::cmp::min;
use std::collections::HashSet;
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
use crate::config::FilenamePattern;
use crate::config::PlatformSuffix;
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

use cargo_lock::package::Checksum;
use cargo_lock::Lockfile;
use cargo_metadata::camino::Utf8PathBuf;
use cyclonedx_bom::external_models::normalized_string::NormalizedString;
use cyclonedx_bom::external_models::spdx::SpdxExpression;
use cyclonedx_bom::external_models::uri::Uri;
use cyclonedx_bom::models::attached_text::AttachedText;
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
use cyclonedx_bom::models::property::{Properties, Property};
use cyclonedx_bom::models::tool::{Tool, Tools};
use cyclonedx_bom::validation::Validate;
use once_cell::sync::Lazy;
use regex::Regex;

use log::Level;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use validator::validate_email;

// Maps from PackageId to Package for efficiency - faster lookups than in a Vec
type PackageMap = BTreeMap<PackageId, Package>;
type ResolveMap = BTreeMap<PackageId, Node>;
type DependencyKindMap = BTreeMap<PackageId, DependencyKind>;

/// The values are ordered from weakest to strongest so that casting to integer would make sense
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
enum PrivateDepKind {
    Development,
    Build,
    Runtime,
}
impl From<PrivateDepKind> for DependencyKind {
    fn from(priv_kind: PrivateDepKind) -> Self {
        match priv_kind {
            PrivateDepKind::Development => DependencyKind::Development,
            PrivateDepKind::Build => DependencyKind::Build,
            PrivateDepKind::Runtime => DependencyKind::Normal,
        }
    }
}

impl From<&DependencyKind> for PrivateDepKind {
    fn from(kind: &DependencyKind) -> Self {
        match kind {
            DependencyKind::Normal => PrivateDepKind::Runtime,
            DependencyKind::Development => PrivateDepKind::Development,
            DependencyKind::Build => PrivateDepKind::Build,
            _ => panic!("Unknown dependency kind"),
        }
    }
}

pub struct SbomGenerator {
    config: SbomConfig,
    workspace_root: Utf8PathBuf,
    crate_hashes: HashMap<cargo_metadata::PackageId, Checksum>,
}

/// Contains a map from `bom_ref` of a subcomponent to the kinds of Cargo targets it has,
/// sourced from `cargo metadata`
#[derive(Debug, Clone)]
pub struct TargetKinds(
    // TODO: refactor it to store Vec<TargetKind> once `cargo_metadata` crate ships enums:
    // https://github.com/oli-obk/cargo_metadata/pull/258
    HashMap<String, Vec<String>>,
);

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

            let dep_kinds = index_dep_kinds(member, &resolve);

            let (dependencies, pruned_resolve) =
                if config.included_dependencies() == IncludedDependencies::AllDependencies {
                    all_dependencies(member, &packages, &resolve, config)
                } else {
                    top_level_dependencies(member, &packages, &resolve, config)
                };

            let manifest_path = packages[member].manifest_path.clone().into_std_path_buf();

            let mut crate_hashes = HashMap::new();
            match locate_cargo_lock(&manifest_path) {
                Ok(path) => match Lockfile::load(path) {
                    Ok(lockfile_contents) => crate_hashes = package_hashes(&lockfile_contents),
                    Err(err) => log::warn!(
                        "Failed to parse `Cargo.lock`: {err}\n\
                        Hashes will not be included in the SBOM."
                    ),
                },
                Err(err) => log::warn!(
                    "Failed to locate `Cargo.lock`: {err}\n\
                    Hashes will not be included in the SBOM."
                ),
            }

            let generator = SbomGenerator {
                config: config.clone(),
                workspace_root: meta.workspace_root.to_owned(),
                crate_hashes,
            };
            let (bom, target_kinds) =
                generator.create_bom(member, &dependencies, &pruned_resolve, &dep_kinds)?;

            let generated = GeneratedSbom {
                bom,
                manifest_path,
                package_name: packages[member].name.clone(),
                sbom_config: generator.config,
                target_kinds,
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
        dep_kinds: &DependencyKindMap,
    ) -> Result<(Bom, TargetKinds), GeneratorError> {
        let mut bom = Bom::default();
        let root_package = &packages[package];

        let components: Vec<_> = packages
            .values()
            .filter(|p| &p.id != package)
            .map(|component| self.create_component(component, root_package, dep_kinds))
            .collect();

        bom.components = Some(Components(components));

        let (metadata, target_kinds) = self.create_metadata(&packages[package])?;

        bom.metadata = Some(metadata);

        bom.dependencies = Some(create_dependencies(resolve));

        Ok((bom, target_kinds))
    }

    fn create_component(
        &self,
        package: &Package,
        root_package: &Package,
        dep_kinds: &DependencyKindMap,
    ) -> Component {
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
        component.scope = match dep_kinds
            .get(&package.id)
            .unwrap_or(&DependencyKind::Normal)
        {
            DependencyKind::Normal => Some(Scope::Required),
            _ => Some(Scope::Excluded),
        };
        component.external_references = Self::get_external_references(package);
        component.licenses = self.get_licenses(package);
        component.hashes = self.get_hashes(package);

        component.description = package
            .description
            .as_ref()
            .map(|s| NormalizedString::new(s));

        // TODO: record in `authors` field rather than `author` when writing v1.6
        if !package.authors.is_empty() {
            component.author = Some(NormalizedString::new(&package.authors.join(", ")));
        }

        component
    }

    /// Same as [Self::create_component] but also includes information
    /// on binaries and libraries comprising it as subcomponents
    fn create_toplevel_component(&self, package: &Package) -> (Component, TargetKinds) {
        let mut top_component = self.create_component(package, package, &DependencyKindMap::new());
        let mut subcomponents: Vec<Component> = Vec::new();
        let mut target_kinds = HashMap::new();
        for tgt in filter_targets(&package.targets) {
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
                subcomponents.len(), // numbers the components
            );

            // record target kinds now that we have the bom_ref
            target_kinds.insert(bom_ref.clone(), tgt.kind.clone());

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
        top_component.components = Some(Components(subcomponents));
        (top_component, TargetKinds(target_kinds))
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
        let mut licenses = vec![];

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
                Ok(expression) => licenses.push(LicenseChoice::Expression(expression)),
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
                    licenses.push(LicenseChoice::License(License::named_license(license)))
                }
            }
        }

        // Check for license file.
        // It is possible to specify both a named license and a license file in Cargo.toml.
        // If that happens, we encode both.
        if let Some(license_file) = package.license_file().as_ref() {
            match std::fs::read_to_string(license_file.as_path()) {
                Ok(content) => {
                    let mut license = License::named_license("Unknown");
                    let encoded_text = AttachedText::new(None, content);
                    license.text = Some(encoded_text);
                    licenses.push(LicenseChoice::License(license));
                }
                Err(error) => {
                    log::warn!(
                        "Failed to read license file '{}' for package {}: {}",
                        package.name,
                        license_file,
                        error
                    );
                }
            }
        }

        if licenses.is_empty() {
            log::trace!(
                "Package {} has no licenses or license file specified",
                package.name
            );
            return None;
        }

        Some(Licenses(licenses))
    }

    fn get_hashes(&self, package: &Package) -> Option<cyclonedx_bom::models::hash::Hashes> {
        match self.crate_hashes.get(&package.id) {
            Some(hash) => Some(cyclonedx_bom::models::hash::Hashes(vec![to_bom_hash(hash)])),
            None => {
                // Log level is set to debug because this is perfectly normal:
                // First, only Rust 1.77 and later has `cargo metadata` output pkgid format,
                // so anything prior to that won't match.
                // Second, only packages coming from registries have a checksum associated with them,
                // while local or git packages do not have a checksum and that too is normal.
                log::debug!(
                    "Hash for package ID {} not found in Cargo.lock",
                    &package.id
                );
                None
            }
        }
    }

    fn create_metadata(
        &self,
        package: &Package,
    ) -> Result<(Metadata, TargetKinds), GeneratorError> {
        let authors = Self::create_authors(package);

        let mut metadata = Metadata::new()?;
        if !authors.is_empty() {
            metadata.authors = Some(authors);
        }

        let (mut component, target_kinds) = self.create_toplevel_component(package);

        component.component_type = Self::get_classification(package);

        metadata.component = Some(component);

        let tool = Tool::new("CycloneDX", "cargo-cyclonedx", env!("CARGO_PKG_VERSION"));

        metadata.tools = Some(Tools::List(vec![tool]));

        use crate::config::Target::*;
        let properties = match self.config.target.as_ref().unwrap() {
            SingleTarget(target) => vec![Property::new("cdx:rustc:sbom:target:triple", target)],
            AllTargets => vec![Property::new("cdx:rustc:sbom:target:all_targets", "true")],
        };
        metadata.properties = Some(Properties(properties));

        Ok((metadata, target_kinds))
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
        static AUTHORS_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(?P<author>[^<]+)\s*(<(?P<email>[^>]+)>)?$")
                .expect("Failed to compile regex.")
        });

        match AUTHORS_REGEX.captures(author) {
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

/// Ignore tests, benches, examples and build scripts.
/// They are not part of the final build artifacts, which is what we are after.
fn filter_targets(
    targets: &[cargo_metadata::Target],
) -> impl Iterator<Item = &cargo_metadata::Target> {
    targets.iter().filter(|tgt| {
        !(tgt.is_bench() || tgt.is_example() || tgt.is_test() || tgt.is_custom_build())
    })
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

fn index_dep_kinds(root: &PackageId, resolve: &ResolveMap) -> DependencyKindMap {
    // cache strongest found dependency kind for every node
    let mut id_to_dep_kind: HashMap<PackageId, PrivateDepKind> = HashMap::new();
    id_to_dep_kind.insert(root.clone(), PrivateDepKind::Runtime);

    type DepNode = (PackageId, PrivateDepKind, PrivateDepKind);

    let mut nodes_to_visit: Vec<DepNode> = vec![];
    nodes_to_visit.push((
        root.clone(),
        PrivateDepKind::Runtime,
        PrivateDepKind::Runtime,
    ));

    let mut visited_nodes: HashSet<DepNode> = HashSet::new();

    // perform a simple iterative DFS over the dependencies,
    // mark child deps with the minimum of parent kind and their own strongest value
    // therefore e.g. mark decendants of build dependencies as build dependencies,
    // as long as they never occur as normal dependency.
    while let Some((pkg_id, node_kind, path_node_kind)) = nodes_to_visit.pop() {
        visited_nodes.insert((pkg_id.clone(), node_kind, path_node_kind));

        let dep_kind_on_previous_visit = id_to_dep_kind.get(&pkg_id);
        // insert/update a nodes dependency kind, when its new or stronger than the previous value
        if dep_kind_on_previous_visit.is_none()
            || path_node_kind > *dep_kind_on_previous_visit.unwrap()
        {
            let _ = id_to_dep_kind.insert(pkg_id.clone(), path_node_kind);
        }

        let node = &resolve[&pkg_id];
        for child_dep in &node.deps {
            for dep_kind in &child_dep.dep_kinds {
                let current_kind = PrivateDepKind::from(&dep_kind.kind);
                let new_path_node_kind = min(current_kind, path_node_kind);

                let dep_node: DepNode = (child_dep.pkg.clone(), current_kind, new_path_node_kind);
                if !visited_nodes.contains(&dep_node) {
                    nodes_to_visit.push(dep_node);
                }
            }
        }
    }

    id_to_dep_kind
        .iter()
        .map(|(x, y)| ((*x).clone(), DependencyKind::from(*y)))
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
    config: &SbomConfig,
) -> (PackageMap, ResolveMap) {
    log::trace!("Adding top-level dependencies to SBOM");

    // Only include packages that have dependency kinds other than "Development"
    let root_node = add_filtered_dependencies(&resolve[root], config);

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
    config: &SbomConfig,
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
                out_resolve.insert(node.id.to_owned(), add_filtered_dependencies(node, config));
                // Queue its dependencies for the next BFS loop iteration
                next_queue.extend(
                    filtered_dependencies(&node.deps, config).map(|dep| &resolve[&dep.pkg]),
                );
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

fn add_filtered_dependencies(node: &Node, config: &SbomConfig) -> Node {
    let mut node = node.clone();
    node.deps = filtered_dependencies(&node.deps, config).cloned().collect();
    node.dependencies = node.deps.iter().map(|d| d.pkg.to_owned()).collect();
    node
}

/// Filters out dependencies only used for development, and not affecting the final binary.
/// These are specified under `[dev-dependencies]` in Cargo.toml.
fn filtered_dependencies<'a>(
    input: &'a [NodeDep],
    config: &'a SbomConfig,
) -> impl Iterator<Item = &'a NodeDep> {
    input.iter().filter(|p| {
        p.dep_kinds.iter().any(|dep| {
            if let Some(true) = config.only_normal_deps {
                dep.kind == DependencyKind::Normal
            } else {
                dep.kind != DependencyKind::Development
            }
        })
    })
}

/// Contains a generated SBOM and context used in its generation
///
/// * `bom` - Generated SBOM
/// * `manifest_path` - Folder containing the `Cargo.toml` manifest
/// * `package_name` - Package from which this SBOM was generated
/// * `sbom_config` - Configuration options used during generation
/// * `target_kinds` - Detailed information on the kinds of targets in `sbom`
#[derive(Debug)]
pub struct GeneratedSbom {
    pub bom: Bom,
    pub manifest_path: PathBuf,
    pub package_name: String,
    pub sbom_config: SbomConfig,
    pub target_kinds: TargetKinds,
}

impl GeneratedSbom {
    /// Writes SBOM to either a JSON or XML file in the same folder as `Cargo.toml` manifest
    pub fn write_to_files(self) -> Result<(), SbomWriterError> {
        match self.sbom_config.describe.unwrap_or_default() {
            Describe::Crate => {
                let path = self.manifest_path.with_file_name(self.filename(None, &[]));
                Self::write_to_file(self.bom, &path, &self.sbom_config)
            }
            pattern @ (Describe::Binaries | Describe::AllCargoTargets) => {
                for (sbom, target_kind) in
                    Self::per_artifact_sboms(&self.bom, &self.target_kinds, pattern)
                {
                    let meta = sbom.metadata.as_ref().unwrap();
                    let name = meta.component.as_ref().unwrap().name.as_ref();
                    let path = self
                        .manifest_path
                        .with_file_name(self.filename(Some(name), &target_kind));
                    Self::write_to_file(sbom, &path, &self.sbom_config)?;
                }
                Ok(())
            }
        }
    }

    fn write_to_file(bom: Bom, path: &Path, config: &SbomConfig) -> Result<(), SbomWriterError> {
        // If running in debug mode, validate that the SBOM is self-consistent and well-formed
        if cfg!(debug_assertions) {
            let result = bom.validate();
            if result.has_errors() {
                panic!(
                    "The generated SBOM failed validation: {:?}",
                    result.errors()
                );
            }
        }

        use cyclonedx_bom::models::bom::SpecVersion::*;
        let spec_version = config.spec_version.unwrap_or(V1_3);

        log::info!("Outputting {}", path.display());
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        match config.format() {
            Format::Json => {
                bom.output_as_json(&mut writer, spec_version)
                    .map_err(SbomWriterError::JsonWriteError)?;
            }
            Format::Xml => {
                bom.output_as_xml(&mut writer, spec_version)
                    .map_err(SbomWriterError::XmlWriteError)?;
            }
        }

        // Flush the writer explicitly to catch and report any I/O errors
        writer.flush()?;

        Ok(())
    }

    /// Returns an iterator over SBOMs and their associated target kinds
    fn per_artifact_sboms<'a>(
        bom: &'a Bom,
        target_kinds: &'a TargetKinds,
        describe: Describe,
    ) -> impl Iterator<Item = (Bom, Vec<String>)> + 'a {
        let meta = bom.metadata.as_ref().unwrap();
        let crate_component = meta.component.as_ref().unwrap();
        let components = crate_component.components.as_ref().unwrap();
        // Narrow down the set of targets for which we emit a SBOM depending on the configuration
        components
            .0
            .iter()
            .filter(move |component| {
                let target_kind = &target_kinds.0[component.bom_ref.as_ref().unwrap()];
                match describe {
                    Describe::Binaries => {
                        // only record binary artifacts
                        // TODO: refactor this to use an enum, coming Soon(tm) to cargo-metadata:
                        // https://github.com/oli-obk/cargo_metadata/pull/258
                        target_kind.contains(&"bin".to_owned())
                            || target_kind.contains(&"cdylib".to_owned())
                    }
                    Describe::AllCargoTargets => true, // pass everything through
                    Describe::Crate => unreachable!(),
                }
            })
            .map(|component| {
                let target_kind = &target_kinds.0[component.bom_ref.as_ref().unwrap()];
                // In the original SBOM the toplevel component describes a crate.
                // We need to change it to describe a specific binary.
                // Most properties apply to the entire package and should be kept;
                // we just need to update the name, type and purl.
                let mut new_bom = bom.clone();
                let metadata = new_bom.metadata.as_mut().unwrap();
                let toplevel_component = metadata.component.as_mut().unwrap();
                toplevel_component.name = component.name.clone();
                toplevel_component.component_type = component.component_type.clone();
                toplevel_component.purl.clone_from(&component.purl);

                (new_bom, target_kind.clone())
            })
    }

    fn filename(&self, binary_name: Option<&str>, target_kind: &[String]) -> String {
        let output_options = self.sbom_config.output_options();
        let describe = self.sbom_config.describe.unwrap_or_default();

        let mut prefix = match describe {
            Describe::Crate => self.package_name.clone(),
            Describe::Binaries => binary_name.unwrap().to_owned(),
            Describe::AllCargoTargets => binary_name.unwrap().to_owned(),
        };
        let mut extension = ".cdx";

        // Handle overridden filename
        match output_options.filename {
            FilenamePattern::CrateName => (), // already handled above, nothing more to do
            FilenamePattern::Custom(name_override) => {
                prefix = name_override.to_string();
                extension = ""; // do not append the extension to allow writing to literally "bom.xml" as per spec
            }
        }

        let target_kind_suffix = if !target_kind.is_empty() {
            debug_assert!(matches!(
                describe,
                Describe::Binaries | Describe::AllCargoTargets
            ));
            format!("_{}", target_kind.join("-"))
        } else {
            "".to_owned()
        };

        let platform_suffix = match output_options.platform_suffix {
            PlatformSuffix::NotIncluded => "".to_owned(),
            PlatformSuffix::Included => {
                extension = ".cdx"; // only a literal "bom.{xml,json}" is allowed not to have .cdx
                let target_string = self.sbom_config.target.as_ref().unwrap();
                format!("_{}", target_string.as_str())
            }
        };

        format!(
            "{}{}{}{}.{}",
            prefix,
            target_kind_suffix,
            platform_suffix,
            extension,
            self.sbom_config.format()
        )
    }
}

/// Locates the corresponding `Cargo.lock` file given the location of `Cargo.toml`.
/// This must be run **after** `cargo metadata` which will generate the `Cargo.lock` file
/// and make sure it's up to date.
fn locate_cargo_lock(manifest_path: &Path) -> Result<PathBuf, std::io::Error> {
    let manifest_path = manifest_path.canonicalize()?;
    let ancestors = manifest_path.as_path().ancestors();

    for path in ancestors {
        let potential_lockfile = path.join("Cargo.lock");
        if potential_lockfile.is_file() {
            return Ok(potential_lockfile);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not find Cargo.lock in any parent directories",
    ))
}

/// Extracts all available package hashes from the provided `Cargo.lock` file
/// and collects them into a HashMap for fast and reasy lookup
fn package_hashes(lockfile: &Lockfile) -> HashMap<cargo_metadata::PackageId, Checksum> {
    let mut result = HashMap::new();
    for pkg in &lockfile.packages {
        if let Some(hash) = pkg.checksum.as_ref() {
            result.insert(cargo_metadata::PackageId { repr: pkgid(pkg) }, hash.clone());
        }
    }
    result
}

/// Returns a Cargo unique identifier for a package.
/// See `cargo help pkgid` for more info.
fn pkgid(pkg: &cargo_lock::Package) -> String {
    match pkg.source.as_ref() {
        Some(source) => format!("{}#{}@{}", source, pkg.name, pkg.version),
        None => format!("{}@{}", pkg.name, pkg.version),
    }
}

/// Converts a checksum from the `cargo-lock` crate format to `cyclonedx-bom` crate format
fn to_bom_hash(hash: &Checksum) -> cyclonedx_bom::models::hash::Hash {
    use cyclonedx_bom::models::hash::{Hash, HashAlgorithm, HashValue};
    // use a match statement to get a compile-time error
    // if/when more variants are added
    match hash {
        Checksum::Sha256(_) => {
            Hash {
                alg: HashAlgorithm::SHA_256,
                // {:x} means "format as lowercase hex"
                content: HashValue(format!("{hash:x}")),
            }
        }
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
