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
pub enum BomError {
    #[error("Failed to serialize BOM to JSON: {0}")]
    JsonSerializationError(#[from] serde_json::Error),

    #[error("Failed to serialize BOM to XML: {0}")]
    XmlSerializationError(String),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum XmlWriteError {
    #[error("Failed to serialize XML while writing {element}: {error}")]
    XmlElementWriteError {
        #[source]
        error: xml::writer::Error,
        element: String,
    },
}
