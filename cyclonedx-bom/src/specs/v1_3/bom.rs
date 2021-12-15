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

use crate::{models, utilities::convert_optional};
use crate::{
    specs::v1_3::{
        component::Component, composition::Composition, dependency::Dependencies,
        external_reference::ExternalReference, metadata::Metadata, property::Properties,
        service::Service,
    },
    utilities::convert_optional_vec,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Bom {
    bom_format: BomFormat,
    spec_version: String,
    version: Option<u32>,
    serial_number: Option<UrnUuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    services: Option<Vec<Service>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_references: Option<Vec<ExternalReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Dependencies>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compositions: Option<Vec<Composition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
}

impl From<models::Bom> for Bom {
    fn from(other: models::Bom) -> Self {
        Self {
            bom_format: BomFormat::CycloneDX,
            spec_version: "1.4".to_string(),
            version: Some(other.version),
            serial_number: convert_optional(other.serial_number),
            metadata: convert_optional(other.metadata),
            components: convert_optional_vec(other.components),
            services: convert_optional_vec(other.services),
            external_references: convert_optional_vec(other.external_references),
            dependencies: convert_optional(other.dependencies),
            compositions: convert_optional_vec(other.compositions),
            properties: convert_optional(other.properties),
        }
    }
}

impl From<Bom> for models::Bom {
    fn from(other: Bom) -> Self {
        Self {
            version: other.version.unwrap_or(1),
            serial_number: convert_optional(other.serial_number),
            metadata: convert_optional(other.metadata),
            components: convert_optional_vec(other.components),
            services: convert_optional_vec(other.services),
            external_references: convert_optional_vec(other.external_references),
            dependencies: convert_optional(other.dependencies),
            compositions: convert_optional_vec(other.compositions),
            properties: convert_optional(other.properties),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum BomFormat {
    CycloneDX,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct UrnUuid(String);

impl From<models::UrnUuid> for UrnUuid {
    fn from(other: models::UrnUuid) -> Self {
        Self(other.0)
    }
}

impl From<UrnUuid> for models::UrnUuid {
    fn from(other: UrnUuid) -> Self {
        Self(other.0)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::specs::v1_3::{
        component::test::{corresponding_component, example_component},
        composition::test::{corresponding_composition, example_composition},
        dependency::test::{corresponding_dependencies, example_dependencies},
        external_reference::test::{corresponding_external_reference, example_external_reference},
        metadata::test::{corresponding_metadata, example_metadata},
        property::test::{corresponding_properties, example_properties},
        service::test::{corresponding_service, example_service},
    };

    use super::*;

    #[test]
    fn it_should_serialize_to_json() {
        let actual = Bom {
            bom_format: BomFormat::CycloneDX,
            spec_version: "1.3".to_string(),
            version: Some(1),
            serial_number: Some(UrnUuid("fake-uuid".to_string())),
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: None,
            properties: None,
        };

        insta::assert_json_snapshot!(actual);
    }

    #[test]
    fn it_should_serialize_a_complex_example_to_json() {
        let actual = full_bom_example();

        insta::assert_json_snapshot!(actual);
    }

    #[test]
    fn it_can_convert_to_the_internal_model() {
        let spec = full_bom_example();
        let model: models::Bom = spec.into();
        assert_eq!(model, corresponding_internal_model());
    }

    #[test]
    fn it_can_convert_from_the_internal_model() {
        let model = corresponding_internal_model();
        let spec: Bom = model.into();
        assert_eq!(spec, full_bom_example());
    }

    pub(crate) fn full_bom_example() -> Bom {
        Bom {
            bom_format: BomFormat::CycloneDX,
            spec_version: "1.4".to_string(),
            version: Some(1),
            serial_number: Some(UrnUuid("fake-uuid".to_string())),
            metadata: Some(example_metadata()),
            components: Some(vec![example_component()]),
            services: Some(vec![example_service()]),
            external_references: Some(vec![example_external_reference()]),
            dependencies: Some(example_dependencies()),
            compositions: Some(vec![example_composition()]),
            properties: Some(example_properties()),
        }
    }

    pub(crate) fn corresponding_internal_model() -> models::Bom {
        models::Bom {
            version: 1,
            serial_number: Some(models::UrnUuid("fake-uuid".to_string())),
            metadata: Some(corresponding_metadata()),
            components: Some(vec![corresponding_component()]),
            services: Some(vec![corresponding_service()]),
            external_references: Some(vec![corresponding_external_reference()]),
            dependencies: Some(corresponding_dependencies()),
            compositions: Some(vec![corresponding_composition()]),
            properties: Some(corresponding_properties()),
        }
    }
}
