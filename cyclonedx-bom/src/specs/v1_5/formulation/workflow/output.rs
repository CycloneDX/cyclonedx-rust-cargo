use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    errors::XmlReadError,
    models,
    specs::{common::property::Properties, v1_5::attachment::Attachment},
    utilities::{convert_optional, convert_vec},
    xml::{
        read_simple_tag, to_xml_read_error, unexpected_element_error, write_close_tag,
        write_simple_option_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

use super::{resource_reference::ResourceReference, EnvironmentVars, ENVIRONMENT_VARS_TAG};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Output {
    #[serde(flatten)]
    pub(crate) required: RequiredOutputField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<ResourceReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) target: Option<ResourceReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::formulation::workflow::output::Output> for Output {
    fn from(output: models::formulation::workflow::output::Output) -> Self {
        Self {
            required: match output.required {
                models::formulation::workflow::output::RequiredOutputField::Resource(resource) => {
                    RequiredOutputField::Resource {
                        resource: resource.into(),
                    }
                }
                models::formulation::workflow::output::RequiredOutputField::EnvironmentVars(
                    environment_vars,
                ) => RequiredOutputField::EnvironmentVars {
                    environment_vars: EnvironmentVars(convert_vec(environment_vars)),
                },
                models::formulation::workflow::output::RequiredOutputField::Data(data) => {
                    RequiredOutputField::Data { data: data.into() }
                }
            },
            r#type: output.r#type.map(|t| t.to_string()),
            source: convert_optional(output.source),
            target: convert_optional(output.target),
            properties: convert_optional(output.properties),
        }
    }
}

impl From<Output> for models::formulation::workflow::output::Output {
    fn from(output: Output) -> Self {
        Self {
            required: match output.required {
                RequiredOutputField::Resource { resource } => {
                    models::formulation::workflow::output::RequiredOutputField::Resource(
                        resource.into(),
                    )
                }
                RequiredOutputField::EnvironmentVars { environment_vars } => {
                    models::formulation::workflow::output::RequiredOutputField::EnvironmentVars(
                        convert_vec(environment_vars.0),
                    )
                }
                RequiredOutputField::Data { data } => {
                    models::formulation::workflow::output::RequiredOutputField::Data(data.into())
                }
            },
            r#type: output
                .r#type
                .map(models::formulation::workflow::output::Type::new_unchecked),
            source: convert_optional(output.source),
            target: convert_optional(output.target),
            properties: convert_optional(output.properties),
        }
    }
}

const OUTPUT_TAG: &str = "output";
const RESOURCE_TAG: &str = "resource";
const DATA_TAG: &str = "data";
const TYPE_TAG: &str = "type";
const SOURCE_TAG: &str = "source";
const TARGET_TAG: &str = "target";
const PROPERTIES_TAG: &str = "properties";

impl ToXml for Output {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, OUTPUT_TAG)?;

        match &self.required {
            RequiredOutputField::Resource { resource } => {
                resource.write_xml_named_element(writer, RESOURCE_TAG)?
            }
            RequiredOutputField::EnvironmentVars { environment_vars } => {
                environment_vars.write_xml_element(writer)?;
            }
            RequiredOutputField::Data { data } => {
                data.write_xml_named_element(writer, DATA_TAG)?;
            }
        }

        write_simple_option_tag(writer, TYPE_TAG, &self.r#type)?;

        if let Some(source) = &self.source {
            source.write_xml_named_element(writer, SOURCE_TAG)?;
        }

        if let Some(target) = &self.target {
            target.write_xml_named_element(writer, TARGET_TAG)?;
        }

        if let Some(properties) = &self.properties {
            properties.write_xml_element(writer)?;
        }

        write_close_tag(writer, OUTPUT_TAG)
    }
}

impl FromXml for Output {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut required = None;
        let mut r#type = None;
        let mut source = None;
        let mut target = None;
        let mut properties = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(OUTPUT_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                } => match name.local_name.as_str() {
                    RESOURCE_TAG => {
                        required = Some(RequiredOutputField::Resource {
                            resource: ResourceReference::read_xml_element(
                                event_reader,
                                name,
                                attributes,
                            )?,
                        })
                    }
                    ENVIRONMENT_VARS_TAG => {
                        required = Some(RequiredOutputField::EnvironmentVars {
                            environment_vars: EnvironmentVars::read_xml_element(
                                event_reader,
                                name,
                                attributes,
                            )?,
                        });
                    }
                    DATA_TAG => {
                        required = Some(RequiredOutputField::Data {
                            data: Attachment::read_xml_element(event_reader, name, attributes)?,
                        });
                    }
                    TYPE_TAG => r#type = Some(read_simple_tag(event_reader, name)?),
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
            r#type,
            source,
            target,
            properties,
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, rename_all_fields = "camelCase")]
pub(crate) enum RequiredOutputField {
    Resource { resource: ResourceReference },
    EnvironmentVars { environment_vars: EnvironmentVars },
    Data { data: Attachment },
}

#[cfg(test)]
mod tests {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;

    fn example_output() -> Output {
        Output {
            required: RequiredOutputField::Resource {
                resource: ResourceReference::Ref {
                    r#ref: "component-14".into(),
                },
            },
            r#type: Some("artifact".into()),
            source: Some(ResourceReference::Ref {
                r#ref: "component-15".into(),
            }),
            target: Some(ResourceReference::Ref {
                r#ref: "component-16".into(),
            }),
            properties: None,
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_output());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<output>
    <resource>
        <ref>component-14</ref>
    </resource>
    <type>artifact</type>
    <source>
        <ref>component-15</ref>
    </source>
    <target>
        <ref>component-16</ref>
    </target>
</output>
"#;
        let actual: Output = read_element_from_string(input);
        let expected = example_output();
        assert_eq!(actual, expected);
    }
}
