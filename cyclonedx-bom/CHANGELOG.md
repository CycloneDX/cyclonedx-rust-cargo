# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.2 - 2023-11-13

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
