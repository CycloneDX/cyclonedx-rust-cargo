/*
 * This file is part of CycloneDX Rust Cargo.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#![deny(clippy::all)]
#![deny(warnings)]

//! The `cyclonedx-bom` library provides JSON and XML serialization and derserialization of Software Bill-of-Materials (SBOM) files.
//!
//! [CycloneDX](https://cyclonedx.org/) is a lightweight SBOM specification that is easily created, human and machine readable, and simple to parse.
//!
//! The library is intended to enable developers to:
//!
//! - Construct SBOM documents that conform the CycloneDX specification
//! - Parse and validate JSON and XML SBOM documents
//! - Perform modifications to BOM documents (e.g. merging multiple BOMs using a variety of algorithms)
//!
//! ## Read and validate an SBOM
//!
//! Given an input implements [std::io::Read], parse the value into a [`Bom`](crate::models::bom::Bom) and then use the [`Validate`](crate::validation::Validate) trait to ensure that it is a valid BOM.
//!
//! ```rust
//! use cyclonedx_bom::models::bom::Bom;
//! use cyclonedx_bom::validation::{Validate, ValidationResult};
//!
//! let bom_json = r#"{
//!   "bomFormat": "CycloneDX",
//!   "specVersion": "1.3",
//!   "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
//!   "version": 1
//! }"#;
//! let bom = Bom::parse_from_json_v1_3(bom_json.as_bytes()).expect("Failed to parse BOM");
//!
//! let validation_result = bom.validate().expect("Failed to validate BOM");
//! assert_eq!(validation_result, ValidationResult::Passed);
//! ```
//!
//! ## Create and output an SBOM
//!
//! Given an output implements [std::io::Write], output the [`Bom`](crate::models::bom::Bom) as JSON.
//!
//! ```rust
//! use cyclonedx_bom::external_models::normalized_string::NormalizedString;
//! use cyclonedx_bom::models::{
//!     bom::{Bom, UrnUuid},
//!     metadata::Metadata,
//!     tool::{Tool, Tools},
//! };
//!
//! let bom = Bom {
//!     serial_number: Some(
//!         UrnUuid::new("urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79".to_string())
//!             .expect("Failed to create UrnUuid"),
//!     ),
//!     metadata: Some(Metadata {
//!         tools: Some(Tools(vec![Tool {
//!             name: Some(NormalizedString::new("my_tool")),
//!             ..Tool::default()
//!         }])),
//!         ..Metadata::default()
//!     }),
//!     ..Bom::default()
//! };
//!
//! let mut output = Vec::<u8>::new();
//!
//! bom.output_as_json_v1_3(&mut output)
//!     .expect("Failed to write BOM");
//! let output = String::from_utf8(output).expect("Failed to read output as a string");
//! assert_eq!(
//!     output,
//!     r#"{
//!   "bomFormat": "CycloneDX",
//!   "specVersion": "1.3",
//!   "version": 1,
//!   "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
//!   "metadata": {
//!     "tools": [
//!       {
//!         "name": "my_tool"
//!       }
//!     ]
//!   }
//! }"#
//! );
//! ```

pub mod errors;
pub mod external_models;
pub mod models;
pub mod validation;

mod specs;
mod utilities;
mod xml;
