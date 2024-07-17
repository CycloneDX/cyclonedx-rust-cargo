[![Build Status](https://github.com/CycloneDX/cyclonedx-rust-cargo/workflows/Rust%20CI/badge.svg)](https://github.com/CycloneDX/cyclonedx-rust-cargo/actions?workflow=Rust+CI)
[![Crates.io](https://img.shields.io/crates/v/cargo-cyclonedx.svg)](https://crates.io/crates/cargo-cyclonedx)
[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)][License]
[![Website](https://img.shields.io/badge/https://-cyclonedx.org-blue.svg)](https://cyclonedx.org/)
[![Slack Invite](https://img.shields.io/badge/Slack-Join-blue?logo=slack&labelColor=393939)](https://cyclonedx.org/slack/invite)
[![Group Discussion](https://img.shields.io/badge/discussion-groups.io-blue.svg)](https://groups.io/g/CycloneDX)
[![Twitter](https://img.shields.io/twitter/url/http/shields.io.svg?style=social&label=Follow)](https://twitter.com/CycloneDX_Spec)

# `cargo-cyclonedx`

This [CycloneDX](https://cyclonedx.org/) plugin for `cargo` creates a [custom `cargo` subcommand](https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands) that generates a Software Bill of Materials (SBOM) file that describes the `cargo` project.

CycloneDX is a lightweight SBOM specification that is easily created, human and machine-readable, and simple to parse.

## Usage

### Installing

``` bash
cargo install cargo-cyclonedx
```

### Executing from `cargo`

``` bash
cargo cyclonedx
```

This produces a `bom.xml` file adjacent to every `Cargo.toml` file that exists in the workspace.

#### Command-line options

```
      --manifest-path <PATH>
          Path to Cargo.toml

  -f, --format <FORMAT>
          Output BOM format: json, xml

      --describe <DESCRIBE>
          Possible values:
          - crate:             Describe the entire crate in a single SBOM file, with Cargo targets as subcomponents. (default)
          - binaries:          A separate SBOM is emitted for each binary (bin, cdylib) while all other targets are ignored
          - all-cargo-targets: A separate SBOM is emitted for each Cargo target, including things that aren't directly executable (e.g rlib)

  -v, --verbose...
          Use verbose output (-vv for debug logging, -vvv for tracing)

  -q, --quiet...
          Disable progress reports (-qq to suppress warnings)

      --all-features
          Activate all available features

      --no-default-features
          Do not activate the `default` feature

  -F, --features <FEATURES>
          Space or comma separated list of features to activate

      --target <TARGET>
          The target to generate the SBOM for, e.g. 'x86_64-unknown-linux-gnu'.
          Use 'all' to include dependencies for all possible targets.
          Defaults to the host target, as printed by 'rustc -vV'

      --target-in-filename
          Include the target platform of the BOM in the filename

  -a, --all
          List all dependencies instead of only top-level ones (default)

      --top-level
          List only top-level dependencies

      --override-filename <FILENAME>
          Custom string to use for the output filename

      --license-strict
          Reject the deprecated '/' separator for licenses, treating 'MIT/Apache-2.0' as an error

      --license-accept-named <LICENSE_ACCEPT_NAMED>
          Add license names which will not be warned about when parsing them as a SPDX expression fails

      --spec-version <SPEC_VERSION>
          The CycloneDX specification version to output: `1.3`, `1.4` or `1.5`. Defaults to 1.3

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Differences from other tools

A number of language-independent tools support generating SBOMs for Rust projects. However, they typically rely on parsing the `Cargo.lock` file, which severely limits the information available to them.

By contrast, `cargo cyclonedx` sources data both from `Cargo.lock` and from [`cargo metadata`](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html), which enables a number of features that tools limited to `Cargo.lock` cannot support:

 - Create a SBOM for a particular crate or a particular binary, as opposed to the entire workspace
 - Honor a particular combination of enabled Cargo features, matching your build configuration
 - Omit dev-dependencies, which cannot affect the final executable
 - Record additional fields such as the license for every component

## Contributing

See [CONTRIBUTING](../CONTRIBUTING.md) for details.

### Bug Bounty

We are running a [Bug Bounty](https://yeswehack.com/programs/cyclonedx-rust-cargo-bounty-program) program financed by the [Bug Resilience Program](https://www.sovereigntechfund.de/programs/bug-resilience/faq) of the [Sovereign Tech Fund](https://www.sovereigntechfund.de/). Thank you very much!

## Copyright & License

CycloneDX Rust Cargo is Copyright (c) OWASP Foundation. All Rights Reserved.

Permission to modify and redistribute is granted under the terms of the Apache 2.0 license. See the [LICENSE] file for the full license.

[License]: https://github.com/CycloneDX/cyclonedx-rust-cargo/blob/main/LICENSE
