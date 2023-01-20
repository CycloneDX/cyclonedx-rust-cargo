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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BomError {
    #[error("Failed to serialize BOM to JSON: {0}")]
    JsonSerializationError(#[from] serde_json::Error),

    #[error("Failed to serialize BOM to XML: {0}")]
    XmlSerializationError(String),

    #[error("Failed to serialize BOM to v1.3: {0}")]
    BomV13SerializationError(String),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum JsonWriteError {
    #[error("Failed to serialize JSON: {error}")]
    JsonElementWriteError {
        #[from]
        error: serde_json::Error,
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
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum JsonReadError {
    #[error("Failed to deserialize JSON: {error}")]
    JsonElementWriteError {
        #[from]
        error: serde_json::Error,
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
