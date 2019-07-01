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
use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::path;
use std::str;

use cargo::core::dependency::Kind;
use cargo::core::package::PackageSet;
use cargo::core::{Package, Resolve, Workspace};
use cargo::ops;
use cargo::util::Config;
use cargo::CargoResult;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
enum Opts {
    #[structopt(
    name = "bom",
    raw(
    setting = "AppSettings::UnifiedHelpMessage",
    setting = "AppSettings::DeriveDisplayOrder",
    setting = "AppSettings::DontCollapseArgsInUsage"
    )
    )]
    /// Display a Bill-of-Materials for Rust project
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
    #[structopt(short = "Z", value_name = "FLAG")]
    /// Unstable (nightly-only) flags to Cargo
    unstable_flags: Vec<String>,
}

fn main() -> Result<(), Error> {
    let mut config = Config::default()?;
    let Opts::Bom(args) = Opts::from_args();
    real_main(&mut config, args)
}

fn real_main(config: &mut Config, args: Args) -> Result<(), Error> {
    config.configure(
        args.verbose,
        args.quiet,
        &args.color,
        args.frozen,
        args.locked,
        &args.target_dir,
        &args.unstable_flags,
    )?;

    let manifest = args
        .manifest_path
        .unwrap_or_else(|| config.cwd().join("Cargo.toml"));
    let ws = Workspace::new(&manifest, &config)?;
    let members: Vec<Package> = ws.members().cloned().collect();
    let (package_ids, resolve) = ops::resolve_ws(&ws)?;

    let dependencies = if args.all {
        all_dependencies(&members, package_ids, resolve)?
    } else {
        top_level_dependencies(&members, package_ids)?
    };

    let mut packages = BTreeSet::new();
    for package in &dependencies {
        let name = package.name().to_owned();
        let version = format!("{}", package.version());
        let licenses = format!("{}", package_licenses(package));
        let license_files = package_license_files(package)?;
        packages.insert((name, version, licenses, license_files));
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    {
        let mut tw = tabwriter::TabWriter::new(&mut out);
        writeln!(tw, "Name\t| Version\t| Licenses")?;
        writeln!(tw, "----\t| -------\t| --------")?;
        for (name, version, licenses, _) in &packages {
            writeln!(tw, "{}\t| {}\t| {}", &name, &version, &licenses)?;
        }

        // TabWriter flush() makes the actual write to stdout.
        tw.flush()?;
    }

    println!();

    for (name, version, _, license_files) in packages {
        if license_files.is_empty() {
            continue;
        }

        println!("-----BEGIN {} {} LICENSES-----", name, version);

        let mut buf = Vec::new();
        let mut licenses_to_print = license_files.len();
        for file in license_files {
            let mut fs = std::fs::File::open(file)?;
            fs.read_to_end(&mut buf)?;
            out.write_all(&buf)?;
            buf.clear();
            if licenses_to_print > 1 {
                out.write_all(b"\n-----NEXT LICENSE-----\n")?;
                licenses_to_print -= 1;
            }
        }

        println!("-----END {} {} LICENSES-----", name, version);
        println!();
    }

    Ok(())
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
                Kind::Normal => (),
                Kind::Build | Kind::Development => continue,
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

fn package_licenses(package: &Package) -> Licenses<'_> {
    let metadata = package.manifest().metadata();

    if let Some(ref license_str) = metadata.license {
        let licenses: BTreeSet<&str> = license_str
            .split("OR")
            .map(|s| s.split("AND"))
            .flatten()
            .map(|s| s.split('/'))
            .flatten()
            .map(str::trim)
            .collect();
        return Licenses::Licenses(licenses);
    }

    if let Some(ref license_file) = metadata.license_file {
        return Licenses::File(license_file);
    }

    Licenses::Missing
}

static LICENCE_FILE_NAMES: &[&str] = &["LICENSE", "UNLICENSE"];

fn package_license_files(package: &Package) -> io::Result<Vec<path::PathBuf>> {
    let mut result = Vec::new();

    if let Some(path) = package.manifest_path().parent() {
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                if let Ok(name) = entry.file_name().into_string() {
                    for license_name in LICENCE_FILE_NAMES {
                        if name.starts_with(license_name) {
                            result.push(entry.path())
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

#[derive(Debug)]
enum Licenses<'a> {
    Licenses(BTreeSet<&'a str>),
    File(&'a str),
    Missing,
}

impl<'a> fmt::Display for Licenses<'a> {
    fn fmt(self: &Self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            Licenses::File(_) => write!(f, "Specified in license file"),
            Licenses::Missing => write!(f, "Missing"),
            Licenses::Licenses(ref lic_names) => {
                let lics: Vec<String> = lic_names.iter().map(|s| String::from(*s)).collect();
                write!(f, "{}", lics.join(", "))
            }
        }
    }
}

#[derive(Debug)]
struct Error;

impl From<failure::Error> for Error {
    fn from(err: failure::Error) -> Self {
        cargo_exit(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        let failure = failure::Error::from_boxed_compat(Box::new(err));
        cargo_exit(failure)
    }
}

fn cargo_exit<E: Into<cargo::CliError>>(err: E) -> ! {
    let mut shell = cargo::core::shell::Shell::new();
    cargo::exit_with_error(err.into(), &mut shell)
}