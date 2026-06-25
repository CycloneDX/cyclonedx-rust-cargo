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
use xml::{
    name::OwnedName,
    reader::{self},
    writer,
};

use crate::{
    errors::XmlReadError,
    models,
    utilities::convert_optional,
    xml::{
        optional_attribute, to_xml_read_error, to_xml_write_error, write_close_tag, FromXml,
        ToInnerXml,
    },
};

/// bom-1.5.schema.json #definitions/attachment
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Attachment {
    pub(crate) content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) encoding: Option<String>,
}

impl From<models::attachment::Attachment> for Attachment {
    fn from(other: models::attachment::Attachment) -> Self {
        Self {
            content: other.content,
            content_type: convert_optional(other.content_type),
            encoding: convert_optional(other.encoding),
        }
    }
}

impl From<Attachment> for models::attachment::Attachment {
    fn from(other: Attachment) -> Self {
        Self {
            content: other.content,
            content_type: convert_optional(other.content_type),
            encoding: convert_optional(other.encoding),
        }
    }
}

const ENCODING_ATTR: &str = "encoding";
const CONTENT_TYPE_ATTR: &str = "content-type";

impl ToInnerXml for Attachment {
    fn write_xml_named_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let mut start_tag = writer::XmlEvent::start_element(tag);
        if let Some(encoding) = &self.encoding {
            start_tag = start_tag.attr(ENCODING_ATTR, encoding);
        }
        if let Some(content_type) = &self.content_type {
            start_tag = start_tag.attr(ENCODING_ATTR, content_type);
        }
        writer.write(start_tag).map_err(to_xml_write_error(tag))?;

        writer
            .write(writer::XmlEvent::characters(&self.content))
            .map_err(to_xml_write_error(tag))?;

        write_close_tag(writer, tag)?;

        Ok(())
    }
}

impl FromXml for Attachment {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let content_type: Option<String> = optional_attribute(attributes, CONTENT_TYPE_ATTR);
        let encoding: Option<String> = optional_attribute(attributes, ENCODING_ATTR);
        let mut content: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::Characters(image_content) => {
                    content = Some(image_content);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        let content = content.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: "inner characters".to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(Self {
            content,
            content_type,
            encoding,
        })
    }
}
