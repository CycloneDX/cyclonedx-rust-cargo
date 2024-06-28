[![Build Status](https://github.com/CycloneDX/cyclonedx-rust-cargo/workflows/Rust%20CI/badge.svg)](https://github.com/CycloneDX/cyclonedx-rust-cargo/actions?workflow=Rust+CI)
[![Crates.io](https://img.shields.io/crates/v/cyclonedx-bom.svg)](https://crates.io/crates/cyclonedx-bom)
[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)][License]
[![Website](https://img.shields.io/badge/https://-cyclonedx.org-blue.svg)](https://cyclonedx.org/)
[![Slack Invite](https://img.shields.io/badge/Slack-Join-blue?logo=slack&labelColor=393939)](https://cyclonedx.org/slack/invite)
[![Group Discussion](https://img.shields.io/badge/discussion-groups.io-blue.svg)](https://groups.io/g/CycloneDX)
[![Twitter](https://img.shields.io/twitter/url/http/shields.io.svg?style=social&label=Follow)](https://twitter.com/CycloneDX_Spec)

# CycloneDX Rust (Cargo) Plugin

The CycloneDX module for Rust (Cargo) creates a valid CycloneDX Software Bill of Materials (SBOM) containing an
aggregate of all project dependencies.
OWASP CycloneDX is a full-stack Bill of Materials (BOM) standard providing advanced supply chain capabilities for cyber risk reduction.

## Structure

This repository contains two separate projects:

- [`cyclonedx-bom`](./cyclonedx-bom/README.md) is a Rust library to read and write CycloneDX SBOMs to and from Rust structs.
- [`cargo-cyclonedx`](./cargo-cyclonedx/README.md) is a Rust application, which generates CycloneDX SBOMs for Cargo based Rust projects (it uses `cyclonedx-bom` for that purpose).

## Usage

Execute `cargo-cyclonedx` from within a Rust project directory containing Cargo.toml.

### Installing

```bash
cargo install cargo-cyclonedx
```

### Executing binary

```bash
~/.cargo/bin/cargo-cyclonedx cyclonedx
```

### Executing from cargo

```bash
cargo cyclonedx
```

## Contributing

Contributions are welcome.
See our [`CONTRIBUTING.md`](CONTRIBUTING.md) for details.

### Bug Bounty

We are running a [Bug Bounty](https://yeswehack.com/programs/cyclonedx-rust-cargo-bounty-program) program financed by the [Bug Resilience Program](https://www.sovereigntechfund.de/programs/bug-resilience/faq) of the [Sovereign Tech Fund](https://www.sovereigntechfund.de/). Thank you very much!

## Copyright & License

CycloneDX Rust Cargo is Copyright (c) OWASP Foundation. All Rights Reserved.

Permission to modify and redistribute is granted under the terms of the Apache 2.0 license. See the [LICENSE] file for the full license.

[License]: https://github.com/CycloneDX/cyclonedx-rust-cargo/blob/main/LICENSE
