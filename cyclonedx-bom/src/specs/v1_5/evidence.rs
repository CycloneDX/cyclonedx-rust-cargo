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
    errors::XmlReadError,
    models,
    specs::common::bom_reference::BomReference,
    utilities::{convert_optional, convert_optional_vec, convert_vec},
    xml::{
        optional_attribute, read_lax_validation_list_tag, read_list_tag, read_simple_tag,
        to_xml_read_error, to_xml_write_error, unexpected_element_error, write_close_tag,
        write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Occurrences(Vec<Occurrence>);

const OCCURRENCE_TAG: &str = "occurrence";
const OCCURRENCES_TAG: &str = "occurrences";

impl ToXml for Occurrences {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        self.write_xml_named_element(writer, OCCURRENCES_TAG)
    }
}

impl ToInnerXml for Occurrences {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, tag)?;

        for occurrence in &self.0 {
            occurrence.write_xml_element(writer)?;
        }

        write_close_tag(writer, tag)?;

        Ok(())
    }
}

impl FromXml for Occurrences {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, OCCURRENCE_TAG).map(Occurrences)
    }
}

impl From<Occurrences> for models::component::Occurrences {
    fn from(other: Occurrences) -> Self {
        Self(convert_vec(other.0))
    }
}

impl From<models::component::Occurrences> for Occurrences {
    fn from(other: models::component::Occurrences) -> Self {
        Self(convert_vec(other.0))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Occurrence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bom_ref: Option<String>,
    pub location: String,
}

impl From<Occurrence> for models::component::Occurrence {
    fn from(other: Occurrence) -> Self {
        Self {
            bom_ref: other.bom_ref.map(crate::models::bom::BomReference::new),
            location: other.location,
        }
    }
}

impl From<models::component::Occurrence> for Occurrence {
    fn from(other: models::component::Occurrence) -> Self {
        Self {
            bom_ref: other.bom_ref.map(|s| s.0),
            location: other.location,
        }
    }
}

const BOM_REF_ATTR: &str = "bom-ref";
const LOCATION_TAG: &str = "location";

impl ToXml for Occurrence {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let mut start_tag = xml::writer::XmlEvent::start_element(OCCURRENCE_TAG);

        if let Some(bom_ref) = &self.bom_ref {
            start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
        }

        writer
            .write(start_tag)
            .map_err(to_xml_write_error(OCCURRENCE_TAG))?;

        write_simple_tag(writer, LOCATION_TAG, &self.location)?;

        write_close_tag(writer, OCCURRENCE_TAG)?;

        Ok(())
    }
}

impl FromXml for Occurrence {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
        let mut location: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(OCCURRENCE_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == LOCATION_TAG => {
                    location = Some(read_simple_tag(event_reader, &name)?);
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let location = location
            .ok_or_else(|| XmlReadError::required_data_missing(LOCATION_TAG, &element_name))?;

        Ok(Self { bom_ref, location })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Callstack(pub Frames);

impl From<Callstack> for models::component::Callstack {
    fn from(other: Callstack) -> Self {
        Self(other.0.into())
    }
}

impl From<models::component::Callstack> for Callstack {
    fn from(other: models::component::Callstack) -> Self {
        Self(other.0.into())
    }
}

const FRAMES_TAG: &str = "frames";
const FRAME_TAG: &str = "frame";

impl FromXml for Callstack {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let frames =
            read_lax_validation_list_tag(event_reader, element_name, FRAMES_TAG).map(Frames)?;
        Ok(Self(frames))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Frames(pub Vec<Frame>);

impl From<Frames> for models::component::Frames {
    fn from(other: Frames) -> Self {
        Self(convert_vec(other.0))
    }
}

impl From<models::component::Frames> for Frames {
    fn from(other: models::component::Frames) -> Self {
        Self(convert_vec(other.0))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Frame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) package: Option<String>,
    pub(crate) module: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parameters: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) full_filename: Option<String>,
}

impl From<Frame> for models::component::Frame {
    fn from(other: Frame) -> Self {
        Self {
            package: convert_optional(other.package),
            module: other.module,
            function: convert_optional(other.function),
            parameters: convert_optional_vec(other.parameters),
            line: convert_optional(other.line),
            column: convert_optional(other.column),
            full_filename: convert_optional(other.full_filename),
        }
    }
}

impl From<models::component::Frame> for Frame {
    fn from(other: models::component::Frame) -> Self {
        Self {
            package: convert_optional(other.package),
            module: other.module,
            function: convert_optional(other.function),
            parameters: convert_optional_vec(other.parameters),
            line: convert_optional(other.line),
            column: convert_optional(other.column),
            full_filename: convert_optional(other.full_filename),
        }
    }
}

impl FromXml for Frame {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(FRAME_TAG))?;

            match next_element {
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                _ => (),
            }
        }

        todo!("")
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Identity {
    pub(crate) field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) methods: Option<Methods>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tools: Option<Vec<BomReference>>,
}

impl From<Identity> for models::component::Identity {
    fn from(other: Identity) -> Self {
        Self {
            field: other.field,
            confidence: other
                .confidence
                .map(models::component::ConfidenceScore::new),
            methods: convert_optional(other.methods),
            tools: convert_optional_vec(other.tools),
        }
    }
}

impl From<models::component::Identity> for Identity {
    fn from(other: models::component::Identity) -> Self {
        Self {
            field: other.field,
            confidence: other.confidence.map(|s| s.get()),
            methods: convert_optional(other.methods),
            tools: convert_optional_vec(other.tools),
        }
    }
}

const IDENTITY_TAG: &str = "identity";
const FIELD_TAG: &str = "field";
const CONFIDENCE_TAG: &str = "confidence";
const METHODS_TAG: &str = "methods";
const METHOD_TAG: &str = "methods";
const TECHNIQUE_TAG: &str = "technique";
const VALUE_TAG: &str = "value";
const TOOLS_TAG: &str = "tools";
const TOOL_TAG: &str = "tool";
const REF_ATTR: &str = "ref";

impl ToXml for Identity {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, IDENTITY_TAG)?;

        write_simple_tag(writer, FIELD_TAG, &self.field)?;

        if let Some(confidence) = self.confidence {
            write_simple_tag(writer, CONFIDENCE_TAG, confidence.to_string().as_str())?;
        }

        if let Some(methods) = &self.methods {
            methods.write_xml_element(writer)?;
        }

        if let Some(tool_refs) = &self.tools {
            write_start_tag(writer, TOOLS_TAG)?;

            for tool_ref in tool_refs {
                let start_tag = xml::writer::XmlEvent::start_element(TOOL_TAG)
                    .attr(REF_ATTR, tool_ref.as_ref());

                writer
                    .write(start_tag)
                    .map_err(to_xml_write_error(TOOL_TAG))?;

                write_close_tag(writer, TOOL_TAG)?;
            }

            write_close_tag(writer, TOOLS_TAG)?;
        }

        write_close_tag(writer, IDENTITY_TAG)?;

        Ok(())
    }
}

impl FromXml for Identity {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut field: Option<String> = None;
        let mut confidence: Option<f32> = None;
        let mut methods: Option<Methods> = None;
        let mut tools: Option<Vec<BomReference>> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(OCCURRENCE_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == FIELD_TAG => {
                    field = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                _ => (),
            }
        }

        let field =
            field.ok_or_else(|| XmlReadError::required_data_missing(FIELD_TAG, &element_name))?;

        Ok(Self {
            field,
            confidence,
            methods,
            tools,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Methods(Vec<Method>);

impl From<Methods> for models::component::Methods {
    fn from(other: Methods) -> Self {
        Self(convert_vec(other.0))
    }
}

impl From<models::component::Methods> for Methods {
    fn from(other: models::component::Methods) -> Self {
        Self(convert_vec(other.0))
    }
}

impl ToInnerXml for Methods {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, tag)?;

        for method in &self.0 {
            method.write_xml_element(writer)?;
        }

        write_close_tag(writer, tag)?;

        Ok(())
    }
}

impl ToXml for Methods {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        self.write_xml_named_element(writer, METHODS_TAG)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Method {
    pub(crate) technique: String,
    pub(crate) confidence: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<String>,
}

impl From<Method> for models::component::Method {
    fn from(other: Method) -> Self {
        Self {
            technique: other.technique,
            confidence: models::component::ConfidenceScore::new(other.confidence),
            value: convert_optional(other.value),
        }
    }
}

impl From<models::component::Method> for Method {
    fn from(other: models::component::Method) -> Self {
        Self {
            technique: other.technique,
            confidence: other.confidence.get(),
            value: convert_optional(other.value),
        }
    }
}

impl ToXml for Method {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, METHOD_TAG)?;

        write_simple_tag(writer, TECHNIQUE_TAG, &self.technique)?;

        write_simple_tag(writer, CONFIDENCE_TAG, self.confidence.to_string().as_str())?;

        if let Some(value) = &self.value {
            write_simple_tag(writer, VALUE_TAG, value)?;
        }

        write_close_tag(writer, METHODS_TAG)?;

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;
    use pretty_assertions::assert_eq;

    pub(crate) fn example_occurrences() -> Occurrences {
        Occurrences(vec![Occurrence {
            bom_ref: Some("occurrence-1".to_string()),
            location: "location-1".to_string(),
        }])
    }

    pub(crate) fn corresponding_occurrences() -> models::component::Occurrences {
        models::component::Occurrences(vec![models::component::Occurrence {
            bom_ref: Some(models::bom::BomReference::new("occurrence-1")),
            location: "location-1".to_string(),
        }])
    }

    pub(crate) fn example_callstack() -> Callstack {
        Callstack(Frames(vec![Frame {
            package: Some("package-1".to_string()),
            module: "module-1".to_string(),
            function: Some("function".to_string()),
            parameters: None,
            line: Some(10),
            column: Some(20),
            full_filename: Some("full-filename".to_string()),
        }]))
    }

    pub(crate) fn corresponding_callstack() -> models::component::Callstack {
        models::component::Callstack(models::component::Frames(vec![models::component::Frame {
            package: Some("package-1".to_string()),
            module: "module-1".to_string(),
            function: Some("function".to_string()),
            parameters: None,
            line: Some(10),
            column: Some(20),
            full_filename: Some("full-filename".to_string()),
        }]))
    }

    pub(crate) fn example_identity() -> Identity {
        Identity {
            field: "field".to_string(),
            confidence: Some(0.5),
            methods: Some(Methods(vec![Method {
                technique: "technique-1".to_string(),
                confidence: 0.8,
                value: Some("identity-value".to_string()),
            }])),
            tools: Some(vec![BomReference::new("identity-tool-ref")]),
        }
    }

    pub(crate) fn corresponding_identity() -> models::component::Identity {
        models::component::Identity {
            field: "field".to_string(),
            confidence: Some(models::component::ConfidenceScore::new(0.5)),
            methods: Some(models::component::Methods(vec![
                models::component::Method {
                    technique: "technique-1".to_string(),
                    confidence: models::component::ConfidenceScore::new(0.8),
                    value: Some("identity-value".to_string()),
                },
            ])),
            tools: None,
        }
    }

    #[test]
    fn it_should_read_xml_occurrences() {
        let input = r#"
<occurrences>
  <occurrence bom-ref="d6bf237e-4e11-4713-9f62-56d18d5e2079">
    <location>/path/to/component</location>
  </occurrence>
  <occurrence bom-ref="b574d5d1-e3cf-4dcd-9ba5-f3507eb1b175">
    <location>/another/path/to/component</location>
  </occurrence>
</occurrences>
"#;
        let actual: Occurrences = read_element_from_string(input);
        let expected = Occurrences(vec![
            Occurrence {
                bom_ref: Some("d6bf237e-4e11-4713-9f62-56d18d5e2079".to_string()),
                location: "/path/to/component".to_string(),
            },
            Occurrence {
                bom_ref: Some("b574d5d1-e3cf-4dcd-9ba5-f3507eb1b175".to_string()),
                location: "/another/path/to/component".to_string(),
            },
        ]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_write_xml_occurrences() {
        let xml_output = write_element_to_string(example_occurrences());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_callstack() {
        let input = r#"
<callstack>
  <frames>
    <frame>
      <package>com.apache.logging.log4j.core</package>
      <module>Logger.class</module>
      <function>logMessage</function>
      <parameters>
        <parameter>com.acme.HelloWorld</parameter>
        <parameter>Level.INFO</parameter>
        <parameter>null</parameter>
        <parameter>Hello World</parameter>
      </parameters>
      <line>150</line>
      <column>17</column>
      <fullFilename>full-filename.java</fullFilename>
    </frame>
    <frame>
      <module>HelloWorld.class</module>
      <function>main</function>
      <line>20</line>
      <column>12</column>
      <fullFilename>/path/to/HelloWorld.class</fullFilename>
    </frame>
  </frames>
</callstack>"#;
        let actual: Callstack = read_element_from_string(input);
        let expected = Callstack(Frames(vec![
            Frame {
                package: Some("com.apache.logging.log4j.core".to_string()),
                module: "Logger.class".to_string(),
                function: Some("logMessage".to_string()),
                parameters: Some(vec![
                    "com.acme.HelloWorld".to_string(),
                    "Level.INFO".to_string(),
                    "null".to_string(),
                    "Hello World".to_string(),
                ]),
                line: Some(150),
                column: Some(17),
                full_filename: Some("full-filename.java".to_string()),
            },
            Frame {
                package: None,
                module: "HelloWorld.class".to_string(),
                function: None,
                parameters: None,
                line: Some(20),
                column: Some(12),
                full_filename: Some("/path/to/HelloWorld.class".to_string()),
            },
        ]));
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_write_xml_identity() {
        let xml_output = write_element_to_string(example_identity());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_identity() {
        let input = r#"
<identity>
  <field>purl</field>
  <confidence>1</confidence>
  <methods>
    <method>
      <technique>filename</technique>
      <confidence>0.1</confidence>
      <value>findbugs-project-3.0.0.jar</value>
    </method>
  </methods>
  <tools>
    <tool ref="bom-ref-of-tool-that-performed-analysis"/>
  </tools>
</identity>"#;
        let actual: Identity = read_element_from_string(input);
        let expected = Identity {
            field: "purl".to_string(),
            confidence: Some(1.0),
            methods: Some(Methods(vec![Method {
                technique: "filename".to_string(),
                confidence: 0.1,
                value: Some("findbugs-project-3.0.0.jar".to_string()),
            }])),
            tools: Some(vec![BomReference::new(
                "bom-ref-of-tool-that-performed-analysis",
            )]),
        };
        assert_eq!(actual, expected);
    }
}
