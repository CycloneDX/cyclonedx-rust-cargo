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

use cyclonedx_bom_macros::versioned;

#[versioned("1.3", "1.4", "1.5")]
pub(crate) mod base {
    #[versioned("1.4")]
    use crate::specs::v1_4::external_reference::ExternalReferences;
    #[versioned("1.5")]
    use crate::specs::v1_5::{
        component::Components, external_reference::ExternalReferences, service::Services,
    };

    use crate::models;
    use crate::{
        errors::{BomError, XmlReadError},
        external_models::normalized_string::NormalizedString,
        specs::common::hash::Hashes,
        utilities::{convert_optional, convert_vec},
        xml::{
            read_lax_validation_tag, read_simple_tag, to_xml_read_error, to_xml_write_error,
            unexpected_element_error, write_simple_tag, FromXml, ToXml,
        },
    };
    use serde::{Deserialize, Serialize};
    use xml::{reader, writer};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase", untagged)]
    pub(crate) enum Tools {
        /// Legacy version: https://cyclonedx.org/docs/1.4/json/#metadata_tools
        List(Vec<Tool>),
        /// Added in 1.5, see https://cyclonedx.org/docs/1.5/json/#metadata_tools
        #[versioned("1.5")]
        Object {
            #[serde(skip_serializing_if = "Option::is_none")]
            services: Option<Services>,
            #[serde(skip_serializing_if = "Option::is_none")]
            components: Option<Components>,
        },
    }

    impl TryFrom<models::tool::Tools> for Tools {
        type Error = BomError;

        fn try_from(other: models::tool::Tools) -> Result<Self, Self::Error> {
            match other {
                models::tool::Tools::List(tools) => Ok(Self::List(convert_vec(tools))),
                #[versioned("1.3", "1.4")]
                models::tool::Tools::Object { .. } => Ok(Self::List(vec![])),
                #[versioned("1.5")]
                models::tool::Tools::Object {
                    services,
                    components,
                } => Ok(Self::Object {
                    services: convert_optional(services),
                    components: crate::utilities::try_convert_optional(components)?,
                }),
            }
        }
    }

    impl From<Tools> for models::tool::Tools {
        fn from(other: Tools) -> Self {
            match other {
                Tools::List(tools) => models::tool::Tools::List(convert_vec(tools)),
                #[versioned("1.5")]
                Tools::Object {
                    services,
                    components,
                } => Self::Object {
                    services: convert_optional(services),
                    components: convert_optional(components),
                },
            }
        }
    }

    const TOOLS_TAG: &str = "tools";
    #[versioned("1.5")]
    const COMPONENTS_TAG: &str = "components";
    #[versioned("1.5")]
    const SERVICES_TAG: &str = "services";

    impl ToXml for Tools {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            writer
                .write(writer::XmlEvent::start_element(TOOLS_TAG))
                .map_err(to_xml_write_error(TOOLS_TAG))?;

            match self {
                Tools::List(tools) => {
                    for tool in tools {
                        tool.write_xml_element(writer)?;
                    }
                }
                #[versioned("1.5")]
                Tools::Object {
                    services,
                    components,
                } => {
                    services.write_xml_element(writer)?;
                    components.write_xml_element(writer)?;
                }
            }

            writer
                .write(writer::XmlEvent::end_element())
                .map_err(to_xml_write_error(TOOLS_TAG))?;
            Ok(())
        }
    }

    impl FromXml for Tools {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            _attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let mut tools: Option<Vec<Tool>> = None;

            #[versioned("1.5")]
            let mut components: Option<Components> = None;
            #[versioned("1.5")]
            let mut services: Option<Services> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader.next().map_err(to_xml_read_error(TOOL_TAG))?;

                match next_element {
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == TOOL_TAG => {
                        tools.get_or_insert(Vec::new()).push(Tool::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }

                    #[versioned("1.5")]
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == COMPONENTS_TAG => {
                        components = Some(Components::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }

                    #[versioned("1.5")]
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == SERVICES_TAG => {
                        services = Some(Services::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }

                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => {
                        return Err(unexpected_element_error(element_name, unexpected));
                    }
                }
            }

            match tools {
                Some(tools) => {
                    #[versioned("1.5")]
                    if components.is_some() || services.is_some() {
                        return Err(XmlReadError::RequiredDataMissing {
                            required_field: "tool array or services & components".to_string(),
                            element: element_name.local_name.to_string(),
                        });
                    }

                    Ok(Self::List(tools))
                }
                #[versioned("1.3", "1.4")]
                None => Ok(Self::List(vec![])),
                #[versioned("1.5")]
                None => Ok(Self::Object {
                    services,
                    components,
                }),
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Tool {
        #[serde(skip_serializing_if = "Option::is_none")]
        vendor: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hashes: Option<Hashes>,
        #[versioned("1.4", "1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        external_references: Option<ExternalReferences>,
    }

    impl From<models::tool::Tool> for Tool {
        fn from(other: models::tool::Tool) -> Self {
            Self {
                vendor: other.vendor.map(|v| v.to_string()),
                name: other.name.map(|n| n.to_string()),
                version: other.version.map(|v| v.to_string()),
                hashes: convert_optional(other.hashes),
                #[versioned("1.4", "1.5")]
                external_references: convert_optional(other.external_references),
            }
        }
    }

    impl From<Tool> for models::tool::Tool {
        fn from(other: Tool) -> Self {
            Self {
                vendor: other.vendor.map(NormalizedString::new_unchecked),
                name: other.name.map(NormalizedString::new_unchecked),
                version: other.version.map(NormalizedString::new_unchecked),
                hashes: convert_optional(other.hashes),
                #[versioned("1.3")]
                external_references: None,
                #[versioned("1.4", "1.5")]
                external_references: convert_optional(other.external_references),
            }
        }
    }

    const TOOL_TAG: &str = "tool";
    const VENDOR_TAG: &str = "vendor";
    const NAME_TAG: &str = "name";
    const VERSION_TAG: &str = "version";

    impl ToXml for Tool {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            writer
                .write(writer::XmlEvent::start_element(TOOL_TAG))
                .map_err(to_xml_write_error(TOOL_TAG))?;

            if let Some(vendor) = &self.vendor {
                write_simple_tag(writer, VENDOR_TAG, vendor)?;
            }

            if let Some(name) = &self.name {
                write_simple_tag(writer, NAME_TAG, name)?;
            }

            if let Some(version) = &self.version {
                write_simple_tag(writer, VERSION_TAG, version)?;
            }

            if let Some(hashes) = &self.hashes {
                if hashes.will_write() {
                    hashes.write_xml_element(writer)?;
                }
            }

            #[versioned("1.4", "1.5")]
            if let Some(external_references) = &self.external_references {
                external_references.write_xml_element(writer)?;
            }

            writer
                .write(writer::XmlEvent::end_element())
                .map_err(to_xml_write_error(TOOL_TAG))?;

            Ok(())
        }

        fn will_write(&self) -> bool {
            self.vendor.is_some()
                || self.name.is_some()
                || self.version.is_some()
                || self.hashes.is_some()
        }
    }

    const HASHES_TAG: &str = "hashes";
    #[versioned("1.4", "1.5")]
    const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";

    impl FromXml for Tool {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            _attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let mut vendor: Option<String> = None;
            let mut tool_name: Option<String> = None;
            let mut version: Option<String> = None;
            let mut hashes: Option<Hashes> = None;
            #[versioned("1.4", "1.5")]
            let mut external_references: Option<ExternalReferences> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader.next().map_err(to_xml_read_error(TOOL_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == VENDOR_TAG =>
                    {
                        vendor = Some(read_simple_tag(event_reader, &name)?)
                    }
                    reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                        tool_name = Some(read_simple_tag(event_reader, &name)?)
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == VERSION_TAG =>
                    {
                        version = Some(read_simple_tag(event_reader, &name)?)
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == HASHES_TAG => {
                        hashes = Some(Hashes::read_xml_element(event_reader, &name, &attributes)?)
                    }
                    #[versioned("1.4", "1.5")]
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == EXTERNAL_REFERENCES_TAG => {
                        external_references = Some(ExternalReferences::read_xml_element(
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
                vendor,
                name: tool_name,
                version,
                hashes,
                #[versioned("1.4", "1.5")]
                external_references,
            })
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        #[versioned("1.4")]
        use crate::specs::common::external_reference::v1_4::test::{
            corresponding_external_references, example_external_references,
        };
        #[versioned("1.5")]
        use crate::specs::{
            common::{
                external_reference::v1_5::test::{
                    corresponding_external_references, example_external_references,
                },
                hash::{Hash, HashValue},
                organization::OrganizationalEntity,
            },
            v1_5::{
                component::{Component, Components},
                service::{Service, Services},
            },
        };

        use crate::{
            specs::common::hash::test::{corresponding_hashes, example_hashes},
            xml::test::{read_element_from_string, write_element_to_string},
        };

        use super::*;
        use pretty_assertions::assert_eq;

        pub(crate) fn example_tools() -> Tools {
            Tools::List(vec![example_tool()])
        }

        pub(crate) fn corresponding_tools() -> models::tool::Tools {
            models::tool::Tools::List(vec![corresponding_tool()])
        }

        #[versioned("1.3")]
        pub(crate) fn example_tool() -> Tool {
            Tool {
                vendor: Some("vendor".to_string()),
                name: Some("name".to_string()),
                version: Some("version".to_string()),
                hashes: Some(example_hashes()),
            }
        }

        #[versioned("1.3")]
        pub(crate) fn corresponding_tool() -> models::tool::Tool {
            models::tool::Tool {
                vendor: Some(NormalizedString::new_unchecked("vendor".to_string())),
                name: Some(NormalizedString::new_unchecked("name".to_string())),
                version: Some(NormalizedString::new_unchecked("version".to_string())),
                hashes: Some(corresponding_hashes()),
                external_references: None,
            }
        }

        #[versioned("1.4", "1.5")]
        pub(crate) fn example_tool() -> Tool {
            Tool {
                vendor: Some("vendor".to_string()),
                name: Some("name".to_string()),
                version: Some("version".to_string()),
                hashes: Some(example_hashes()),
                external_references: Some(example_external_references()),
            }
        }

        #[versioned("1.4", "1.5")]
        pub(crate) fn corresponding_tool() -> models::tool::Tool {
            models::tool::Tool {
                vendor: Some(NormalizedString::new_unchecked("vendor".to_string())),
                name: Some(NormalizedString::new_unchecked("name".to_string())),
                version: Some(NormalizedString::new_unchecked("version".to_string())),
                hashes: Some(corresponding_hashes()),
                external_references: Some(corresponding_external_references()),
            }
        }

        #[test]
        fn it_should_write_xml_full() {
            let xml_output = write_element_to_string(example_tools());
            insta::assert_snapshot!(xml_output);
        }

        #[test]
        fn it_should_read_xml_full() {
            let input = r#"
<tools>
  <tool>
    <vendor>vendor</vendor>
    <name>name</name>
    <version>version</version>
    <hashes>
      <hash alg="algorithm">hash value</hash>
    </hashes>
    <externalReferences>
      <reference type="external reference type">
        <url>url</url>
        <comment>comment</comment>
        <hashes>
          <hash alg="algorithm">hash value</hash>
        </hashes>
      </reference>
    </externalReferences>
  </tool>
</tools>
"#;
            let actual: Tools = read_element_from_string(input);
            let expected = example_tools();
            assert_eq!(actual, expected);
        }

        #[test]
        #[versioned("1.5")]
        fn it_should_read_xml_with_services_and_components() {
            let input = r#"
<tools>
  <components>
    <component type="application">
      <group>Awesome Vendor</group>
      <name>Awesome Tool</name>
      <version>9.1.2</version>
      <hashes>
        <hash alg="SHA-1">abcdefgh</hash>
      </hashes>
    </component>
  </components>
  <services>
    <service>
      <provider>
        <name>Acme Org</name>
        <url>https://example.com</url>
      </provider>
      <group>com.example</group>
      <name>Acme Signing Server</name>
      <description>Signs artifacts</description>
    </service>
  </services>
</tools>
"#;
            let actual: Tools = read_element_from_string(input);
            let service = Service {
                bom_ref: None,
                provider: Some(OrganizationalEntity {
                    bom_ref: None,
                    name: Some("Acme Org".to_string()),
                    url: Some(vec!["https://example.com".to_string()]),
                    contact: None,
                }),
                group: Some("com.example".to_string()),
                name: "Acme Signing Server".to_string(),
                version: None,
                description: Some("Signs artifacts".to_string()),
                endpoints: None,
                authenticated: None,
                x_trust_boundary: None,
                data: None,
                licenses: None,
                external_references: None,
                properties: None,
                services: None,
                signature: None,
                trust_zone: None,
            };
            let component = Component {
                component_type: "application".to_string(),
                mime_type: None,
                bom_ref: None,
                supplier: None,
                author: None,
                publisher: None,
                group: Some("Awesome Vendor".to_string()),
                name: "Awesome Tool".to_string(),
                version: Some("9.1.2".to_string()),
                description: None,
                scope: None,
                hashes: Some(Hashes(vec![Hash {
                    alg: "SHA-1".to_string(),
                    content: HashValue("abcdefgh".to_string()),
                }])),
                licenses: None,
                copyright: None,
                cpe: None,
                purl: None,
                swid: None,
                modified: None,
                pedigree: None,
                external_references: None,
                properties: None,
                components: None,
                evidence: None,
                signature: None,
                model_card: None,
                data: None,
            };
            let expected = Tools::Object {
                services: Some(Services(vec![service])),
                components: Some(Components(vec![component])),
            };
            assert_eq!(actual, expected);
        }
    }
}
