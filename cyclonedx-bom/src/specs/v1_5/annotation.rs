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

use serde::{Deserialize, Serialize};

use crate::models;
use crate::specs::common::organization::{OrganizationalContact, OrganizationalEntity};
use crate::specs::common::signature::Signature;
use crate::specs::v1_5::{component::Component, service::Service};
use crate::utilities::{convert_optional, convert_vec};

/// Represents the `Annotations` field, see https://cyclonedx.org/docs/1.5/json/#annotations.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Annotations(Vec<Annotation>);

impl From<models::annotation::Annotations> for Annotations {
    fn from(other: models::annotation::Annotations) -> Self {
        Annotations(convert_vec(other.0))
    }
}

/// A single annotation.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Annotation {
    /// Optional identifier to reference the annotation elsewhere in the Bom.
    bom_ref: Option<String>,
    /// A list of BOM references
    subjects: Vec<String>,
    /// The annotator
    annotator: Annotator,
    /// The timestamp when this annotation was created.
    timestamp: String,
    /// The textual content of the annotation.
    text: String,
    /// The optional signature
    signature: Option<Signature>,
}

impl From<models::annotation::Annotation> for Annotation {
    fn from(other: models::annotation::Annotation) -> Self {
        Self {
            bom_ref: convert_optional(other.bom_ref),
            subjects: convert_vec(other.subjects),
            annotator: other.annotator.into(),
            timestamp: other.timestamp.to_string(),
            text: other.text.clone(),
            signature: convert_optional(other.signature),
        }
    }
}

/// Represents the 'Annotator' field, see https://cyclonedx.org/docs/1.5/json/#annotations_items_annotator
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Annotator {
    Organization(OrganizationalEntity),
    Individual(OrganizationalContact),
    Component(Component),
    Service(Service),
}

impl From<models::annotation::Annotator> for Annotator {
    fn from(other: models::annotation::Annotator) -> Self {
        match other {
            models::annotation::Annotator::Organization(org) => Self::Organization(org.into()),
            models::annotation::Annotator::Individual(contact) => Self::Individual(contact.into()),
            models::annotation::Annotator::Component(component) => {
                Self::Component(component.into())
            }
            models::annotation::Annotator::Service(service) => Self::Service(service.into()),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{
        models,
        specs::common::{organization::test::example_entity, signature::test::example_signature},
    };

    use super::{Annotation, Annotator};

    pub(crate) fn example_annotation() -> Annotation {
        Annotation {
            bom_ref: Some("annotation-1".to_string()),
            subjects: vec!["subject1".to_string()],
            annotator: example_annotator(),
            timestamp: "timestamp".to_string(),
            text: "Annotation text".to_string(),
            signature: Some(example_signature()),
        }
    }

    fn example_annotator() -> Annotator {
        Annotator::Organization(example_entity())
    }

    pub(crate) fn corresponding_annotation() -> models::annotation::Annotation {
        todo!("")
    }
}
