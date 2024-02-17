use cargo_cyclonedx::{
    config::{
        CdxExtension, CustomPrefix, Features, IncludedDependencies, LicenseParserOptions,
        OutputOptions, ParseMode, Pattern, PlatformSuffix, Prefix, PrefixError, SbomConfig, Target,
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
#[clap(version)]
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

    /// Include the target platform of the BOM in the filename. Implies --output-cdx
    #[clap(long = "target-in-filename")]
    pub target_in_filename: bool,

    /// List all dependencies instead of only top-level ones (default)
    #[clap(long = "all", short = 'a')]
    pub all: bool,

    /// List only top-level dependencies
    #[clap(name = "top-level", long = "top-level", conflicts_with = "all")]
    pub top_level: bool,

    /// Prepend file extension with .cdx
    #[clap(long = "output-cdx")]
    pub output_cdx: bool,

    /// Prefix patterns to use for the filename: bom, package, binary, cargo-target
    /// Values other than 'bom' imply --output-cdx
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
        value_name = "FILENAME_PREFIX",
        conflicts_with = "output-pattern"
    )]
    pub output_prefix: Option<String>,

    /// Reject the deprecated '/' separator for licenses, treating 'MIT/Apache-2.0' as an error
    #[clap(long = "license-strict")]
    pub license_strict: bool,

    /// Add license names which will not be warned about when parsing them as a SPDX expression fails
    #[clap(long = "license-accept-named", action=ArgAction::Append)]
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

        let mut cdx_extension = match self.output_cdx {
            true => Some(CdxExtension::Included),
            false => None,
        };

        let platform_suffix = match self.target_in_filename {
            true => PlatformSuffix::Included,
            false => PlatformSuffix::NotIncluded,
        };

        // according to the CycloneDX spec, the file has either be called 'bom.xml'
        // or include the .cdx extension:
        // https://cyclonedx.org/specification/overview/#recognized-file-patterns
        if self.target_in_filename {
            cdx_extension = Some(CdxExtension::Included)
        }
        // Ditto for any kind of prefix or anything not named 'bom'
        if prefix.is_some() {
            cdx_extension = Some(CdxExtension::Included)
        };

        let output_options =
            if cdx_extension.is_none() && prefix.is_none() && !self.target_in_filename {
                None
            } else {
                Some(OutputOptions {
                    cdx_extension: cdx_extension.unwrap_or_default(),
                    prefix: prefix.unwrap_or_default(),
                    platform_suffix,
                })
            };

        let license_parser = Some(LicenseParserOptions {
            mode: match self.license_strict {
                true => ParseMode::Strict,
                false => ParseMode::Lax,
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
