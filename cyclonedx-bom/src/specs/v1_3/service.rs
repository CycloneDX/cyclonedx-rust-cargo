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
    utilities::{convert_optional, convert_optional_vec},
};
use serde::{Deserialize, Serialize};

use crate::specs::v1_3::{
    external_reference::ExternalReference, license::LicenseChoice,
    organization::OrganizationalEntity, property::Properties,
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Service {
    #[serde(rename = "bom-ref")]
    bom_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<OrganizationalEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    authenticated: Option<bool>,
    #[serde(rename = "x-trust-boundary", skip_serializing_if = "Option::is_none")]
    x_trust_boundary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Vec<DataClassification>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    licenses: Option<Vec<LicenseChoice>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_references: Option<Vec<ExternalReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    services: Option<Vec<Service>>,
}

impl From<models::Service> for Service {
    fn from(other: models::Service) -> Self {
        Self {
            bom_ref: other.bom_ref,
            provider: convert_optional(other.provider),
            group: other.group.map(|g| g.to_string()),
            name: other.name.to_string(),
            version: other.version.map(|v| v.to_string()),
            description: other.description.map(|d| d.to_string()),
            endpoints: other
                .endpoints
                .map(|endpoints| endpoints.into_iter().map(|e| e.to_string()).collect()),
            authenticated: other.authenticated,
            x_trust_boundary: other.x_trust_boundary,
            data: convert_optional_vec(other.data),
            licenses: convert_optional_vec(other.licenses),
            external_references: convert_optional_vec(other.external_references),
            properties: convert_optional(other.properties),
            services: convert_optional_vec(other.services),
        }
    }
}

impl From<Service> for models::Service {
    fn from(other: Service) -> Self {
        Self {
            bom_ref: other.bom_ref,
            provider: convert_optional(other.provider),
            group: other.group.map(NormalizedString::new_unchecked),
            name: NormalizedString::new_unchecked(other.name),
            version: other.version.map(NormalizedString::new_unchecked),
            description: other.description.map(NormalizedString::new_unchecked),
            endpoints: other
                .endpoints
                .map(|endpoints| endpoints.into_iter().map(Uri).collect()),
            authenticated: other.authenticated,
            x_trust_boundary: other.x_trust_boundary,
            data: convert_optional_vec(other.data),
            licenses: convert_optional_vec(other.licenses),
            external_references: convert_optional_vec(other.external_references),
            properties: convert_optional(other.properties),
            services: convert_optional_vec(other.services),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DataClassification {
    flow: String,
    classification: String,
}

impl From<models::DataClassification> for DataClassification {
    fn from(other: models::DataClassification) -> Self {
        Self {
            flow: other.flow.to_string(),
            classification: other.classification.to_string(),
        }
    }
}

impl From<DataClassification> for models::DataClassification {
    fn from(other: DataClassification) -> Self {
        Self {
            flow: models::DataFlowType::new_unchecked(&other.flow),
            classification: NormalizedString::new_unchecked(other.classification),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::specs::v1_3::{
        external_reference::test::{corresponding_external_reference, example_external_reference},
        license::test::{corresponding_license_expression, example_license_expression},
        organization::test::{corresponding_entity, example_entity},
        property::test::{corresponding_properties, example_properties},
    };

    pub(crate) fn example_service() -> Service {
        Service {
            bom_ref: Some("bom-ref".to_string()),
            provider: Some(example_entity()),
            group: Some("group".to_string()),
            name: "name".to_string(),
            version: Some("version".to_string()),
            description: Some("description".to_string()),
            endpoints: Some(vec!["endpoint".to_string()]),
            authenticated: Some(true),
            x_trust_boundary: Some(true),
            data: Some(vec![example_data_classification()]),
            licenses: Some(vec![example_license_expression()]),
            external_references: Some(vec![example_external_reference()]),
            properties: Some(example_properties()),
            services: Some(vec![]),
        }
    }

    pub(crate) fn corresponding_service() -> models::Service {
        models::Service {
            bom_ref: Some("bom-ref".to_string()),
            provider: Some(corresponding_entity()),
            group: Some(NormalizedString::new_unchecked("group".to_string())),
            name: NormalizedString::new_unchecked("name".to_string()),
            version: Some(NormalizedString::new_unchecked("version".to_string())),
            description: Some(NormalizedString::new_unchecked("description".to_string())),
            endpoints: Some(vec![Uri("endpoint".to_string())]),
            authenticated: Some(true),
            x_trust_boundary: Some(true),
            data: Some(vec![corresponding_data_classification()]),
            licenses: Some(vec![corresponding_license_expression()]),
            external_references: Some(vec![corresponding_external_reference()]),
            properties: Some(corresponding_properties()),
            services: Some(vec![]),
        }
    }

    fn example_data_classification() -> DataClassification {
        DataClassification {
            flow: "flow".to_string(),
            classification: "classification".to_string(),
        }
    }

    fn corresponding_data_classification() -> models::DataClassification {
        models::DataClassification {
            flow: models::DataFlowType::UnknownDataFlow("flow".to_string()),
            classification: NormalizedString::new_unchecked("classification".to_string()),
        }
    }
}
