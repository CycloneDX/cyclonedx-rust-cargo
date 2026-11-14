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
use std::ffi::OsString;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use cargo_cyclonedx::{
    config::{Describe, SbomConfig},
    generator::SbomGenerator,
};
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
    cyclonedx_flag: Option<String>,
}

/// Options of cargo-cyclonedx that `cargo build` does not have
const CYCLONEDX_FLAGS: &[&str] = &[
    "--format",
    "-f",
    "--describe",
    "--target-in-filename",
    "--top-level",
    "--override-filename",
    "--license-strict",
    "--license-accept-named",
    "--spec-version",
    "--no-build-deps",
];

impl CargoArgs {
    fn parse(args: &[OsString]) -> CargoArgs {
        let mut parsed = CargoArgs::default();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            let arg = match arg.to_str() {
                Some(arg) => arg,
                None => continue,
            };
            // Values are given either as `--flag=value` or as the next argument
            let (flag, inline_value) = match arg.split_once('=') {
                Some((flag, value)) => (flag, Some(value)),
                None => (arg, None),
            };
            let mut value = || {
                inline_value
                    .map(OsString::from)
                    .or_else(|| args.next().cloned())
            };
            match flag {
                "--" => break,
                "--offline" => parsed.offline = true,
                "--frozen" => parsed.frozen = true,
                "--manifest-path" => parsed.manifest_path = value().map(PathBuf::from),
                "--config" => parsed
                    .config
                    .extend(value().map(|config| config.to_string_lossy().into_owned())),
                "--message-format" => parsed.message_format = true,
                flag if CYCLONEDX_FLAGS.contains(&flag) && parsed.cyclonedx_flag.is_none() => {
                    parsed.cyclonedx_flag = Some(flag.to_owned())
                }
                _ => (),
            }
        }
        parsed
    }
}

pub fn build_and_generate(args: &Args, config: &SbomConfig, cargo_args: &[OsString]) -> Result<()> {
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
        ("--describe crate", args.describe == Some(Describe::Crate)),
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
    if let Some(flag) = &cargo_flags.cyclonedx_flag {
        bail!("Pass `{flag}` before `build`");
    }

    log::trace!("Running `cargo build` started");
    let artifacts = run_build(args, &cargo_flags, cargo_args)?;
    log::trace!("Running `cargo build` finished");

    log::trace!("Running `cargo metadata` started");
    let metadata = run_metadata(args, &cargo_flags)?;
    log::trace!("Running `cargo metadata` finished");

    // Dependencies get precursor files too, but only the workspace's own artifacts are described
    let mut generated = 0;
    for artifact in artifacts.iter().filter(|artifact| {
        metadata.workspace_members.contains(&artifact.package_id) && gets_sbom(artifact)
    }) {
        for filename in &artifact.filenames {
            // Cargo only copies the artifacts of the requested packages out of `deps/`
            if filename.parent().and_then(|dir| dir.file_name()) == Some("deps") {
                continue;
            }
            if args.describe == Some(Describe::Binaries) && filename.extension() == Some("rlib") {
                continue;
            }
            // Only executable and linkable outputs have a precursor, e.g. `.rmeta` files do not
            let precursor_path = cargo_sbom_precursor::path_for_artifact(filename);
            if !precursor_path.is_file() {
                continue;
            }

            log::trace!("SBOM generation for {} started", filename);
            let precursor = Precursor::from_path(&precursor_path)
                .with_context(|| precursor_path.display().to_string())?;
            let sbom = SbomGenerator::create_sbom_from_precursor(
                &metadata,
                &precursor,
                &artifact.target,
                config,
            )?;
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

/// Proc-macros, build scripts, tests, benches and examples get no SBOM
fn gets_sbom(artifact: &Artifact) -> bool {
    let target = &artifact.target;
    !(artifact.profile.test
        || target.is_proc_macro()
        || target.is_custom_build()
        || target.is_test()
        || target.is_bench()
        || target.is_example())
}

/// Whether the cargo running us supports `-Z sbom`, i.e. whether `build` would work
pub fn sbom_supported() -> bool {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    Command::new(cargo)
        .arg("-Z")
        .arg("sbom")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_build(
    args: &Args,
    cargo_flags: &CargoArgs,
    cargo_args: &[OsString],
) -> Result<Vec<Artifact>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.args([
        "build",
        "-Z",
        "sbom",
        "--message-format=json-render-diagnostics",
    ]);
    // A `--manifest-path` among the Cargo arguments is passed on as is
    if cargo_flags.manifest_path.is_none() {
        if let Some(manifest_path) = &args.manifest_path {
            command.arg("--manifest-path").arg(manifest_path);
        }
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
    if let Some(manifest_path) = cargo_flags
        .manifest_path
        .as_ref()
        .or(args.manifest_path.as_ref())
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
        CargoArgs::parse(&args.iter().map(OsString::from).collect::<Vec<_>>())
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
                cyclonedx_flag: None,
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

    #[test]
    fn detects_cyclonedx_flags() {
        assert_eq!(
            parse(&["--release", "--format", "json"]).cyclonedx_flag,
            Some("--format".to_owned())
        );
        assert_eq!(
            parse(&["--spec-version=1.5"]).cyclonedx_flag,
            Some("--spec-version".to_owned())
        );
        assert_eq!(parse(&["--all", "-F", "foo"]).cyclonedx_flag, None);
        assert_eq!(parse(&["--", "--format", "json"]).cyclonedx_flag, None);
    }
}
