[![Build Status](https://github.com/CycloneDX/cyclonedx-rust-cargo/workflows/Rust%20CI/badge.svg)](https://github.com/CycloneDX/cyclonedx-rust-cargo/actions?workflow=Rust+CI)
[![Crates.io](https://img.shields.io/crates/v/cyclonedx-bom.svg)](https://crates.io/crates/cyclonedx-bom)
[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)][License]
[![Website](https://img.shields.io/badge/https://-cyclonedx.org-blue.svg)](https://cyclonedx.org/)
[![Slack Invite](https://img.shields.io/badge/Slack-Join-blue?logo=slack&labelColor=393939)](https://cyclonedx.org/slack/invite)
[![Group Discussion](https://img.shields.io/badge/discussion-groups.io-blue.svg)](https://groups.io/g/CycloneDX)
[![Twitter](https://img.shields.io/twitter/url/http/shields.io.svg?style=social&label=Follow)](https://twitter.com/CycloneDX_Spec)

# `cyclonedx-bom`

The [CycloneDX](https://cyclonedx.org/) library provides JSON and XML serialization and derserialization of Software Bill-of-Materials (SBOM) files.

CycloneDX is a lightweight SBOM specification that is easily created, human and machine readable, and simple to parse.

The library is intended to enable developers to:

- Construct SBOM documents that conform the CycloneDX specification
- Parse and validate JSON and XML SBOM documents
- Perform modifications to BOM documents (e.g. merging multiple BOMs using a variety of algorithms)

## Usage

### Read and validate an SBOM

```rust
use cyclonedx_bom::prelude::*;

let bom_json = r#"{
  "bomFormat": "CycloneDX",
  "specVersion": "1.3",
  "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
  "version": 1
}"#;
let bom = Bom::parse_from_json_v1_3(bom_json.as_bytes()).expect("Failed to parse BOM");

let validation_result = bom.validate().expect("Failed to validate BOM");
assert_eq!(validation_result, ValidationResult::Passed);
```

### Create and output an SBOM

```rust
use cyclonedx_bom::prelude::*;
use cyclonedx_bom::models::{
    tool::{Tool, Tools},
};

let bom = Bom {
    serial_number: Some(
        UrnUuid::new("urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79".to_string())
            .expect("Failed to create UrnUuid"),
    ),
    metadata: Some(Metadata {
        tools: Some(Tools(vec![Tool {
            name: Some(NormalizedString::new("my_tool")),
            ..Tool::default()
        }])),
        ..Metadata::default()
    }),
    ..Bom::default()
};

let mut output = Vec::<u8>::new();

bom.output_as_json_v1_3(&mut output)
    .expect("Failed to write BOM");
let output = String::from_utf8(output).expect("Failed to read output as a string");
assert_eq!(
    output,
    r#"{
  "bomFormat": "CycloneDX",
  "specVersion": "1.3",
  "version": 1,
  "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
  "metadata": {
    "tools": [
      {
        "name": "my_tool"
      }
    ]
  }
}"#
);
```

## Verification and Validation

see [README](./tests/README.md) for details.

## Contributing

see [CONTRIBUTING](./CONTRIBUTING.md) for details.

## Copyright & License

CycloneDX Rust Cargo is Copyright (c) OWASP Foundation. All Rights Reserved.

Permission to modify and redistribute is granted under the terms of the Apache 2.0 license. See the [LICENSE] file for the full license.

[License]: https://github.com/CycloneDX/cyclonedx-rust-cargo/blob/main/LICENSE
