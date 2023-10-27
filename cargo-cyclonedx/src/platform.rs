use std::{
    ffi::{OsStr, OsString},
    io::BufRead,
    process::Command,
};

/// Returns the host target triple, e.g. `x86_64-unknown-linux-gnu`
pub fn host_platform() -> String {
    rustc_host_target_triple(&rustc_location())
}

pub fn rustc_location() -> OsString {
    // Honor the environment variable used by Cargo to locate `rustc`:
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html
    std::env::var_os("RUSTC").unwrap_or("rustc".into())
}

/// Returns the default target triple for the rustc we're running
pub fn rustc_host_target_triple(rustc_path: &OsStr) -> String {
    // While this feels somewhat insane, this is how `cargo` determines the host platform
    Command::new(rustc_path)
        .arg("-vV")
        .output()
        .expect("Failed to invoke rustc! Is it in your $PATH?")
        .stdout
        .lines()
        .map(|l| l.unwrap())
        .find(|l| l.starts_with("host: "))
        .map(|l| l[6..].to_string())
        .expect("Failed to parse rustc output to determine the current platform. Please report this bug!")
}
