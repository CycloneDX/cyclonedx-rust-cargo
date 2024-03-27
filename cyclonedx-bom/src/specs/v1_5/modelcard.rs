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
use xml::{name::OwnedName, reader};

use crate::{
    errors::XmlReadError,
    models::{self, bom::BomReference},
    prelude::Uri,
    specs::common::{
        organization::{OrganizationalContact, OrganizationalEntity},
        property::Properties,
    },
    utilities::{convert_optional, convert_vec},
    xml::{
        optional_attribute, read_list_tag, read_simple_tag, to_xml_read_error,
        unexpected_element_error, FromXml,
    },
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelCard {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bom_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) model_parameters: Option<ModelParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) quantitative_analysis: Option<QuantitativeAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) considerations: Option<Considerations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::modelcard::ModelCard> for ModelCard {
    fn from(other: models::modelcard::ModelCard) -> Self {
        Self {
            bom_ref: other.bom_ref.map(|r| r.0),
            model_parameters: convert_optional(other.model_parameters),
            quantitative_analysis: convert_optional(other.quantitative_analysis),
            considerations: convert_optional(other.considerations),
            properties: convert_optional(other.properties),
        }
    }
}

impl From<ModelCard> for models::modelcard::ModelCard {
    fn from(other: ModelCard) -> Self {
        Self {
            bom_ref: other.bom_ref.map(BomReference::new),
            model_parameters: convert_optional(other.model_parameters),
            quantitative_analysis: convert_optional(other.quantitative_analysis),
            considerations: convert_optional(other.considerations),
            properties: convert_optional(other.properties),
        }
    }
}

const MODEL_PARAMETERS_TAG: &str = "modelParameters";
const BOM_REF_ATTR: &str = "bom-ref";

impl FromXml for ModelCard {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
        let mut model_parameters: Option<ModelParameters> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == MODEL_PARAMETERS_TAG => {
                    model_parameters = Some(ModelParameters::read_xml_element(
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
            bom_ref,
            model_parameters,
            quantitative_analysis: None,
            considerations: None,
            properties: None,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) approach: Option<ModelParametersApproach>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) architecture_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) model_architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) datasets: Option<Datasets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) inputs: Option<Inputs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) outputs: Option<Outputs>,
}

impl From<models::modelcard::ModelParameters> for ModelParameters {
    fn from(other: models::modelcard::ModelParameters) -> Self {
        Self {
            approach: convert_optional(other.approach),
            task: other.task,
            architecture_family: other.architecture_family,
            model_architecture: other.model_architecture,
            datasets: convert_optional(other.datasets),
            inputs: convert_optional(other.inputs),
            outputs: convert_optional(other.outputs),
        }
    }
}

impl From<ModelParameters> for models::modelcard::ModelParameters {
    fn from(other: ModelParameters) -> Self {
        Self {
            approach: convert_optional(other.approach),
            task: other.task,
            architecture_family: other.architecture_family,
            model_architecture: other.model_architecture,
            datasets: convert_optional(other.datasets),
            inputs: convert_optional(other.inputs),
            outputs: convert_optional(other.outputs),
        }
    }
}

const APPROACH_TAG: &str = "approach";
const TASK_TAG: &str = "task";
const ARCHITECTURE_FAMILY_TAG: &str = "architectureFamily";
const MODEL_ARCHITECTURE_TAG: &str = "modelArchitecture";
const INPUTS_TAG: &str = "inputs";
const INPUT_TAG: &str = "input";
const OUTPUTS_TAG: &str = "outputs";
const OUTPUT_TAG: &str = "output";
const FORMAT_TAG: &str = "format";
const ATTACHMENT_TAG: &str = "attachment";

impl FromXml for ModelParameters {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut approach: Option<ModelParametersApproach> = None;
        let mut task: Option<String> = None;
        let mut architecture_family: Option<String> = None;
        let mut model_architecture: Option<String> = None;
        let mut datasets: Option<Datasets> = None;
        let mut inputs: Option<Inputs> = None;
        let mut outputs: Option<Outputs> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == APPROACH_TAG => {
                    approach = Some(ModelParametersApproach::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == TASK_TAG => {
                    task = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == ARCHITECTURE_FAMILY_TAG =>
                {
                    architecture_family = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == MODEL_ARCHITECTURE_TAG =>
                {
                    model_architecture = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == DATASETS_TAG => {
                    datasets = Some(Datasets::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == INPUTS_TAG => {
                    inputs = Some(Inputs::read_xml_element(event_reader, &name, &attributes)?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == OUTPUTS_TAG => {
                    outputs = Some(Outputs::read_xml_element(event_reader, &name, &attributes)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                _ => (),
            }
        }

        Ok(Self {
            approach,
            task,
            architecture_family,
            model_architecture,
            datasets,
            inputs,
            outputs,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelParametersApproach {
    #[serde(rename = "type")]
    pub(crate) approach_type: Option<String>,
}

impl From<models::modelcard::ModelParametersApproach> for ModelParametersApproach {
    fn from(other: models::modelcard::ModelParametersApproach) -> Self {
        Self {
            approach_type: other.approach_type.map(|at| at.to_string()),
        }
    }
}

impl From<ModelParametersApproach> for models::modelcard::ModelParametersApproach {
    fn from(other: ModelParametersApproach) -> Self {
        Self {
            approach_type: other
                .approach_type
                .map(models::modelcard::ApproachType::new_unchecked),
        }
    }
}

const TYPE_TAG: &str = "type";

impl FromXml for ModelParametersApproach {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut approach_type: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == TYPE_TAG => {
                    approach_type = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self { approach_type })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Datasets(pub Vec<Dataset>);

impl From<models::modelcard::Datasets> for Datasets {
    fn from(other: models::modelcard::Datasets) -> Self {
        Datasets(convert_vec(other.0))
    }
}

impl From<Datasets> for models::modelcard::Datasets {
    fn from(other: Datasets) -> Self {
        models::modelcard::Datasets(convert_vec(other.0))
    }
}

impl FromXml for Datasets {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut datasets = Vec::new();

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == DATASET_TAG => {
                    datasets.push(Dataset::read_xml_element(event_reader, &name, &attributes)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self(datasets))
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", untagged)]
pub(crate) enum Dataset {
    Component(ComponentData),
    Reference(String),
}

impl From<models::modelcard::Dataset> for Dataset {
    fn from(other: models::modelcard::Dataset) -> Self {
        match other {
            models::modelcard::Dataset::Component(component) => Self::Component(component.into()),
            models::modelcard::Dataset::Reference(reference) => Self::Reference(reference),
        }
    }
}

impl From<Dataset> for models::modelcard::Dataset {
    fn from(other: Dataset) -> Self {
        match other {
            Dataset::Component(component) => {
                models::modelcard::Dataset::Component(component.into())
            }
            Dataset::Reference(reference) => models::modelcard::Dataset::Reference(reference),
        }
    }
}

const DATASETS_TAG: &str = "datasets";
const DATASET_TAG: &str = "dataset";
const CONTENTS_TAG: &str = "contents";
const GRAPHICS_TAG: &str = "graphics";
const NAME_TAG: &str = "name";
const CLASSIFICATION_TAG: &str = "classification";
const SENSITIVE_DATA_TAG: &str = "sensitiveData";
const GOVERNANCE_TAG: &str = "governance";

impl FromXml for Dataset {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
        let mut data_type: Option<String> = None;
        let mut data_name: Option<String> = None;
        let mut contents: Option<DataContents> = None;
        let mut classification: Option<String> = None;
        let mut graphics: Option<Graphics> = None;
        let mut description: Option<String> = None;
        let mut governance: Option<DataGovernance> = None;
        let sensitive_data: Option<Vec<String>> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(DATASET_TAG))?;

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
                    graphics = Some(Graphics::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == SENSITIVE_DATA_TAG =>
                {
                    // NOTE: it's not fully clear how this tag works
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

        Ok(Self::Component(ComponentData {
            bom_ref,
            data_type,
            name: data_name,
            contents,
            classification,
            sensitive_data,
            graphics,
            description,
            governance,
        }))
    }
}

/// Dataset component, for more details see:
/// https://cyclonedx.org/docs/1.5/json/#tab-pane_components_items_modelCard_modelParameters_datasets_items_oneOf_i1
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComponentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bom_ref: Option<String>,
    #[serde(rename = "type")]
    pub(crate) data_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) contents: Option<DataContents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sensitive_data: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) graphics: Option<Graphics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) governance: Option<DataGovernance>,
}

impl From<models::modelcard::ComponentData> for ComponentData {
    fn from(other: models::modelcard::ComponentData) -> Self {
        Self {
            bom_ref: other.bom_ref,
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

impl From<ComponentData> for models::modelcard::ComponentData {
    fn from(other: ComponentData) -> Self {
        Self {
            bom_ref: other.bom_ref,
            data_type: models::modelcard::ComponentDataType::new_unchecked(other.data_type),
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

impl From<models::modelcard::DataContents> for DataContents {
    fn from(other: models::modelcard::DataContents) -> Self {
        Self {
            attachment: convert_optional(other.attachment),
            url: other.url.map(|url| url.to_string()),
            properties: convert_optional(other.properties),
        }
    }
}

impl From<DataContents> for models::modelcard::DataContents {
    fn from(other: DataContents) -> Self {
        Self {
            attachment: convert_optional(other.attachment),
            url: other.url.map(Uri),
            properties: convert_optional(other.properties),
        }
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Attachment {
    content: String,
    content_type: Option<String>,
    encoding: Option<String>,
}

impl From<models::modelcard::Attachment> for Attachment {
    fn from(other: models::modelcard::Attachment) -> Self {
        Self {
            content: other.content,
            content_type: convert_optional(other.content_type),
            encoding: convert_optional(other.encoding),
        }
    }
}

impl From<Attachment> for models::modelcard::Attachment {
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuantitativeAnalysis {}

impl From<models::modelcard::QuantitativeAnalysis> for QuantitativeAnalysis {
    fn from(_other: models::modelcard::QuantitativeAnalysis) -> Self {
        Self {}
    }
}

impl From<QuantitativeAnalysis> for models::modelcard::QuantitativeAnalysis {
    fn from(_other: QuantitativeAnalysis) -> Self {
        Self {}
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Considerations {}

impl From<models::modelcard::Considerations> for Considerations {
    fn from(_other: models::modelcard::Considerations) -> Self {
        Self {}
    }
}

impl From<Considerations> for models::modelcard::Considerations {
    fn from(_other: Considerations) -> Self {
        Self {}
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct Inputs(pub Vec<MLParameter>);

impl From<models::modelcard::Inputs> for Inputs {
    fn from(other: models::modelcard::Inputs) -> Self {
        Self(convert_vec(other.0))
    }
}

impl From<Inputs> for models::modelcard::Inputs {
    fn from(other: Inputs) -> Self {
        Self(convert_vec(other.0))
    }
}

impl FromXml for Inputs {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut inputs: Vec<MLParameter> = Vec::new();

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == INPUT_TAG => {
                    let parameter =
                        MLParameter::read_xml_element(event_reader, &name, &attributes)?;
                    inputs.push(parameter);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self(inputs))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct Outputs(pub Vec<MLParameter>);

impl From<models::modelcard::Outputs> for Outputs {
    fn from(other: models::modelcard::Outputs) -> Self {
        Self(convert_vec(other.0))
    }
}

impl From<Outputs> for models::modelcard::Outputs {
    fn from(other: Outputs) -> Self {
        Self(convert_vec(other.0))
    }
}

impl FromXml for Outputs {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut outputs: Vec<MLParameter> = Vec::new();

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == OUTPUT_TAG => {
                    let parameter =
                        MLParameter::read_xml_element(event_reader, &name, &attributes)?;
                    outputs.push(parameter);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self(outputs))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct MLParameter {
    format: Option<String>,
}

impl MLParameter {
    #[allow(unused)]
    pub fn new(format: &str) -> Self {
        Self {
            format: Some(format.to_string()),
        }
    }
}

impl From<models::modelcard::MLParameter> for MLParameter {
    fn from(other: models::modelcard::MLParameter) -> Self {
        Self {
            format: convert_optional(other.format),
        }
    }
}

impl From<MLParameter> for models::modelcard::MLParameter {
    fn from(other: MLParameter) -> Self {
        Self {
            format: convert_optional(other.format),
        }
    }
}

impl FromXml for MLParameter {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut format: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(OUTPUT_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == FORMAT_TAG => {
                    format = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self { format })
    }
}

/// For more details see:
/// https://cyclonedx.org/docs/1.5/json/#components_items_modelCard_modelParameters_datasets_items_oneOf_i0_graphics
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Graphics {
    description: Option<String>,
    collection: Option<Collection>,
}

impl From<models::modelcard::Graphics> for Graphics {
    fn from(other: models::modelcard::Graphics) -> Self {
        Self {
            description: convert_optional(other.description),
            collection: convert_optional(other.collection),
        }
    }
}

impl From<Graphics> for models::modelcard::Graphics {
    fn from(other: Graphics) -> Self {
        Self {
            description: convert_optional(other.description),
            collection: convert_optional(other.collection),
        }
    }
}

/// Helper struct to collect all [`Graphic`].
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Collection(pub Vec<Graphic>);

impl From<Vec<models::modelcard::Graphic>> for Collection {
    fn from(other: Vec<models::modelcard::Graphic>) -> Self {
        Self(convert_vec(other))
    }
}

impl From<Collection> for Vec<models::modelcard::Graphic> {
    fn from(other: Collection) -> Self {
        convert_vec(other.0)
    }
}

const GRAPHIC_TAG: &str = "graphic";

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

const COLLECTION_TAG: &str = "collection";
const DESCRIPTION_TAG: &str = "description";

impl FromXml for Graphics {
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Graphic {
    name: Option<String>,
    image: Option<Attachment>,
}

impl From<models::modelcard::Graphic> for Graphic {
    fn from(other: models::modelcard::Graphic) -> Self {
        Self {
            name: convert_optional(other.name),
            image: convert_optional(other.image),
        }
    }
}

impl From<Graphic> for models::modelcard::Graphic {
    fn from(other: Graphic) -> Self {
        Self {
            name: convert_optional(other.name),
            image: convert_optional(other.image),
        }
    }
}

const IMAGE_TAG: &str = "image";

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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DataGovernance {
    custodians: Option<Vec<DataGovernanceResponsibleParty>>,
    stewards: Option<Vec<DataGovernanceResponsibleParty>>,
    owners: Option<Vec<DataGovernanceResponsibleParty>>,
}

impl From<models::modelcard::DataGovernance> for DataGovernance {
    fn from(other: models::modelcard::DataGovernance) -> Self {
        Self {
            custodians: other.custodians.map(convert_vec),
            stewards: other.stewards.map(convert_vec),
            owners: other.owners.map(convert_vec),
        }
    }
}

impl From<DataGovernance> for models::modelcard::DataGovernance {
    fn from(other: DataGovernance) -> Self {
        Self {
            custodians: other.custodians.map(convert_vec),
            stewards: other.stewards.map(convert_vec),
            owners: other.owners.map(convert_vec),
        }
    }
}

const CUSTODIANS_TAG: &str = "custodians";
const CUSTODIAN_TAG: &str = "custodian";
const STEWARDS_TAG: &str = "stewards";
const STEWARD_TAG: &str = "steward";
const OWNERS_TAG: &str = "owners";
const OWNER_TAG: &str = "owner";

impl FromXml for DataGovernance {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut custodians: Option<Vec<DataGovernanceResponsibleParty>> = None;
        let mut stewards: Option<Vec<DataGovernanceResponsibleParty>> = None;
        let mut owners: Option<Vec<DataGovernanceResponsibleParty>> = None;
        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == CUSTODIANS_TAG =>
                {
                    custodians = Some(read_list_tag(event_reader, &name, CUSTODIAN_TAG)?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == STEWARDS_TAG => {
                    stewards = Some(read_list_tag(event_reader, &name, STEWARD_TAG)?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == OWNERS_TAG => {
                    owners = Some(read_list_tag(event_reader, &name, OWNER_TAG)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                _ => (),
            }
        }

        Ok(Self {
            custodians,
            stewards,
            owners,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum DataGovernanceResponsibleParty {
    Organization(OrganizationalEntity),
    Contact(OrganizationalContact),
}

impl From<models::modelcard::DataGovernanceResponsibleParty> for DataGovernanceResponsibleParty {
    fn from(other: models::modelcard::DataGovernanceResponsibleParty) -> Self {
        match other {
            models::modelcard::DataGovernanceResponsibleParty::Organization(organization) => {
                Self::Organization(organization.into())
            }
            models::modelcard::DataGovernanceResponsibleParty::Contact(contact) => {
                Self::Contact(contact.into())
            }
        }
    }
}

impl From<DataGovernanceResponsibleParty> for models::modelcard::DataGovernanceResponsibleParty {
    fn from(other: DataGovernanceResponsibleParty) -> Self {
        match other {
            DataGovernanceResponsibleParty::Organization(organization) => {
                Self::Organization(organization.into())
            }
            DataGovernanceResponsibleParty::Contact(contact) => Self::Contact(contact.into()),
        }
    }
}

const ORGANIZATION_TAG: &str = "organization";
const CONTACT_TAG: &str = "contact";

impl FromXml for DataGovernanceResponsibleParty {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut party: Option<DataGovernanceResponsibleParty> = None;
        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ORGANIZATION_TAG => {
                    let organization =
                        OrganizationalEntity::read_xml_element(event_reader, &name, &attributes)?;
                    party = Some(DataGovernanceResponsibleParty::Organization(organization));
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == CONTACT_TAG => {
                    let contact =
                        OrganizationalContact::read_xml_element(event_reader, &name, &attributes)?;
                    party = Some(DataGovernanceResponsibleParty::Contact(contact));
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }

                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let party = party.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: "organization or contact".to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(party)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        models,
        specs::{
            common::organization::{OrganizationalContact, OrganizationalEntity},
            v1_5::modelcard::{
                Attachment, Collection, ComponentData, DataContents, DataGovernance,
                DataGovernanceResponsibleParty, Dataset, Datasets, Graphic, Graphics, Inputs,
                MLParameter, ModelCard, ModelParameters, ModelParametersApproach, Outputs,
            },
        },
        xml::test::read_element_from_string,
    };

    pub(crate) fn example_modelcard() -> ModelCard {
        ModelCard {
            bom_ref: None,
            model_parameters: None,
            quantitative_analysis: None,
            considerations: None,
            properties: None,
        }
    }

    pub(crate) fn corresponding_modelcard() -> models::modelcard::ModelCard {
        models::modelcard::ModelCard {
            bom_ref: None,
            model_parameters: None,
            quantitative_analysis: None,
            considerations: None,
            properties: None,
        }
    }

    #[test]
    fn it_should_read_xml_image_attachment() {
        let input = r#"
<image encoding="base64" content-type="image/jpeg">abcdefgh</image>
"#;
        let actual: Attachment = read_element_from_string(input);
        let expected = Attachment {
            content: "abcdefgh".to_string(),
            content_type: Some("image/jpeg".to_string()),
            encoding: Some("base64".to_string()),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_read_xml_graphic() {
        let input = r#"
<graphic>
  <name>FID vs CLIP Scores on 512x512 samples for different v1-versions</name>
  <image encoding="base64" content-type="image/jpeg">abcdefgh</image>
</graphic>
"#;
        let actual: Graphic = read_element_from_string(input);
        let expected = Graphic {
            name: Some(
                "FID vs CLIP Scores on 512x512 samples for different v1-versions".to_string(),
            ),
            image: Some(Attachment {
                content: "abcdefgh".to_string(),
                content_type: Some("image/jpeg".to_string()),
                encoding: Some("base64".to_string()),
            }),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_read_xml_graphics() {
        let input = r#"
<graphics>
  <description>Performance images</description>
  <collection>
    <graphic>
      <name>FID vs CLIP Scores on 512x512 samples for different v1-versions</name>
      <image encoding="base64" content-type="image/jpeg">abcdefgh</image>
    </graphic>
  </collection>
</graphics>
"#;
        let actual: Graphics = read_element_from_string(input);
        let expected = Graphics {
            description: Some("Performance images".to_string()),
            collection: Some(Collection(vec![Graphic {
                name: Some(
                    "FID vs CLIP Scores on 512x512 samples for different v1-versions".to_string(),
                ),
                image: Some(Attachment {
                    content: "abcdefgh".to_string(),
                    content_type: Some("image/jpeg".to_string()),
                    encoding: Some("base64".to_string()),
                }),
            }])),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_read_xml_ml_parameter() {
        let input = r#"
<input>
  <format>string</format>
</input>
"#;
        let actual: MLParameter = read_element_from_string(input);
        let expected = MLParameter::new("string");
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_parse_xml_inputs() {
        let input = r#"
<inputs>
  <input>
    <format>string</format>
  </input>
  <input>
    <format>input</format>
  </input>
</inputs>
"#;
        let actual: Inputs = read_element_from_string(input);
        let expected = Inputs(vec![MLParameter::new("string"), MLParameter::new("input")]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_read_xml_governance() {
        let input = r#"
<governance>
  <owners>
    <owner>
      <organization>
        <name>Organization 1</name>
      </organization>
    </owner>
  </owners>
  <custodians>
    <custodian>
      <contact bom-ref="custodian-1">
        <name>Custodian 1</name>
        <email>custodian@example.com</email>
      </contact>
    </custodian>
  </custodians>
</governance>
"#;
        let actual: DataGovernance = read_element_from_string(input);
        let expected = DataGovernance {
            custodians: Some(vec![DataGovernanceResponsibleParty::Contact(
                OrganizationalContact {
                    bom_ref: Some("custodian-1".to_string()),
                    name: Some("Custodian 1".to_string()),
                    email: Some("custodian@example.com".to_string()),
                    phone: None,
                },
            )]),
            stewards: None,
            owners: Some(vec![DataGovernanceResponsibleParty::Organization(
                OrganizationalEntity {
                    contact: None,
                    name: Some("Organization 1".to_string()),
                    url: None,
                },
            )]),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_read_xml_dataset() {
        let input = r#"
<dataset bom-ref="dataset-a">
  <type>dataset</type>
  <name>Training Data</name>
  <contents>
    <url>https://example.com/path/to/dataset</url>
  </contents>
  <classification>public</classification>
  <description>data description</description>
  <governance>
    <owners>
      <owner>
        <organization>
          <name>Organization name</name>
        </organization>
      </owner>
    </owners>
  </governance>
</dataset>
"#;
        let actual: Dataset = read_element_from_string(input);
        let expected = Dataset::Component(ComponentData {
            bom_ref: Some("dataset-a".to_string()),
            data_type: "dataset".to_string(),
            name: Some("Training Data".to_string()),
            contents: Some(DataContents {
                attachment: None,
                url: Some("https://example.com/path/to/dataset".to_string()),
                properties: None,
            }),
            sensitive_data: None,
            classification: Some("public".to_string()),
            graphics: None,
            description: Some("data description".to_string()),
            governance: Some(DataGovernance {
                custodians: None,
                stewards: None,
                owners: Some(vec![DataGovernanceResponsibleParty::Organization(
                    OrganizationalEntity {
                        contact: None,
                        name: Some("Organization name".to_string()),
                        url: None,
                    },
                )]),
            }),
        });
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_read_xml_model_parameters_approach() {
        let input = r#"
<approach>
  <type>supervised</type>
</approach>
"#;
        let actual: ModelParametersApproach = read_element_from_string(input);
        let expected = ModelParametersApproach {
            approach_type: Some("supervised".to_string()),
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_should_read_xml_model_parameters() {
        let input = r#"
<modelParameters>
  <approach>
    <type>supervised</type>
  </approach>
  <task>Task</task>
  <architectureFamily>Architecture</architectureFamily>
  <modelArchitecture>Model</modelArchitecture>
</modelParameters>
"#;
        let actual: ModelParameters = read_element_from_string(input);
        let expected = ModelParameters {
            approach: Some(ModelParametersApproach {
                approach_type: Some("supervised".to_string()),
            }),
            task: Some("Task".to_string()),
            architecture_family: Some("Architecture".to_string()),
            model_architecture: Some("Model".to_string()),
            datasets: None,
            inputs: None,
            outputs: None,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn it_sould_read_xml_model_card() {
        let input = r#"
<modelCard>
  <modelParameters>
    <approach>
      <type>supervised</type>
    </approach>
    <task>Task</task>
    <architectureFamily>Architecture</architectureFamily>
    <modelArchitecture>Model</modelArchitecture>
    <datasets>
      <dataset>
        <type>dataset</type>
        <name>Training Data</name>
        <contents>
          <url>https://example.com/path/to/dataset</url>
        </contents>
        <classification>public</classification>
      </dataset>
    </datasets>
    <inputs>
      <input><format>string</format></input>
    </inputs>
    <outputs>
      <output><format>image</format></output>
    </outputs>
  </modelParameters>
</modelCard>
"#;
        let actual: ModelCard = read_element_from_string(input);
        let expected = ModelCard {
            bom_ref: None,
            model_parameters: Some(ModelParameters {
                approach: Some(ModelParametersApproach {
                    approach_type: Some("supervised".to_string()),
                }),
                task: Some("Task".to_string()),
                architecture_family: Some("Architecture".to_string()),
                model_architecture: Some("Model".to_string()),
                datasets: Some(Datasets(vec![Dataset::Component(ComponentData {
                    bom_ref: None,
                    data_type: "dataset".to_string(),
                    name: Some("Training Data".to_string()),
                    contents: Some(DataContents {
                        attachment: None,
                        url: Some("https://example.com/path/to/dataset".to_string()),
                        properties: None,
                    }),
                    classification: Some("public".to_string()),
                    sensitive_data: None,
                    graphics: None,
                    description: None,
                    governance: None,
                })])),
                inputs: Some(Inputs(vec![MLParameter::new("string")])),
                outputs: Some(Outputs(vec![MLParameter::new("image")])),
            }),
            quantitative_analysis: None,
            considerations: None,
            properties: None,
        };
        assert_eq!(expected, actual);
    }
}
