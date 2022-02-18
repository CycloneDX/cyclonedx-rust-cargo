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
    external_models::normalized_string::NormalizedString,
    specs::v1_3::hash::Hashes,
    utilities::convert_vec,
    xml::{to_xml_write_error, write_simple_tag, ToXml},
};
use crate::{models, utilities::convert_optional};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Tools(Vec<Tool>);

impl From<models::Tools> for Tools {
    fn from(other: models::Tools) -> Self {
        Tools(convert_vec(other.0))
    }
}

impl From<Tools> for models::Tools {
    fn from(other: Tools) -> Self {
        models::Tools(convert_vec(other.0))
    }
}

const TOOLS_TAG: &str = "tools";

impl ToXml for Tools {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(TOOLS_TAG))
            .map_err(to_xml_write_error(TOOLS_TAG))?;

        for tool in &self.0 {
            tool.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(TOOLS_TAG))?;
        Ok(())
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

impl From<models::Tool> for Tool {
    fn from(other: models::Tool) -> Self {
        Self {
            vendor: other.vendor.map(|v| v.to_string()),
            name: other.name.map(|n| n.to_string()),
            version: other.version.map(|v| v.to_string()),
            hashes: convert_optional(other.hashes),
        }
    }
}

impl From<Tool> for models::Tool {
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
            .write(XmlEvent::start_element(TOOL_TAG))
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
            .write(XmlEvent::end_element())
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

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        specs::v1_3::hash::test::{corresponding_hashes, example_hashes},
        xml::test::write_element_to_string,
    };

    use super::*;

    pub(crate) fn example_tools() -> Tools {
        Tools(vec![example_tool()])
    }

    pub(crate) fn corresponding_tools() -> models::Tools {
        models::Tools(vec![corresponding_tool()])
    }

    pub(crate) fn example_tool() -> Tool {
        Tool {
            vendor: Some("vendor".to_string()),
            name: Some("name".to_string()),
            version: Some("version".to_string()),
            hashes: Some(example_hashes()),
        }
    }

    pub(crate) fn corresponding_tool() -> models::Tool {
        models::Tool {
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
}
