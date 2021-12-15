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
    external_models::{normalized_string::NormalizedString, uri::Uri},
    models,
    utilities::convert_optional_vec,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrganizationalContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
}

impl From<models::OrganizationalContact> for OrganizationalContact {
    fn from(other: models::OrganizationalContact) -> Self {
        Self {
            name: other.name.map(|n| n.to_string()),
            email: other.email.map(|e| e.to_string()),
            phone: other.phone.map(|p| p.to_string()),
        }
    }
}

impl From<OrganizationalContact> for models::OrganizationalContact {
    fn from(other: OrganizationalContact) -> Self {
        Self {
            name: other.name.map(NormalizedString::new_unchecked),
            email: other.email.map(NormalizedString::new_unchecked),
            phone: other.phone.map(NormalizedString::new_unchecked),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrganizationalEntity {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact: Option<Vec<OrganizationalContact>>,
}

impl From<models::OrganizationalEntity> for OrganizationalEntity {
    fn from(other: models::OrganizationalEntity) -> Self {
        Self {
            name: other.name.map(|n| n.to_string()),
            url: other
                .url
                .map(|urls| urls.into_iter().map(|url| url.0).collect()),
            contact: convert_optional_vec(other.contact),
        }
    }
}

impl From<OrganizationalEntity> for models::OrganizationalEntity {
    fn from(other: OrganizationalEntity) -> Self {
        Self {
            name: other.name.map(NormalizedString::new_unchecked),
            url: other.url.map(|urls| urls.into_iter().map(Uri).collect()),
            contact: convert_optional_vec(other.contact),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub(crate) fn example_contact() -> OrganizationalContact {
        OrganizationalContact {
            name: Some("name".to_string()),
            email: Some("email".to_string()),
            phone: Some("phone".to_string()),
        }
    }

    pub(crate) fn corresponding_contact() -> models::OrganizationalContact {
        models::OrganizationalContact {
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            email: Some(NormalizedString::new_unchecked("email".to_string())),
            phone: Some(NormalizedString::new_unchecked("phone".to_string())),
        }
    }

    pub(crate) fn example_entity() -> OrganizationalEntity {
        OrganizationalEntity {
            name: Some("name".to_string()),
            url: Some(vec!["url".to_string()]),
            contact: Some(vec![example_contact()]),
        }
    }

    pub(crate) fn corresponding_entity() -> models::OrganizationalEntity {
        models::OrganizationalEntity {
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            url: Some(vec![Uri("url".to_string())]),
            contact: Some(vec![corresponding_contact()]),
        }
    }
}
