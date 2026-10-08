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

use cyclonedx_bom_macros::versioned;

#[versioned("1.6", "1.7")]
pub(crate) mod base {
    use serde::{Deserialize, Serialize};
    use xml::{reader, writer::XmlEvent};

    #[versioned("1.6")]
    use crate::specs::v1_6::external_reference::ExternalReferences;
    #[versioned("1.7")]
    use crate::specs::v1_7::{
        external_reference::ExternalReferences,
        patent::{Patent, PatentDefinition, PatentFamily, PATENT_FAMILY_TAG, PATENT_TAG},
    };
    #[versioned("1.7")]
    use crate::utilities::convert_optional_vec;
    use crate::{
        errors::{XmlReadError, XmlWriteError},
        models::{self, bom::BomReference},
        specs::common::{property::Properties, signature::Signature},
        utilities::convert_optional,
        xml::{
            optional_attribute, read_lax_validation_list_tag, read_lax_validation_tag,
            read_list_tag, read_simple_tag, to_xml_read_error, to_xml_write_error,
            unexpected_element_error, write_close_tag, write_list_string_tag, write_simple_tag,
            write_start_tag, FromXml, ToXml,
        },
    };

    const BOM_REF_ATTR: &str = "bom-ref";
    const IDENTIFIER_TAG: &str = "identifier";
    const TITLE_TAG: &str = "title";
    const DESCRIPTION_TAG: &str = "description";
    const REQUIREMENTS_TAG: &str = "requirements";
    const REQUIREMENT_TAG: &str = "requirement";
    const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Definitions {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) standards: Option<Vec<Standard>>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) patents: Option<Vec<PatentDefinition>>,
    }

    impl From<models::definitions::Definitions> for Definitions {
        fn from(other: models::definitions::Definitions) -> Self {
            Self {
                standards: other
                    .standards
                    .map(|standards| standards.into_iter().map(Into::into).collect()),
                #[versioned("1.7")]
                patents: convert_optional_vec(other.patents),
            }
        }
    }

    impl From<Definitions> for models::definitions::Definitions {
        fn from(other: Definitions) -> Self {
            Self {
                standards: other
                    .standards
                    .map(|standards| standards.into_iter().map(Into::into).collect()),
                #[versioned("1.6")]
                patents: None,
                #[versioned("1.7")]
                patents: convert_optional_vec(other.patents),
            }
        }
    }

    const DEFINITIONS_TAG: &str = "definitions";
    const STANDARDS_TAG: &str = "standards";
    const STANDARD_TAG: &str = "standard";
    #[versioned("1.7")]
    const PATENTS_TAG: &str = "patents";

    impl ToXml for Definitions {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, DEFINITIONS_TAG)?;

            if let Some(standards) = &self.standards {
                write_start_tag(writer, STANDARDS_TAG)?;
                for standard in standards {
                    standard.write_xml_element(writer)?;
                }
                write_close_tag(writer, STANDARDS_TAG)?;
            }

            #[versioned("1.7")]
            if let Some(patents) = &self.patents {
                write_start_tag(writer, PATENTS_TAG)?;
                for patent in patents {
                    patent.write_xml_element(writer)?;
                }
                write_close_tag(writer, PATENTS_TAG)?;
            }

            write_close_tag(writer, DEFINITIONS_TAG)
        }
    }

    impl FromXml for Definitions {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            _attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let mut standards: Option<Vec<Standard>> = None;
            #[versioned("1.7")]
            let mut patents: Option<Vec<PatentDefinition>> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(&element_name.local_name))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == STANDARDS_TAG =>
                    {
                        standards = Some(read_lax_validation_list_tag(
                            event_reader,
                            &name,
                            STANDARD_TAG,
                        )?);
                    }
                    #[versioned("1.7")]
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == PATENTS_TAG =>
                    {
                        patents = Some(read_patents(event_reader, &name)?);
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
                standards,
                #[versioned("1.7")]
                patents,
            })
        }
    }

    #[versioned("1.7")]
    fn read_patents<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
    ) -> Result<Vec<PatentDefinition>, XmlReadError> {
        let mut patents = Vec::new();

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PATENT_TAG => {
                    patents.push(PatentDefinition::Patent(Patent::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?));
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PATENT_FAMILY_TAG => {
                    patents.push(PatentDefinition::PatentFamily(
                        PatentFamily::read_xml_element(event_reader, &name, &attributes)?,
                    ));
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

        Ok(patents)
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Standard {
        #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
        pub(crate) bom_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) owner: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) requirements: Option<Vec<Requirement>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) levels: Option<Vec<Level>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) external_references: Option<ExternalReferences>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) signature: Option<Signature>,
    }

    impl From<models::definitions::Standard> for Standard {
        fn from(other: models::definitions::Standard) -> Self {
            Self {
                bom_ref: other.bom_ref.map(|r| r.0),
                name: other.name,
                version: other.version,
                description: other.description,
                owner: other.owner,
                requirements: other
                    .requirements
                    .map(|requirements| requirements.into_iter().map(Into::into).collect()),
                levels: other
                    .levels
                    .map(|levels| levels.into_iter().map(Into::into).collect()),
                external_references: convert_optional(other.external_references),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl From<Standard> for models::definitions::Standard {
        fn from(other: Standard) -> Self {
            Self {
                bom_ref: other.bom_ref.map(BomReference::new),
                name: other.name,
                version: other.version,
                description: other.description,
                owner: other.owner,
                requirements: other
                    .requirements
                    .map(|requirements| requirements.into_iter().map(Into::into).collect()),
                levels: other
                    .levels
                    .map(|levels| levels.into_iter().map(Into::into).collect()),
                external_references: convert_optional(other.external_references),
                signature: convert_optional(other.signature),
            }
        }
    }

    const NAME_TAG: &str = "name";
    const VERSION_TAG: &str = "version";
    const OWNER_TAG: &str = "owner";
    const LEVELS_TAG: &str = "levels";
    const LEVEL_TAG: &str = "level";

    impl ToXml for Standard {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            let mut start_tag = XmlEvent::start_element(STANDARD_TAG);
            if let Some(bom_ref) = &self.bom_ref {
                start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
            }
            writer
                .write(start_tag)
                .map_err(to_xml_write_error(STANDARD_TAG))?;

            if let Some(name) = &self.name {
                write_simple_tag(writer, NAME_TAG, name)?;
            }
            if let Some(version) = &self.version {
                write_simple_tag(writer, VERSION_TAG, version)?;
            }
            if let Some(description) = &self.description {
                write_simple_tag(writer, DESCRIPTION_TAG, description)?;
            }
            if let Some(owner) = &self.owner {
                write_simple_tag(writer, OWNER_TAG, owner)?;
            }
            if let Some(requirements) = &self.requirements {
                write_start_tag(writer, REQUIREMENTS_TAG)?;
                for requirement in requirements {
                    requirement.write_xml_element(writer)?;
                }
                write_close_tag(writer, REQUIREMENTS_TAG)?;
            }
            if let Some(levels) = &self.levels {
                write_start_tag(writer, LEVELS_TAG)?;
                for level in levels {
                    level.write_xml_element(writer)?;
                }
                write_close_tag(writer, LEVELS_TAG)?;
            }
            if let Some(external_references) = &self.external_references {
                external_references.write_xml_element(writer)?;
            }
            // The XSD expects an XML-DSig `ds:Signature` here, while the crate's `Signature` is
            // written as a CycloneDX-namespaced JSF element, which the XSD rejects in this
            // position. The signature is therefore only kept in JSON.

            write_close_tag(writer, STANDARD_TAG)
        }
    }

    impl FromXml for Standard {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
            let mut standard_name: Option<String> = None;
            let mut version: Option<String> = None;
            let mut description: Option<String> = None;
            let mut owner: Option<String> = None;
            let mut requirements: Option<Vec<Requirement>> = None;
            let mut levels: Option<Vec<Level>> = None;
            let mut external_references: Option<ExternalReferences> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(&element_name.local_name))?;
                match next_element {
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_str() {
                        NAME_TAG => standard_name = Some(read_simple_tag(event_reader, &name)?),
                        VERSION_TAG => version = Some(read_simple_tag(event_reader, &name)?),
                        DESCRIPTION_TAG => {
                            description = Some(read_simple_tag(event_reader, &name)?)
                        }
                        OWNER_TAG => owner = Some(read_simple_tag(event_reader, &name)?),
                        REQUIREMENTS_TAG => {
                            requirements =
                                Some(read_list_tag(event_reader, &name, REQUIREMENT_TAG)?)
                        }
                        LEVELS_TAG => levels = Some(read_list_tag(event_reader, &name, LEVEL_TAG)?),
                        EXTERNAL_REFERENCES_TAG => {
                            external_references = Some(ExternalReferences::read_xml_element(
                                event_reader,
                                &name,
                                &attributes,
                            )?)
                        }
                        // lax validation of any elements from a different schema
                        _ => read_lax_validation_tag(event_reader, &name)?,
                    },
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            Ok(Self {
                bom_ref,
                name: standard_name,
                version,
                description,
                owner,
                requirements,
                levels,
                external_references,
                signature: None,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Requirement {
        #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
        pub(crate) bom_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) identifier: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) descriptions: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) open_cre: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) parent: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) properties: Option<Properties>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) external_references: Option<ExternalReferences>,
    }

    impl From<models::definitions::Requirement> for Requirement {
        fn from(other: models::definitions::Requirement) -> Self {
            Self {
                bom_ref: other.bom_ref.map(|r| r.0),
                identifier: other.identifier,
                title: other.title,
                text: other.text,
                descriptions: other.descriptions,
                open_cre: other.open_cre,
                parent: other.parent.map(|r| r.0),
                properties: convert_optional(other.properties),
                external_references: convert_optional(other.external_references),
            }
        }
    }

    impl From<Requirement> for models::definitions::Requirement {
        fn from(other: Requirement) -> Self {
            Self {
                bom_ref: other.bom_ref.map(BomReference::new),
                identifier: other.identifier,
                title: other.title,
                text: other.text,
                descriptions: other.descriptions,
                open_cre: other.open_cre,
                parent: other.parent.map(BomReference::new),
                properties: convert_optional(other.properties),
                external_references: convert_optional(other.external_references),
            }
        }
    }

    const TEXT_TAG: &str = "text";
    const DESCRIPTIONS_TAG: &str = "descriptions";
    const OPEN_CRE_TAG: &str = "openCre";
    const PARENT_TAG: &str = "parent";
    const PROPERTIES_TAG: &str = "properties";

    impl ToXml for Requirement {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            let mut start_tag = XmlEvent::start_element(REQUIREMENT_TAG);
            if let Some(bom_ref) = &self.bom_ref {
                start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
            }
            writer
                .write(start_tag)
                .map_err(to_xml_write_error(REQUIREMENT_TAG))?;

            if let Some(identifier) = &self.identifier {
                write_simple_tag(writer, IDENTIFIER_TAG, identifier)?;
            }
            if let Some(title) = &self.title {
                write_simple_tag(writer, TITLE_TAG, title)?;
            }
            if let Some(text) = &self.text {
                write_simple_tag(writer, TEXT_TAG, text)?;
            }
            if let Some(descriptions) = &self.descriptions {
                write_list_string_tag(writer, DESCRIPTIONS_TAG, DESCRIPTION_TAG, descriptions)?;
            }
            // `openCre` repeats without a wrapper element in the XSD.
            if let Some(open_cre) = &self.open_cre {
                for cre in open_cre {
                    write_simple_tag(writer, OPEN_CRE_TAG, cre)?;
                }
            }
            if let Some(parent) = &self.parent {
                write_simple_tag(writer, PARENT_TAG, parent)?;
            }
            if let Some(properties) = &self.properties {
                properties.write_xml_element(writer)?;
            }
            if let Some(external_references) = &self.external_references {
                external_references.write_xml_element(writer)?;
            }

            write_close_tag(writer, REQUIREMENT_TAG)
        }
    }

    impl FromXml for Requirement {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
            let mut identifier: Option<String> = None;
            let mut title: Option<String> = None;
            let mut text: Option<String> = None;
            let mut descriptions: Option<Vec<String>> = None;
            let mut open_cre: Option<Vec<String>> = None;
            let mut parent: Option<String> = None;
            let mut properties: Option<Properties> = None;
            let mut external_references: Option<ExternalReferences> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(&element_name.local_name))?;
                match next_element {
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_str() {
                        IDENTIFIER_TAG => identifier = Some(read_simple_tag(event_reader, &name)?),
                        TITLE_TAG => title = Some(read_simple_tag(event_reader, &name)?),
                        TEXT_TAG => text = Some(read_simple_tag(event_reader, &name)?),
                        DESCRIPTIONS_TAG => {
                            descriptions =
                                Some(read_list_tag(event_reader, &name, DESCRIPTION_TAG)?)
                        }
                        OPEN_CRE_TAG => open_cre
                            .get_or_insert_with(Vec::new)
                            .push(read_simple_tag(event_reader, &name)?),
                        PARENT_TAG => parent = Some(read_simple_tag(event_reader, &name)?),
                        PROPERTIES_TAG => {
                            properties = Some(Properties::read_xml_element(
                                event_reader,
                                &name,
                                &attributes,
                            )?)
                        }
                        EXTERNAL_REFERENCES_TAG => {
                            external_references = Some(ExternalReferences::read_xml_element(
                                event_reader,
                                &name,
                                &attributes,
                            )?)
                        }
                        // lax validation of any elements from a different schema
                        _ => read_lax_validation_tag(event_reader, &name)?,
                    },
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            Ok(Self {
                bom_ref,
                identifier,
                title,
                text,
                descriptions,
                open_cre,
                parent,
                properties,
                external_references,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Level {
        #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
        pub(crate) bom_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) identifier: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) requirements: Option<Vec<String>>,
    }

    impl From<models::definitions::Level> for Level {
        fn from(other: models::definitions::Level) -> Self {
            Self {
                bom_ref: other.bom_ref.map(|r| r.0),
                identifier: other.identifier,
                title: other.title,
                description: other.description,
                requirements: other
                    .requirements
                    .map(|refs| refs.into_iter().map(|r| r.0).collect()),
            }
        }
    }

    impl From<Level> for models::definitions::Level {
        fn from(other: Level) -> Self {
            Self {
                bom_ref: other.bom_ref.map(BomReference::new),
                identifier: other.identifier,
                title: other.title,
                description: other.description,
                requirements: other
                    .requirements
                    .map(|refs| refs.into_iter().map(BomReference::new).collect()),
            }
        }
    }

    impl ToXml for Level {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            let mut start_tag = XmlEvent::start_element(LEVEL_TAG);
            if let Some(bom_ref) = &self.bom_ref {
                start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
            }
            writer
                .write(start_tag)
                .map_err(to_xml_write_error(LEVEL_TAG))?;

            if let Some(identifier) = &self.identifier {
                write_simple_tag(writer, IDENTIFIER_TAG, identifier)?;
            }
            if let Some(title) = &self.title {
                write_simple_tag(writer, TITLE_TAG, title)?;
            }
            if let Some(description) = &self.description {
                write_simple_tag(writer, DESCRIPTION_TAG, description)?;
            }
            if let Some(requirements) = &self.requirements {
                write_list_string_tag(writer, REQUIREMENTS_TAG, REQUIREMENT_TAG, requirements)?;
            }

            write_close_tag(writer, LEVEL_TAG)
        }
    }

    impl FromXml for Level {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
            let mut identifier: Option<String> = None;
            let mut title: Option<String> = None;
            let mut description: Option<String> = None;
            let mut requirements: Option<Vec<String>> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(&element_name.local_name))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. } => {
                        match name.local_name.as_str() {
                            IDENTIFIER_TAG => {
                                identifier = Some(read_simple_tag(event_reader, &name)?)
                            }
                            TITLE_TAG => title = Some(read_simple_tag(event_reader, &name)?),
                            DESCRIPTION_TAG => {
                                description = Some(read_simple_tag(event_reader, &name)?)
                            }
                            REQUIREMENTS_TAG => {
                                requirements =
                                    Some(read_list_tag(event_reader, &name, REQUIREMENT_TAG)?)
                            }
                            // lax validation of any elements from a different schema
                            _ => read_lax_validation_tag(event_reader, &name)?,
                        }
                    }
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            Ok(Self {
                bom_ref,
                identifier,
                title,
                description,
                requirements,
            })
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use pretty_assertions::assert_eq;

        use super::*;
        #[versioned("1.6")]
        use crate::specs::v1_6::external_reference::test::{
            corresponding_external_references, example_external_references,
        };
        #[versioned("1.7")]
        use crate::specs::v1_7::{
            external_reference::test::{
                corresponding_external_references, example_external_references,
            },
            patent::test::{corresponding_patent_definitions, example_patent_definitions},
        };
        use crate::{
            specs::common::{
                property::test::{corresponding_properties, example_properties},
                signature::test::{corresponding_signature, example_signature},
            },
            xml::test::{read_element_from_string, write_element_to_string},
        };

        fn example_standard(signature: Option<Signature>) -> Standard {
            Standard {
                bom_ref: Some("standard-1".to_string()),
                name: Some("name".to_string()),
                version: Some("1.0.0".to_string()),
                description: Some("description".to_string()),
                owner: Some("owner".to_string()),
                requirements: Some(vec![Requirement {
                    bom_ref: Some("requirement-1".to_string()),
                    identifier: Some("v1".to_string()),
                    title: Some("title".to_string()),
                    text: Some("text".to_string()),
                    descriptions: Some(vec!["first".to_string(), "second".to_string()]),
                    open_cre: Some(vec!["CRE:616-305".to_string(), "CRE:764-507".to_string()]),
                    parent: Some("requirement-0".to_string()),
                    properties: Some(example_properties()),
                    external_references: Some(example_external_references()),
                }]),
                levels: Some(vec![Level {
                    bom_ref: Some("level-1".to_string()),
                    identifier: Some("Level 1".to_string()),
                    title: Some("title".to_string()),
                    description: Some("description".to_string()),
                    requirements: Some(vec!["requirement-1".to_string()]),
                }]),
                external_references: Some(example_external_references()),
                signature,
            }
        }

        pub(crate) fn example_definitions() -> Definitions {
            Definitions {
                standards: Some(vec![example_standard(Some(example_signature()))]),
                #[versioned("1.7")]
                patents: Some(example_patent_definitions()),
            }
        }

        pub(crate) fn corresponding_definitions() -> models::definitions::Definitions {
            models::definitions::Definitions {
                standards: Some(vec![models::definitions::Standard {
                    bom_ref: Some(BomReference::new("standard-1")),
                    name: Some("name".to_string()),
                    version: Some("1.0.0".to_string()),
                    description: Some("description".to_string()),
                    owner: Some("owner".to_string()),
                    requirements: Some(vec![models::definitions::Requirement {
                        bom_ref: Some(BomReference::new("requirement-1")),
                        identifier: Some("v1".to_string()),
                        title: Some("title".to_string()),
                        text: Some("text".to_string()),
                        descriptions: Some(vec!["first".to_string(), "second".to_string()]),
                        open_cre: Some(vec!["CRE:616-305".to_string(), "CRE:764-507".to_string()]),
                        parent: Some(BomReference::new("requirement-0")),
                        properties: Some(corresponding_properties()),
                        external_references: Some(corresponding_external_references()),
                    }]),
                    levels: Some(vec![models::definitions::Level {
                        bom_ref: Some(BomReference::new("level-1")),
                        identifier: Some("Level 1".to_string()),
                        title: Some("title".to_string()),
                        description: Some("description".to_string()),
                        requirements: Some(vec![BomReference::new("requirement-1")]),
                    }]),
                    external_references: Some(corresponding_external_references()),
                    signature: Some(corresponding_signature()),
                }]),
                #[versioned("1.6")]
                patents: None,
                #[versioned("1.7")]
                patents: Some(corresponding_patent_definitions()),
            }
        }

        #[test]
        fn it_should_convert_definitions_both_ways() {
            let model: models::definitions::Definitions = example_definitions().into();
            assert_eq!(model, corresponding_definitions());

            let spec: Definitions = corresponding_definitions().into();
            assert_eq!(spec, example_definitions());
        }

        #[test]
        fn it_should_write_xml_full() {
            insta::assert_snapshot!(write_element_to_string(example_definitions()));
        }

        #[test]
        fn it_should_read_xml_full() {
            #[versioned("1.6")]
            let patents = "";
            #[versioned("1.7")]
            let patents = r#"
  <patents>
    <patent bom-ref="patent-1">
      <patentNumber>US1234567890</patentNumber>
      <applicationNumber>12345</applicationNumber>
      <jurisdiction>US</jurisdiction>
      <priorityApplication>
        <applicationNumber>US1234567890</applicationNumber>
        <jurisdiction>US</jurisdiction>
        <filingDate>2021-01-15</filingDate>
      </priorityApplication>
      <publicationNumber>US-12345</publicationNumber>
      <title>title</title>
      <abstract>abstract</abstract>
      <filingDate>2021-01-15</filingDate>
      <grantDate>2022-06-01</grantDate>
      <patentExpirationDate>2042-01-15</patentExpirationDate>
      <patentLegalStatus>in-force</patentLegalStatus>
      <patentAssignee>
        <organization>
          <name>name</name>
          <url>url</url>
          <contact>
            <name>name</name>
            <email>email</email>
            <phone>phone</phone>
          </contact>
        </organization>
      </patentAssignee>
      <patentAssignee>
        <individual>
          <name>name</name>
          <email>email</email>
          <phone>phone</phone>
        </individual>
      </patentAssignee>
    </patent>
    <patentFamily bom-ref="patent-family-1">
      <familyId>PF-2023001</familyId>
      <priorityApplication>
        <applicationNumber>US1234567890</applicationNumber>
        <jurisdiction>US</jurisdiction>
        <filingDate>2021-01-15</filingDate>
      </priorityApplication>
      <members>
        <ref>patent-1</ref>
      </members>
    </patentFamily>
  </patents>"#;
            let input = format!(
                r#"
<definitions>
  <standards>
    <standard bom-ref="standard-1">
      <name>name</name>
      <version>1.0.0</version>
      <description>description</description>
      <owner>owner</owner>
      <requirements>
        <requirement bom-ref="requirement-1">
          <identifier>v1</identifier>
          <title>title</title>
          <text>text</text>
          <descriptions>
            <description>first</description>
            <description>second</description>
          </descriptions>
          <openCre>CRE:616-305</openCre>
          <openCre>CRE:764-507</openCre>
          <parent>requirement-0</parent>
          <properties>
            <property name="name">value</property>
          </properties>
          <externalReferences>
            <reference type="external reference type">
              <url>url</url>
              <comment>comment</comment>
              <hashes>
                <hash alg="algorithm">hash value</hash>
              </hashes>
            </reference>
          </externalReferences>
        </requirement>
      </requirements>
      <levels>
        <level bom-ref="level-1">
          <identifier>Level 1</identifier>
          <title>title</title>
          <description>description</description>
          <requirements>
            <requirement>requirement-1</requirement>
          </requirements>
        </level>
      </levels>
      <externalReferences>
        <reference type="external reference type">
          <url>url</url>
          <comment>comment</comment>
          <hashes>
            <hash alg="algorithm">hash value</hash>
          </hashes>
        </reference>
      </externalReferences>
      <ds:Signature xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
        <ds:SignatureValue>abc</ds:SignatureValue>
      </ds:Signature>
    </standard>
  </standards>{patents}
</definitions>
"#
            );
            let actual: Definitions = read_element_from_string(input);
            let expected = Definitions {
                standards: Some(vec![example_standard(None)]),
                #[versioned("1.7")]
                patents: Some(example_patent_definitions()),
            };
            assert_eq!(actual, expected);
        }
    }
}
