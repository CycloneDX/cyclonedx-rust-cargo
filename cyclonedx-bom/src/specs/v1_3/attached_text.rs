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

use crate::external_models::normalized_string::NormalizedString;
use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttachedText {
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding: Option<String>,
    content: String,
}

impl From<models::AttachedText> for AttachedText {
    fn from(other: models::AttachedText) -> Self {
        Self {
            content_type: other.content_type.map(|n| n.0),
            encoding: other.encoding.map(|e| e.to_string()),
            content: other.content,
        }
    }
}

impl From<AttachedText> for models::AttachedText {
    fn from(other: AttachedText) -> Self {
        Self {
            content_type: other.content_type.map(NormalizedString::new_unchecked),
            encoding: other.encoding.map(models::Encoding::new_unchecked),
            content: other.content,
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub(crate) fn example_attached_text() -> AttachedText {
        AttachedText {
            content_type: Some("content type".to_string()),
            encoding: Some("encoding".to_string()),
            content: "content".to_string(),
        }
    }

    pub(crate) fn corresponding_attached_text() -> models::AttachedText {
        models::AttachedText {
            content_type: Some(NormalizedString::new_unchecked("content type".to_string())),
            encoding: Some(models::Encoding::UnknownEncoding("encoding".to_string())),
            content: "content".to_string(),
        }
    }
}
