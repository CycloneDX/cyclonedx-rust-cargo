# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.5.0 - 2024-03-01

### Added

 - Added `--describe` flag to control what is described by the SBOM: the crate as a whole in a single SBOM file, a separate SBOM file for every binary (executable or cdylib), or a separate SBOM file for every [Cargo target](https://doc.rust-lang.org/cargo/reference/cargo-targets.html) including rlibs and other kinds that do not produce executable artifacts. ([#619]) ([#630]) ([#634])
 - Added an option to output CycloneDX v1.4 with `--spec-version=1.4`. The recorded data are the same between v1.3 and v1.4 outputs. ([#634])
 - When using Rust 1.77 and later, the package hashes for crates originating from package registries are now recorded. ([#620])

### Changed
 - `cargo cyclonedx` now displays the progress information for Cargo operations, such as updating the crates.io index. This can be suppressed with the `-q` flag. `-qq` is now required to suppress warnings. ([#634])
 - Introduced the `--override-filename` flag replacing the `--output-prefix` and `--output-pattern` flags. ([#634])
 - The `.cdx` suffix is now always added to the end of the filename in all cases when the filename isn't manually overridden, to comply with the CycloneDX specification. The `--output-cdx` flag that previously controlled this behavior is removed. ([#602]) ([#634])

## 0.4.1 - 2023-11-23

### Added

- Added the `--version` flag to print the version ([#561])

### Changed

- Print a more helpful message on errors related to crate types ([#553])

### Fixed

- The type for procedural macros and certain other exotic crate types is now correctly recorded as library type ([#554])

## 0.4.0 - 2023-11-13

### Added

- The SBOM now includes the dependency tree ([#504])
- It is now possible to configure Cargo features via `--all-features`, `--no-default-features` and `--features=...` flags. Previously the tool always recorded the default configuration. ([#512])
- It is now possible to select the target for which the SBOM will be recorded with the `--target=...` flag. ([#513])
- Added a flag to include the target platform into the SBOM filename ([#535])
- Added finer-grained controls for the behavior of the license parsing. ([#363])
- Record the binary targets of the top-level package into the SBOM ([#533])

### Changed

- Switched to `cargo metadata` as the backend. This brings lower binary size, shorter compile times, faster crate downloads and lower maintenance burden. ([#496])
- Increased MSRV to match the dependencies, switched to 2021 edition ([#457])
- Dev-dependencies that cannot affect the final executables are no longer recorded ([#498]) ([#525])
- Default to recording the dependencies used on the current platform when writing the SBOM. The old behavior of recording the dependencies from all possible targets is still available via `--target=all`. ([#513])
- Default to recording all dependencies instead of only the top-level ones. This is the behavior recommended by the standard. ([#526])

### Fixed

- Fix recording of top-level dependencies ([#443])
- Correctly record overridden top-level dependencies ([#365])
- Use buffered I/O when writing SBOM files for much higher performance ([#497])
- Fixed incorrect recording of dependencies. Previously all dependencies of the workspace would be included instead of the dependencies of the specified package ([#498])
- Use actually unique identifiers in `bom-ref` field ([#503])
- Passing mutually exclusive command-line flags now results in a properly reported error, instead of being silently ignored. ([#526]) ([#535])
- Present non-fatal issues as warnings rather than errors ([#542])
- Removed `#[deny(warnings)]` to avoid breakage in production if newer compilers add more warnings ([#496])
- Encode the origin of the package into the PURL instead of pretending that everything always comes from crates.io ([#523])

### Removed

- Removed the configuration through `Cargo.toml`. This is a fundamentally wrong place to record it. ([#520]) If you have use cases for a configuration file, please let us know by [filing an issue](https://github.com/CycloneDX/cyclonedx-rust-cargo/issues).

[#363]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/363
[#365]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/365
[#443]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/443
[#457]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/457
[#496]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/496
[#497]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/497
[#498]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/498
[#503]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/503
[#504]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/504
[#512]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/512
[#513]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/513
[#520]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/520
[#523]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/523
[#525]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/525
[#526]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/526
[#533]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/533
[#535]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/535
[#542]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/542
[#553]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/553
[#554]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/554
[#561]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/561
[#602]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/602
[#619]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/619
[#620]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/620
[#630]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/630
[#634]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/634
