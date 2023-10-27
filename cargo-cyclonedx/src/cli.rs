use cargo_cyclonedx::{
    config::{
        CdxExtension, CustomPrefix, IncludedDependencies, OutputOptions, Pattern, Prefix,
        PrefixError, SbomConfig,
    },
    format::Format,
};
use clap::{ArgGroup, Parser};
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

    // The feature selection flags are not mutually exclusive in Cargo,
    // so we keep the same behavior here too.
    /// Activate all available features
    #[clap(long = "all-features")]
    pub all_features: bool,

    /// Do not activate the `default` feature
    #[clap(long = "no-default-features")]
    pub no_default_features: bool,

    /// Space or comma separated list of features to activate
    #[clap(long = "features", short = 'F')]
    pub features: Option<Vec<String>>,

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

        Ok(SbomConfig {
            format: self.format,
            included_dependencies,
            output_options,
            features: None, // TODO
        })
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ArgsError {
    #[error("Invalid prefix from CLI")]
    CustomPrefixError(#[from] PrefixError),
}
