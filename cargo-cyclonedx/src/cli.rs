use cargo_cyclonedx::{
    config::{
        Describe, Features, FilenameOverride, FilenameOverrideError, FilenamePattern,
        IncludedDependencies, LicenseParserOptions, OutputOptions, ParseMode, PlatformSuffix,
        SbomConfig, Target,
    },
    format::Format,
    platform::host_platform,
};
use clap::{ArgAction, ArgGroup, Parser};
use cyclonedx_bom::models::bom::SpecVersion;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::path;
use thiserror::Error;

#[derive(Parser, Debug)]
#[clap(bin_name = "cargo")]
pub enum Opts {
    #[clap(name = "cyclonedx")]
    /// Creates a CycloneDX Software Bill-of-Materials (SBOM) for Rust project
    Bom(Args),
}

#[derive(Parser, Debug)]
#[clap(version)]
#[clap(group(ArgGroup::new("dependencies-group").required(false).args(&["all", "top-level"])))]
pub struct Args {
    /// Path to Cargo.toml
    #[clap(long = "manifest-path", value_name = "PATH", value_hint = clap::ValueHint::FilePath)]
    pub manifest_path: Option<path::PathBuf>,

    /// Output BOM format: json, xml
    #[clap(long = "format", short = 'f', value_name = "FORMAT")]
    pub format: Option<Format>,

    // the ValueEnum derive provides ample help text
    #[clap(long = "describe")]
    pub describe: Option<Describe>,

    /// Use verbose output (-vv for debug logging, -vvv for tracing)
    #[clap(long = "verbose", short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Disable progress reports (-qq to suppress warnings)
    #[clap(long = "quiet", short = 'q', action = clap::ArgAction::Count)]
    pub quiet: u8,

    // `--all-features`, `--no-default-features` and `--features`
    // are not mutually exclusive in Cargo, so we keep the same behavior here too.
    /// Activate all available features
    #[clap(long = "all-features")]
    pub all_features: bool,

    /// Do not activate the `default` feature
    #[clap(long = "no-default-features")]
    pub no_default_features: bool,

    /// Space or comma separated list of features to activate
    #[clap(long = "features", short = 'F')]
    pub features: Vec<String>,

    /// The target platform to generate the SBOM for, or 'all' for all targets.
    #[clap(
        long = "target",
        long_help = "The target to generate the SBOM for, e.g. 'x86_64-unknown-linux-gnu'.
Use 'all' to include dependencies for all possible targets.
Defaults to the host target, as printed by 'rustc -vV'"
    )]
    pub target: Option<String>,

    /// Include the target platform of the BOM in the filename
    #[clap(long = "target-in-filename")]
    pub target_in_filename: bool,

    /// List all dependencies instead of only top-level ones (default)
    #[clap(long = "all", short = 'a')]
    pub all: bool,

    /// List only top-level dependencies
    #[clap(name = "top-level", long = "top-level", conflicts_with = "all")]
    pub top_level: bool,

    /// Custom string to use for the output filename
    #[clap(
        long = "override-filename",
        value_name = "FILENAME",
        conflicts_with = "describe"
    )]
    pub filename_override: Option<String>,

    /// Reject the deprecated '/' separator for licenses, treating 'MIT/Apache-2.0' as an error
    #[clap(long = "license-strict")]
    pub license_strict: bool,

    /// Add license names which will not be warned about when parsing them as a SPDX expression fails
    #[clap(long = "license-accept-named", action=ArgAction::Append)]
    pub license_accept_named: Vec<String>,

    /// The CycloneDX specification version to output: `1.3`, `1.4` or `1.5`. Defaults to 1.3
    #[clap(long = "spec-version")]
    pub spec_version: Option<SpecVersion>,

    /// Do not include build-time dependencies in the SBOM
    #[clap(long = "no-build-deps")]
    pub no_build_deps: bool,
}

impl Args {
    pub fn as_config(&self) -> Result<SbomConfig, ArgsError> {
        let included_dependencies = match (self.all, self.top_level) {
            (true, _) => Some(IncludedDependencies::AllDependencies),
            (_, true) => Some(IncludedDependencies::TopLevelDependencies),
            _ => None,
        };

        let features =
            if !self.all_features && !self.no_default_features && self.features.is_empty() {
                None
            } else {
                let mut feature_list: Vec<String> = Vec::new();
                // Features can be comma- or space-separated for compatibility with Cargo,
                // but only in command-line arguments.
                for comma_separated_features in &self.features {
                    // Feature names themselves never contain commas.
                    for space_separated_features in comma_separated_features.split(',') {
                        for feature in space_separated_features.split(' ') {
                            if !feature.is_empty() {
                                feature_list.push(feature.to_owned());
                            }
                        }
                    }
                }

                Some(Features {
                    all_features: self.all_features,
                    no_default_features: self.no_default_features,
                    features: feature_list,
                })
            };

        let target_string = self.target.clone().unwrap_or_else(host_platform);
        let target = Some(if &target_string == "all" {
            Target::AllTargets
        } else {
            Target::SingleTarget(target_string)
        });

        let platform_suffix = match self.target_in_filename {
            true => PlatformSuffix::Included,
            false => PlatformSuffix::NotIncluded,
        };

        let filename_pattern = match &self.filename_override {
            Some(string) => {
                let name_override = FilenameOverride::new(string)?;
                FilenamePattern::Custom(name_override)
            }
            None => FilenamePattern::CrateName,
        };

        let output_options = Some(OutputOptions {
            filename: filename_pattern,
            platform_suffix,
        });

        let license_parser = Some(LicenseParserOptions {
            mode: match self.license_strict {
                true => ParseMode::Strict,
                false => ParseMode::Lax,
            },
            accept_named: HashSet::from_iter(self.license_accept_named.clone()),
        });

        let describe = self.describe;
        let spec_version = self.spec_version;
        let only_normal_deps = Some(self.no_build_deps);

        Ok(SbomConfig {
            format: self.format,
            included_dependencies,
            output_options,
            features,
            target,
            license_parser,
            describe,
            spec_version,
            only_normal_deps,
        })
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ArgsError {
    #[error("Invalid filename")]
    FilenameOverrideError(#[from] FilenameOverrideError),
}

#[cfg(test)]
pub fn parse_to_config(args: &[&str]) -> SbomConfig {
    Args::parse_from(args.iter()).as_config().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_features() {
        let args = vec!["cyclonedx"];
        let config = parse_to_config(&args);
        assert!(config.features.is_none());

        let args = vec!["cyclonedx", "--features=foo"];
        let config = parse_to_config(&args);
        assert!(contains_feature(&config, "foo"));

        let args = vec!["cyclonedx", "--features=foo", "--features=bar"];
        let config = parse_to_config(&args);
        assert!(contains_feature(&config, "foo"));
        assert!(contains_feature(&config, "bar"));

        let args = vec!["cyclonedx", "--features=foo,bar baz"];
        let config = parse_to_config(&args);
        assert!(contains_feature(&config, "foo"));
        assert!(contains_feature(&config, "bar"));
        assert!(contains_feature(&config, "baz"));

        let args = vec!["cyclonedx", "--features=foo, bar"];
        let config = parse_to_config(&args);
        assert!(contains_feature(&config, "foo"));
        assert!(contains_feature(&config, "bar"));
        assert!(!contains_feature(&config, ""));
    }

    fn contains_feature(config: &SbomConfig, feature: &str) -> bool {
        config
            .features
            .as_ref()
            .unwrap()
            .features
            .contains(&feature.to_owned())
    }
}
