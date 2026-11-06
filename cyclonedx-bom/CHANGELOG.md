# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.9.0 - 2026-11-06

### Added

- Support for CycloneDX specification versions 1.6 and 1.7, including reading
  and writing both JSON and XML. ([#872])
- Added the 1.6 and 1.7 models and fields: BOM `declarations`, `definitions`
  and `citations`; component `manufacturer`, `authors`, `omniborId`, `swhid`,
  `cryptoProperties`, `isExternal`, `versionRange`, `patentAssertions` and
  `tags`; metadata `manufacturer` and `distributionConstraints`; service
  `tags` and `patentAssertions`; dependency `provides`; and license
  `acknowledgement` and expression `details`, `licensing` and `properties`.
  ([#872])
- Added `Bom::parse_json_value_with_version(value, version)` to parse a
  `serde_json::Value` with an explicit `SpecVersion`, supporting versions
  1.3–1.7. `parse_from_json` and `parse_json_value` continue to detect the
  version automatically.
- When writing CycloneDX 1.5 and earlier, `Component::authors` is now converted
  to the legacy `author` field when `author` is not explicitly set. ([#882])

### Changed

- Removed the version-specific `Bom` parsing and output methods. Pass a
  `SpecVersion` as the final argument to `parse_from_json_with_version`,
  `parse_from_xml_with_version`, `output_as_json`, or `output_as_xml` instead.
  ([#879])
- `ComponentEvidence::identity` is now `Option<Vec<Identity>>` because CycloneDX
  1.6 allows multiple identities; version 1.5 allows at most one. ([#872])
- Added public fields to the `Bom`, `Component`, `Occurrence`, `Dependency`,
  `ExternalReference`, `License`, `Metadata` and `Service` models for the 1.6
  and 1.7 fields. Code that constructs these structs with literals must add the
  new fields. ([#872])
- Added `SpecVersion::V1_6`, `SpecVersion::V1_7`,
  `Classification::CryptographicAsset` and the new 1.6/1.7
  `ExternalReferenceType` variants. Exhaustive matches over these enums must be
  updated. ([#872])
- Renamed `ExternalReferenceType::CondifiedInfrastructure` to
  `CodifiedInfrastructure`. It is now written as `codified-infrastructure`,
  the value in the schema, instead of `condified-infrastructure`. ([#886])
- Increased the MSRV (minimum supported Rust version) to 1.86. ([#881])

### Fixed

- Vulnerability analysis is now serialized with the `response` field name
  required by the schema, rather than `responses`. ([#869])
- XML output for formulation tasks now includes the required `taskTypes`, and
  formulation steps and `tools` with components and services are written in the
  element order the XSD requires. ([#889])
- XML element ordering now matches the XSD `xs:sequence` across all spec
  versions, and `proofOfConcept` is now written to XML. ([#872])
- Do not write JSON signatures into XML documents, which produced invalid XML
  that failed validation against the official schemas. ([#872])
- Fixed several XML round-trip bugs that also affected version 1.5: `bom-ref`
  was serialized as `bomRef` on occurrences, licenses and organizations;
  attachments wrote `encoding` twice; license licensing, evidence identity,
  occurrences and callstack were dropped; and annotation subjects were written
  as text. ([#872])
- JSON properties without a `value` are now parsed; a missing `value` defaults
  to an empty string, matching the schema. ([#887])
- Service dataflow XML now writes the description as the `description`
  attribute. ([#885])
- Validation rejects a `bom-ref` on a license or an SPDX license expression for
  spec versions 1.3 and 1.4, which do not define one. Such a BOM previously
  passed `validate_version` and was then written as output that the 1.4 schema
  rejects. ([#890])
- `ConfidenceScore` validation now correctly rejects values outside the
  0.0–1.0 range. ([#872])

## 0.8.1 - 2025-03-19

- Increased the MSRV to 1.85.0 ([#845])
- Updated the `spdx` crate to support additional SPDX expressions ([#845])
- Extended the invalid SPDX expression error to include the expression ([#844])
- Several `Option`al fields are not serialized when they're None ([#847], [#848], [#849])

## 0.8.0 - 2024-11-07

### Added

 - Support parsing of empty XML string tags ([#761])
 - Add spec version to `bom` struct and make validation honor it ([#767])

## 0.7.0 - 2024-08-06

### Changed

 - Made model types `pub` instead of `pub(crate)`, which allows client code to write more fields in SBOMs ([#758])
 - Removed `#[non_exhaustive]` from `SpecVersion`, which was a source of bugs in client code ([#749])
 - Switched from `packageurl` to `purl` crate as the PURL implementation ([#746])
 - Removed JSON schema validation from the public API and moved `jsonschema` to dev-dependencies to combat dependency bloat ([#750])

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
[#723]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/723
[#724]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/724
[#725]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/725
[#746]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/746
[#749]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/749
[#750]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/750
[#758]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/758
[#761]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/761
[#767]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/767
[#844]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/844
[#845]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/845
[#847]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/847
[#848]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/848
[#849]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/849
[#869]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/869
[#872]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/872
[#879]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/879
[#881]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/881
[#882]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/882
[#885]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/885
[#886]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/886
[#887]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/887
[#889]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/889
[#890]: https://github.com/CycloneDX/cyclonedx-rust-cargo/pull/890
