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

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    models::{
        bom::{validate_bom_ref, BomReference, SpecVersion},
        external_reference::ExternalReferences,
        organization::{OrganizationalContact, OrganizationalEntity},
    },
    validation::{Validate, ValidationContext, ValidationError, ValidationResult},
};

/// A list of assertions made regarding patents associated with a component or service.
///
/// Defined via the [CycloneDX JSON schema](https://cyclonedx.org/docs/1.7/json/#components_items_patentAssertions)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PatentAssertions(pub Vec<PatentAssertion>);

impl Validate for PatentAssertions {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |assertion| {
                assertion.validate_version(version)
            })
            .into()
    }
}

/// An assertion linking a patent or patent family to a component or service.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PatentAssertion {
    pub bom_ref: Option<BomReference>,
    pub assertion_type: PatentAssertionType,
    pub patent_refs: Option<Vec<BomReference>>,
    pub asserter: PatentAsserter,
    pub notes: Option<String>,
}

impl Validate for PatentAssertion {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("bom-ref", self.bom_ref.as_ref(), |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .add_enum(
                "assertion_type",
                &self.assertion_type,
                validate_patent_assertion_type,
            )
            .add_struct("asserter", &self.asserter, version)
            .into()
    }
}

pub fn validate_patent_assertion_type(
    assertion_type: &PatentAssertionType,
) -> Result<(), ValidationError> {
    if matches!(assertion_type, PatentAssertionType::Unknown(_)) {
        return Err(ValidationError::new("Unknown patent assertion type"));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum PatentAssertionType {
    Ownership,
    License,
    ThirdPartyClaim,
    StandardsInclusion,
    PriorArt,
    ExclusiveRights,
    NonAssertion,
    ResearchOrEvaluation,
    #[doc(hidden)]
    #[strum(default)]
    Unknown(String),
}

impl PatentAssertionType {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "ownership" => Self::Ownership,
            "license" => Self::License,
            "third-party-claim" => Self::ThirdPartyClaim,
            "standards-inclusion" => Self::StandardsInclusion,
            "prior-art" => Self::PriorArt,
            "exclusive-rights" => Self::ExclusiveRights,
            "non-assertion" => Self::NonAssertion,
            "research-or-evaluation" => Self::ResearchOrEvaluation,
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

/// The party making a [`PatentAssertion`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatentAsserter {
    Organization(OrganizationalEntity),
    Contact(OrganizationalContact),
    /// A `bom-ref` to an organizational entity or contact defined elsewhere in the BOM.
    Reference(BomReference),
}

impl Validate for PatentAsserter {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match self {
            Self::Organization(organization) => organization.validate_version(version),
            Self::Contact(contact) => contact.validate_version(version),
            Self::Reference(_) => ValidationResult::new(),
        }
    }
}

/// An individual patent or a patent family listed in `definitions.patents`.
// Boxing `Patent` would only save a few hundred bytes per item and complicate construction.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatentDefinition {
    Patent(Patent),
    PatentFamily(PatentFamily),
}

impl Validate for PatentDefinition {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match self {
            Self::Patent(patent) => patent.validate_version(version),
            Self::PatentFamily(family) => family.validate_version(version),
        }
    }
}

/// Defined via the [CycloneDX JSON schema](https://cyclonedx.org/docs/1.7/json/#definitions_patents)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Patent {
    pub bom_ref: Option<BomReference>,
    pub patent_number: String,
    pub application_number: Option<String>,
    pub jurisdiction: String,
    pub priority_application: Option<PriorityApplication>,
    pub publication_number: Option<String>,
    pub title: Option<String>,
    pub abstract_text: Option<String>,
    /// ISO 8601 date (`YYYY-MM-DD`)
    pub filing_date: Option<String>,
    /// ISO 8601 date (`YYYY-MM-DD`)
    pub grant_date: Option<String>,
    /// ISO 8601 date (`YYYY-MM-DD`)
    pub patent_expiration_date: Option<String>,
    pub patent_legal_status: PatentLegalStatus,
    pub patent_assignee: Option<Vec<PatentAssignee>>,
    pub external_references: Option<ExternalReferences>,
}

impl Validate for Patent {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("bom-ref", self.bom_ref.as_ref(), |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .add_field(
                "patent_number",
                self.patent_number.as_str(),
                validate_patent_number,
            )
            .add_field_option(
                "application_number",
                self.application_number.as_deref(),
                validate_patent_number,
            )
            .add_field(
                "jurisdiction",
                self.jurisdiction.as_str(),
                validate_jurisdiction,
            )
            .add_struct_option(
                "priority_application",
                self.priority_application.as_ref(),
                version,
            )
            .add_field_option(
                "publication_number",
                self.publication_number.as_deref(),
                validate_patent_number,
            )
            .add_field_option("filing_date", self.filing_date.as_deref(), validate_date)
            .add_field_option("grant_date", self.grant_date.as_deref(), validate_date)
            .add_field_option(
                "patent_expiration_date",
                self.patent_expiration_date.as_deref(),
                validate_date,
            )
            .add_enum(
                "patent_legal_status",
                &self.patent_legal_status,
                validate_patent_legal_status,
            )
            .add_list_option(
                "patent_assignee",
                self.patent_assignee.as_ref(),
                |assignee| assignee.validate_version(version),
            )
            .add_struct_option(
                "external_references",
                self.external_references.as_ref(),
                version,
            )
            .into()
    }
}

pub fn validate_patent_legal_status(status: &PatentLegalStatus) -> Result<(), ValidationError> {
    if matches!(status, PatentLegalStatus::Unknown(_)) {
        return Err(ValidationError::new("Unknown patent legal status"));
    }
    Ok(())
}

/// Legal status of a patent, based on WIPO ST.27.
#[derive(Clone, Debug, PartialEq, Eq, Hash, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum PatentLegalStatus {
    Pending,
    Granted,
    Revoked,
    Expired,
    Lapsed,
    Withdrawn,
    Abandoned,
    Suspended,
    Reinstated,
    Opposed,
    Terminated,
    Invalidated,
    InForce,
    #[doc(hidden)]
    #[strum(default)]
    Unknown(String),
}

impl PatentLegalStatus {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "pending" => Self::Pending,
            "granted" => Self::Granted,
            "revoked" => Self::Revoked,
            "expired" => Self::Expired,
            "lapsed" => Self::Lapsed,
            "withdrawn" => Self::Withdrawn,
            "abandoned" => Self::Abandoned,
            "suspended" => Self::Suspended,
            "reinstated" => Self::Reinstated,
            "opposed" => Self::Opposed,
            "terminated" => Self::Terminated,
            "invalidated" => Self::Invalidated,
            "in-force" => Self::InForce,
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

/// An organization or individual to whom patent rights are assigned.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatentAssignee {
    Organization(OrganizationalEntity),
    Individual(OrganizationalContact),
}

impl Validate for PatentAssignee {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match self {
            Self::Organization(organization) => organization.validate_version(version),
            Self::Individual(contact) => contact.validate_version(version),
        }
    }
}

/// Defined via the [CycloneDX JSON schema](https://cyclonedx.org/docs/1.7/json/#definitions_patents)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PatentFamily {
    pub bom_ref: Option<BomReference>,
    pub family_id: String,
    pub priority_application: Option<PriorityApplication>,
    pub members: Option<Vec<BomReference>>,
    pub external_references: Option<ExternalReferences>,
}

impl Validate for PatentFamily {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("bom-ref", self.bom_ref.as_ref(), |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .add_struct_option(
                "priority_application",
                self.priority_application.as_ref(),
                version,
            )
            .add_struct_option(
                "external_references",
                self.external_references.as_ref(),
                version,
            )
            .into()
    }
}

/// The earlier patent filing a patent or patent family claims priority from.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PriorityApplication {
    pub application_number: String,
    pub jurisdiction: String,
    /// ISO 8601 date (`YYYY-MM-DD`)
    pub filing_date: String,
}

impl Validate for PriorityApplication {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field(
                "application_number",
                self.application_number.as_str(),
                validate_patent_number,
            )
            .add_field(
                "jurisdiction",
                self.jurisdiction.as_str(),
                validate_jurisdiction,
            )
            .add_field("filing_date", self.filing_date.as_str(), validate_date)
            .into()
    }
}

fn validate_patent_number(number: &str) -> Result<(), ValidationError> {
    static PATENT_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[A-Za-z0-9][A-Za-z0-9\-/.()\s]{0,28}[A-Za-z0-9]$")
            .expect("Failed to compile regex.")
    });

    if !PATENT_NUMBER_REGEX.is_match(number) {
        return Err(ValidationError::new(
            "Patent number does not match regular expression",
        ));
    }
    Ok(())
}

fn validate_jurisdiction(jurisdiction: &str) -> Result<(), ValidationError> {
    static JURISDICTION_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[A-Z]{2}$").expect("Failed to compile regex."));

    if !JURISDICTION_REGEX.is_match(jurisdiction) {
        return Err(ValidationError::new(
            "Jurisdiction must be a two letter WIPO ST.3 code",
        ));
    }
    Ok(())
}

fn validate_date(date: &str) -> Result<(), ValidationError> {
    static DATE_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").expect("Failed to compile regex."));

    let is_valid = DATE_REGEX.captures(date).is_some_and(|captures| {
        let year = captures[1].parse::<i32>();
        let month = captures[2].parse::<u8>().map(time::Month::try_from);
        let day = captures[3].parse::<u8>();
        match (year, month, day) {
            (Ok(year), Ok(Ok(month)), Ok(day)) => {
                time::Date::from_calendar_date(year, month, day).is_ok()
            }
            _ => false,
        }
    });

    if !is_valid {
        return Err(ValidationError::new("Date does not conform to YYYY-MM-DD"));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::validation;

    fn example_patent() -> Patent {
        Patent {
            bom_ref: Some(BomReference::new("patent-1")),
            patent_number: "US1234567890".to_string(),
            application_number: Some("12345".to_string()),
            jurisdiction: "US".to_string(),
            priority_application: Some(PriorityApplication {
                application_number: "US1234567890".to_string(),
                jurisdiction: "US".to_string(),
                filing_date: "2021-01-15".to_string(),
            }),
            publication_number: None,
            title: Some("Title".to_string()),
            abstract_text: None,
            filing_date: Some("2021-01-15".to_string()),
            grant_date: None,
            patent_expiration_date: None,
            patent_legal_status: PatentLegalStatus::InForce,
            patent_assignee: Some(vec![PatentAssignee::Individual(
                OrganizationalContact::new("Jane Smith", None),
            )]),
            external_references: None,
        }
    }

    #[test]
    fn valid_patent_should_pass_validation() {
        let result = PatentDefinition::Patent(example_patent()).validate_version(SpecVersion::V1_7);

        assert!(result.passed());
    }

    #[test]
    fn invalid_patent_should_fail_validation() {
        let patent = Patent {
            jurisdiction: "usa".to_string(),
            filing_date: Some("15-01-2021".to_string()),
            patent_legal_status: PatentLegalStatus::new_unchecked("dormant"),
            ..example_patent()
        };

        let result = patent.validate_version(SpecVersion::V1_7);

        assert_eq!(
            result,
            vec![
                validation::field(
                    "jurisdiction",
                    "Jurisdiction must be a two letter WIPO ST.3 code"
                ),
                validation::field("filing_date", "Date does not conform to YYYY-MM-DD"),
                validation::r#enum("patent_legal_status", "Unknown patent legal status"),
            ]
            .into()
        );
    }

    #[test]
    fn invalid_patent_assertion_should_fail_validation() {
        let assertions = PatentAssertions(vec![PatentAssertion {
            bom_ref: None,
            assertion_type: PatentAssertionType::new_unchecked("borrowed"),
            patent_refs: Some(vec![BomReference::new("patent-1")]),
            asserter: PatentAsserter::Reference(BomReference::new("org-1")),
            notes: None,
        }]);

        let result = assertions.validate_version(SpecVersion::V1_7);

        assert_eq!(
            result,
            validation::list(
                "inner",
                [(
                    0,
                    validation::r#enum("assertion_type", "Unknown patent assertion type")
                )]
            )
        );
    }
}
