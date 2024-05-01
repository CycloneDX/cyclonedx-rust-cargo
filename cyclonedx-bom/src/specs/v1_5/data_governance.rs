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
use xml::{name::OwnedName, reader};

use crate::{
    errors::XmlReadError,
    models,
    specs::common::organization::{OrganizationalContact, OrganizationalEntity},
    utilities::convert_vec,
    xml::{
        read_list_tag, to_xml_read_error, unexpected_element_error, write_close_tag,
        write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

/// bom-1.5.schema.json #definitions/dataGovernance
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DataGovernance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) custodians: Option<Vec<DataGovernanceResponsibleParty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) stewards: Option<Vec<DataGovernanceResponsibleParty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) owners: Option<Vec<DataGovernanceResponsibleParty>>,
}

impl From<models::data_governance::DataGovernance> for DataGovernance {
    fn from(other: models::data_governance::DataGovernance) -> Self {
        Self {
            custodians: other.custodians.map(convert_vec),
            stewards: other.stewards.map(convert_vec),
            owners: other.owners.map(convert_vec),
        }
    }
}

impl From<DataGovernance> for models::data_governance::DataGovernance {
    fn from(other: DataGovernance) -> Self {
        Self {
            custodians: other.custodians.map(convert_vec),
            stewards: other.stewards.map(convert_vec),
            owners: other.owners.map(convert_vec),
        }
    }
}

const CUSTODIANS_TAG: &str = "custodians";
const CUSTODIAN_TAG: &str = "custodian";
const STEWARDS_TAG: &str = "stewards";
const STEWARD_TAG: &str = "steward";
const OWNERS_TAG: &str = "owners";
const OWNER_TAG: &str = "owner";

impl ToInnerXml for DataGovernance {
    fn write_xml_named_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, tag)?;

        if let Some(owners) = &self.owners {
            write_start_tag(writer, OWNERS_TAG)?;
            for owner in owners {
                write_start_tag(writer, OWNER_TAG)?;
                owner.write_xml_element(writer)?;
                write_close_tag(writer, OWNER_TAG)?;
            }
            write_close_tag(writer, OWNERS_TAG)?;
        }

        if let Some(custodians) = &self.custodians {
            write_start_tag(writer, CUSTODIANS_TAG)?;
            for custodian in custodians {
                write_start_tag(writer, CUSTODIAN_TAG)?;
                custodian.write_xml_element(writer)?;
                write_close_tag(writer, CUSTODIAN_TAG)?;
            }
            write_close_tag(writer, CUSTODIANS_TAG)?;
        }

        if let Some(stewards) = &self.stewards {
            write_start_tag(writer, STEWARDS_TAG)?;
            for steward in stewards {
                write_start_tag(writer, STEWARD_TAG)?;
                steward.write_xml_element(writer)?;
                write_close_tag(writer, STEWARD_TAG)?;
            }
            write_close_tag(writer, STEWARDS_TAG)?;
        }

        write_close_tag(writer, tag)?;

        Ok(())
    }
}

impl FromXml for DataGovernance {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut custodians: Option<Vec<DataGovernanceResponsibleParty>> = None;
        let mut stewards: Option<Vec<DataGovernanceResponsibleParty>> = None;
        let mut owners: Option<Vec<DataGovernanceResponsibleParty>> = None;
        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == CUSTODIANS_TAG =>
                {
                    custodians = Some(read_list_tag(event_reader, &name, CUSTODIAN_TAG)?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == STEWARDS_TAG => {
                    stewards = Some(read_list_tag(event_reader, &name, STEWARD_TAG)?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == OWNERS_TAG => {
                    owners = Some(read_list_tag(event_reader, &name, OWNER_TAG)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self {
            custodians,
            stewards,
            owners,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum DataGovernanceResponsibleParty {
    Organization(OrganizationalEntity),
    Contact(OrganizationalContact),
}

impl From<models::data_governance::DataGovernanceResponsibleParty>
    for DataGovernanceResponsibleParty
{
    fn from(other: models::data_governance::DataGovernanceResponsibleParty) -> Self {
        match other {
            models::data_governance::DataGovernanceResponsibleParty::Organization(organization) => {
                Self::Organization(organization.into())
            }
            models::data_governance::DataGovernanceResponsibleParty::Contact(contact) => {
                Self::Contact(contact.into())
            }
        }
    }
}

impl From<DataGovernanceResponsibleParty>
    for models::data_governance::DataGovernanceResponsibleParty
{
    fn from(other: DataGovernanceResponsibleParty) -> Self {
        match other {
            DataGovernanceResponsibleParty::Organization(organization) => {
                Self::Organization(organization.into())
            }
            DataGovernanceResponsibleParty::Contact(contact) => Self::Contact(contact.into()),
        }
    }
}

const ORGANIZATION_TAG: &str = "organization";
const CONTACT_TAG: &str = "contact";

impl ToXml for DataGovernanceResponsibleParty {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        match self {
            DataGovernanceResponsibleParty::Organization(organization) => {
                organization.write_xml_named_element(writer, ORGANIZATION_TAG)?;
            }
            DataGovernanceResponsibleParty::Contact(contact) => {
                contact.write_xml_named_element(writer, CONTACT_TAG)?;
            }
        }

        Ok(())
    }
}

impl FromXml for DataGovernanceResponsibleParty {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut party: Option<DataGovernanceResponsibleParty> = None;
        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ORGANIZATION_TAG => {
                    let organization =
                        OrganizationalEntity::read_xml_element(event_reader, &name, &attributes)?;
                    party = Some(DataGovernanceResponsibleParty::Organization(organization));
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == CONTACT_TAG => {
                    let contact =
                        OrganizationalContact::read_xml_element(event_reader, &name, &attributes)?;
                    party = Some(DataGovernanceResponsibleParty::Contact(contact));
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let party = party.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: "organization or contact".to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(party)
    }
}
