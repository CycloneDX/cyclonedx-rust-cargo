[![Build Status](https://github.com/CycloneDX/cyclonedx-rust-cargo/workflows/Rust%20CI/badge.svg)](https://github.com/CycloneDX/cyclonedx-rust-cargo/actions?workflow=Rust+CI)
[![Crates.io](https://img.shields.io/crates/v/cargo-cyclonedx.svg)](https://crates.io/crates/cargo-cyclonedx)
[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)][License]
[![Website](https://img.shields.io/badge/https://-cyclonedx.org-blue.svg)](https://cyclonedx.org/)
[![Slack Invite](https://img.shields.io/badge/Slack-Join-blue?logo=slack&labelColor=393939)](https://cyclonedx.org/slack/invite)
[![Group Discussion](https://img.shields.io/badge/discussion-groups.io-blue.svg)](https://groups.io/g/CycloneDX)
[![Twitter](https://img.shields.io/twitter/url/http/shields.io.svg?style=social&label=Follow)](https://twitter.com/CycloneDX_Spec)

# `cargo-cyclonedx`

The [CycloneDX](https://cyclonedx.org/) plugin for `cargo` creates a [custom `cargo` subcommand](https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands) that generates a Software Bill-of-Materials (SBOM) file that describes the `cargo` project.

CycloneDX is a lightweight SBOM specification that is easily created, human and machine readable, and simple to parse.

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

* `--format` (`xml` or `json`): Defaults to XML output
* `--all`: Include the transitive dependencies for the project rather than only the top-level dependencies
* `--manifest-path`: where to find the `Cargo.toml` file if other than the default `cargo` location of the current directory
* `--output-cdx`: Include `.cdx` in the filename as described in [the recognized file patterns](https://cyclonedx.org/specification/overview/#recognized-file-patterns)
* `--output-pattern` (`bom` or `package`)
  * `bom`: Outputs a prefix of `bom` for the filename
  * `package`: Outputs a prefix using the `Cargo.toml` package name for the filename
* `--output-prefix`: Outputs a custom prefix for the filename

Notes:

* `--output-cdx`, `--output-pattern`, and `--output-prefix` are a group of options. Passing any of them as arguments will override any `output_options` configurations in `Cargo.toml` files.
* `--output-pattern` and `--output-prefix` cannot be passed as arguments at the same time.

### Manifest Configuration

There are several locations you can set configuration options for convenience. If your project uses a
[Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html), you can set configuration as
toml values under `[workspace.metadata.cyclonedx]` in your workspace manifest. These configuration values will
propagate to your workspace packages unless you override the values either by specifying toml values under
`[package.metadata.cyclonedx]` in your package manifest or with command-line options.

Option                  | Values (*default)   | Description
----------------------- | ------------------- | --------------------------
`included_dependencies` | `top-level`*, `all` | Either only direct (`top-level`) or including transitive (`all`) dependencies
`format`                | `xml`*, `json`      | Output format for the SBOM
`output_options`        | `<defined below>`   | A collection of options for file output

#### Output Options

Option    | Values (*default)   | Description
--------- | ------------------- | --------------------------
`cdx`     | `true` / `false`*   | Determines if `.cdx` is included in the filename as described in [the recognized file patterns](https://cyclonedx.org/specification/overview/#recognized-file-patterns)
`pattern` | `bom`*, `package`   | `bom` outputs `bom`, while `package` outputs the `Cargo.toml` package name as the prefix
`prefix`  | `<filename prefix>` | Outputs a custom value for the prefix

Notes:

* `output_options` values are merged as a single configuration, so a package-level configuration will override the whole workspace-level configuration.
* `pattern` and `prefix` cannot be configured at the same time.

#### Precedence

Configuration options will be merged and applied in the following order from lowest to highest precedence.

1. Defaults
2. Workspace manifest metadata
3. Package manifest metadata
4. Command-line options

#### Example Workspace Configuration

``` toml
[workspace.metadata.cyclonedx]
included_dependencies = "top-level"
format = "xml"
output_options = { cdx = false, prefix = "cyclonedx" }
```

#### Example Package Configuration

You can also specify your configuration in using package metadata in your package manifest.

``` toml
[package.metadata.cyclonedx]
included_dependencies = "all"
format = "json"
output_options = { cdx = true, pattern = "package" }
```

## Copyright & License

CycloneDX Rust Cargo is Copyright (c) OWASP Foundation. All Rights Reserved.

Permission to modify and redistribute is granted under the terms of the Apache 2.0 license. See the [LICENSE] file for the full license.

[License]: https://github.com/CycloneDX/cyclonedx-rust-cargo/blob/main/LICENSE
