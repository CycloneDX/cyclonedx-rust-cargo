# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.6.1 - 2024-06-04

### Added

 - A series of APIs that serialize and deserialize in the format specified with the `SpecVersion` enum ([#725])

### Fixed

 - Fixed a panic when parsing CycloneDX v1.5 from a `serde_json::Value` ([#723])

### Changed

 - Removed `--allow-dirty` flag from the publishing workflow so that the provenance of the package uploaded to crates.io can be established ([#724])

## 0.6.0 - 2024-05-22

### Added

 - Added support for CycloneDX version 1.5, necessitating a number of breaking changes to the API.
 - Added the ability to turn a `NormalizedString` into a `String` without cloning ([#707])
 - Added the ability to view a number of types as a `&str` to reduce the necessary cloning ([#708])
 - Added an ability to parse a `serde_json::Value` into a CycloneDX document ([#705])
 - Added automatic validation of generated JSON against the official CycloneDX schemas ([#653])

### Fixed

 - Added support for `external_references` field on `Tool` introduced in CycloneDX 1.4 but accidentally omitted from the parser ([#709])

### Changed

 - Introduced the `cyclonedx-bom-macros` crate with a proc macro to eliminate copy-pasted code between various spec versions
 - Multiple refactors to make the code simpler and easier to maintain

## 0.5.0 - 2024-02-21

### Changed

 - Added support for CycloneDX v1.4 ([#575]) ([#588])
 - Added a function to deserialize from JSON without knowing the spec version in advance ([#585])
 - Implemented `Display` and `AsRef<str>` for `NormalizedString` ([#550])
 - Turned specification version into an enum ([#583])
 - Made the toplevel `version` field required ([#618])
 - Made `dependencies.dependsOn` field optional ([#616])
 - The output of methods on `Validate` trait is no longer wrapped in a `Result` and can be used directly ([#606]) ([#609])

## 0.4.3 - 2023-11-13

### Added

- Added the ability to parse from and serialize to a `JsonValue` ([#518])
- Added `FromStr` implementations for `Purl` and `Cpe` to enable writing them ([#381])
- Made the field of the `Dependencies` struct public so that it could be read and written ([#504])
- Made the field of the `HashValue` struct public so that it could be read and written ([#519])

### Fixed

- Removed `#[deny(warnings)]` to avoid breakage in production if newer compilers add more warnings ([#530])

[#381]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/381
[#504]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/504
[#518]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/518
[#519]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/519
[#530]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/530
[#550]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/550
[#575]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/575
[#583]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/583
[#585]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/585
[#588]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/588
[#606]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/606
[#609]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/609
[#616]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/616
[#618]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/618
[#653]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/653
[#705]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/705
[#707]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/707
[#708]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/708
[#709]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/709
