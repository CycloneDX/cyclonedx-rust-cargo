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
 * Copyright (c) OWASP Foundation. All Rights Reserved.
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
use cargo_cyclonedx::{
    config::{SbomConfig, Target},
    generator::SbomGenerator,
    GeneratedSbom,
};

use std::{
    io::{self},
    path::{Path, PathBuf},
};

use cargo_metadata::{self, CargoOpt, Metadata};

use anyhow::Result;
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;

mod cli;
use cli::{Args, Opts};

fn generate_sboms(args: &Args) -> Result<Vec<GeneratedSbom>> {
    let config = resolve_config(args)?;
    let manifest_path = locate_manifest(args)?;
    log::debug!("Found the Cargo.toml file at {}", manifest_path.display());

    log::trace!("Running `cargo metadata` started");
    let metadata = get_metadata(args, &manifest_path, &config)?;
    log::trace!("Running `cargo metadata` finished");

    let hakari_id = if config.ignore_hakari == Some(true) {
        detect_hakari_package(&metadata)
    } else {
        None
    };

    log::trace!("SBOM generation started");
    let boms = SbomGenerator::create_sboms(metadata, &config, hakari_id.as_ref())?;
    log::trace!("SBOM generation finished");

    Ok(boms)
}

/// Detects the hakari workspace-hack package by reading `.config/hakari.toml`
/// from the workspace root. Returns `None` if the file doesn't exist, can't be
/// parsed, or the named package isn't in the dependency graph.
fn detect_hakari_package(metadata: &cargo_metadata::Metadata) -> Option<cargo_metadata::PackageId> {
    let hakari_toml_path = metadata.workspace_root.join(".config/hakari.toml");
    let contents = std::fs::read_to_string(hakari_toml_path.as_str()).ok()?;
    let table: toml::Table = contents.parse().ok()?;
    let name = table.get("hakari-package")?.as_str()?;
    let pkg = metadata.packages.iter().find(|p| p.name == name)?;
    log::info!("Detected hakari package '{}'", name);
    Some(pkg.id.clone())
}

fn main() -> anyhow::Result<()> {
    let Opts::Bom(args) = Opts::parse();
    setup_logging(&args)?;

    let boms = generate_sboms(&args)?;

    log::trace!("SBOM output started");
    for bom in boms {
        bom.write_to_files()?;
    }
    log::trace!("SBOM output finished");

    Ok(())
}

fn setup_logging(args: &Args) -> anyhow::Result<()> {
    let mut builder = Builder::new();

    let level_filter = if args.quiet >= 2 {
        LevelFilter::Off
    } else {
        match args.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    };
    builder.filter_level(level_filter);
    builder.parse_default_env(); // allow overriding CLI arguments
    builder.try_init()?;

    Ok(())
}

/// Constructs the resolved configuration based on env vars and CLI arguments
fn resolve_config(args: &Args) -> Result<SbomConfig> {
    let env_config = SbomConfig::from_env();
    let cli_config = args.as_config()?;
    // CLI options take precedence over environment variables:
    // https://doc.rust-lang.org/cargo/reference/config.html#command-line-overrides
    let mut merged = cli_config.merge(&env_config);
    // If the target is not specified on the CLI or via an env var,
    // default to the host platform. This is resolved here only once
    // because calls to rustc can be expensive (hundreds of milliseconds)
    // if it's installed with rustup, because rustup needs to parse a config every time.
    if merged.target.is_none() {
        merged.target = Some(Target::SingleTarget(
            cargo_cyclonedx::platform::host_platform(),
        ));
    }
    Ok(merged)
}

fn locate_manifest(args: &Args) -> Result<PathBuf, io::Error> {
    if let Some(manifest_path) = &args.manifest_path {
        let manifest_path = std::path::absolute(manifest_path)?;
        log::info!(
            "Using manually specified Cargo.toml manifest located at: {}",
            manifest_path.to_string_lossy()
        );
        Ok(manifest_path)
    } else {
        let manifest_path = std::env::current_dir()?.join("Cargo.toml");
        log::info!(
            "Using Cargo.toml manifest located at: {}",
            manifest_path.to_string_lossy()
        );
        Ok(manifest_path)
    }
}

fn get_metadata(
    args: &Args,
    manifest_path: &Path,
    config: &SbomConfig,
) -> anyhow::Result<Metadata> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.manifest_path(manifest_path);

    if let Some(feature_configuration) = config.features.as_ref() {
        if feature_configuration.all_features {
            cmd.features(CargoOpt::AllFeatures);
        }
        if feature_configuration.no_default_features {
            cmd.features(CargoOpt::NoDefaultFeatures);
        }
        if !feature_configuration.features.is_empty() {
            cmd.features(CargoOpt::SomeFeatures(
                feature_configuration.features.clone(),
            ));
        }
    }

    if args.quiet == 0 {
        // Contrary to the name, this does not enable verbose output.
        // It merely forwards the cargo stdout to our stdout,
        // so that `cargo metadata` can show a progressbar on long-running operations.
        cmd.verbose(true);
    }

    if let Some(Target::SingleTarget(target)) = config.target.as_ref() {
        cmd.other_options(vec!["--filter-platform".to_owned(), target.to_owned()]);
    }

    Ok(cmd.exec()?)
}

#[cfg(test)]
mod tests {
    use cyclonedx_bom::prelude::NormalizedString;

    #[test]
    fn parse_toml_only_normal() {
        use crate::cli;
        use crate::generate_sboms;
        use clap::Parser;
        use cyclonedx_bom::models::component::Scope;
        use std::path::PathBuf;

        let mut test_cargo_toml = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_cargo_toml.push("tests/fixtures/build_then_runtime_dep/Cargo.toml");

        let path_arg = &format!("--manifest-path={}", test_cargo_toml.display());
        let args = ["cyclonedx", path_arg, "--no-build-deps"];
        let args_parsed = cli::Args::parse_from(args.iter());

        let sboms = generate_sboms(&args_parsed).unwrap();

        let components = sboms[0].bom.components.as_ref().unwrap();
        assert!(components
            .0
            .iter()
            .all(|f| f.scope == Some(Scope::Required)));
    }

    #[test]
    fn parse_toml_with_excluded() {
        use crate::cli;
        use crate::generate_sboms;
        use clap::Parser;
        use cyclonedx_bom::models::component::Scope;
        use std::path::PathBuf;

        let mut test_cargo_toml = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_cargo_toml.push("tests/fixtures/build_then_runtime_dep/Cargo.toml");

        let path_arg = &format!("--manifest-path={}", test_cargo_toml.display());
        let args = ["cyclonedx", path_arg];
        let args_parsed = cli::Args::parse_from(args.iter());

        let sboms = generate_sboms(&args_parsed).unwrap();

        // build_dep is a build dependency -> excluded
        // runtime_dep_of_build_dep is a dependency of a build dependency -> excluded
        let components = sboms[0].bom.components.as_ref().unwrap();
        assert!(components
            .0
            .iter()
            .all(|c| c.name != NormalizedString::new("build_dep")
                || c.scope == Some(Scope::Excluded)));
        assert!(components.0.iter().all(|c| c.name
            != NormalizedString::new("runtime_dep_of_build_dep")
            || c.scope == Some(Scope::Excluded)));
    }

    /// Without --ignore-hakari, the workspace-hack and proptest (promoted
    /// from dev-dep to normal dep by hakari) both appear in the SBOM.
    #[test]
    fn hakari_present_without_flag() {
        use crate::cli;
        use crate::generate_sboms;
        use clap::Parser;
        use std::path::PathBuf;

        let mut test_cargo_toml = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_cargo_toml.push("tests/fixtures/with_hakari/Cargo.toml");

        let path_arg = &format!("--manifest-path={}", test_cargo_toml.display());
        let args = ["cyclonedx", path_arg];
        let args_parsed = cli::Args::parse_from(args.iter());

        let sboms = generate_sboms(&args_parsed).unwrap();

        let app_sbom = sboms
            .iter()
            .find(|s| s.package_name == "app")
            .expect("should have an SBOM for app");

        let components = app_sbom.bom.components.as_ref().unwrap();
        let names: Vec<String> = components.0.iter().map(|c| c.name.to_string()).collect();

        assert!(
            names.contains(&"my-workspace-hack".to_string()),
            "workspace-hack should be present without --ignore-hakari, got: {:?}",
            names,
        );
        assert!(
            names.contains(&"proptest".to_string()),
            "proptest should be present (promoted by hakari), got: {:?}",
            names,
        );
    }

    /// With --ignore-hakari, the workspace-hack and deps only reachable
    /// through it (like proptest, promoted from dev-dep) are pruned.
    /// Shared deps reachable through real paths survive.
    #[test]
    fn hakari_pruned_with_flag() {
        use crate::cli;
        use crate::generate_sboms;
        use clap::Parser;
        use std::path::PathBuf;

        let mut test_cargo_toml = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_cargo_toml.push("tests/fixtures/with_hakari/Cargo.toml");

        let path_arg = &format!("--manifest-path={}", test_cargo_toml.display());
        let args = ["cyclonedx", path_arg, "--ignore-hakari"];
        let args_parsed = cli::Args::parse_from(args.iter());

        let sboms = generate_sboms(&args_parsed).unwrap();

        // The workspace-hack crate itself should not get its own SBOM
        assert!(
            !sboms.iter().any(|s| s.package_name == "my-workspace-hack"),
            "workspace-hack should not get its own SBOM with --ignore-hakari",
        );

        let app_sbom = sboms
            .iter()
            .find(|s| s.package_name == "app")
            .expect("should have an SBOM for app");

        let components = app_sbom.bom.components.as_ref().unwrap();
        let names: Vec<String> = components.0.iter().map(|c| c.name.to_string()).collect();

        // Workspace-hack and proptest (only reachable via hack) are pruned
        assert!(
            !names.contains(&"my-workspace-hack".to_string()),
            "workspace-hack should be pruned, got: {:?}",
            names,
        );
        assert!(
            !names.contains(&"proptest".to_string()),
            "proptest should be pruned (only reachable via hack), got: {:?}",
            names,
        );

        // Shared deps reachable through real paths survive
        assert!(
            names.contains(&"real_dep".to_string()),
            "real_dep should survive, got: {:?}",
            names,
        );
        assert!(
            names.contains(&"regex".to_string()),
            "regex should survive (shared dep), got: {:?}",
            names,
        );

        // Dependency graph: no refs or edges mention pruned packages
        let deps = app_sbom.bom.dependencies.as_ref().unwrap();
        let pruned = ["my-workspace-hack", "proptest"];
        for dep in &deps.0 {
            for name in &pruned {
                assert!(
                    !dep.dependency_ref.contains(name),
                    "dependency_ref should not reference {}: {}",
                    name,
                    dep.dependency_ref,
                );
                for child in &dep.dependencies {
                    assert!(
                        !child.contains(name),
                        "dependsOn should not reference {}: {}",
                        name,
                        child,
                    );
                }
            }
        }
    }
}
