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

use serde::{Deserialize, Deserializer, Serialize};
use xml::{reader, writer::XmlEvent};

use crate::{
    errors::{XmlReadError, XmlWriteError},
    models::{self, bom::BomReference},
    specs::{
        common::organization::{OrganizationalContact, OrganizationalEntity},
        v1_7::external_reference::ExternalReferences,
    },
    utilities::{convert_optional, convert_optional_vec, convert_vec},
    xml::{
        optional_attribute, read_lax_validation_tag, read_list_tag, read_simple_tag,
        to_xml_read_error, to_xml_write_error, unexpected_element_error, write_close_tag,
        write_list_string_tag, write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

const BOM_REF_ATTR: &str = "bom-ref";
const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";
const ORGANIZATION_TAG: &str = "organization";
const INDIVIDUAL_TAG: &str = "individual";
const CONTACT_TAG: &str = "contact";
const REF_TAG: &str = "ref";
const JURISDICTION_TAG: &str = "jurisdiction";
const APPLICATION_NUMBER_TAG: &str = "applicationNumber";
const FILING_DATE_TAG: &str = "filingDate";
const PRIORITY_APPLICATION_TAG: &str = "priorityApplication";

fn required<T>(value: Option<T>, field: &str, element: &str) -> Result<T, XmlReadError> {
    value.ok_or_else(|| XmlReadError::RequiredDataMissing {
        required_field: field.to_string(),
        element: element.to_string(),
    })
}

/// JSON accepts either an `organizationalEntity` or an `organizationalContact` object without a
/// discriminator, so the object is read once with the fields of both and classified afterwards.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrganizationOrIndividualJson {
    #[serde(rename = "bom-ref")]
    bom_ref: Option<String>,
    name: Option<String>,
    url: Option<Vec<String>>,
    contact: Option<Vec<OrganizationalContact>>,
    email: Option<String>,
    phone: Option<String>,
}

enum OrganizationOrIndividual {
    Organization(OrganizationalEntity),
    Individual(OrganizationalContact),
}

impl From<OrganizationOrIndividualJson> for OrganizationOrIndividual {
    fn from(other: OrganizationOrIndividualJson) -> Self {
        // An object carrying only shared fields (`bom-ref`, `name`) is ambiguous; treating it as
        // an organization keeps every field, because an entity is a superset of a contact here.
        let is_individual = (other.email.is_some() || other.phone.is_some())
            && other.url.is_none()
            && other.contact.is_none();
        if is_individual {
            Self::Individual(OrganizationalContact {
                bom_ref: other.bom_ref,
                name: other.name,
                email: other.email,
                phone: other.phone,
            })
        } else {
            Self::Organization(OrganizationalEntity {
                bom_ref: other.bom_ref,
                name: other.name,
                url: other.url,
                contact: other.contact,
            })
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct PatentAssertions(pub(crate) Vec<PatentAssertion>);

impl From<models::patent::PatentAssertions> for PatentAssertions {
    fn from(other: models::patent::PatentAssertions) -> Self {
        Self(convert_vec(other.0))
    }
}

impl From<PatentAssertions> for models::patent::PatentAssertions {
    fn from(other: PatentAssertions) -> Self {
        Self(convert_vec(other.0))
    }
}

const PATENT_ASSERTIONS_TAG: &str = "patentAssertions";
const PATENT_ASSERTION_TAG: &str = "patentAssertion";

impl ToXml for PatentAssertions {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        write_start_tag(writer, PATENT_ASSERTIONS_TAG)?;
        for assertion in &self.0 {
            assertion.write_xml_element(writer)?;
        }
        write_close_tag(writer, PATENT_ASSERTIONS_TAG)
    }
}

impl FromXml for PatentAssertions {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, PATENT_ASSERTION_TAG).map(Self)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PatentAssertion {
    #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
    pub(crate) bom_ref: Option<String>,
    pub(crate) assertion_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) patent_refs: Option<Vec<String>>,
    pub(crate) asserter: PatentAsserter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) notes: Option<String>,
}

impl From<models::patent::PatentAssertion> for PatentAssertion {
    fn from(other: models::patent::PatentAssertion) -> Self {
        Self {
            bom_ref: other.bom_ref.map(|r| r.0),
            assertion_type: other.assertion_type.to_string(),
            patent_refs: other
                .patent_refs
                .map(|refs| refs.into_iter().map(|r| r.0).collect()),
            asserter: other.asserter.into(),
            notes: other.notes,
        }
    }
}

impl From<PatentAssertion> for models::patent::PatentAssertion {
    fn from(other: PatentAssertion) -> Self {
        Self {
            bom_ref: other.bom_ref.map(BomReference::new),
            assertion_type: models::patent::PatentAssertionType::new_unchecked(
                other.assertion_type,
            ),
            patent_refs: other
                .patent_refs
                .map(|refs| refs.into_iter().map(BomReference::new).collect()),
            asserter: other.asserter.into(),
            notes: other.notes,
        }
    }
}

const ASSERTION_TYPE_TAG: &str = "assertionType";
const PATENT_REFS_TAG: &str = "patentRefs";
const ASSERTER_TAG: &str = "asserter";
const NOTES_TAG: &str = "notes";

impl ToXml for PatentAssertion {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        let mut start_tag = XmlEvent::start_element(PATENT_ASSERTION_TAG);
        if let Some(bom_ref) = &self.bom_ref {
            start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
        }
        writer
            .write(start_tag)
            .map_err(to_xml_write_error(PATENT_ASSERTION_TAG))?;

        write_simple_tag(writer, ASSERTION_TYPE_TAG, &self.assertion_type)?;

        if let Some(patent_refs) = &self.patent_refs {
            write_list_string_tag(writer, PATENT_REFS_TAG, BOM_REF_ATTR, patent_refs)?;
        }

        self.asserter.write_xml_element(writer)?;

        if let Some(notes) = &self.notes {
            write_simple_tag(writer, NOTES_TAG, notes)?;
        }

        write_close_tag(writer, PATENT_ASSERTION_TAG)
    }
}

impl FromXml for PatentAssertion {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
        let mut assertion_type: Option<String> = None;
        let mut patent_refs: Option<Vec<String>> = None;
        let mut asserter: Option<PatentAsserter> = None;
        let mut notes: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == ASSERTION_TYPE_TAG =>
                {
                    assertion_type = Some(read_simple_tag(event_reader, &name)?);
                }
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == PATENT_REFS_TAG =>
                {
                    patent_refs = Some(read_list_tag(event_reader, &name, BOM_REF_ATTR)?);
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ASSERTER_TAG => {
                    asserter = Some(PatentAsserter::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NOTES_TAG => {
                    notes = Some(read_simple_tag(event_reader, &name)?);
                }
                // lax validation of any elements from a different schema
                reader::XmlEvent::StartElement { name, .. } => {
                    read_lax_validation_tag(event_reader, &name)?
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            bom_ref,
            assertion_type: required(assertion_type, ASSERTION_TYPE_TAG, PATENT_ASSERTION_TAG)?,
            patent_refs,
            asserter: required(asserter, ASSERTER_TAG, PATENT_ASSERTION_TAG)?,
            notes,
        })
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum PatentAsserter {
    Organization(OrganizationalEntity),
    Contact(OrganizationalContact),
    Reference(String),
}

impl<'de> Deserialize<'de> for PatentAsserter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum AsserterJson {
            Reference(String),
            Party(OrganizationOrIndividualJson),
        }

        Ok(match AsserterJson::deserialize(deserializer)? {
            AsserterJson::Reference(reference) => Self::Reference(reference),
            AsserterJson::Party(party) => match OrganizationOrIndividual::from(party) {
                OrganizationOrIndividual::Organization(organization) => {
                    Self::Organization(organization)
                }
                OrganizationOrIndividual::Individual(contact) => Self::Contact(contact),
            },
        })
    }
}

impl From<models::patent::PatentAsserter> for PatentAsserter {
    fn from(other: models::patent::PatentAsserter) -> Self {
        match other {
            models::patent::PatentAsserter::Organization(organization) => {
                Self::Organization(organization.into())
            }
            models::patent::PatentAsserter::Contact(contact) => Self::Contact(contact.into()),
            models::patent::PatentAsserter::Reference(reference) => Self::Reference(reference.0),
        }
    }
}

impl From<PatentAsserter> for models::patent::PatentAsserter {
    fn from(other: PatentAsserter) -> Self {
        match other {
            PatentAsserter::Organization(organization) => Self::Organization(organization.into()),
            PatentAsserter::Contact(contact) => Self::Contact(contact.into()),
            PatentAsserter::Reference(reference) => Self::Reference(BomReference::new(reference)),
        }
    }
}

impl ToXml for PatentAsserter {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        write_start_tag(writer, ASSERTER_TAG)?;
        match self {
            Self::Organization(organization) => {
                organization.write_xml_named_element(writer, ORGANIZATION_TAG)?
            }
            Self::Contact(contact) => contact.write_xml_named_element(writer, CONTACT_TAG)?,
            Self::Reference(reference) => write_simple_tag(writer, REF_TAG, reference)?,
        }
        write_close_tag(writer, ASSERTER_TAG)
    }
}

impl FromXml for PatentAsserter {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut asserter: Option<PatentAsserter> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ORGANIZATION_TAG => {
                    asserter = Some(Self::Organization(OrganizationalEntity::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?));
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == CONTACT_TAG => {
                    asserter = Some(Self::Contact(OrganizationalContact::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?));
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == REF_TAG => {
                    asserter = Some(Self::Reference(read_simple_tag(event_reader, &name)?));
                }
                // lax validation of any elements from a different schema
                reader::XmlEvent::StartElement { name, .. } => {
                    read_lax_validation_tag(event_reader, &name)?
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        required(asserter, "organization|contact|ref", ASSERTER_TAG)
    }
}

/// An item of `definitions.patents`; the JSON schema distinguishes the two by their required
/// fields (`patentNumber` vs. `familyId`), which is what untagged deserialization relies on.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum PatentDefinition {
    Patent(Patent),
    PatentFamily(PatentFamily),
}

impl From<models::patent::PatentDefinition> for PatentDefinition {
    fn from(other: models::patent::PatentDefinition) -> Self {
        match other {
            models::patent::PatentDefinition::Patent(patent) => Self::Patent(patent.into()),
            models::patent::PatentDefinition::PatentFamily(family) => {
                Self::PatentFamily(family.into())
            }
        }
    }
}

impl From<PatentDefinition> for models::patent::PatentDefinition {
    fn from(other: PatentDefinition) -> Self {
        match other {
            PatentDefinition::Patent(patent) => Self::Patent(patent.into()),
            PatentDefinition::PatentFamily(family) => Self::PatentFamily(family.into()),
        }
    }
}

pub(crate) const PATENT_TAG: &str = "patent";
pub(crate) const PATENT_FAMILY_TAG: &str = "patentFamily";

impl ToXml for PatentDefinition {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        match self {
            Self::Patent(patent) => patent.write_xml_element(writer),
            Self::PatentFamily(family) => family.write_xml_element(writer),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Patent {
    #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
    pub(crate) bom_ref: Option<String>,
    pub(crate) patent_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) application_number: Option<String>,
    pub(crate) jurisdiction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) priority_application: Option<PriorityApplication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) publication_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
    #[serde(rename = "abstract", skip_serializing_if = "Option::is_none")]
    pub(crate) abstract_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) filing_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) grant_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) patent_expiration_date: Option<String>,
    pub(crate) patent_legal_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) patent_assignee: Option<Vec<PatentAssignee>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) external_references: Option<ExternalReferences>,
}

impl From<models::patent::Patent> for Patent {
    fn from(other: models::patent::Patent) -> Self {
        Self {
            bom_ref: other.bom_ref.map(|r| r.0),
            patent_number: other.patent_number,
            application_number: other.application_number,
            jurisdiction: other.jurisdiction,
            priority_application: convert_optional(other.priority_application),
            publication_number: other.publication_number,
            title: other.title,
            abstract_text: other.abstract_text,
            filing_date: other.filing_date,
            grant_date: other.grant_date,
            patent_expiration_date: other.patent_expiration_date,
            patent_legal_status: other.patent_legal_status.to_string(),
            patent_assignee: convert_optional_vec(other.patent_assignee),
            external_references: convert_optional(other.external_references),
        }
    }
}

impl From<Patent> for models::patent::Patent {
    fn from(other: Patent) -> Self {
        Self {
            bom_ref: other.bom_ref.map(BomReference::new),
            patent_number: other.patent_number,
            application_number: other.application_number,
            jurisdiction: other.jurisdiction,
            priority_application: convert_optional(other.priority_application),
            publication_number: other.publication_number,
            title: other.title,
            abstract_text: other.abstract_text,
            filing_date: other.filing_date,
            grant_date: other.grant_date,
            patent_expiration_date: other.patent_expiration_date,
            patent_legal_status: models::patent::PatentLegalStatus::new_unchecked(
                other.patent_legal_status,
            ),
            patent_assignee: convert_optional_vec(other.patent_assignee),
            external_references: convert_optional(other.external_references),
        }
    }
}

const PATENT_NUMBER_TAG: &str = "patentNumber";
const PUBLICATION_NUMBER_TAG: &str = "publicationNumber";
const TITLE_TAG: &str = "title";
const ABSTRACT_TAG: &str = "abstract";
const GRANT_DATE_TAG: &str = "grantDate";
const PATENT_EXPIRATION_DATE_TAG: &str = "patentExpirationDate";
const PATENT_LEGAL_STATUS_TAG: &str = "patentLegalStatus";
const PATENT_ASSIGNEE_TAG: &str = "patentAssignee";

impl ToXml for Patent {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        let mut start_tag = XmlEvent::start_element(PATENT_TAG);
        if let Some(bom_ref) = &self.bom_ref {
            start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
        }
        writer
            .write(start_tag)
            .map_err(to_xml_write_error(PATENT_TAG))?;

        write_simple_tag(writer, PATENT_NUMBER_TAG, &self.patent_number)?;
        if let Some(application_number) = &self.application_number {
            write_simple_tag(writer, APPLICATION_NUMBER_TAG, application_number)?;
        }
        write_simple_tag(writer, JURISDICTION_TAG, &self.jurisdiction)?;
        if let Some(priority_application) = &self.priority_application {
            priority_application.write_xml_element(writer)?;
        }
        if let Some(publication_number) = &self.publication_number {
            write_simple_tag(writer, PUBLICATION_NUMBER_TAG, publication_number)?;
        }
        if let Some(title) = &self.title {
            write_simple_tag(writer, TITLE_TAG, title)?;
        }
        if let Some(abstract_text) = &self.abstract_text {
            write_simple_tag(writer, ABSTRACT_TAG, abstract_text)?;
        }
        if let Some(filing_date) = &self.filing_date {
            write_simple_tag(writer, FILING_DATE_TAG, filing_date)?;
        }
        if let Some(grant_date) = &self.grant_date {
            write_simple_tag(writer, GRANT_DATE_TAG, grant_date)?;
        }
        if let Some(patent_expiration_date) = &self.patent_expiration_date {
            write_simple_tag(writer, PATENT_EXPIRATION_DATE_TAG, patent_expiration_date)?;
        }
        write_simple_tag(writer, PATENT_LEGAL_STATUS_TAG, &self.patent_legal_status)?;
        if let Some(patent_assignee) = &self.patent_assignee {
            for assignee in patent_assignee {
                assignee.write_xml_element(writer)?;
            }
        }
        if let Some(external_references) = &self.external_references {
            external_references.write_xml_element(writer)?;
        }

        write_close_tag(writer, PATENT_TAG)
    }
}

impl FromXml for Patent {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
        let mut patent_number: Option<String> = None;
        let mut application_number: Option<String> = None;
        let mut jurisdiction: Option<String> = None;
        let mut priority_application: Option<PriorityApplication> = None;
        let mut publication_number: Option<String> = None;
        let mut title: Option<String> = None;
        let mut abstract_text: Option<String> = None;
        let mut filing_date: Option<String> = None;
        let mut grant_date: Option<String> = None;
        let mut patent_expiration_date: Option<String> = None;
        let mut patent_legal_status: Option<String> = None;
        let mut patent_assignee: Option<Vec<PatentAssignee>> = None;
        let mut external_references: Option<ExternalReferences> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } => match name.local_name.as_str() {
                    PATENT_NUMBER_TAG => {
                        patent_number = Some(read_simple_tag(event_reader, &name)?)
                    }
                    APPLICATION_NUMBER_TAG => {
                        application_number = Some(read_simple_tag(event_reader, &name)?)
                    }
                    JURISDICTION_TAG => jurisdiction = Some(read_simple_tag(event_reader, &name)?),
                    PRIORITY_APPLICATION_TAG => {
                        priority_application = Some(PriorityApplication::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }
                    PUBLICATION_NUMBER_TAG => {
                        publication_number = Some(read_simple_tag(event_reader, &name)?)
                    }
                    TITLE_TAG => title = Some(read_simple_tag(event_reader, &name)?),
                    ABSTRACT_TAG => abstract_text = Some(read_simple_tag(event_reader, &name)?),
                    FILING_DATE_TAG => filing_date = Some(read_simple_tag(event_reader, &name)?),
                    GRANT_DATE_TAG => grant_date = Some(read_simple_tag(event_reader, &name)?),
                    PATENT_EXPIRATION_DATE_TAG => {
                        patent_expiration_date = Some(read_simple_tag(event_reader, &name)?)
                    }
                    PATENT_LEGAL_STATUS_TAG => {
                        patent_legal_status = Some(read_simple_tag(event_reader, &name)?)
                    }
                    PATENT_ASSIGNEE_TAG => patent_assignee.get_or_insert_with(Vec::new).push(
                        PatentAssignee::read_xml_element(event_reader, &name, &attributes)?,
                    ),
                    EXTERNAL_REFERENCES_TAG => {
                        external_references = Some(ExternalReferences::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }
                    // lax validation of any elements from a different schema
                    _ => read_lax_validation_tag(event_reader, &name)?,
                },
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            bom_ref,
            patent_number: required(patent_number, PATENT_NUMBER_TAG, PATENT_TAG)?,
            application_number,
            jurisdiction: required(jurisdiction, JURISDICTION_TAG, PATENT_TAG)?,
            priority_application,
            publication_number,
            title,
            abstract_text,
            filing_date,
            grant_date,
            patent_expiration_date,
            patent_legal_status: required(
                patent_legal_status,
                PATENT_LEGAL_STATUS_TAG,
                PATENT_TAG,
            )?,
            patent_assignee,
            external_references,
        })
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum PatentAssignee {
    Organization(OrganizationalEntity),
    Individual(OrganizationalContact),
}

impl<'de> Deserialize<'de> for PatentAssignee {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(
            match OrganizationOrIndividual::from(OrganizationOrIndividualJson::deserialize(
                deserializer,
            )?) {
                OrganizationOrIndividual::Organization(organization) => {
                    Self::Organization(organization)
                }
                OrganizationOrIndividual::Individual(contact) => Self::Individual(contact),
            },
        )
    }
}

impl From<models::patent::PatentAssignee> for PatentAssignee {
    fn from(other: models::patent::PatentAssignee) -> Self {
        match other {
            models::patent::PatentAssignee::Organization(organization) => {
                Self::Organization(organization.into())
            }
            models::patent::PatentAssignee::Individual(contact) => Self::Individual(contact.into()),
        }
    }
}

impl From<PatentAssignee> for models::patent::PatentAssignee {
    fn from(other: PatentAssignee) -> Self {
        match other {
            PatentAssignee::Organization(organization) => Self::Organization(organization.into()),
            PatentAssignee::Individual(contact) => Self::Individual(contact.into()),
        }
    }
}

impl ToXml for PatentAssignee {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        write_start_tag(writer, PATENT_ASSIGNEE_TAG)?;
        match self {
            Self::Organization(organization) => {
                organization.write_xml_named_element(writer, ORGANIZATION_TAG)?
            }
            Self::Individual(contact) => contact.write_xml_named_element(writer, INDIVIDUAL_TAG)?,
        }
        write_close_tag(writer, PATENT_ASSIGNEE_TAG)
    }
}

impl FromXml for PatentAssignee {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut assignee: Option<PatentAssignee> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ORGANIZATION_TAG => {
                    assignee = Some(Self::Organization(OrganizationalEntity::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?));
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == INDIVIDUAL_TAG => {
                    assignee = Some(Self::Individual(OrganizationalContact::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?));
                }
                // lax validation of any elements from a different schema
                reader::XmlEvent::StartElement { name, .. } => {
                    read_lax_validation_tag(event_reader, &name)?
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        required(assignee, "organization|individual", PATENT_ASSIGNEE_TAG)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PatentFamily {
    #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
    pub(crate) bom_ref: Option<String>,
    pub(crate) family_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) priority_application: Option<PriorityApplication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) members: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) external_references: Option<ExternalReferences>,
}

impl From<models::patent::PatentFamily> for PatentFamily {
    fn from(other: models::patent::PatentFamily) -> Self {
        Self {
            bom_ref: other.bom_ref.map(|r| r.0),
            family_id: other.family_id,
            priority_application: convert_optional(other.priority_application),
            members: other
                .members
                .map(|members| members.into_iter().map(|r| r.0).collect()),
            external_references: convert_optional(other.external_references),
        }
    }
}

impl From<PatentFamily> for models::patent::PatentFamily {
    fn from(other: PatentFamily) -> Self {
        Self {
            bom_ref: other.bom_ref.map(BomReference::new),
            family_id: other.family_id,
            priority_application: convert_optional(other.priority_application),
            members: other
                .members
                .map(|members| members.into_iter().map(BomReference::new).collect()),
            external_references: convert_optional(other.external_references),
        }
    }
}

const FAMILY_ID_TAG: &str = "familyId";
const MEMBERS_TAG: &str = "members";

impl ToXml for PatentFamily {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        let mut start_tag = XmlEvent::start_element(PATENT_FAMILY_TAG);
        if let Some(bom_ref) = &self.bom_ref {
            start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
        }
        writer
            .write(start_tag)
            .map_err(to_xml_write_error(PATENT_FAMILY_TAG))?;

        write_simple_tag(writer, FAMILY_ID_TAG, &self.family_id)?;
        if let Some(priority_application) = &self.priority_application {
            priority_application.write_xml_element(writer)?;
        }
        // The XSD requires at least one `ref` inside `members`.
        if let Some(members) = self.members.as_ref().filter(|m| !m.is_empty()) {
            write_list_string_tag(writer, MEMBERS_TAG, REF_TAG, members)?;
        }
        if let Some(external_references) = &self.external_references {
            external_references.write_xml_element(writer)?;
        }

        write_close_tag(writer, PATENT_FAMILY_TAG)
    }
}

impl FromXml for PatentFamily {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
        let mut family_id: Option<String> = None;
        let mut priority_application: Option<PriorityApplication> = None;
        let mut members: Option<Vec<String>> = None;
        let mut external_references: Option<ExternalReferences> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } => match name.local_name.as_str() {
                    FAMILY_ID_TAG => family_id = Some(read_simple_tag(event_reader, &name)?),
                    PRIORITY_APPLICATION_TAG => {
                        priority_application = Some(PriorityApplication::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }
                    MEMBERS_TAG => members = Some(read_list_tag(event_reader, &name, REF_TAG)?),
                    EXTERNAL_REFERENCES_TAG => {
                        external_references = Some(ExternalReferences::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }
                    // lax validation of any elements from a different schema
                    _ => read_lax_validation_tag(event_reader, &name)?,
                },
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            bom_ref,
            family_id: required(family_id, FAMILY_ID_TAG, PATENT_FAMILY_TAG)?,
            priority_application,
            members,
            external_references,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PriorityApplication {
    pub(crate) application_number: String,
    pub(crate) jurisdiction: String,
    pub(crate) filing_date: String,
}

impl From<models::patent::PriorityApplication> for PriorityApplication {
    fn from(other: models::patent::PriorityApplication) -> Self {
        Self {
            application_number: other.application_number,
            jurisdiction: other.jurisdiction,
            filing_date: other.filing_date,
        }
    }
}

impl From<PriorityApplication> for models::patent::PriorityApplication {
    fn from(other: PriorityApplication) -> Self {
        Self {
            application_number: other.application_number,
            jurisdiction: other.jurisdiction,
            filing_date: other.filing_date,
        }
    }
}

impl ToXml for PriorityApplication {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        write_start_tag(writer, PRIORITY_APPLICATION_TAG)?;
        write_simple_tag(writer, APPLICATION_NUMBER_TAG, &self.application_number)?;
        write_simple_tag(writer, JURISDICTION_TAG, &self.jurisdiction)?;
        write_simple_tag(writer, FILING_DATE_TAG, &self.filing_date)?;
        write_close_tag(writer, PRIORITY_APPLICATION_TAG)
    }
}

impl FromXml for PriorityApplication {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut application_number: Option<String> = None;
        let mut jurisdiction: Option<String> = None;
        let mut filing_date: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                    APPLICATION_NUMBER_TAG => {
                        application_number = Some(read_simple_tag(event_reader, &name)?)
                    }
                    JURISDICTION_TAG => jurisdiction = Some(read_simple_tag(event_reader, &name)?),
                    FILING_DATE_TAG => filing_date = Some(read_simple_tag(event_reader, &name)?),
                    // lax validation of any elements from a different schema
                    _ => read_lax_validation_tag(event_reader, &name)?,
                },
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            application_number: required(
                application_number,
                APPLICATION_NUMBER_TAG,
                PRIORITY_APPLICATION_TAG,
            )?,
            jurisdiction: required(jurisdiction, JURISDICTION_TAG, PRIORITY_APPLICATION_TAG)?,
            filing_date: required(filing_date, FILING_DATE_TAG, PRIORITY_APPLICATION_TAG)?,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        specs::common::organization::test::{
            corresponding_contact, corresponding_entity, example_contact, example_entity,
        },
        xml::test::{read_element_from_string, write_element_to_string},
    };

    pub(crate) fn example_patent_assertions() -> PatentAssertions {
        PatentAssertions(vec![PatentAssertion {
            bom_ref: Some("patent-assertion-1".to_string()),
            assertion_type: "ownership".to_string(),
            patent_refs: Some(vec!["patent-1".to_string()]),
            asserter: PatentAsserter::Reference("org-acme".to_string()),
            notes: Some("notes".to_string()),
        }])
    }

    pub(crate) fn corresponding_patent_assertions() -> models::patent::PatentAssertions {
        models::patent::PatentAssertions(vec![models::patent::PatentAssertion {
            bom_ref: Some(BomReference::new("patent-assertion-1")),
            assertion_type: models::patent::PatentAssertionType::Ownership,
            patent_refs: Some(vec![BomReference::new("patent-1")]),
            asserter: models::patent::PatentAsserter::Reference(BomReference::new("org-acme")),
            notes: Some("notes".to_string()),
        }])
    }

    fn example_priority_application() -> PriorityApplication {
        PriorityApplication {
            application_number: "US1234567890".to_string(),
            jurisdiction: "US".to_string(),
            filing_date: "2021-01-15".to_string(),
        }
    }

    pub(crate) fn example_patent_definitions() -> Vec<PatentDefinition> {
        vec![
            PatentDefinition::Patent(Patent {
                bom_ref: Some("patent-1".to_string()),
                patent_number: "US1234567890".to_string(),
                application_number: Some("12345".to_string()),
                jurisdiction: "US".to_string(),
                priority_application: Some(example_priority_application()),
                publication_number: Some("US-12345".to_string()),
                title: Some("title".to_string()),
                abstract_text: Some("abstract".to_string()),
                filing_date: Some("2021-01-15".to_string()),
                grant_date: Some("2022-06-01".to_string()),
                patent_expiration_date: Some("2042-01-15".to_string()),
                patent_legal_status: "in-force".to_string(),
                patent_assignee: Some(vec![
                    PatentAssignee::Organization(example_entity()),
                    PatentAssignee::Individual(example_contact()),
                ]),
                external_references: None,
            }),
            PatentDefinition::PatentFamily(PatentFamily {
                bom_ref: Some("patent-family-1".to_string()),
                family_id: "PF-2023001".to_string(),
                priority_application: Some(example_priority_application()),
                members: Some(vec!["patent-1".to_string()]),
                external_references: None,
            }),
        ]
    }

    pub(crate) fn corresponding_patent_definitions() -> Vec<models::patent::PatentDefinition> {
        let priority_application = || models::patent::PriorityApplication {
            application_number: "US1234567890".to_string(),
            jurisdiction: "US".to_string(),
            filing_date: "2021-01-15".to_string(),
        };
        vec![
            models::patent::PatentDefinition::Patent(models::patent::Patent {
                bom_ref: Some(BomReference::new("patent-1")),
                patent_number: "US1234567890".to_string(),
                application_number: Some("12345".to_string()),
                jurisdiction: "US".to_string(),
                priority_application: Some(priority_application()),
                publication_number: Some("US-12345".to_string()),
                title: Some("title".to_string()),
                abstract_text: Some("abstract".to_string()),
                filing_date: Some("2021-01-15".to_string()),
                grant_date: Some("2022-06-01".to_string()),
                patent_expiration_date: Some("2042-01-15".to_string()),
                patent_legal_status: models::patent::PatentLegalStatus::InForce,
                patent_assignee: Some(vec![
                    models::patent::PatentAssignee::Organization(corresponding_entity()),
                    models::patent::PatentAssignee::Individual(corresponding_contact()),
                ]),
                external_references: None,
            }),
            models::patent::PatentDefinition::PatentFamily(models::patent::PatentFamily {
                bom_ref: Some(BomReference::new("patent-family-1")),
                family_id: "PF-2023001".to_string(),
                priority_application: Some(priority_application()),
                members: Some(vec![BomReference::new("patent-1")]),
                external_references: None,
            }),
        ]
    }

    #[test]
    fn it_should_convert_patent_definitions_both_ways() {
        let model: Vec<models::patent::PatentDefinition> =
            convert_vec(example_patent_definitions());
        assert_eq!(model, corresponding_patent_definitions());

        let spec: Vec<PatentDefinition> = convert_vec(corresponding_patent_definitions());
        assert_eq!(spec, example_patent_definitions());
    }

    #[test]
    fn it_should_write_xml_patent_assertions() {
        let mut assertions = example_patent_assertions();
        assertions.0.extend([
            PatentAssertion {
                bom_ref: None,
                assertion_type: "license".to_string(),
                patent_refs: None,
                asserter: PatentAsserter::Organization(example_entity()),
                notes: None,
            },
            PatentAssertion {
                bom_ref: None,
                assertion_type: "prior-art".to_string(),
                patent_refs: None,
                asserter: PatentAsserter::Contact(example_contact()),
                notes: None,
            },
        ]);
        insta::assert_snapshot!(write_element_to_string(assertions));
    }

    #[test]
    fn it_should_read_xml_patent_assertions() {
        let input = r#"
<patentAssertions>
  <patentAssertion bom-ref="patent-assertion-1">
    <assertionType>ownership</assertionType>
    <patentRefs>
      <bom-ref>patent-1</bom-ref>
    </patentRefs>
    <asserter>
      <ref>org-acme</ref>
    </asserter>
    <notes>notes</notes>
  </patentAssertion>
</patentAssertions>
"#;
        let actual: PatentAssertions = read_element_from_string(input);
        assert_eq!(actual, example_patent_assertions());
    }

    #[test]
    fn it_should_read_xml_patent_and_family() {
        let patent = r#"
<patent bom-ref="patent-1">
  <patentNumber>US1234567890</patentNumber>
  <applicationNumber>12345</applicationNumber>
  <jurisdiction>US</jurisdiction>
  <priorityApplication>
    <applicationNumber>US1234567890</applicationNumber>
    <jurisdiction>US</jurisdiction>
    <filingDate>2021-01-15</filingDate>
  </priorityApplication>
  <publicationNumber>US-12345</publicationNumber>
  <title>title</title>
  <abstract>abstract</abstract>
  <filingDate>2021-01-15</filingDate>
  <grantDate>2022-06-01</grantDate>
  <patentExpirationDate>2042-01-15</patentExpirationDate>
  <patentLegalStatus>in-force</patentLegalStatus>
  <patentAssignee>
    <organization>
      <name>name</name>
      <url>url</url>
      <contact>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </contact>
    </organization>
  </patentAssignee>
  <patentAssignee>
    <individual>
      <name>name</name>
      <email>email</email>
      <phone>phone</phone>
    </individual>
  </patentAssignee>
</patent>
"#;
        let family = r#"
<patentFamily bom-ref="patent-family-1">
  <familyId>PF-2023001</familyId>
  <priorityApplication>
    <applicationNumber>US1234567890</applicationNumber>
    <jurisdiction>US</jurisdiction>
    <filingDate>2021-01-15</filingDate>
  </priorityApplication>
  <members>
    <ref>patent-1</ref>
  </members>
</patentFamily>
"#;
        let actual = vec![
            PatentDefinition::Patent(read_element_from_string(patent)),
            PatentDefinition::PatentFamily(read_element_from_string(family)),
        ];
        assert_eq!(actual, example_patent_definitions());
    }

    #[test]
    fn it_should_fail_reading_patent_without_legal_status() {
        let input = r#"
<patent>
  <patentNumber>US1234567890</patentNumber>
  <jurisdiction>US</jurisdiction>
</patent>
"#;
        let mut event_reader = xml::EventReader::new_with_config(
            input.as_bytes(),
            xml::ParserConfig::default().trim_whitespace(true),
        );
        event_reader.next().expect("Failed to read start document");
        let result = match event_reader.next().expect("Failed to read start element") {
            reader::XmlEvent::StartElement {
                name, attributes, ..
            } => Patent::read_xml_element(&mut event_reader, &name, &attributes),
            unexpected => panic!("Unexpected event {unexpected:?}"),
        };
        assert!(matches!(
            result,
            Err(XmlReadError::RequiredDataMissing { required_field, .. })
                if required_field == PATENT_LEGAL_STATUS_TAG
        ));
    }

    #[test]
    fn it_should_classify_json_asserter_and_assignee() {
        let asserters: Vec<PatentAsserter> = serde_json::from_str(
            r#"[
                "org-acme",
                { "name": "Partner", "contact": [{ "name": "Sam", "phone": "800" }] },
                { "bom-ref": "jane", "name": "Jane", "email": "jane@example.com" },
                { "name": "Ambiguous" }
            ]"#,
        )
        .expect("Failed to parse asserters");

        assert_eq!(
            asserters,
            vec![
                PatentAsserter::Reference("org-acme".to_string()),
                PatentAsserter::Organization(OrganizationalEntity {
                    bom_ref: None,
                    name: Some("Partner".to_string()),
                    url: None,
                    contact: Some(vec![OrganizationalContact {
                        bom_ref: None,
                        name: Some("Sam".to_string()),
                        email: None,
                        phone: Some("800".to_string()),
                    }]),
                }),
                PatentAsserter::Contact(OrganizationalContact {
                    bom_ref: Some("jane".to_string()),
                    name: Some("Jane".to_string()),
                    email: Some("jane@example.com".to_string()),
                    phone: None,
                }),
                PatentAsserter::Organization(OrganizationalEntity::new("Ambiguous")),
            ]
        );

        let assignee: PatentAssignee =
            serde_json::from_str(r#"{ "name": "Jane", "email": "jane@example.com" }"#)
                .expect("Failed to parse assignee");
        assert!(matches!(assignee, PatentAssignee::Individual(_)));
    }

    #[test]
    fn it_should_distinguish_json_patent_from_family() {
        let definitions: Vec<PatentDefinition> = serde_json::from_str(
            r#"[
                { "patentNumber": "US1", "jurisdiction": "US", "patentLegalStatus": "pending" },
                { "familyId": "PF-1", "members": ["patent-1"] }
            ]"#,
        )
        .expect("Failed to parse patents");

        assert!(matches!(definitions[0], PatentDefinition::Patent(_)));
        assert!(matches!(definitions[1], PatentDefinition::PatentFamily(_)));
    }
}
