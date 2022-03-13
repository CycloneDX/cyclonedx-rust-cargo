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
    errors::XmlWriteError, external_models::normalized_string::NormalizedString, xml::ToInnerXml,
};
use crate::{models, xml::to_xml_write_error};
use serde::{Deserialize, Serialize};
use xml::writer::{EventWriter, XmlEvent};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttachedText {
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding: Option<String>,
    content: String,
}

impl From<models::attached_text::AttachedText> for AttachedText {
    fn from(other: models::attached_text::AttachedText) -> Self {
        Self {
            content_type: other.content_type.map(|n| n.0),
            encoding: other.encoding.map(|e| e.to_string()),
            content: other.content,
        }
    }
}

impl From<AttachedText> for models::attached_text::AttachedText {
    fn from(other: AttachedText) -> Self {
        Self {
            content_type: other.content_type.map(NormalizedString::new_unchecked),
            encoding: other
                .encoding
                .map(models::attached_text::Encoding::new_unchecked),
            content: other.content,
        }
    }
}

impl ToInnerXml for AttachedText {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut EventWriter<W>,
        tag: &str,
    ) -> Result<(), XmlWriteError> {
        let mut attached_text_tag = XmlEvent::start_element(tag);

        if let Some(content_type) = &self.content_type {
            attached_text_tag = attached_text_tag.attr("content-type", content_type);
        }

        if let Some(encoding) = &self.encoding {
            attached_text_tag = attached_text_tag.attr("encoding", encoding);
        }
        writer
            .write(attached_text_tag)
            .map_err(to_xml_write_error(tag))?;

        writer
            .write(XmlEvent::characters(&self.content))
            .map_err(to_xml_write_error(tag))?;
        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(tag))?;

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::xml::test::write_named_element_to_string;

    pub(crate) fn example_attached_text() -> AttachedText {
        AttachedText {
            content_type: Some("content type".to_string()),
            encoding: Some("encoding".to_string()),
            content: "content".to_string(),
        }
    }

    pub(crate) fn corresponding_attached_text() -> models::attached_text::AttachedText {
        models::attached_text::AttachedText {
            content_type: Some(NormalizedString::new_unchecked("content type".to_string())),
            encoding: Some(models::attached_text::Encoding::UnknownEncoding(
                "encoding".to_string(),
            )),
            content: "content".to_string(),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_named_element_to_string(example_attached_text(), "text");
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_write_xml_no_attributes() {
        let xml_output = write_named_element_to_string(
            AttachedText {
                content_type: None,
                encoding: None,
                content: "content".to_string(),
            },
            "text",
        );
        insta::assert_snapshot!(xml_output);
    }
}
