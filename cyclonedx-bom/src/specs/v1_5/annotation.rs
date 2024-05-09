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
use xml::name::OwnedName;
use xml::{reader, writer};

use crate::errors::{BomError, XmlReadError};
use crate::models;
use crate::prelude::DateTime;
use crate::specs::common::organization::{OrganizationalContact, OrganizationalEntity};
use crate::specs::common::service::v1_5::Service;
use crate::specs::common::signature::Signature;
use crate::specs::v1_5::component::Component;
use crate::utilities::{convert_optional, convert_vec, try_convert_vec};
use crate::xml::{
    read_simple_tag, to_xml_read_error, to_xml_write_error, unexpected_element_error,
    write_close_tag, write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
};

/// Represents the `Annotations` field, see https://cyclonedx.org/docs/1.5/json/#annotations.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Annotations(Vec<Annotation>);

impl TryFrom<models::annotation::Annotations> for Annotations {
    type Error = BomError;

    fn try_from(other: models::annotation::Annotations) -> Result<Self, Self::Error> {
        try_convert_vec(other.0).map(Self)
    }
}

impl From<Annotations> for models::annotation::Annotations {
    fn from(other: Annotations) -> Self {
        models::annotation::Annotations(convert_vec(other.0))
    }
}

const ANNOTATIONS_TAG: &str = "annotations";

impl ToXml for Annotations {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, ANNOTATIONS_TAG)?;

        for annotation in &self.0 {
            annotation.write_xml_element(writer)?;
        }

        write_close_tag(writer, ANNOTATIONS_TAG)?;

        Ok(())
    }
}

impl FromXml for Annotations {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut annotations = Vec::new();

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(ANNOTATIONS_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ANNOTATION_TAG => {
                    annotations.push(Annotation::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Annotations(annotations))
    }
}

/// A single annotation.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Annotation {
    /// Optional identifier to reference the annotation elsewhere in the Bom.
    #[serde(skip_serializing_if = "Option::is_none")]
    bom_ref: Option<String>,
    /// A list of BOM references, TODO change to `Subjects`
    subjects: Vec<String>,
    /// The annotator
    annotator: Annotator,
    /// The timestamp when this annotation was created.
    timestamp: String,
    /// The textual content of the annotation.
    text: String,
    /// The optional signature
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<Signature>,
}

impl TryFrom<models::annotation::Annotation> for Annotation {
    type Error = BomError;

    fn try_from(other: models::annotation::Annotation) -> Result<Self, Self::Error> {
        Ok(Self {
            bom_ref: convert_optional(other.bom_ref),
            subjects: convert_vec(other.subjects),
            annotator: other.annotator.try_into()?,
            timestamp: other.timestamp.to_string(),
            text: other.text.clone(),
            signature: convert_optional(other.signature),
        })
    }
}

impl From<Annotation> for models::annotation::Annotation {
    fn from(other: Annotation) -> Self {
        Self {
            bom_ref: convert_optional(other.bom_ref),
            subjects: convert_vec(other.subjects),
            annotator: other.annotator.into(),
            timestamp: DateTime(other.timestamp),
            text: other.text.clone(),
            signature: convert_optional(other.signature),
        }
    }
}

/// Represents the 'Annotator' field, see https://cyclonedx.org/docs/1.5/json/#annotations_items_annotator
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Annotator {
    Organization(OrganizationalEntity),
    Individual(OrganizationalContact),
    Component(Component),
    Service(Service),
}

impl TryFrom<models::annotation::Annotator> for Annotator {
    type Error = BomError;

    fn try_from(other: models::annotation::Annotator) -> Result<Self, Self::Error> {
        match other {
            models::annotation::Annotator::Organization(org) => Ok(Self::Organization(org.into())),
            models::annotation::Annotator::Individual(contact) => {
                Ok(Self::Individual(contact.into()))
            }
            models::annotation::Annotator::Component(component) => {
                component.try_into().map(Self::Component)
            }
            models::annotation::Annotator::Service(service) => Ok(Self::Service(service.into())),
        }
    }
}

impl From<Annotator> for models::annotation::Annotator {
    fn from(other: Annotator) -> Self {
        match other {
            Annotator::Organization(org) => models::annotation::Annotator::Organization(org.into()),
            Annotator::Individual(contact) => {
                models::annotation::Annotator::Individual(contact.into())
            }
            Annotator::Component(component) => {
                models::annotation::Annotator::Component(component.into())
            }
            Annotator::Service(service) => models::annotation::Annotator::Service(service.into()),
        }
    }
}

const ORGANIZATION_TAG: &str = "organization";
const INDIVIDUAL_TAG: &str = "individual";
const COMPONENT_TAG: &str = "component";
const SERVICE_TAG: &str = "service";

impl FromXml for Annotator {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut got_end_tag = false;
        let mut annotator: Option<Annotator> = None;

        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ORGANIZATION_TAG => {
                    annotator = Some(Self::Organization(OrganizationalEntity::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?))
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == INDIVIDUAL_TAG => {
                    annotator = Some(Self::Individual(OrganizationalContact::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?))
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COMPONENT_TAG => {
                    annotator = Some(Self::Component(Component::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?))
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SERVICE_TAG => {
                    annotator = Some(Self::Service(Service::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?))
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let annotator = annotator.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: "organization, individual, component, service".to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(annotator)
    }
}

impl ToXml for Annotator {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, ANNOTATOR_TAG)?;
        match self {
            Annotator::Organization(organization) => {
                organization.write_xml_named_element(writer, ORGANIZATION_TAG)?;
            }
            Annotator::Individual(contact) => {
                contact.write_xml_named_element(writer, INDIVIDUAL_TAG)?;
            }
            Annotator::Component(component) => {
                component.write_xml_element(writer)?;
            }
            Annotator::Service(service) => {
                service.write_xml_element(writer)?;
            }
        }
        write_close_tag(writer, ANNOTATOR_TAG)?;
        Ok(())
    }
}

const ANNOTATION_TAG: &str = "annotation";

impl ToXml for Annotation {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let mut attribute_element = writer::XmlEvent::start_element(ANNOTATION_TAG);
        if let Some(bom_ref) = &self.bom_ref {
            attribute_element = attribute_element.attr("bom-ref", bom_ref);
        }

        writer
            .write(attribute_element)
            .map_err(to_xml_write_error(ANNOTATION_TAG))?;

        if !self.subjects.is_empty() {
            write_start_tag(writer, SUBJECTS_TAG)?;

            for subject in &self.subjects {
                write_simple_tag(writer, SUBJECT_TAG, subject)?;
            }

            write_close_tag(writer, SUBJECTS_TAG)?;
        }

        self.annotator.write_xml_element(writer)?;

        write_simple_tag(writer, TIMESTAMP_TAG, &self.timestamp)?;

        write_simple_tag(writer, TEXT_TAG, &self.text)?;

        if let Some(signature) = &self.signature {
            signature.write_xml_element(writer)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(ANNOTATION_TAG))?;

        Ok(())
    }
}

const SUBJECTS_TAG: &str = "subjects";
const SUBJECT_TAG: &str = "subject";
const ANNOTATOR_TAG: &str = "annotator";
const TIMESTAMP_TAG: &str = "timestamp";
const TEXT_TAG: &str = "text";
const SIGNATURE_TAG: &str = "signature";

fn read_subject<R: std::io::Read>(
    event_reader: &mut xml::EventReader<R>,
    element_name: &OwnedName,
    attributes: &[xml::attribute::OwnedAttribute],
) -> Result<String, XmlReadError> {
    if element_name.local_name.as_str() != SUBJECT_TAG {
        return Err(XmlReadError::UnexpectedElementReadError {
            error: format!("Unexpected tag '{}' found", element_name),
            element: SUBJECT_TAG.to_string(),
        });
    }

    let ref_name = attributes
        .iter()
        .find(|a| a.name.local_name == "ref")
        .map(|a| a.value.clone())
        .ok_or_else(|| XmlReadError::RequiredAttributeMissing {
            attribute: "ref".to_string(),
            element: element_name.local_name.clone(),
        })?;

    let mut got_end_tag = false;
    while !got_end_tag {
        let next_element = event_reader
            .next()
            .map_err(to_xml_read_error(ANNOTATION_TAG))?;

        match next_element {
            reader::XmlEvent::EndElement { name } if &name == element_name => {
                got_end_tag = true;
            }
            unexpected => return Err(unexpected_element_error(element_name, unexpected)),
        }
    }

    Ok(ref_name)
}

fn read_subjects<R: std::io::Read>(
    event_reader: &mut xml::EventReader<R>,
    element_name: &OwnedName,
    _attributes: &[xml::attribute::OwnedAttribute],
) -> Result<Vec<String>, XmlReadError> {
    if element_name.local_name.as_str() != SUBJECTS_TAG {
        return Err(XmlReadError::UnexpectedElementReadError {
            error: format!("Unexpected tag '{}' found", element_name),
            element: SUBJECTS_TAG.to_string(),
        });
    }

    let mut subjects = Vec::new();

    let mut got_end_tag = false;
    while !got_end_tag {
        let next_element = event_reader
            .next()
            .map_err(to_xml_read_error(ANNOTATION_TAG))?;

        match next_element {
            reader::XmlEvent::StartElement {
                name, attributes, ..
            } if name.local_name == SUBJECT_TAG => {
                subjects.push(read_subject(event_reader, &name, &attributes)?);
            }

            reader::XmlEvent::EndElement { name } if &name == element_name => {
                got_end_tag = true;
            }
            unexpected => return Err(unexpected_element_error(element_name, unexpected)),
        }
    }

    Ok(subjects)
}

impl FromXml for Annotation {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        // read the 'bom-ref' attribute from `<annotation>` tag
        let bom_ref = attributes
            .iter()
            .find(|a| a.name.local_name == "bom-ref")
            .map(|a| a.value.clone());

        let mut subjects = Vec::new();
        let mut annotator: Option<Annotator> = None;
        let mut timestamp: Option<String> = None;
        let mut text: Option<String> = None;
        let mut signature: Option<Signature> = None;

        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(ANNOTATION_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SUBJECTS_TAG => {
                    subjects = read_subjects(event_reader, &name, &attributes)?;
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ANNOTATOR_TAG => {
                    annotator = Some(Annotator::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == TIMESTAMP_TAG => {
                    timestamp = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == TEXT_TAG => {
                    text = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SIGNATURE_TAG => {
                    signature = Some(Signature::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let annotator = annotator.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: ANNOTATOR_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        let timestamp = timestamp.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: TIMESTAMP_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        let text = text.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: TEXT_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(Self {
            bom_ref,
            subjects,
            annotator,
            timestamp,
            text,
            signature,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use pretty_assertions::assert_eq;
    use xml::{EventReader, ParserConfig};

    use crate::{
        models,
        specs::{
            common::{
                organization::{
                    test::{example_contact, example_entity},
                    OrganizationalContact, OrganizationalEntity,
                },
                service::v1_5::test::example_service,
                signature::test::example_signature,
            },
            v1_5::component::test::example_component,
        },
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::{read_subject, read_subjects, Annotation, Annotations, Annotator};

    pub(crate) fn example_annotations() -> Annotations {
        Annotations(vec![example_annotation()])
    }

    pub(crate) fn corresponding_annotations() -> models::annotation::Annotations {
        models::annotation::Annotations(vec![corresponding_annotation()])
    }

    pub(crate) fn example_annotation() -> Annotation {
        Annotation {
            bom_ref: Some("annotation-1".to_string()),
            subjects: vec!["subject1".to_string()],
            annotator: example_annotator(),
            timestamp: "timestamp".to_string(),
            text: "Annotation text".to_string(),
            signature: Some(example_signature()),
        }
    }

    fn example_annotator() -> Annotator {
        Annotator::Organization(example_entity())
    }

    pub(crate) fn corresponding_annotation() -> models::annotation::Annotation {
        example_annotation().into()
    }

    fn event_reader<R: std::io::Read>(input: R) -> EventReader<R> {
        EventReader::new_with_config(input, ParserConfig::default().trim_whitespace(true))
    }

    #[test]
    fn it_should_read_subjects() {
        let input = r#"
<subjects>
  <subject ref="component-a"/>
</subjects>
"#;
        let mut event_reader = event_reader(input.as_bytes());
        event_reader.next().expect("Failed to get start document");

        match event_reader.next().expect("Failed to read") {
            xml::reader::XmlEvent::StartElement {
                name, attributes, ..
            } => {
                let subjects = read_subjects(&mut event_reader, &name, &attributes)
                    .expect("Failed to read subjects");
                assert_eq!(vec!["component-a".to_string()], subjects);
            }
            _ => panic!("Should not land here"),
        }
    }

    #[test]
    fn it_should_read_single_subject() {
        let input = r#"<subject ref="component-a" />"#;
        let mut event_reader = event_reader(input.as_bytes());
        let _event = event_reader.next().expect("Failed to get start document");

        match event_reader.next().expect("Failed to get next") {
            xml::reader::XmlEvent::StartElement {
                name, attributes, ..
            } if name.local_name == "subject" => {
                let result = read_subject(&mut event_reader, &name, &attributes);
                assert!(result.is_ok());
                let value = result.expect("Failed to get string");
                assert_eq!("component-a", &value);
            }
            _ => panic!("Unexpected"),
        }
    }

    #[test]
    fn it_should_read_xml_annotator() {
        let input = r#"
<annotator>
  <individual bom-ref="contact">
    <name>Samantha Wright</name>
    <email>samantha.wright@example.com</email>
    <phone>800-555-1212</phone>
  </individual>
</annotator>
"#;
        let actual: Annotator = read_element_from_string(input);
        let expected = Annotator::Individual(OrganizationalContact {
            bom_ref: Some("contact".to_string()),
            name: Some("Samantha Wright".to_string()),
            email: Some("samantha.wright@example.com".to_string()),
            phone: Some("800-555-1212".to_string()),
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_xml_full_annotations() {
        let input = r#"
<annotations>
  <annotation bom-ref="annotation-1">
    <subjects>
      <subject ref="component-a" />
    </subjects>
    <annotator>
      <organization bom-ref="Acme">
        <name>Acme, Inc.</name>
        <url>https://example.com</url>
        <contact bom-ref="contact-1">
          <name>Acme Professional Services</name>
          <email>professional.services@example.com</email>
        </contact>
      </organization>
    </annotator>
    <timestamp>2020-04-07T07:01:00Z</timestamp>
    <text>This is a sample annotation made by an organization</text>
  </annotation>
</annotations>
"#;
        let actual: Annotations = read_element_from_string(input);
        let expected = Annotations(vec![Annotation {
            bom_ref: Some("annotation-1".to_string()),
            subjects: vec!["component-a".to_string()],
            annotator: Annotator::Organization(OrganizationalEntity {
                bom_ref: Some("Acme".to_string()),
                name: Some(String::from("Acme, Inc.")),
                url: Some(vec!["https://example.com".to_string()]),
                contact: Some(vec![OrganizationalContact {
                    bom_ref: Some("contact-1".to_string()),
                    name: Some("Acme Professional Services".to_string()),
                    email: Some("professional.services@example.com".to_string()),
                    phone: None,
                }]),
            }),
            timestamp: "2020-04-07T07:01:00Z".to_string(),
            text: "This is a sample annotation made by an organization".to_string(),
            signature: None,
        }]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_write_xml_full_annotations() {
        let annotations = vec![
            Annotation {
                bom_ref: Some("annotation-1".to_string()),
                subjects: vec!["component-a".to_string()],
                annotator: Annotator::Individual(example_contact()),
                timestamp: "2024-04-07T07:01:00Z".to_string(),
                text: "Contact annotation text".to_string(),
                signature: Some(example_signature()),
            },
            Annotation {
                bom_ref: Some("annotation-2".to_string()),
                subjects: vec!["component-b".to_string()],
                annotator: Annotator::Service(example_service()),
                timestamp: "2024-04-07T07:01:00Z".to_string(),
                text: "Service annotation text".to_string(),
                signature: None,
            },
            Annotation {
                bom_ref: Some("annotation-2".to_string()),
                subjects: vec!["component-b".to_string()],
                annotator: Annotator::Component(example_component()),
                timestamp: "2024-04-07T07:01:00Z".to_string(),
                text: "Component annotation text".to_string(),
                signature: None,
            },
        ];

        let xml_output = write_element_to_string(Annotations(annotations));
        insta::assert_snapshot!(xml_output);
    }
}
