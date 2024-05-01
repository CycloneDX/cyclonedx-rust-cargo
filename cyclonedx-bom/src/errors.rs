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

use std::convert::Infallible;

use xml::name::OwnedName;

use crate::models::bom::SpecVersion;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BomError {
    #[error("Failed to serialize BOM to JSON: {0}")]
    JsonSerializationError(#[from] serde_json::Error),

    #[error("Failed to serialize BOM to XML: {0}")]
    XmlSerializationError(String),

    #[error("Failed to serialize BOM with version {0:?}: {1}")]
    BomSerializationError(SpecVersion, String),

    #[error("Unsupported Spec Version '{0}'")]
    UnsupportedSpecVersion(String),
}

// This allows to use `TryFrom` when a type only implements `From` inside a
// `TryFrom<Error = BomError>` implementation.
impl From<Infallible> for BomError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum JsonWriteError {
    #[error("Failed to serialize JSON: {error}")]
    JsonElementWriteError {
        #[from]
        error: serde_json::Error,
    },
    #[error("Failed to convert Bom: {error}")]
    BomError {
        #[from]
        error: BomError,
    },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum XmlWriteError {
    #[error("Failed to serialize XML while writing {element}: {error}")]
    XmlElementWriteError {
        #[source]
        error: xml::writer::Error,
        element: String,
    },
    #[error("Failed to convert Bom: {error}")]
    BomError {
        #[from]
        error: BomError,
    },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum JsonReadError {
    #[error("Failed to deserialize JSON: {error}")]
    JsonElementReadError {
        #[from]
        error: serde_json::Error,
    },
    #[error("Invalid input format found: {error}")]
    BomError {
        #[from]
        error: BomError,
    },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum XmlReadError {
    #[error("Failed to deserialize XML while reading {element}: {error}")]
    ElementReadError {
        #[source]
        error: xml::reader::Error,
        element: String,
    },
    #[error("Got unexpected XML element when reading {element}: {error}")]
    UnexpectedElementReadError { error: String, element: String },

    #[error("Ended element {element} without data for required field {required_field}")]
    RequiredDataMissing {
        required_field: String,
        element: String,
    },

    #[error("Required attribute {attribute} not found in element {element}")]
    RequiredAttributeMissing { attribute: String, element: String },

    #[error("Could not parse {value} as {data_type} on {element}")]
    InvalidParseError {
        value: String,
        data_type: String,
        element: String,
    },

    #[error(
        "Expected document to be in the form {expected_namespace}, but received {}", .actual_namespace.as_ref().unwrap_or(&"no CycloneDX namespace".to_string())
    )]
    InvalidNamespaceError {
        expected_namespace: String,
        actual_namespace: Option<String>,
    },
}

impl XmlReadError {
    pub fn required_data_missing(required_field: &str, element: &OwnedName) -> Self {
        Self::RequiredDataMissing {
            required_field: required_field.to_string(),
            element: element.local_name.to_string(),
        }
    }
}
