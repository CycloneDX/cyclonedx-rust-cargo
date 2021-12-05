use crate::format::Format;
use std::path;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Opts {
    #[structopt(name = "cyclonedx")]
    /// Creates a CycloneDX Software Bill-of-Materials (SBOM) for Rust project
    Bom(Args),
}

#[derive(StructOpt)]
pub struct Args {
    /// Path to Cargo.toml
    #[structopt(long = "manifest-path", value_name = "PATH", parse(from_os_str))]
    pub manifest_path: Option<path::PathBuf>,

    /// Output BOM format: json, xml
    #[structopt(long = "format", short = "f", value_name = "FORMAT", default_value)]
    pub format: Format,

    /// Use verbose output (-vv very verbose/build.rs output)
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    pub verbose: u32,

    /// No output printed to stdout other than the tree
    #[structopt(long = "quiet", short = "q")]
    pub quiet: bool,

    /// List all dependencies instead of only top level ones
    #[structopt(long = "all", short = "a")]
    pub all: bool,
}
