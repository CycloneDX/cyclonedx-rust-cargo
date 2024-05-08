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
    prelude::Uri,
    specs::{
        common::{bom_reference::BomReference, property::Properties},
        v1_5::data_governance::DataGovernance,
    },
    utilities::{convert_optional, convert_vec},
    xml::{
        optional_attribute, read_simple_tag, to_xml_read_error, to_xml_write_error,
        write_close_tag, write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

use super::attachment::Attachment;

/// Component's Data.
///
/// bom-1.5.schema.json #definitions/componentData
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComponentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bom_ref: Option<BomReference>,
    #[serde(rename = "type")]
    pub(crate) data_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) contents: Option<DataContents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // NOTE: this should be DataClassification but specs and examples differ.
    pub(crate) classification: Option<String>,
    /// Marked as an array of `String`, but examples use a single entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sensitive_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) graphics: Option<GraphicsCollection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) governance: Option<DataGovernance>,
}

impl From<models::component_data::ComponentData> for ComponentData {
    fn from(other: models::component_data::ComponentData) -> Self {
        Self {
            bom_ref: other.bom_ref.clone().map(Into::into),
            data_type: other.data_type.to_string(),
            name: other.name,
            contents: convert_optional(other.contents),
            classification: convert_optional(other.classification),
            sensitive_data: convert_optional(other.sensitive_data),
            graphics: convert_optional(other.graphics),
            description: convert_optional(other.description),
            governance: convert_optional(other.governance),
        }
    }
}

impl From<ComponentData> for models::component_data::ComponentData {
    fn from(other: ComponentData) -> Self {
        Self {
            bom_ref: other.bom_ref.map(models::bom::BomReference::from),
            data_type: other.data_type.into(),
            name: other.name,
            contents: convert_optional(other.contents),
            classification: convert_optional(other.classification),
            sensitive_data: convert_optional(other.sensitive_data),
            graphics: convert_optional(other.graphics),
            description: convert_optional(other.description),
            governance: convert_optional(other.governance),
        }
    }
}

const CONTENTS_TAG: &str = "contents";
const GRAPHICS_TAG: &str = "graphics";
const NAME_TAG: &str = "name";
const CLASSIFICATION_TAG: &str = "classification";
const SENSITIVE_DATA_TAG: &str = "sensitiveData";
const GOVERNANCE_TAG: &str = "governance";
const BOM_REF_ATTR: &str = "bom-ref";

impl ToInnerXml for ComponentData {
    fn write_xml_named_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let Self {
            bom_ref,
            data_type,
            name,
            contents,
            classification,
            sensitive_data,
            graphics,
            description,
            governance,
        } = self;

        let mut start_tag = writer::XmlEvent::start_element(tag);
        if let Some(bom_ref) = bom_ref.as_ref() {
            start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref.as_ref());
        }
        writer.write(start_tag).map_err(to_xml_write_error(tag))?;

        write_simple_tag(writer, TYPE_TAG, data_type)?;

        if let Some(name) = name.as_ref() {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(contents) = contents.as_ref() {
            contents.write_xml_element(writer)?;
        }

        if let Some(classification) = classification.as_ref() {
            write_simple_tag(writer, CLASSIFICATION_TAG, classification)?;
        }

        if let Some(sensitive_data) = sensitive_data.as_ref() {
            write_simple_tag(writer, SENSITIVE_DATA_TAG, sensitive_data)?;
        }

        if let Some(graphics) = graphics.as_ref() {
            graphics.write_xml_named_element(writer, GRAPHICS_TAG)?;
        }

        if let Some(description) = description.as_ref() {
            write_simple_tag(writer, DESCRIPTION_TAG, description)?;
        }

        if let Some(governance) = governance.as_ref() {
            governance.write_xml_named_element(writer, GOVERNANCE_TAG)?;
        }

        write_close_tag(writer, tag)?;

        Ok(())
    }
}

const TYPE_TAG: &str = "type";

impl FromXml for ComponentData {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR).map(BomReference::new);
        let mut data_type: Option<String> = None;
        let mut data_name: Option<String> = None;
        let mut contents: Option<DataContents> = None;
        let mut classification: Option<String> = None;
        let mut graphics: Option<GraphicsCollection> = None;
        let mut description: Option<String> = None;
        let mut governance: Option<DataGovernance> = None;
        let mut sensitive_data: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == TYPE_TAG => {
                    data_type = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    data_name = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == CONTENTS_TAG => {
                    contents = Some(DataContents::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == DESCRIPTION_TAG =>
                {
                    description = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == CLASSIFICATION_TAG =>
                {
                    classification = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == GOVERNANCE_TAG => {
                    governance = Some(DataGovernance::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == GRAPHICS_TAG => {
                    graphics = Some(GraphicsCollection::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == SENSITIVE_DATA_TAG =>
                {
                    sensitive_data = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        let data_type = data_type.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: TYPE_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(ComponentData {
            bom_ref,
            data_type,
            name: data_name,
            contents,
            classification,
            sensitive_data,
            graphics,
            description,
            governance,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DataContents {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) attachment: Option<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::component_data::DataContents> for DataContents {
    fn from(other: models::component_data::DataContents) -> Self {
        Self {
            attachment: convert_optional(other.attachment),
            url: other.url.map(|url| url.to_string()),
            properties: convert_optional(other.properties),
        }
    }
}

impl From<DataContents> for models::component_data::DataContents {
    fn from(other: DataContents) -> Self {
        Self {
            attachment: convert_optional(other.attachment),
            url: other.url.map(Uri),
            properties: convert_optional(other.properties),
        }
    }
}

const ATTACHMENT_TAG: &str = "attachment";

impl ToXml for DataContents {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, CONTENTS_TAG)?;

        if let Some(attachment) = &self.attachment {
            attachment.write_xml_named_element(writer, ATTACHMENT_TAG)?;
        }

        if let Some(url) = &self.url {
            write_simple_tag(writer, URL_TAG, url)?;
        }

        if let Some(properties) = &self.properties {
            properties.write_xml_element(writer)?;
        }

        write_close_tag(writer, CONTENTS_TAG)?;

        Ok(())
    }
}

const URL_TAG: &str = "url";
const PROPERTIES_TAG: &str = "properties";

impl FromXml for DataContents {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut url: Option<String> = None;
        let mut attachment: Option<Attachment> = None;
        let mut properties: Option<Properties> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == URL_TAG => {
                    url = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ATTACHMENT_TAG => {
                    attachment = Some(Attachment::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PROPERTIES_TAG => {
                    properties = Some(Properties::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self {
            attachment,
            url,
            properties,
        })
    }
}

/// For more details see:
/// https://cyclonedx.org/docs/1.5/json/#components_items_modelCard_modelParameters_datasets_items_oneOf_i0_graphics
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct GraphicsCollection {
    pub(crate) description: Option<String>,
    pub(crate) collection: Option<Collection>,
}

impl From<models::component_data::GraphicsCollection> for GraphicsCollection {
    fn from(other: models::component_data::GraphicsCollection) -> Self {
        Self {
            description: convert_optional(other.description),
            collection: convert_optional(other.collection),
        }
    }
}

impl From<GraphicsCollection> for models::component_data::GraphicsCollection {
    fn from(other: GraphicsCollection) -> Self {
        Self {
            description: convert_optional(other.description),
            collection: convert_optional(other.collection),
        }
    }
}

const COLLECTION_TAG: &str = "collection";
const DESCRIPTION_TAG: &str = "description";

impl ToInnerXml for GraphicsCollection {
    fn write_xml_named_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, tag)?;

        if let Some(description) = &self.description {
            write_simple_tag(writer, DESCRIPTION_TAG, description)?;
        }

        if let Some(collection) = &self.collection {
            collection.write_xml_element(writer)?;
        }

        write_close_tag(writer, tag)?;

        Ok(())
    }
}

const OUTPUT_TAG: &str = "output";

impl FromXml for GraphicsCollection {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut description: Option<String> = None;
        let mut collection: Option<Collection> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(OUTPUT_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == DESCRIPTION_TAG =>
                {
                    description = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COLLECTION_TAG => {
                    collection = Some(Collection::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self {
            description,
            collection,
        })
    }
}

/// Helper struct to collect all [`Graphic`].
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct Collection(pub(crate) Vec<Graphic>);

impl From<Vec<Graphic>> for Collection {
    fn from(value: Vec<Graphic>) -> Self {
        Self(value)
    }
}

impl From<Vec<models::component_data::Graphic>> for Collection {
    fn from(other: Vec<models::component_data::Graphic>) -> Self {
        Self(convert_vec(other))
    }
}

impl From<Collection> for Vec<models::component_data::Graphic> {
    fn from(other: Collection) -> Self {
        convert_vec(other.0)
    }
}

const GRAPHIC_TAG: &str = "graphic";

impl ToXml for Collection {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, COLLECTION_TAG)?;

        for graphic in &self.0 {
            graphic.write_xml_element(writer)?;
        }

        write_close_tag(writer, COLLECTION_TAG)?;

        Ok(())
    }
}

impl FromXml for Collection {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut collection: Vec<Graphic> = Vec::new();
        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(OUTPUT_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == GRAPHIC_TAG => {
                    collection.push(Graphic::read_xml_element(event_reader, &name, &attributes)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self(collection))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct Graphic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) image: Option<Attachment>,
}

impl From<models::component_data::Graphic> for Graphic {
    fn from(other: models::component_data::Graphic) -> Self {
        Self {
            name: convert_optional(other.name),
            image: convert_optional(other.image),
        }
    }
}

impl From<Graphic> for models::component_data::Graphic {
    fn from(other: Graphic) -> Self {
        Self {
            name: convert_optional(other.name),
            image: convert_optional(other.image),
        }
    }
}

const IMAGE_TAG: &str = "image";

impl ToXml for Graphic {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, GRAPHIC_TAG)?;

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(image) = &self.image {
            image.write_xml_named_element(writer, IMAGE_TAG)?;
        }

        write_close_tag(writer, GRAPHIC_TAG)?;

        Ok(())
    }
}

impl FromXml for Graphic {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut graphic_name: Option<String> = None;
        let mut image: Option<Attachment> = None;

        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(OUTPUT_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    graphic_name = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == IMAGE_TAG => {
                    image = Some(Attachment::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self {
            name: graphic_name,
            image,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn example_component_data() -> ComponentData {
        ComponentData {
            bom_ref: None,
            data_type: "configuration".into(),
            name: Some("config".into()),
            contents: Some(DataContents {
                attachment: Some(Attachment {
                    content: "foo: bar".into(),
                    content_type: None,
                    encoding: None,
                }),
                url: None,
                properties: None,
            }),
            classification: None,
            sensitive_data: None,
            graphics: None,
            description: None,
            governance: None,
        }
    }

    pub(crate) fn corresponding_component_data() -> models::component_data::ComponentData {
        models::component_data::ComponentData {
            bom_ref: None,
            data_type: models::component_data::ComponentDataType::Configuration,
            name: Some("config".into()),
            contents: Some(models::component_data::DataContents {
                attachment: Some(models::attachment::Attachment {
                    content: "foo: bar".into(),
                    content_type: None,
                    encoding: None,
                }),
                url: None,
                properties: None,
            }),
            classification: None,
            sensitive_data: None,
            graphics: None,
            description: None,
            governance: None,
        }
    }
}
