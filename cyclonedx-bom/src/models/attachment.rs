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

use crate::{
    prelude::{Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

use super::bom::SpecVersion;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Attachment {
    pub content: String,
    pub content_type: Option<String>,
    pub encoding: Option<String>,
}

impl Validate for Attachment {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("encoding", self.encoding.as_ref(), validate_encoding)
            .into()
    }
}

fn validate_encoding(encoding: &String) -> Result<(), ValidationError> {
    if encoding != "base64" {
        return Err("Unsupported encoding found.".into());
    }
    Ok(())
}
