use cargo_cyclonedx::{
    config::{IncludedDependencies, SbomConfig},
    format::Format,
};
use clap::{ArgGroup, Parser};
use std::path;

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
    #[clap(long = "manifest-path", value_name = "PATH", parse(from_os_str))]
    pub manifest_path: Option<path::PathBuf>,

    /// Output BOM format: json, xml
    #[clap(long = "format", short = 'f', value_name = "FORMAT")]
    pub format: Option<Format>,

    /// Use verbose output (-vv very verbose/build.rs output)
    #[clap(long = "verbose", short = 'v', parse(from_occurrences))]
    pub verbose: u32,

    /// No output printed to stdout
    #[clap(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// List all dependencies instead of only top-level ones
    #[clap(long = "all", short = 'a')]
    pub all: bool,

    /// List only top-level dependencies (default)
    #[clap(long = "top-level")]
    pub top_level: bool,
}

impl Args {
    pub fn as_config(&self) -> SbomConfig {
        let included_dependencies = match (self.all, self.top_level) {
            (true, _) => Some(IncludedDependencies::AllDependencies),
            (_, true) => Some(IncludedDependencies::TopLevelDependencies),
            _ => None,
        };

        SbomConfig {
            format: self.format,
            included_dependencies,
            // TODO - Make this real. None while I test TOML parsing
            output_options: None,
        }
    }
}
