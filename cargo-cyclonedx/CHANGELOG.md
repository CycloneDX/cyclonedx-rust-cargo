# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2023-10-23

### Changed

 - `cargo cyclonedx` now generates multiple SBOM files, one per target. This means that in a project with several binaries, each binary gets its own SBOM. This is a far more correct and compliant behavior than the previous default of rolling up everything into one SBOM file. ([#441])
 - Reporting all dependencies as opposed to the toplevel ones is now the default, following the CycloneDX recommendation. ([#441])
 - Messages reporting the use of lax license field parsing are no longer surfaced as errors. They will only be reported at the debug log level from now on. ([#441])
 - The minimum supported Rust version (MSRV) is now 1.70 following an MSRV in our dependency, `clap`. ([#457]) 

### Fixed

 - Fixed an empty component list being returned for toplevel dependencies. This was a regression introduced in 0.3.8. ([#443])
 - Fixed a build error on latest nightly compiler. ([#456])

### Removed

 - The `--output-pattern`, `--output-prefix` and `--output-cdx` flags were removed. They do not make sense for the new behavior of writing multiple files. ([#441]) If you have requirements for customizing the file names and locations that are not covered by the new system, please [file an issue on Github](https://github.com/CycloneDX/cyclonedx-rust-cargo/issues).

[#441]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/#441
[#443]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/#443
[#456]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/#456
[#457]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/#457
