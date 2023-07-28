use cargo_cyclonedx::{
    config::{
        CdxExtension, IncludedDependencies, OutputOptions,
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

    /// List all dependencies instead of only top-level ones
    #[clap(long = "all", short = 'a')]
    pub all: bool,

    /// List only top-level dependencies (default)
    #[clap(name = "top-level", long = "top-level")]
    pub top_level: bool,

    /// Prepend file extension with .cdx
    #[clap(long = "output-cdx")]
    pub output_cdx: bool,

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

        let cdx_extension = match self.output_cdx {
            true => Some(CdxExtension::Included),
            false => None,
        };

        let output_options = match cdx_extension {
            Some(cdx_extension) => Some(OutputOptions {
                cdx_extension,
            }),
            _ => Some(OutputOptions {
                cdx_extension: CdxExtension::default(),
            }),
        };

        Ok(SbomConfig {
            format: self.format,
            included_dependencies,
            output_options,
        })
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ArgsError {
    #[error("Invalid prefix from CLI")]
    CustomPrefixError(#[from] PrefixError),
}
