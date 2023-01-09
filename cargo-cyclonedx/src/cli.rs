use cargo_cyclonedx::{
    config::{
        CdxExtension, CustomPrefix, Features, IncludedDependencies, LicenseParserOptions,
        OutputOptions, ParseMode, Pattern, Prefix, PrefixError, SbomConfig, Target,
    },
    format::Format,
    platform::host_platform,
};
use clap::{ArgAction, ArgGroup, Parser};
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
#[clap(group(ArgGroup::new("dependencies-group").required(false).args(&["all", "top-level"])))]
#[clap(group(ArgGroup::new("prefix-or-pattern-group").required(false).args(&["output-prefix", "output-pattern"])))]
pub struct Args {
    /// Path to Cargo.toml
    #[clap(long = "manifest-path", value_name = "PATH")]
    pub manifest_path: Option<path::PathBuf>,

    /// Output BOM format: json, xml
    #[clap(long = "format", short = 'f', value_name = "FORMAT")]
    pub format: Option<Format>,

    /// Use verbose output (-vv very verbose/build.rs output)
    #[clap(long = "verbose", short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// No output printed to stdout
    #[clap(long = "quiet", short = 'q')]
    pub quiet: bool,

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

    /// The target to generate the SBOM for, or 'all' for all targets.
    #[clap(
        long = "target",
        long_help = "The target to generate the SBOM for, e.g. 'x86_64-unknown-linux-gnu'.
Use 'all' to include dependencies for all possible targets.
Defaults to the host target, as printed by 'rustc -vV'"
    )]
    pub target: Option<String>,

    /// List all dependencies instead of only top-level ones
    #[clap(long = "all", short = 'a')]
    pub all: bool,

    /// List only top-level dependencies (default)
    #[clap(name = "top-level", long = "top-level")]
    pub top_level: bool,

    /// Prepend file extension with .cdx
    #[clap(long = "output-cdx")]
    pub output_cdx: bool,

    /// Prefix patterns to use for the filename: bom, package
    #[clap(
        name = "output-pattern",
        long = "output-pattern",
        value_name = "PATTERN"
    )]
    pub output_pattern: Option<Pattern>,

    /// Custom prefix string to use for the filename
    #[clap(
        name = "output-prefix",
        long = "output-prefix",
        value_name = "FILENAME_PREFIX"
    )]
    pub output_prefix: Option<String>,

    /// Perform a best-effort interpretation of invalid SPDX license expressions (lax parsing)
    #[clap(long = "license-lax")]
    pub license_lax: bool,

    #[clap(long = "license-accept-name", action=ArgAction::Append)]
    pub license_accept_named: Vec<String>,
}

impl Args {
    pub fn as_config(&self) -> Result<SbomConfig, ArgsError> {
        let included_dependencies = match (self.all, self.top_level) {
            (true, _) => Some(IncludedDependencies::AllDependencies),
            (_, true) => Some(IncludedDependencies::TopLevelDependencies),
            _ => None,
        };

        let prefix = match (self.output_pattern, &self.output_prefix) {
            (Some(pattern), _) => Some(Prefix::Pattern(pattern)),
            (_, Some(prefix)) => {
                let prefix = CustomPrefix::new(prefix)?;
                Some(Prefix::Custom(prefix))
            }
            (_, _) => None,
        };

        let cdx_extension = match self.output_cdx {
            true => Some(CdxExtension::Included),
            false => None,
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

        let output_options = match (cdx_extension, prefix) {
            (Some(cdx_extension), Some(prefix)) => Some(OutputOptions {
                cdx_extension,
                prefix,
            }),
            (Some(cdx_extension), _) => Some(OutputOptions {
                cdx_extension,
                prefix: Prefix::default(),
            }),
            (_, Some(prefix)) => Some(OutputOptions {
                cdx_extension: CdxExtension::default(),
                prefix,
            }),
            (_, _) => None,
        };

        let license_parser = Some(LicenseParserOptions {
            parse_mode: match self.license_lax {
                true => ParseMode::Lax,
                false => ParseMode::Strict,
            },
            accept_named: HashSet::from_iter(self.license_accept_named.clone()),
        });

        Ok(SbomConfig {
            format: self.format,
            included_dependencies,
            output_options,
            features,
            target,
            license_parser,
        })
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ArgsError {
    #[error("Invalid prefix from CLI")]
    CustomPrefixError(#[from] PrefixError),
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

    fn parse_to_config(args: &[&str]) -> SbomConfig {
        Args::parse_from(args.iter()).as_config().unwrap()
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
