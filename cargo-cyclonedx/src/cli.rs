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

    // the following arguments are deprecated and will be removed in the next major release
    /// Deprecated (will be removed in the next release)
    /// Directory for all generated artifacts
    #[allow(dead_code)]
    #[structopt(long = "target-dir", value_name = "DIRECTORY", parse(from_os_str))]
    target_dir: Option<path::PathBuf>,

    /// Deprecated (will be removed in the next release)
    /// Coloring: auto, always, never
    #[allow(dead_code)]
    #[structopt(long = "color", value_name = "WHEN")]
    color: Option<String>,

    /// Deprecated (will be removed in the next release)
    /// Require Cargo.lock and cache are up to date
    #[allow(dead_code)]
    #[structopt(long = "frozen")]
    frozen: bool,

    /// Deprecated (will be removed in the next release)
    /// Require Cargo.lock is up to date
    #[allow(dead_code)]
    #[structopt(long = "locked")]
    locked: bool,

    /// Deprecated (will be removed in the next release)
    /// Run without accessing the network
    #[allow(dead_code)]
    #[structopt(long = "offline")]
    offline: bool,

    /// Deprecated (will be removed in the next release)
    /// Unstable (nightly-only) flags to Cargo
    #[allow(dead_code)]
    #[structopt(short = "Z", value_name = "FLAG")]
    unstable_flags: Vec<String>,

    /// Deprecated (will be removed in the next release)
    /// Override a configuration value
    #[allow(dead_code)]
    #[structopt(long = "config", value_name = "KEY=VALUE")]
    config_args: Vec<String>,
}
