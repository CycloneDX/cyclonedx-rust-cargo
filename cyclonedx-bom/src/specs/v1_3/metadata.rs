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

use crate::errors::BomError;
use crate::{
    external_models::date_time::DateTime,
    models,
    specs::common::{
        organization::OrganizationalContact, organization::OrganizationalEntity,
        property::Properties,
    },
    specs::v1_3::{component::Component, license::Licenses, tool::Tools},
    utilities::{convert_optional, convert_optional_vec, try_convert_optional},
    xml::{
        read_lax_validation_tag, read_list_tag, read_simple_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_simple_tag, FromXml, ToInnerXml, ToXml,
    },
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use xml::{reader, writer::XmlEvent};

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

impl TryFrom<models::metadata::Metadata> for Metadata {
    type Error = BomError;

    fn try_from(other: models::metadata::Metadata) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: other.timestamp.map(|t| t.to_string()),
            tools: convert_optional(other.tools),
            authors: convert_optional_vec(other.authors),
            component: try_convert_optional(other.component)?,
            manufacture: convert_optional(other.manufacture),
            supplier: convert_optional(other.supplier),
            licenses: convert_optional(other.licenses),
            properties: convert_optional(other.properties),
        })
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
            lifecycles: None,
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

const TOOLS_TAG: &str = "tools";
const COMPONENT_TAG: &str = "component";
const LICENSES_TAG: &str = "licenses";
const PROPERTIES_TAG: &str = "properties";

impl FromXml for Metadata {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut timestamp: Option<String> = None;
        let mut tools: Option<Tools> = None;
        let mut authors: Option<Vec<OrganizationalContact>> = None;
        let mut component: Option<Component> = None;
        let mut manufacture: Option<OrganizationalEntity> = None;
        let mut supplier: Option<OrganizationalEntity> = None;
        let mut licenses: Option<Licenses> = None;
        let mut properties: Option<Properties> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(METADATA_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == TIMESTAMP_TAG => {
                    timestamp = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == TOOLS_TAG => {
                    tools = Some(Tools::read_xml_element(event_reader, &name, &attributes)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == AUTHORS_TAG => {
                    authors = Some(read_list_tag(event_reader, &name, AUTHOR_TAG)?);
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COMPONENT_TAG => {
                    component = Some(Component::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == MANUFACTURE_TAG => {
                    manufacture = Some(OrganizationalEntity::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SUPPLIER_TAG => {
                    supplier = Some(OrganizationalEntity::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == LICENSES_TAG => {
                    licenses = Some(Licenses::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PROPERTIES_TAG => {
                    properties = Some(Properties::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
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
            timestamp,
            tools,
            authors,
            component,
            manufacture,
            supplier,
            licenses,
            properties,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        specs::common::{
            organization::test::{
                corresponding_contact, corresponding_entity, example_contact, example_entity,
            },
            property::test::{corresponding_properties, example_properties},
        },
        specs::v1_3::{
            component::test::{corresponding_component, example_component},
            license::test::{corresponding_licenses, example_licenses},
            tool::test::{corresponding_tools, example_tools},
        },
        xml::test::{read_element_from_string, write_element_to_string},
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
            lifecycles: None,
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_metadata());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<metadata>
  <timestamp>timestamp</timestamp>
  <tools>
    <tool>
      <vendor>vendor</vendor>
      <name>name</name>
      <version>version</version>
      <hashes>
        <hash alg="algorithm">hash value</hash>
      </hashes>
    </tool>
  </tools>
  <authors>
    <author>
      <name>name</name>
      <email>email</email>
      <phone>phone</phone>
    </author>
  </authors>
  <component type="component type" mime-type="mime type" bom-ref="bom ref">
    <supplier>
      <name>name</name>
      <url>url</url>
      <contact>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </contact>
    </supplier>
    <author>author</author>
    <publisher>publisher</publisher>
    <group>group</group>
    <name>name</name>
    <version>version</version>
    <description>description</description>
    <scope>scope</scope>
    <hashes>
      <hash alg="algorithm">hash value</hash>
    </hashes>
    <licenses>
      <expression>expression</expression>
    </licenses>
    <copyright>copyright</copyright>
    <cpe>cpe</cpe>
    <purl>purl</purl>
    <swid tagId="tag id" name="name" version="version" tagVersion="1" patch="true">
      <text content-type="content type" encoding="encoding">content</text>
      <url>url</url>
    </swid>
    <modified>true</modified>
    <pedigree>
      <ancestors />
      <descendants />
      <variants />
      <commits>
        <commit>
          <uid>uid</uid>
          <url>url</url>
          <author>
            <timestamp>timestamp</timestamp>
            <name>name</name>
            <email>email</email>
          </author>
          <committer>
            <timestamp>timestamp</timestamp>
            <name>name</name>
            <email>email</email>
          </committer>
          <message>message</message>
        </commit>
      </commits>
      <patches>
        <patch type="patch type">
          <diff>
            <text content-type="content type" encoding="encoding">content</text>
            <url>url</url>
          </diff>
          <resolves>
            <issue type="issue type">
              <id>id</id>
              <name>name</name>
              <description>description</description>
              <source>
                <name>name</name>
                <url>url</url>
              </source>
              <references>
                <url>reference</url>
              </references>
            </issue>
          </resolves>
        </patch>
      </patches>
      <notes>notes</notes>
    </pedigree>
    <externalReferences>
      <reference type="external reference type">
        <url>url</url>
        <comment>comment</comment>
        <hashes>
          <hash alg="algorithm">hash value</hash>
        </hashes>
      </reference>
    </externalReferences>
    <properties>
      <property name="name">value</property>
    </properties>
    <components />
    <evidence>
      <licenses>
        <expression>expression</expression>
      </licenses>
      <copyright>
        <text><![CDATA[copyright]]></text>
      </copyright>
    </evidence>
  </component>
  <manufacture>
    <name>name</name>
    <url>url</url>
    <contact>
      <name>name</name>
      <email>email</email>
      <phone>phone</phone>
    </contact>
  </manufacture>
  <supplier>
    <name>name</name>
    <url>url</url>
    <contact>
      <name>name</name>
      <email>email</email>
      <phone>phone</phone>
    </contact>
  </supplier>
  <licenses>
    <expression>expression</expression>
  </licenses>
  <properties>
    <property name="name">value</property>
  </properties>
</metadata>
"#;
        let actual: Metadata = read_element_from_string(input);
        let expected = example_metadata();
        assert_eq!(actual, expected);
    }
}
