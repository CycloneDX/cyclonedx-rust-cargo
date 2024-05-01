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
use xml::reader;

use crate::{
    errors::XmlReadError,
    external_models::uri::Uri,
    prelude::NormalizedString,
    utilities::convert_optional,
    xml::{
        optional_attribute, read_list_tag, to_xml_read_error, to_xml_write_error, write_close_tag,
        write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

use crate::models;
use crate::specs::v1_5::{data_governance::DataGovernance, service::DataClassification};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServiceData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(flatten)]
    pub(crate) classification: DataClassification,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) governance: Option<DataGovernance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) destination: Option<Vec<String>>,
}

impl From<models::service::ServiceData> for ServiceData {
    fn from(other: models::service::ServiceData) -> Self {
        Self {
            name: other.name.map(|n| n.0),
            description: other.description.map(|d| d.0),
            classification: other.classification.into(),
            governance: convert_optional(other.governance),
            source: other
                .source
                .map(|uris| uris.into_iter().map(|url| url.0).collect()),
            destination: other
                .destination
                .map(|uris| uris.into_iter().map(|url| url.0).collect()),
        }
    }
}

impl From<ServiceData> for models::service::ServiceData {
    fn from(other: ServiceData) -> Self {
        Self {
            name: other.name.map(NormalizedString::new_unchecked),
            description: other.description.map(NormalizedString::new_unchecked),
            classification: other.classification.into(),
            governance: convert_optional(other.governance),
            source: other.source.map(|uris| uris.into_iter().map(Uri).collect()),
            destination: other
                .destination
                .map(|uris| uris.into_iter().map(Uri).collect()),
        }
    }
}

const NAME_ATTR: &str = "name";
const DESCRIPTION_ATTR: &str = "description";
const DATAFLOW_TAG: &str = "dataflow";
const CLASSIFICATION_TAG: &str = "classification";
const GOVERNANCE_TAG: &str = "governance";
const SOURCE_TAG: &str = "source";
const DESTINATION_TAG: &str = "destination";
const URL_TAG: &str = "url";

impl FromXml for ServiceData {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let name = optional_attribute(attributes, NAME_ATTR);
        let description = optional_attribute(attributes, DESCRIPTION_ATTR);
        let mut classification: Option<DataClassification> = None;
        let mut governance: Option<DataGovernance> = None;
        let mut source: Option<Vec<String>> = None;
        let mut destination: Option<Vec<String>> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == CLASSIFICATION_TAG => {
                    classification = Some(DataClassification::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == GOVERNANCE_TAG => {
                    governance = Some(DataGovernance::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == SOURCE_TAG => {
                    source = Some(read_list_tag(event_reader, &name, URL_TAG)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == DESTINATION_TAG =>
                {
                    destination = Some(read_list_tag(event_reader, &name, URL_TAG)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                _ => (),
            }
        }

        let classification = classification.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: CLASSIFICATION_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(Self {
            name,
            description,
            classification,
            governance,
            source,
            destination,
        })
    }
}

impl ToXml for ServiceData {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let mut start_tag = xml::writer::XmlEvent::start_element(DATAFLOW_TAG);

        if let Some(name) = &self.name {
            start_tag = start_tag.attr(NAME_ATTR, name);
        }

        if let Some(description) = &self.description {
            start_tag = start_tag.attr(NAME_ATTR, description);
        }

        writer
            .write(start_tag)
            .map_err(to_xml_write_error(DATAFLOW_TAG))?;

        self.classification.write_xml_element(writer)?;

        if let Some(governance) = &self.governance {
            governance.write_xml_named_element(writer, GOVERNANCE_TAG)?;
        }

        if let Some(uris) = &self.source {
            write_start_tag(writer, SOURCE_TAG)?;
            for uri in uris {
                write_simple_tag(writer, URL_TAG, uri.as_str())?;
            }
            write_close_tag(writer, SOURCE_TAG)?;
        }

        if let Some(uris) = &self.destination {
            write_start_tag(writer, DESTINATION_TAG)?;
            for uri in uris {
                write_simple_tag(writer, URL_TAG, uri.as_str())?;
            }
            write_close_tag(writer, DESTINATION_TAG)?;
        }

        write_close_tag(writer, DATAFLOW_TAG)?;

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        specs::{
            common::{
                organization::{OrganizationalContact, OrganizationalEntity},
                service::v1_5::{Data, DataClassification},
            },
            v1_5::{
                data_governance::{DataGovernance, DataGovernanceResponsibleParty},
                service_data::ServiceData,
            },
        },
        xml::test::{read_element_from_string, write_element_to_string},
    };

    #[test]
    fn it_should_write_xml_15_service_data() {
        let actual = Data::ServiceData(vec![ServiceData {
            name: Some("Consumer to Price".to_string()),
            description: Some("Consumer to Price description".to_string()),
            classification: DataClassification {
                flow: "data flow".to_string(),
                classification: "bi-directional".to_string(),
            },
            governance: Some(DataGovernance {
                owners: Some(vec![DataGovernanceResponsibleParty::Contact(
                    OrganizationalContact {
                        bom_ref: Some("owner-1".to_string()),
                        name: Some("owner".to_string()),
                        email: None,
                        phone: None,
                    },
                )]),
                custodians: None,
                stewards: None,
            }),
            source: None,
            destination: None,
        }]);
        insta::assert_snapshot!(write_element_to_string(actual));
    }

    #[test]
    fn it_should_read_xml_service_data() {
        let input = r#"
<dataflow name="Consumer to Stock Service" description="Traffic to/from consumer to service">
  <classification flow="bi-directional">Customer</classification>
  <governance>
    <owners>
      <owner>
        <organization>
          <name>Customer Name</name>
        </organization>
      </owner>
    </owners>
  </governance>
  <source>
    <url>https://0.0.0.0</url>
  </source>
  <destination>
    <url>https://0.0.0.0</url>
  </destination>
</dataflow>"#;
        let actual: ServiceData = read_element_from_string(input);
        let expected = ServiceData {
            name: Some("Consumer to Stock Service".to_string()),
            description: Some("Traffic to/from consumer to service".to_string()),
            classification: DataClassification {
                flow: "bi-directional".to_string(),
                classification: "Customer".to_string(),
            },
            governance: Some(DataGovernance {
                owners: Some(vec![DataGovernanceResponsibleParty::Organization(
                    OrganizationalEntity {
                        bom_ref: None,
                        name: Some("Customer Name".to_string()),
                        url: None,
                        contact: None,
                    },
                )]),
                custodians: None,
                stewards: None,
            }),
            source: Some(vec!["https://0.0.0.0".to_string()]),
            destination: Some(vec!["https://0.0.0.0".to_string()]),
        };
        assert_eq!(actual, expected);
    }
}
