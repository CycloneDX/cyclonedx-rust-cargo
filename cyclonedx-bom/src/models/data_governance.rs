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
    prelude::{Validate, ValidationResult},
    validation::ValidationContext,
};

use super::{
    bom::SpecVersion,
    organization::{OrganizationalContact, OrganizationalEntity},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataGovernance {
    pub custodians: Option<Vec<DataGovernanceResponsibleParty>>,
    pub stewards: Option<Vec<DataGovernanceResponsibleParty>>,
    pub owners: Option<Vec<DataGovernanceResponsibleParty>>,
}

impl Validate for DataGovernance {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("custodians", self.custodians.as_ref(), |c| {
                c.validate_version(version)
            })
            .add_list_option("stewards", self.stewards.as_ref(), |c| {
                c.validate_version(version)
            })
            .add_list_option("owners", self.owners.as_ref(), |c| {
                c.validate_version(version)
            })
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataGovernanceResponsibleParty {
    Organization(OrganizationalEntity),
    Contact(OrganizationalContact),
}

impl Validate for DataGovernanceResponsibleParty {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match self {
            DataGovernanceResponsibleParty::Organization(org) => org.validate_version(version),
            DataGovernanceResponsibleParty::Contact(contact) => contact.validate_version(version),
        }
    }
}
