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
use xml::writer;

use crate::{
    elem_tag,
    errors::XmlReadError,
    get_elements_lax,
    models::{self, bom::BomReference},
    prelude::DateTime,
    specs::common::signature::Signature,
    utilities::{convert_optional, convert_vec},
    xml::{
        optional_attribute, read_list_tag, to_xml_write_error, write_close_tag,
        write_list_string_tag, write_simple_option_tag, write_simple_tag, write_start_tag, FromXml,
        ToXml, VecElemTag, VecXmlReader,
    },
};

/// bom-1.7.schema.json #properties/citations
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Citations(Vec<Citation>);

impl From<models::citation::Citations> for Citations {
    fn from(other: models::citation::Citations) -> Self {
        Self(convert_vec(other.0))
    }
}

impl From<Citations> for models::citation::Citations {
    fn from(other: Citations) -> Self {
        Self(convert_vec(other.0))
    }
}

const CITATIONS_TAG: &str = "citations";
const CITATION_TAG: &str = "citation";
const BOM_REF_ATTR: &str = "bom-ref";
const POINTERS_TAG: &str = "pointers";
const EXPRESSIONS_TAG: &str = "expressions";
const TIMESTAMP_TAG: &str = "timestamp";
const ATTRIBUTED_TO_TAG: &str = "attributedTo";
const PROCESS_TAG: &str = "process";
const NOTE_TAG: &str = "note";
const SIGNATURE_TAG: &str = "signature";

elem_tag!(PointerTag = "pointer");
elem_tag!(ExpressionTag = "expression");

impl ToXml for Citations {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, CITATIONS_TAG)?;
        for citation in &self.0 {
            citation.write_xml_element(writer)?;
        }
        write_close_tag(writer, CITATIONS_TAG)
    }
}

impl FromXml for Citations {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, CITATION_TAG).map(Self)
    }
}

/// bom-1.7.schema.json #definitions/citation
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Citation {
    #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
    bom_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pointers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expressions: Option<Vec<String>>,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributed_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    process: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<Signature>,
}

impl From<models::citation::Citation> for Citation {
    fn from(other: models::citation::Citation) -> Self {
        Self {
            bom_ref: other.bom_ref.map(|r| r.0),
            pointers: other.pointers,
            expressions: other.expressions,
            timestamp: other.timestamp.0,
            attributed_to: other.attributed_to.map(|r| r.0),
            process: other.process.map(|r| r.0),
            note: other.note,
            signature: convert_optional(other.signature),
        }
    }
}

impl From<Citation> for models::citation::Citation {
    fn from(other: Citation) -> Self {
        Self {
            bom_ref: other.bom_ref.map(BomReference),
            pointers: other.pointers,
            expressions: other.expressions,
            timestamp: DateTime(other.timestamp),
            attributed_to: other.attributed_to.map(BomReference),
            process: other.process.map(BomReference),
            note: other.note,
            signature: convert_optional(other.signature),
        }
    }
}

impl ToXml for Citation {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let mut start = writer::XmlEvent::start_element(CITATION_TAG);
        if let Some(bom_ref) = &self.bom_ref {
            start = start.attr(BOM_REF_ATTR, bom_ref);
        }
        writer
            .write(start)
            .map_err(to_xml_write_error(CITATION_TAG))?;

        if let Some(pointers) = &self.pointers {
            write_list_string_tag(writer, POINTERS_TAG, PointerTag::VALUE, pointers)?;
        }
        if let Some(expressions) = &self.expressions {
            write_list_string_tag(writer, EXPRESSIONS_TAG, ExpressionTag::VALUE, expressions)?;
        }
        write_simple_tag(writer, TIMESTAMP_TAG, &self.timestamp)?;
        write_simple_option_tag(writer, ATTRIBUTED_TO_TAG, &self.attributed_to)?;
        write_simple_option_tag(writer, PROCESS_TAG, &self.process)?;
        write_simple_option_tag(writer, NOTE_TAG, &self.note)?;
        self.signature.write_xml_element(writer)?;

        write_close_tag(writer, CITATION_TAG)
    }
}

impl FromXml for Citation {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);

        get_elements_lax! {
            event_reader, element_name,
            POINTERS_TAG => pointers: VecXmlReader<String, PointerTag>,
            EXPRESSIONS_TAG => expressions: VecXmlReader<String, ExpressionTag>,
            TIMESTAMP_TAG => timestamp: String,
            ATTRIBUTED_TO_TAG => attributed_to: String,
            PROCESS_TAG => process: String,
            NOTE_TAG => note: String,
            SIGNATURE_TAG => signature: Signature,
        };

        Ok(Self {
            bom_ref,
            pointers: pointers.map(Vec::from),
            expressions: expressions.map(Vec::from),
            timestamp: timestamp
                .ok_or_else(|| XmlReadError::required_data_missing(TIMESTAMP_TAG, element_name))?,
            attributed_to,
            process,
            note,
            signature,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::xml::test::{read_element_from_string, write_element_to_string};
    use pretty_assertions::assert_eq;

    pub(crate) fn example_citations() -> Citations {
        Citations(vec![
            Citation {
                bom_ref: Some("citation-1".to_string()),
                pointers: Some(vec![
                    "/components/0/name".to_string(),
                    "/components/0/version".to_string(),
                ]),
                expressions: None,
                timestamp: "2025-05-01T14:00:00Z".to_string(),
                attributed_to: Some("person-1".to_string()),
                process: None,
                note: Some("Manually entered".to_string()),
                signature: None,
            },
            Citation {
                bom_ref: None,
                pointers: None,
                expressions: Some(vec!["$.components[*].licenses[*].license.id".to_string()]),
                timestamp: "2025-05-01T14:05:00Z".to_string(),
                attributed_to: Some("scan-tool-1".to_string()),
                process: Some("task-license-scan".to_string()),
                note: None,
                signature: None,
            },
        ])
    }

    pub(crate) fn corresponding_citations() -> models::citation::Citations {
        models::citation::Citations(vec![
            models::citation::Citation {
                bom_ref: Some(BomReference::new("citation-1")),
                pointers: Some(vec![
                    "/components/0/name".to_string(),
                    "/components/0/version".to_string(),
                ]),
                expressions: None,
                timestamp: DateTime("2025-05-01T14:00:00Z".to_string()),
                attributed_to: Some(BomReference::new("person-1")),
                process: None,
                note: Some("Manually entered".to_string()),
                signature: None,
            },
            models::citation::Citation {
                bom_ref: None,
                pointers: None,
                expressions: Some(vec!["$.components[*].licenses[*].license.id".to_string()]),
                timestamp: DateTime("2025-05-01T14:05:00Z".to_string()),
                attributed_to: Some(BomReference::new("scan-tool-1")),
                process: Some(BomReference::new("task-license-scan")),
                note: None,
                signature: None,
            },
        ])
    }

    #[test]
    fn it_should_convert_to_model() {
        let actual: models::citation::Citations = example_citations().into();
        assert_eq!(actual, corresponding_citations());
    }

    #[test]
    fn it_should_convert_from_model() {
        let actual: Citations = corresponding_citations().into();
        assert_eq!(actual, example_citations());
    }

    #[test]
    fn it_should_round_trip_json() {
        let json = r#"[
  {
    "bom-ref": "citation-1",
    "pointers": ["/components/0/name", "/components/0/version"],
    "timestamp": "2025-05-01T14:00:00Z",
    "attributedTo": "person-1",
    "note": "Manually entered"
  },
  {
    "expressions": ["$.components[*].licenses[*].license.id"],
    "timestamp": "2025-05-01T14:05:00Z",
    "attributedTo": "scan-tool-1",
    "process": "task-license-scan"
  }
]"#;
        let actual: Citations = serde_json::from_str(json).expect("valid citations JSON");
        assert_eq!(actual, example_citations());

        let written = serde_json::to_value(&actual).expect("serializable");
        let expected: serde_json::Value = serde_json::from_str(json).expect("valid JSON");
        assert_eq!(written, expected);
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_citations());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<citations>
  <citation bom-ref="citation-1">
    <pointers>
      <pointer>/components/0/name</pointer>
      <pointer>/components/0/version</pointer>
    </pointers>
    <timestamp>2025-05-01T14:00:00Z</timestamp>
    <attributedTo>person-1</attributedTo>
    <note>Manually entered</note>
  </citation>
  <citation>
    <expressions>
      <expression>$.components[*].licenses[*].license.id</expression>
    </expressions>
    <timestamp>2025-05-01T14:05:00Z</timestamp>
    <attributedTo>scan-tool-1</attributedTo>
    <process>task-license-scan</process>
  </citation>
</citations>
"#;
        let actual: Citations = read_element_from_string(input);
        assert_eq!(actual, example_citations());
    }
}
