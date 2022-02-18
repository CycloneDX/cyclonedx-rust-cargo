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

use crate::external_models::uri::Uri;
use crate::specs::v1_3::hash::Hashes;
use crate::{models, utilities::convert_optional};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExternalReference {
    #[serde(rename = "type")]
    external_reference_type: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hashes: Option<Hashes>,
}

impl From<models::ExternalReference> for ExternalReference {
    fn from(other: models::ExternalReference) -> Self {
        Self {
            external_reference_type: other.external_reference_type.to_string(),
            url: other.url.to_string(),
            comment: other.comment,
            hashes: convert_optional(other.hashes),
        }
    }
}

impl From<ExternalReference> for models::ExternalReference {
    fn from(other: ExternalReference) -> Self {
        Self {
            external_reference_type: models::ExternalReferenceType::new_unchecked(
                other.external_reference_type,
            ),
            url: Uri(other.url),
            comment: other.comment,
            hashes: convert_optional(other.hashes),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::specs::v1_3::hash::test::{corresponding_hashes, example_hashes};

    pub(crate) fn example_external_reference() -> ExternalReference {
        ExternalReference {
            external_reference_type: "external reference type".to_string(),
            url: "url".to_string(),
            comment: Some("comment".to_string()),
            hashes: Some(example_hashes()),
        }
    }

    pub(crate) fn corresponding_external_reference() -> models::ExternalReference {
        models::ExternalReference {
            external_reference_type: models::ExternalReferenceType::UnknownExternalReferenceType(
                "external reference type".to_string(),
            ),
            url: Uri("url".to_string()),
            comment: Some("comment".to_string()),
            hashes: Some(corresponding_hashes()),
        }
    }
}
