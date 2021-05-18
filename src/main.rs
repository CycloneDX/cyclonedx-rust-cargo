/*
 * This file is part of CycloneDX Rust (Cargo) Plugin.
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
 */

/**
* A special acknowledgement Ossi Herrala from SensorFu for providing a
* starting point in which to develop this plugin. The original project
* this plugin was derived from is sensorfu/cargo-bom v0.3.1 (MIT licensed)
* https://github.com/sensorfu/cargo-bom
*
* MIT License
*
* Copyright (c) 2017-2019 SensorFu Oy
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs::File,
    io::{self, LineWriter},
    iter, path, str,
};

use cargo::{
    core::{
        dependency::DepKind, package::PackageSet, Dependency, Package, PackageId, Resolve,
        Workspace,
    },
    ops,
    util::Config,
    CargoResult,
};
use structopt::StructOpt;
use xml_writer::XmlWriter;

mod author;
mod bom;
mod component;
mod format;
mod traits;

use bom::Bom;
pub(crate) use component::Component;
use format::Format;
pub(crate) use traits::{IsEmpty, ToXml};

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
enum Opts {
    #[structopt(name = "cyclonedx")]
    /// Creates a CycloneDX Software Bill-of-Materials (SBOM) for Rust project
    Bom(Args),
}

#[derive(StructOpt)]
struct Args {
    /// List all dependencies instead of only top level ones
    #[structopt(long = "all", short = "a")]
    all: bool,
    /// Directory for all generated artifacts
    #[structopt(long = "target-dir", value_name = "DIRECTORY", parse(from_os_str))]
    target_dir: Option<path::PathBuf>,
    #[structopt(long = "manifest-path", value_name = "PATH", parse(from_os_str))]
    /// Path to Cargo.toml
    manifest_path: Option<path::PathBuf>,
    /// Output BOM format: json, xml
    #[structopt(long = "format", short = "f", value_name = "FORMAT", default_value)]
    format: Format,
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    /// Use verbose output (-vv very verbose/build.rs output)
    verbose: u32,
    #[structopt(long = "quiet", short = "q")]
    /// No output printed to stdout other than the tree
    quiet: Option<bool>,
    #[structopt(long = "color", value_name = "WHEN")]
    /// Coloring: auto, always, never
    color: Option<String>,
    #[structopt(long = "frozen")]
    /// Require Cargo.lock and cache are up to date
    frozen: bool,
    #[structopt(long = "locked")]
    /// Require Cargo.lock is up to date
    locked: bool,
    #[structopt(long = "offline")]
    /// Run without accessing the network
    offline: bool,
    #[structopt(short = "Z", value_name = "FLAG")]
    /// Unstable (nightly-only) flags to Cargo
    unstable_flags: Vec<String>,
    #[structopt(long = "config", value_name = "KEY=VALUE")]
    /// Override a configuration value
    config_args: Vec<String>,
}

fn main() -> Result<(), Error> {
    let mut config = Config::default()?;
    let Opts::Bom(args) = Opts::from_args();
    real_main(&mut config, args)
}

fn real_main(config: &mut Config, args: Args) -> Result<(), Error> {
    config.configure(
        args.verbose,
        args.quiet.unwrap_or(false),
        args.color.as_deref(),
        args.frozen,
        args.locked,
        args.offline,
        &args.target_dir,
        &args.unstable_flags,
        &args.config_args,
    )?;

    let manifest = args
        .manifest_path
        .unwrap_or_else(|| config.cwd().join("Cargo.toml"));
    let ws = Workspace::new(&manifest, &config)?;
    let members: Vec<Package> = ws.members().cloned().collect();
    let (package_ids, resolve) = ops::resolve_ws(&ws)?;

    let packages = packages_by_id(&package_ids, resolve)?;

    // If the caller provided a crate manifest path, generate a BOM for that
    // crate; otherwise generate one for the entire workspace.
    let bom = match ws.current_opt() {
        Some(pkg) => {
            // Convert dependency specs to packages, recursing if the `--all` flag was passed
            let dependency_ids = if args.all {
                resolve_dependencies_recursive(iter::once(pkg), &package_ids)?
            } else {
                resolve_direct_dependencies(iter::once(pkg), &package_ids)
            };

            Bom::new(pkg).with_dependencies(
                dependency_ids
                    .iter()
                    .filter_map(|pkg_id| packages.get(pkg_id)),
            )
        }
        None => {
            // Convert dependency specs to packages, recursing if the `--all` flag was passed
            let dependencies = if args.all {
                resolve_dependencies_recursive(&members, &package_ids)?
            } else {
                resolve_direct_dependencies(&members, &package_ids)
            };

            // In the full-workspace case, workspace members shouldn't be part of the `components`
            // BOM section, so we filter members out of the dependencies list.
            Bom::default().with_dependencies(
                dependencies
                    .iter()
                    .filter_map(|pkg_id| packages.get(pkg_id))
                    .filter(|pkg| !members.contains(pkg)),
            )
        }
    };

    match args.format {
        Format::Json => {
            serde_json::to_writer_pretty(File::create("bom.json")?, &bom)
                .map_err(anyhow::Error::from)?;
        }
        Format::Xml => {
            let file = File::create("bom.xml")?;
            let file = LineWriter::new(file);
            let mut xml = XmlWriter::new(file);

            bom.to_xml(&mut xml)?;
            xml.close()?;
            xml.flush()?;
            let _actual = xml.into_inner();
        }
    }

    Ok(())
}

fn is_runtime_dependency(dep: &&Dependency) -> bool {
    dep.kind() == DepKind::Normal
}

/// Resolve a set of dependencies to package IDs in the set that will satisfy those dependencies.
fn resolve_direct_dependencies<'a>(
    packages: impl IntoIterator<Item = &'a Package>,
    package_ids: &'a PackageSet,
) -> BTreeSet<PackageId> {
    let mut dependencies = BTreeSet::new();
    for dependency in packages
        .into_iter()
        .flat_map(|pkg| pkg.dependencies())
        .filter(is_runtime_dependency)
    {
        if let Some(dep) = package_ids
            .package_ids()
            .find(|id| dependency.matches_id(*id))
        {
            dependencies.insert(dep);
        }
    }

    dependencies
}

fn resolve_dependencies_recursive<'a>(
    packages: impl IntoIterator<Item = &'a Package>,
    package_ids: &'a PackageSet,
) -> CargoResult<BTreeSet<PackageId>> {
    let mut output = BTreeSet::new();
    let mut packages_to_visit = VecDeque::new();
    packages_to_visit.extend(packages);

    while let Some(pkg) = packages_to_visit.pop_front() {
        for dep in pkg.dependencies().iter().filter(is_runtime_dependency) {
            if let Some(pkg_id) = package_ids.package_ids().find(|id| dep.matches_id(*id)) {
                if output.insert(pkg_id.clone()) {
                    packages_to_visit.push_back(package_ids.get_one(pkg_id)?);
                }
            }
        }
    }

    Ok(output)
}

fn packages_by_id(
    package_ids: &PackageSet<'_>,
    resolve: Resolve,
) -> CargoResult<BTreeMap<PackageId, Package>> {
    Ok(package_ids
        .get_many(resolve.iter())?
        .into_iter()
        .map(|pkg| (pkg.package_id(), pkg.to_owned()))
        .collect())
}

#[derive(Debug)]
struct Error;

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        cargo_exit(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        cargo_exit(anyhow::Error::new(err))
    }
}

fn cargo_exit<E: Into<cargo::CliError>>(err: E) -> ! {
    let mut shell = cargo::core::shell::Shell::new();
    cargo::exit_with_error(err.into(), &mut shell)
}
