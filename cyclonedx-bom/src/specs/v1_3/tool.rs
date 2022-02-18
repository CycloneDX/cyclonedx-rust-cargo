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

use crate::{external_models::normalized_string::NormalizedString, specs::v1_3::hash::Hashes};
use crate::{models, utilities::convert_optional};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tool {
    #[serde(skip_serializing_if = "Option::is_none")]
    vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hashes: Option<Hashes>,
}

impl From<models::Tool> for Tool {
    fn from(other: models::Tool) -> Self {
        Self {
            vendor: other.vendor.map(|v| v.to_string()),
            name: other.name.map(|n| n.to_string()),
            version: other.version.map(|v| v.to_string()),
            hashes: convert_optional(other.hashes),
        }
    }
}

impl From<Tool> for models::Tool {
    fn from(other: Tool) -> Self {
        Self {
            vendor: other.vendor.map(NormalizedString::new_unchecked),
            name: other.name.map(NormalizedString::new_unchecked),
            version: other.version.map(NormalizedString::new_unchecked),
            hashes: convert_optional(other.hashes),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::specs::v1_3::hash::test::{corresponding_hashes, example_hashes};

    use super::*;

    pub(crate) fn example_tool() -> Tool {
        Tool {
            vendor: Some("vendor".to_string()),
            name: Some("name".to_string()),
            version: Some("version".to_string()),
            hashes: Some(example_hashes()),
        }
    }

    pub(crate) fn corresponding_tool() -> models::Tool {
        models::Tool {
            vendor: Some(NormalizedString::new_unchecked("vendor".to_string())),
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            version: Some(NormalizedString::new_unchecked("version".to_string())),
            hashes: Some(corresponding_hashes()),
        }
    }
}
