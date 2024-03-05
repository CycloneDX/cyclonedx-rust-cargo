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
    models::{
        component::Component,
        organization::{OrganizationalContact, OrganizationalEntity},
        service::Service,
        signature::Signature,
    },
    prelude::{Validate, ValidationResult},
    validation::{ValidationContext, ValidationPathComponent},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotations(pub Vec<Annotation>);

impl Validate for Annotations {
    fn validate_with_context(&self, context: ValidationContext) -> ValidationResult {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, annotation) in self.0.iter().enumerate() {
            let annotation_context =
                context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(annotation.validate_with_context(annotation_context));
        }

        results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
    pub(crate) bom_ref: Option<String>,
    pub(crate) subjects: Vec<String>,
    pub(crate) annotator: Annotator,
    pub(crate) timestamp: DateTime,
    pub(crate) text: String,
    pub(crate) signature: Option<Signature>,
}

impl Validate for Annotation {
    fn validate_with_context(&self, context: ValidationContext) -> ValidationResult {
        let mut results: Vec<ValidationResult> = vec![];

        let timestamp_context = context.with_struct("Annotation", "timestamp");
        results.push(self.timestamp.validate_with_context(timestamp_context));

        let annotator_context = context.with_struct("Annotation", "annotator");
        results.push(self.annotator.validate_with_context(annotator_context));

        results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result))
    }
}

/// Represents an Annotator: organization, individual, component or service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Annotator {
    Organization(OrganizationalEntity),
    Individual(OrganizationalContact),
    Component(Component),
    Service(Service),
}

impl Validate for Annotator {
    fn validate_with_context(&self, context: ValidationContext) -> ValidationResult {
        match self {
            Annotator::Organization(organization) => {
                let organization_context = context
                    .extend_context(vec![ValidationPathComponent::enum_variant("organization")]);
                organization.validate_with_context(organization_context)
            }
            Annotator::Individual(contact) => {
                let individual_context = context
                    .extend_context(vec![ValidationPathComponent::enum_variant("individual")]);
                contact.validate_with_context(individual_context)
            }
            Annotator::Component(component) => {
                let component_context = context
                    .extend_context(vec![ValidationPathComponent::enum_variant("component")]);
                component.validate_with_context(component_context)
            }
            Annotator::Service(service) => {
                let service_context =
                    context.extend_context(vec![ValidationPathComponent::enum_variant("service")]);
                service.validate_with_context(service_context)
            }
        }
    }
}
