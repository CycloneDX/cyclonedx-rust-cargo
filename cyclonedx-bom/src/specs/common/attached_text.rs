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
    errors::XmlWriteError,
    external_models::normalized_string::NormalizedString,
    xml::{closing_tag_or_error, inner_text_or_error, to_xml_read_error, FromXml, ToInnerXml},
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

const CONTENT_TYPE_ATTR: &str = "content-type";
const ENCODING_ATTR: &str = "encoding";

impl ToInnerXml for AttachedText {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut EventWriter<W>,
        tag: &str,
    ) -> Result<(), XmlWriteError> {
        let mut attached_text_tag = XmlEvent::start_element(tag);

        if let Some(content_type) = &self.content_type {
            attached_text_tag = attached_text_tag.attr(CONTENT_TYPE_ATTR, content_type);
        }

        if let Some(encoding) = &self.encoding {
            attached_text_tag = attached_text_tag.attr(ENCODING_ATTR, encoding);
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

impl FromXml for AttachedText {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut content_type: Option<String> = None;
        let mut encoding: Option<String> = None;

        for attribute in attributes {
            match attribute.name.local_name.as_ref() {
                CONTENT_TYPE_ATTR => content_type = Some(attribute.value.clone()),
                ENCODING_ATTR => encoding = Some(attribute.value.clone()),
                _ => (),
            }
        }

        let content = event_reader
            .next()
            .map_err(to_xml_read_error(&element_name.local_name))
            .and_then(inner_text_or_error(&element_name.local_name))?;

        event_reader
            .next()
            .map_err(to_xml_read_error(&element_name.local_name))
            .and_then(closing_tag_or_error(element_name))?;

        Ok(Self {
            content_type,
            encoding,
            content,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::xml::test::{read_element_from_string, write_named_element_to_string};

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

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<text content-type="content type" encoding="encoding">content</text>
"#;
        let actual: AttachedText = read_element_from_string(input);
        let expected = example_attached_text();
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_xml_no_attributes() {
        let input = r#"
<text>content</text>
"#;
        let actual: AttachedText = read_element_from_string(input);
        let expected = AttachedText {
            content_type: None,
            encoding: None,
            content: "content".to_string(),
        };
        assert_eq!(actual, expected);
    }
}
