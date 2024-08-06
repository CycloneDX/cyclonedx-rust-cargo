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
    external_models::date_time::{validate_date_time, DateTime},
    models::{
        component::Component,
        organization::{OrganizationalContact, OrganizationalEntity},
        service::Service,
        signature::Signature,
    },
    prelude::{Validate, ValidationResult},
    validation::ValidationContext,
};

use super::bom::SpecVersion;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotations(pub Vec<Annotation>);

impl Validate for Annotations {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |annotation| {
                annotation.validate_version(version)
            })
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
    pub bom_ref: Option<String>,
    pub subjects: Vec<String>,
    pub annotator: Annotator,
    pub timestamp: DateTime,
    pub text: String,
    pub signature: Option<Signature>,
}

impl Validate for Annotation {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("timestamp", &self.timestamp, validate_date_time)
            .add_struct("annotator", &self.annotator, version)
            .into()
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
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();
        match self {
            Annotator::Organization(organization) => {
                context.add_struct("organization", organization, version);
            }
            Annotator::Individual(contact) => {
                context.add_struct("contact", contact, version);
            }
            Annotator::Component(component) => {
                context.add_struct("component", component, version);
            }
            Annotator::Service(service) => {
                context.add_struct("service", service, version);
            }
        }
        context.into()
    }
}

#[cfg(test)]
mod test {}
