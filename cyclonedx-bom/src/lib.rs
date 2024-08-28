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

//! The `cyclonedx-bom` library provides JSON and XML serialization and deserialization of Software
//! Bill-of-Materials (SBOM) files.
//!
//! [CycloneDX](https://cyclonedx.org/) is a lightweight SBOM specification that is easily created,
//! human and machine readable, and simple to parse.
//!
//! The library is intended to enable developers to:
//!
//! - Construct SBOM documents that conform the CycloneDX specification
//! - Parse and validate JSON and XML SBOM documents
//! - Perform modifications to BOM documents (e.g. merging multiple BOMs using a variety of
//!   algorithms)
//!
//! ## Read and validate an SBOM
//!
//! Given an input implements [std::io::Read], parse the value into a
//! [`Bom`](crate::models::bom::Bom) and then use the [`Validate`](crate::validation::Validate)
//! trait to ensure that it is a valid BOM.
//!
//! ```rust
//! use cyclonedx_bom::prelude::*;
//!
//! let bom_json = r#"{
//!   "bomFormat": "CycloneDX",
//!   "specVersion": "1.3",
//!   "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
//!   "version": 1
//! }"#;
//! let bom = Bom::parse_from_json_v1_3(bom_json.as_bytes()).expect("Failed to parse BOM");
//!
//! let validation_result = bom.validate();
//! assert!(validation_result.passed());
//! ```
//!
//! ## Create and output an SBOM
//!
//! Given an output implements [std::io::Write], output the [`Bom`](crate::models::bom::Bom) as
//! JSON.
//!
//! ```rust
//! use cyclonedx_bom::prelude::*;
//! use cyclonedx_bom::models::{
//!     tool::{Tool, Tools},
//! };
//!
//! let bom = Bom {
//!     serial_number: Some(
//!         UrnUuid::new("urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79".to_string())
//!             .expect("Failed to create UrnUuid"),
//!     ),
//!     metadata: Some(Metadata {
//!         tools: Some(Tools::List(vec![Tool {
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
//!
//! ## Library design notes
//!
//! ### Correctness
//!
//! The library is designed to perform a "best-effort" processing of CycloneDX SBOM documents. This
//! is accomplished by having a library-facing interface that does not allow the programmer to
//! construct invalid documents, but does have a more lax internal representation to accommodate
//! invalid SBOM documents from other sources that are parsed by the library. The enums in the
//! `models` module demonstrate this pattern. They include an `Unknown-X` variant, which should not
//! be created manually, but might occur as a result of parsing a SBOM.
//!
//! In order to be confident that you are working with valid data, the library provides a
//! [`Validate`](crate::validation::Validate) trait to enable you to find invalid data in a parsed
//! SBOM. An example of this can be seen in the "Read and validate an SBOM" code snippet.
//!
//! ### Prelude
//!
//! The library provides a prelude (similar to the [Rust Standard Library's prelude](https://doc.rust-lang.org/std/prelude/index.html)) to make it easier to use the code. The prelude contains commonly used types and traits. To use this in your library, include the following code snippet:
//!
//! ```
//! use cyclonedx_bom::prelude::*;
//! ```

pub mod errors;
pub mod external_models;
pub mod models;
pub mod prelude;
pub mod validation;

mod specs;
mod utilities;
mod xml;
