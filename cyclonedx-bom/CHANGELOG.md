# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
