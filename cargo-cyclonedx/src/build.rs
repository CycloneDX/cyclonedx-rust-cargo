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
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use cargo_cyclonedx::{config::SbomConfig, generator::SbomGenerator};
use cargo_metadata::{Artifact, Message, Metadata, MetadataCommand};
use cargo_sbom_precursor::Precursor;

use crate::cli::Args;

/// The `cargo build` flags that also have to be passed to `cargo metadata`
#[derive(Debug, Default, PartialEq, Eq)]
struct CargoArgs {
    manifest_path: Option<PathBuf>,
    offline: bool,
    frozen: bool,
    config: Vec<String>,
    message_format: bool,
}

impl CargoArgs {
    fn parse(args: &[String]) -> CargoArgs {
        let mut parsed = CargoArgs::default();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--" => break,
                "--offline" => parsed.offline = true,
                "--frozen" => parsed.frozen = true,
                "--manifest-path" => parsed.manifest_path = args.next().map(PathBuf::from),
                "--config" => parsed.config.extend(args.next().cloned()),
                "--message-format" => parsed.message_format = true,
                other => {
                    if other.starts_with("--message-format=") {
                        parsed.message_format = true;
                    } else if let Some(path) = other.strip_prefix("--manifest-path=") {
                        parsed.manifest_path = Some(PathBuf::from(path));
                    } else if let Some(config) = other.strip_prefix("--config=") {
                        parsed.config.push(config.to_owned());
                    }
                }
            }
        }
        parsed
    }
}

pub fn build_and_generate(args: &Args, config: &SbomConfig, cargo_args: &[String]) -> Result<()> {
    for (flag, set) in [
        ("--all-features", args.all_features),
        ("--no-default-features", args.no_default_features),
        ("--features", !args.features.is_empty()),
        ("--target", args.target.is_some()),
    ] {
        if set {
            bail!("Pass `{flag}` after `build`");
        }
    }
    for (flag, set) in [
        ("--describe", args.describe.is_some()),
        ("--override-filename", args.filename_override.is_some()),
        ("--target-in-filename", args.target_in_filename),
    ] {
        if set {
            bail!("Unsupported with `build`: `{flag}`");
        }
    }
    let cargo_flags = CargoArgs::parse(cargo_args);
    if cargo_flags.message_format {
        bail!("Unsupported with `build`: `--message-format`");
    }

    log::trace!("Running `cargo build` started");
    let artifacts = run_build(args, cargo_args)?;
    log::trace!("Running `cargo build` finished");

    log::trace!("Running `cargo metadata` started");
    let metadata = run_metadata(args, &cargo_flags)?;
    log::trace!("Running `cargo metadata` finished");

    // Dependencies get precursor files too, but only the workspace's own artifacts are described
    let mut generated = 0;
    for artifact in artifacts
        .iter()
        .filter(|artifact| metadata.workspace_members.contains(&artifact.package_id))
    {
        for filename in &artifact.filenames {
            // Only executable and linkable outputs have a precursor, e.g. `.rmeta` files do not
            let precursor_path = cargo_sbom_precursor::path_for_artifact(filename);
            if !precursor_path.is_file() {
                continue;
            }

            log::trace!("SBOM generation for {} started", filename);
            let precursor = Precursor::from_path(&precursor_path)
                .with_context(|| precursor_path.display().to_string())?;
            let sbom =
                SbomGenerator::create_sbom_from_precursor(metadata.clone(), &precursor, config)?;
            log::trace!("SBOM generation for {} finished", filename);

            sbom.write_to(Path::new(&format!("{filename}.cdx.{}", config.format())))?;
            generated += 1;
        }
    }

    if generated == 0 {
        log::warn!("The build produced no SBOM precursor files, so no SBOMs were generated");
    }

    Ok(())
}

fn run_build(args: &Args, cargo_args: &[String]) -> Result<Vec<Artifact>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.args([
        "build",
        "-Z",
        "sbom",
        "--message-format=json-render-diagnostics",
    ]);
    if let Some(manifest_path) = &args.manifest_path {
        command.arg("--manifest-path").arg(manifest_path);
    }
    command.args(cargo_args);
    command.env("CARGO_BUILD_SBOM", "true");
    command.stdout(Stdio::piped());

    let mut child = command.spawn()?;
    let mut artifacts = Vec::new();
    for message in Message::parse_stream(BufReader::new(child.stdout.take().unwrap())) {
        match message? {
            Message::CompilerArtifact(artifact) => artifacts.push(artifact),
            Message::TextLine(line) => println!("{line}"),
            _ => (),
        }
    }

    let status = child.wait()?;
    if !status.success() {
        bail!("`cargo build` failed: {status}");
    }
    Ok(artifacts)
}

fn run_metadata(args: &Args, cargo_flags: &CargoArgs) -> Result<Metadata> {
    let mut cmd = MetadataCommand::new();
    if let Some(manifest_path) = args
        .manifest_path
        .as_ref()
        .or(cargo_flags.manifest_path.as_ref())
    {
        cmd.manifest_path(manifest_path);
    }

    let mut other_options = vec!["--locked".to_owned()];
    if cargo_flags.offline {
        other_options.push("--offline".to_owned());
    }
    if cargo_flags.frozen {
        other_options.push("--frozen".to_owned());
    }
    for config in &cargo_flags.config {
        other_options.push("--config".to_owned());
        other_options.push(config.to_owned());
    }
    cmd.other_options(other_options);

    if args.quiet == 0 {
        cmd.verbose(true);
    }

    Ok(cmd.exec()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> CargoArgs {
        CargoArgs::parse(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn forwards_relevant_flags() {
        let parsed = parse(&[
            "--release",
            "--offline",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--config",
            "net.git-fetch-with-cli=true",
            "--config=build.jobs=2",
            "--manifest-path=foo/Cargo.toml",
        ]);
        assert_eq!(
            parsed,
            CargoArgs {
                manifest_path: Some(PathBuf::from("foo/Cargo.toml")),
                offline: true,
                frozen: false,
                config: vec![
                    "net.git-fetch-with-cli=true".to_owned(),
                    "build.jobs=2".to_owned()
                ],
                message_format: false,
            }
        );
    }

    #[test]
    fn ignores_flags_after_double_dash() {
        let parsed = parse(&["--frozen", "--", "--offline"]);
        assert!(parsed.frozen);
        assert!(!parsed.offline);
    }

    #[test]
    fn detects_message_format() {
        assert!(parse(&["--message-format", "short"]).message_format);
        assert!(parse(&["--message-format=json"]).message_format);
        assert!(!parse(&["--", "--message-format=json"]).message_format);
    }
}
