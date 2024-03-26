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
    errors::XmlReadError,
    external_models::normalized_string::NormalizedString,
    specs::common::hash::Hashes,
    utilities::convert_vec,
    xml::{
        read_lax_validation_tag, read_list_tag, read_simple_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_simple_tag, FromXml, ToXml,
    },
};
use crate::{models, utilities::convert_optional};
use serde::{Deserialize, Serialize};
use xml::{reader, writer};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Tools(Vec<Tool>);

impl From<models::tool::Tools> for Tools {
    fn from(other: models::tool::Tools) -> Self {
        match other {
            models::tool::Tools::List(tools) => Tools(convert_vec(tools)),
            models::tool::Tools::Object { .. } => Tools(vec![]),
        }
    }
}

impl From<Tools> for models::tool::Tools {
    fn from(other: Tools) -> Self {
        models::tool::Tools::List(convert_vec(other.0))
    }
}

const TOOLS_TAG: &str = "tools";

impl ToXml for Tools {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(TOOLS_TAG))
            .map_err(to_xml_write_error(TOOLS_TAG))?;

        for tool in &self.0 {
            tool.write_xml_element(writer)?;
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
        read_list_tag(event_reader, element_name, TOOL_TAG).map(Tools)
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
}

impl From<models::tool::Tool> for Tool {
    fn from(other: models::tool::Tool) -> Self {
        Self {
            vendor: other.vendor.map(|v| v.to_string()),
            name: other.name.map(|n| n.to_string()),
            version: other.version.map(|v| v.to_string()),
            hashes: convert_optional(other.hashes),
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

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(TOOL_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == VENDOR_TAG => {
                    vendor = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    tool_name = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == VERSION_TAG => {
                    version = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == HASHES_TAG => {
                    hashes = Some(Hashes::read_xml_element(event_reader, &name, &attributes)?)
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
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        specs::common::hash::test::{corresponding_hashes, example_hashes},
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::*;

    pub(crate) fn example_tools() -> Tools {
        Tools(vec![example_tool()])
    }

    pub(crate) fn corresponding_tools() -> models::tool::Tools {
        models::tool::Tools::List(vec![corresponding_tool()])
    }

    pub(crate) fn example_tool() -> Tool {
        Tool {
            vendor: Some("vendor".to_string()),
            name: Some("name".to_string()),
            version: Some("version".to_string()),
            hashes: Some(example_hashes()),
        }
    }

    pub(crate) fn corresponding_tool() -> models::tool::Tool {
        models::tool::Tool {
            vendor: Some(NormalizedString::new_unchecked("vendor".to_string())),
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            version: Some(NormalizedString::new_unchecked("version".to_string())),
            hashes: Some(corresponding_hashes()),
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
  </tool>
</tools>
"#;
        let actual: Tools = read_element_from_string(input);
        let expected = example_tools();
        assert_eq!(actual, expected);
    }
}
