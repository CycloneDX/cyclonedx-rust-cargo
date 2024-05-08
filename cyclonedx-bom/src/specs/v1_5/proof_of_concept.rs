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
use xml::reader;

use crate::{
    models,
    utilities::{convert_optional, convert_vec},
    xml::{
        read_list_tag, read_simple_tag, to_xml_read_error, write_close_tag, write_simple_tag,
        write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

/// Represents the `ProofOfConcept` field.
/// See https://cyclonedx.org/docs/1.5/json/#vulnerabilities_items_proofOfConcept
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProofOfConcept {
    #[serde(skip_serializing_if = "Option::is_none")]
    reproduction_steps: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supporting_material: Option<Vec<crate::specs::v1_5::attachment::Attachment>>,
}

impl From<ProofOfConcept> for models::vulnerability::VulnerabilityProofOfConcept {
    fn from(other: ProofOfConcept) -> Self {
        Self {
            reproduction_steps: convert_optional(other.reproduction_steps),
            environment: convert_optional(other.environment),
            supporting_material: other.supporting_material.map(convert_vec),
        }
    }
}

impl From<models::vulnerability::VulnerabilityProofOfConcept> for ProofOfConcept {
    fn from(other: models::vulnerability::VulnerabilityProofOfConcept) -> Self {
        Self {
            reproduction_steps: convert_optional(other.reproduction_steps),
            environment: convert_optional(other.environment),
            supporting_material: other.supporting_material.map(convert_vec),
        }
    }
}

const PROOF_OF_CONCEPT_TAG: &str = "proofOfConcept";
const REPRODUCTION_STEPS_TAG: &str = "reproductionSteps";
const ENVIRONMENT_TAG: &str = "environment";
const SUPPORTING_MATERIAL_TAG: &str = "supportingMaterial";
const ATTACHMENT_TAG: &str = "attachment";

impl ToXml for ProofOfConcept {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, PROOF_OF_CONCEPT_TAG)?;

        if let Some(steps) = &self.reproduction_steps {
            write_simple_tag(writer, REPRODUCTION_STEPS_TAG, steps)?;
        }

        if let Some(environment) = &self.environment {
            write_simple_tag(writer, ENVIRONMENT_TAG, environment)?;
        }

        if let Some(attachments) = &self.supporting_material {
            write_start_tag(writer, SUPPORTING_MATERIAL_TAG)?;

            for attachment in attachments {
                attachment.write_xml_named_element(writer, ATTACHMENT_TAG)?;
            }

            write_close_tag(writer, SUPPORTING_MATERIAL_TAG)?;
        }

        write_close_tag(writer, PROOF_OF_CONCEPT_TAG)?;

        Ok(())
    }
}

impl FromXml for ProofOfConcept {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut reproduction_steps: Option<String> = None;
        let mut environment: Option<String> = None;
        let mut supporting_material: Option<Vec<crate::specs::v1_5::attachment::Attachment>> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == REPRODUCTION_STEPS_TAG =>
                {
                    reproduction_steps = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == ENVIRONMENT_TAG =>
                {
                    environment = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == SUPPORTING_MATERIAL_TAG =>
                {
                    supporting_material = Some(read_list_tag(event_reader, &name, ATTACHMENT_TAG)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                _ => (),
            }
        }

        Ok(Self {
            reproduction_steps,
            environment,
            supporting_material,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        models,
        specs::v1_5::attachment::Attachment,
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::ProofOfConcept;

    pub(crate) fn example_proof_of_concept() -> ProofOfConcept {
        ProofOfConcept {
            reproduction_steps: Some("reproduction steps".to_string()),
            environment: Some("production".to_string()),
            supporting_material: Some(vec![Attachment {
                content: "abcdefgh".to_string(),
                content_type: Some("image/jpeg".to_string()),
                encoding: Some("base64".to_string()),
            }]),
        }
    }

    pub(crate) fn corresponding_proof_of_concept(
    ) -> models::vulnerability::VulnerabilityProofOfConcept {
        models::vulnerability::VulnerabilityProofOfConcept {
            reproduction_steps: Some("reproduction steps".to_string()),
            environment: Some("production".to_string()),
            supporting_material: Some(vec![models::attachment::Attachment {
                content: "abcdefgh".to_string(),
                content_type: Some("image/jpeg".to_string()),
                encoding: Some("base64".to_string()),
            }]),
        }
    }

    #[test]
    fn it_should_write_xml() {
        let xml_output = write_element_to_string(example_proof_of_concept());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml() {
        let input = r#"
<proofOfConcept>
  <reproductionSteps>Reproduction Steps</reproductionSteps>
  <environment>Describe the environment</environment>
  <supportingMaterial>
    <attachment content-type="image/jpeg" encoding="base64">abcdfgh</attachment>
  </supportingMaterial>
</proofOfConcept>
        "#;
        let actual: ProofOfConcept = read_element_from_string(input);
        let expected = ProofOfConcept {
            reproduction_steps: Some("Reproduction Steps".to_string()),
            environment: Some("Describe the environment".to_string()),
            supporting_material: Some(vec![Attachment {
                content: "abcdfgh".to_string(),
                content_type: Some("image/jpeg".to_string()),
                encoding: Some("base64".to_string()),
            }]),
        };
        assert_eq!(actual, expected);
    }
}
