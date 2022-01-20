use crate::format::Format;
use clap::Parser;
use std::path;

#[derive(Parser, Debug)]
#[clap(bin_name = "cargo")]
pub enum Opts {
    #[clap(name = "cyclonedx")]
    /// Creates a CycloneDX Software Bill-of-Materials (SBOM) for Rust project
    Bom(Args),
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Path to Cargo.toml
    #[clap(long = "manifest-path", value_name = "PATH", parse(from_os_str))]
    pub manifest_path: Option<path::PathBuf>,

    /// Output BOM format: json, xml
    #[clap(long = "format", short = 'f', value_name = "FORMAT", default_value_t)]
    pub format: Format,

    /// Use verbose output (-vv very verbose/build.rs output)
    #[clap(long = "verbose", short = 'v', parse(from_occurrences))]
    pub verbose: u32,

    /// No output printed to stdout other than the tree
    #[clap(long = "quiet", short = 'q')]
    pub quiet: bool,

    /// List all dependencies instead of only top level ones
    #[clap(long = "all", short = 'a')]
    pub all: bool,
}
