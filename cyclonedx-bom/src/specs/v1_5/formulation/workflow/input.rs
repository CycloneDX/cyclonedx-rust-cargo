use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    errors::XmlReadError,
    models,
    specs::{common::property::Properties, v1_5::attachment::Attachment},
    utilities::{convert_optional, convert_vec},
    xml::{
        read_simple_tag, to_xml_read_error, unexpected_element_error, write_close_tag,
        write_list_tag, write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

use super::{resource_reference::ResourceReference, EnvironmentVars, ENVIRONMENT_VARS_TAG};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Input {
    #[serde(flatten)]
    pub(crate) required: RequiredInputField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<ResourceReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) target: Option<ResourceReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::formulation::workflow::input::Input> for Input {
    fn from(input: models::formulation::workflow::input::Input) -> Self {
        Self {
            required: match input.required {
                models::formulation::workflow::input::RequiredInputField::Resource(resource) => {
                    RequiredInputField::Resource {
                        resource: resource.into(),
                    }
                }
                models::formulation::workflow::input::RequiredInputField::Parameters(
                    parameters,
                ) => RequiredInputField::Parameters {
                    parameters: convert_vec(parameters),
                },
                models::formulation::workflow::input::RequiredInputField::EnvironmentVars(
                    environment_vars,
                ) => RequiredInputField::EnvironmentVars {
                    environment_vars: EnvironmentVars(convert_vec(environment_vars)),
                },
                models::formulation::workflow::input::RequiredInputField::Data(data) => {
                    RequiredInputField::Data { data: data.into() }
                }
            },
            source: convert_optional(input.source),
            target: convert_optional(input.target),
            properties: convert_optional(input.properties),
        }
    }
}

impl From<Input> for models::formulation::workflow::input::Input {
    fn from(input: Input) -> Self {
        Self {
            required: match input.required {
                RequiredInputField::Resource { resource } => {
                    models::formulation::workflow::input::RequiredInputField::Resource(
                        resource.into(),
                    )
                }
                RequiredInputField::Parameters { parameters } => {
                    models::formulation::workflow::input::RequiredInputField::Parameters(
                        convert_vec(parameters),
                    )
                }
                RequiredInputField::EnvironmentVars { environment_vars } => {
                    models::formulation::workflow::input::RequiredInputField::EnvironmentVars(
                        convert_vec(environment_vars.0),
                    )
                }
                RequiredInputField::Data { data } => {
                    models::formulation::workflow::input::RequiredInputField::Data(data.into())
                }
            },
            source: convert_optional(input.source),
            target: convert_optional(input.target),
            properties: convert_optional(input.properties),
        }
    }
}

const INPUT_TAG: &str = "input";
const RESOURCE_TAG: &str = "resource";
const DATA_TAG: &str = "data";
const PARAMETERS_TAG: &str = "parameters";
const SOURCE_TAG: &str = "source";
const TARGET_TAG: &str = "target";
const PROPERTIES_TAG: &str = "properties";

impl ToXml for Input {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, INPUT_TAG)?;

        match &self.required {
            RequiredInputField::Resource { resource } => {
                resource.write_xml_named_element(writer, RESOURCE_TAG)?
            }
            RequiredInputField::Parameters { parameters } => {
                write_list_tag(writer, PARAMETERS_TAG, parameters)?
            }
            RequiredInputField::EnvironmentVars { environment_vars } => {
                environment_vars.write_xml_element(writer)?;
            }
            RequiredInputField::Data { data } => {
                data.write_xml_named_element(writer, DATA_TAG)?;
            }
        }

        if let Some(source) = &self.source {
            source.write_xml_named_element(writer, SOURCE_TAG)?;
        }

        if let Some(target) = &self.target {
            target.write_xml_named_element(writer, TARGET_TAG)?;
        }

        if let Some(properties) = &self.properties {
            properties.write_xml_element(writer)?;
        }

        write_close_tag(writer, INPUT_TAG)
    }
}

impl FromXml for Input {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut required = None;
        let mut source = None;
        let mut target = None;
        let mut properties = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(INPUT_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                } => match name.local_name.as_str() {
                    RESOURCE_TAG => {
                        required = Some(RequiredInputField::Resource {
                            resource: ResourceReference::read_xml_element(
                                event_reader,
                                name,
                                attributes,
                            )?,
                        })
                    }
                    ENVIRONMENT_VARS_TAG => {
                        required = Some(RequiredInputField::EnvironmentVars {
                            environment_vars: EnvironmentVars::read_xml_element(
                                event_reader,
                                name,
                                attributes,
                            )?,
                        });
                    }
                    DATA_TAG => {
                        required = Some(RequiredInputField::Data {
                            data: Attachment::read_xml_element(event_reader, name, attributes)?,
                        });
                    }
                    SOURCE_TAG => {
                        source = Some(ResourceReference::read_xml_element(
                            event_reader,
                            name,
                            attributes,
                        )?);
                    }
                    TARGET_TAG => {
                        target = Some(ResourceReference::read_xml_element(
                            event_reader,
                            name,
                            attributes,
                        )?);
                    }
                    PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            name,
                            attributes,
                        )?)
                    }
                    _ => {
                        return Err(unexpected_element_error(
                            name.local_name.clone(),
                            next_element,
                        ))
                    }
                },
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let required = required
            .ok_or_else(|| XmlReadError::required_data_missing(RESOURCE_TAG, element_name))?;

        Ok(Self {
            required,
            source,
            target,
            properties,
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, rename_all_fields = "camelCase")]
pub(crate) enum RequiredInputField {
    Resource { resource: ResourceReference },
    Parameters { parameters: Vec<Parameter> },
    EnvironmentVars { environment_vars: EnvironmentVars },
    Data { data: Attachment },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Parameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_type: Option<String>,
}

impl From<models::formulation::workflow::input::Parameter> for Parameter {
    fn from(parameter: models::formulation::workflow::input::Parameter) -> Self {
        Self {
            name: parameter.name,
            value: parameter.value,
            data_type: parameter.data_type,
        }
    }
}

impl From<Parameter> for models::formulation::workflow::input::Parameter {
    fn from(parameter: Parameter) -> Self {
        Self {
            name: parameter.name,
            value: parameter.value,
            data_type: parameter.data_type,
        }
    }
}

const PARAMETER_TAG: &str = "parameter";
const NAME_TAG: &str = "name";
const VALUE_TAG: &str = "value";
const DATA_TYPE_TAG: &str = "dataType";

impl ToXml for Parameter {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, PARAMETER_TAG)?;

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(value) = &self.value {
            write_simple_tag(writer, VALUE_TAG, value)?;
        }

        if let Some(data_type) = &self.data_type {
            write_simple_tag(writer, DATA_TYPE_TAG, data_type)?;
        }

        write_close_tag(writer, PARAMETER_TAG)
    }
}

impl FromXml for Parameter {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut name = None;
        let mut value = None;
        let mut data_type = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(PARAMETER_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name: ref elem_name,
                    ..
                } => match elem_name.local_name.as_str() {
                    NAME_TAG => name = Some(read_simple_tag(event_reader, elem_name)?),
                    VALUE_TAG => value = Some(read_simple_tag(event_reader, elem_name)?),
                    DATA_TYPE_TAG => data_type = Some(read_simple_tag(event_reader, elem_name)?),
                    _ => {
                        return Err(unexpected_element_error(
                            elem_name.to_string(),
                            next_element,
                        ))
                    }
                },
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            name,
            value,
            data_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;

    fn example_input() -> Input {
        Input {
            required: RequiredInputField::Resource {
                resource: ResourceReference::Ref {
                    r#ref: "component-10".into(),
                },
            },
            source: Some(ResourceReference::Ref {
                r#ref: "component-11".into(),
            }),
            target: Some(ResourceReference::Ref {
                r#ref: "component-12".into(),
            }),
            properties: None,
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_input = write_element_to_string(example_input());
        insta::assert_snapshot!(xml_input);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<input>
    <resource>
        <ref>component-10</ref>
    </resource>
    <source>
        <ref>component-11</ref>
    </source>
    <target>
        <ref>component-12</ref>
    </target>
</input>
"#;
        let actual: Input = read_element_from_string(input);
        let expected = example_input();
        assert_eq!(actual, expected);
    }
}
