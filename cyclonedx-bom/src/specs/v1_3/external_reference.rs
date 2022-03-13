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
    external_models::uri::Uri,
    xml::{write_simple_tag, ToXml},
};
use crate::{
    models,
    utilities::{convert_optional, convert_vec},
};
use crate::{specs::v1_3::hash::Hashes, xml::to_xml_write_error};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct ExternalReferences(Vec<ExternalReference>);

impl From<models::external_reference::ExternalReferences> for ExternalReferences {
    fn from(other: models::external_reference::ExternalReferences) -> Self {
        ExternalReferences(convert_vec(other.0))
    }
}

impl From<ExternalReferences> for models::external_reference::ExternalReferences {
    fn from(other: ExternalReferences) -> Self {
        models::external_reference::ExternalReferences(convert_vec(other.0))
    }
}

const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";

impl ToXml for ExternalReferences {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(EXTERNAL_REFERENCES_TAG))
            .map_err(to_xml_write_error(EXTERNAL_REFERENCES_TAG))?;

        for external_reference in &self.0 {
            external_reference.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(EXTERNAL_REFERENCES_TAG))?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExternalReference {
    #[serde(rename = "type")]
    external_reference_type: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hashes: Option<Hashes>,
}

impl From<models::external_reference::ExternalReference> for ExternalReference {
    fn from(other: models::external_reference::ExternalReference) -> Self {
        Self {
            external_reference_type: other.external_reference_type.to_string(),
            url: other.url.to_string(),
            comment: other.comment,
            hashes: convert_optional(other.hashes),
        }
    }
}

impl From<ExternalReference> for models::external_reference::ExternalReference {
    fn from(other: ExternalReference) -> Self {
        Self {
            external_reference_type:
                models::external_reference::ExternalReferenceType::new_unchecked(
                    other.external_reference_type,
                ),
            url: Uri(other.url),
            comment: other.comment,
            hashes: convert_optional(other.hashes),
        }
    }
}

const REFERENCE_TAG: &str = "reference";
const TYPE_ATTR: &str = "type";
const URL_TAG: &str = "url";
const COMMENT_TAG: &str = "comment";

impl ToXml for ExternalReference {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(
                XmlEvent::start_element(REFERENCE_TAG)
                    .attr(TYPE_ATTR, &self.external_reference_type),
            )
            .map_err(to_xml_write_error(REFERENCE_TAG))?;

        write_simple_tag(writer, URL_TAG, &self.url)?;

        if let Some(comment) = &self.comment {
            write_simple_tag(writer, COMMENT_TAG, comment)?;
        }

        if let Some(hashes) = &self.hashes {
            hashes.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(REFERENCE_TAG))?;

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::{
        specs::v1_3::hash::test::{corresponding_hashes, example_hashes},
        xml::test::write_element_to_string,
    };

    pub(crate) fn example_external_references() -> ExternalReferences {
        ExternalReferences(vec![example_external_reference()])
    }

    pub(crate) fn corresponding_external_references(
    ) -> models::external_reference::ExternalReferences {
        models::external_reference::ExternalReferences(vec![corresponding_external_reference()])
    }

    pub(crate) fn example_external_reference() -> ExternalReference {
        ExternalReference {
            external_reference_type: "external reference type".to_string(),
            url: "url".to_string(),
            comment: Some("comment".to_string()),
            hashes: Some(example_hashes()),
        }
    }

    pub(crate) fn corresponding_external_reference() -> models::external_reference::ExternalReference
    {
        models::external_reference::ExternalReference {
            external_reference_type:
                models::external_reference::ExternalReferenceType::UnknownExternalReferenceType(
                    "external reference type".to_string(),
                ),
            url: Uri("url".to_string()),
            comment: Some("comment".to_string()),
            hashes: Some(corresponding_hashes()),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_external_references());
        insta::assert_snapshot!(xml_output);
    }
}
