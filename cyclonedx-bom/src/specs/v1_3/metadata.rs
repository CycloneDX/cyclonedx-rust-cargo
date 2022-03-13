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
    specs::v1_3::{
        component::Component, license::Licenses, organization::OrganizationalContact,
        organization::OrganizationalEntity, property::Properties, tool::Tools,
    },
    xml::{to_xml_write_error, write_simple_tag, ToInnerXml, ToXml},
};
use crate::{
    models,
    utilities::{convert_optional, convert_optional_vec},
};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Tools>,
    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<OrganizationalContact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manufacture: Option<OrganizationalEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supplier: Option<OrganizationalEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    licenses: Option<Licenses>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
}

impl From<models::metadata::Metadata> for Metadata {
    fn from(other: models::metadata::Metadata) -> Self {
        Self {
            timestamp: other.timestamp.map(|t| t.to_string()),
            tools: convert_optional(other.tools),
            authors: convert_optional_vec(other.authors),
            component: convert_optional(other.component),
            manufacture: convert_optional(other.manufacture),
            supplier: convert_optional(other.supplier),
            licenses: convert_optional(other.licenses),
            properties: convert_optional(other.properties),
        }
    }
}

impl From<Metadata> for models::metadata::Metadata {
    fn from(other: Metadata) -> Self {
        Self {
            timestamp: other.timestamp.map(DateTime),
            tools: convert_optional(other.tools),
            authors: convert_optional_vec(other.authors),
            component: convert_optional(other.component),
            manufacture: convert_optional(other.manufacture),
            supplier: convert_optional(other.supplier),
            licenses: convert_optional(other.licenses),
            properties: convert_optional(other.properties),
        }
    }
}

const METADATA_TAG: &str = "metadata";
const TIMESTAMP_TAG: &str = "timestamp";
const AUTHORS_TAG: &str = "authors";
const AUTHOR_TAG: &str = "author";
const MANUFACTURE_TAG: &str = "manufacture";
const SUPPLIER_TAG: &str = "supplier";

impl ToXml for Metadata {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(METADATA_TAG))
            .map_err(to_xml_write_error(METADATA_TAG))?;

        if let Some(timestamp) = &self.timestamp {
            write_simple_tag(writer, TIMESTAMP_TAG, timestamp)?;
        }

        if let Some(tools) = &self.tools {
            tools.write_xml_element(writer)?;
        }

        if let Some(authors) = &self.authors {
            writer
                .write(XmlEvent::start_element(AUTHORS_TAG))
                .map_err(to_xml_write_error(AUTHORS_TAG))?;

            for author in authors {
                if author.will_write() {
                    author.write_xml_named_element(writer, AUTHOR_TAG)?;
                }
            }

            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(AUTHORS_TAG))?;
        }

        if let Some(component) = &self.component {
            component.write_xml_element(writer)?;
        }

        if let Some(manufacture) = &self.manufacture {
            manufacture.write_xml_named_element(writer, MANUFACTURE_TAG)?
        }

        if let Some(supplier) = &self.supplier {
            supplier.write_xml_named_element(writer, SUPPLIER_TAG)?;
        }

        if let Some(licenses) = &self.licenses {
            licenses.write_xml_element(writer)?;
        }

        if let Some(properties) = &self.properties {
            properties.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(METADATA_TAG))?;

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.timestamp.is_some()
            || self.tools.is_some()
            || self.authors.is_some()
            || self.component.is_some()
            || self.manufacture.is_some()
            || self.supplier.is_some()
            || self.licenses.is_some()
            || self.properties.is_some()
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        specs::v1_3::{
            component::test::{corresponding_component, example_component},
            license::test::{corresponding_licenses, example_licenses},
            organization::test::{
                corresponding_contact, corresponding_entity, example_contact, example_entity,
            },
            property::test::{corresponding_properties, example_properties},
            tool::test::{corresponding_tools, example_tools},
        },
        xml::test::write_element_to_string,
    };

    use super::*;

    pub(crate) fn example_metadata() -> Metadata {
        Metadata {
            timestamp: Some("timestamp".to_string()),
            tools: Some(example_tools()),
            authors: Some(vec![example_contact()]),
            component: Some(example_component()),
            manufacture: Some(example_entity()),
            supplier: Some(example_entity()),
            licenses: Some(example_licenses()),
            properties: Some(example_properties()),
        }
    }

    pub(crate) fn corresponding_metadata() -> models::metadata::Metadata {
        models::metadata::Metadata {
            timestamp: Some(DateTime("timestamp".to_string())),
            tools: Some(corresponding_tools()),
            authors: Some(vec![corresponding_contact()]),
            component: Some(corresponding_component()),
            manufacture: Some(corresponding_entity()),
            supplier: Some(corresponding_entity()),
            licenses: Some(corresponding_licenses()),
            properties: Some(corresponding_properties()),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_metadata());
        insta::assert_snapshot!(xml_output);
    }
}
