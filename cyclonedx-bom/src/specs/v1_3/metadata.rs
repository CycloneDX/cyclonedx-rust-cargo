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
    external_models::date_time::DateTime,
    specs::v1_3::{
        component::Component, license::Licenses, organization::OrganizationalContact,
        organization::OrganizationalEntity, property::Properties, tool::Tools,
    },
};
use crate::{
    models,
    utilities::{convert_optional, convert_optional_vec},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Tools>,
    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<OrganizationalContact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manufacture: Option<OrganizationalEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supplier: Option<OrganizationalEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    licenses: Option<Licenses>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
}

impl From<models::Metadata> for Metadata {
    fn from(other: models::Metadata) -> Self {
        Self {
            timestamp: other.timestamp.map(|t| t.to_string()),
            tools: convert_optional(other.tools),
            authors: convert_optional_vec(other.authors),
            component: convert_optional(other.component),
            manufacture: convert_optional(other.manufacture),
            supplier: convert_optional(other.supplier),
            licenses: convert_optional(other.licenses),
            properties: convert_optional(other.properties),
        }
    }
}

impl From<Metadata> for models::Metadata {
    fn from(other: Metadata) -> Self {
        Self {
            timestamp: other.timestamp.map(DateTime),
            tools: convert_optional(other.tools),
            authors: convert_optional_vec(other.authors),
            component: convert_optional(other.component),
            manufacture: convert_optional(other.manufacture),
            supplier: convert_optional(other.supplier),
            licenses: convert_optional(other.licenses),
            properties: convert_optional(other.properties),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::specs::v1_3::{
        component::test::{corresponding_component, example_component},
        license::test::{corresponding_licenses, example_licenses},
        organization::test::{
            corresponding_contact, corresponding_entity, example_contact, example_entity,
        },
        property::test::{corresponding_properties, example_properties},
        tool::test::{corresponding_tools, example_tools},
    };

    use super::*;

    pub(crate) fn example_metadata() -> Metadata {
        Metadata {
            timestamp: Some("timestamp".to_string()),
            tools: Some(example_tools()),
            authors: Some(vec![example_contact()]),
            component: Some(example_component()),
            manufacture: Some(example_entity()),
            supplier: Some(example_entity()),
            licenses: Some(example_licenses()),
            properties: Some(example_properties()),
        }
    }

    pub(crate) fn corresponding_metadata() -> models::Metadata {
        models::Metadata {
            timestamp: Some(DateTime("timestamp".to_string())),
            tools: Some(corresponding_tools()),
            authors: Some(vec![corresponding_contact()]),
            component: Some(corresponding_component()),
            manufacture: Some(corresponding_entity()),
            supplier: Some(corresponding_entity()),
            licenses: Some(corresponding_licenses()),
            properties: Some(corresponding_properties()),
        }
    }
}
