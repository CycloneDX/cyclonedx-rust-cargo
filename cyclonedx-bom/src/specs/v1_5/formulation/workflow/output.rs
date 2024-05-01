use serde::{Deserialize, Serialize};
use xml::{reader, writer};

use crate::{
    errors::XmlReadError,
    specs::{common::property::Properties, v1_5::attachment::Attachment},
    xml::{
        attribute_or_error, read_lax_validation_tag, read_simple_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_close_tag, write_simple_tag,
        write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

use super::resource_reference::ResourceReference;
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Outputs(pub(crate) Vec<Output>);

const OUTPUTS_TAG: &str = "outputs";

impl ToXml for Outputs {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, OUTPUTS_TAG)?;

        for output in &self.0 {
            output.write_xml_element(writer)?;
        }

        write_close_tag(writer, OUTPUTS_TAG)
    }
}

impl FromXml for Outputs {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut outputs = vec![];

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(OUTPUTS_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == OUTPUT_TAG => {
                    outputs.push(Output::read_xml_element(event_reader, &name, &attributes)?);
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

        Ok(Self(outputs))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Output {
    #[serde(flatten)]
    pub(crate) required: RequiredOutputField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) r#type: Option<Type>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<ResourceReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) target: Option<ResourceReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

const OUTPUT_TAG: &str = "output";
const RESOURCE_TAG: &str = "resource";
const DATA_TAG: &str = "data";
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

        if let Some(r#type) = &self.r#type {
            r#type.write_xml_element(writer)?;
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
                                &name,
                                &attributes,
                            )?,
                        })
                    }
                    ENVIRONMENT_VARS_TAG => {
                        required = Some(RequiredOutputField::EnvironmentVars {
                            environment_vars: EnvironmentVars::read_xml_element(
                                event_reader,
                                &name,
                                &attributes,
                            )?,
                        });
                    }
                    DATA_TAG => {
                        required = Some(RequiredOutputField::Data {
                            data: Attachment::read_xml_element(event_reader, &name, &attributes)?,
                        });
                    }
                    TYPE_TAG => {
                        r#type = Some(Type::read_xml_element(event_reader, &name, &attributes)?)
                    }
                    SOURCE_TAG => {
                        source = Some(ResourceReference::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    TARGET_TAG => {
                        target = Some(ResourceReference::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
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
            .ok_or_else(|| XmlReadError::required_data_missing(RESOURCE_TAG, &element_name))?;

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
#[serde(untagged, rename_all = "camelCase")]
pub(crate) enum RequiredOutputField {
    Resource { resource: ResourceReference },
    EnvironmentVars { environment_vars: EnvironmentVars },
    Data { data: Attachment },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct EnvironmentVars(Vec<EnvironmentVar>);

const ENVIRONMENT_VARS_TAG: &str = "environmentVars";

impl ToXml for EnvironmentVars {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, ENVIRONMENT_VARS_TAG)?;

        for environment_var in &self.0 {
            environment_var.write_xml_element(writer)?;
        }

        write_close_tag(writer, ENVIRONMENT_VARS_TAG)
    }
}

impl FromXml for EnvironmentVars {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut environment_vars = vec![];

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(ENVIRONMENT_VARS_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ENVIRONMENT_VARS_TAG => {
                    environment_vars.push(EnvironmentVar::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
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

        Ok(Self(environment_vars))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub(crate) enum EnvironmentVar {
    Property { name: String, value: String },
    Value(String),
}

const ENVIRONMENT_VAR_TAG: &str = "environmentVar";
const VALUE_TAG: &str = "value";
const NAME_ATTR: &str = "name";

impl ToXml for EnvironmentVar {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        match self {
            Self::Property { name, value } => {
                writer
                    .write(writer::XmlEvent::start_element(ENVIRONMENT_VAR_TAG).attr("name", name))
                    .map_err(to_xml_write_error(ENVIRONMENT_VAR_TAG))?;

                writer
                    .write(writer::XmlEvent::characters(value))
                    .map_err(to_xml_write_error(ENVIRONMENT_VAR_TAG))?;

                write_close_tag(writer, ENVIRONMENT_VAR_TAG)?;
            }
            Self::Value(value) => {
                write_simple_tag(writer, VALUE_TAG, value)?;
            }
        }

        Ok(())
    }
}

impl FromXml for EnvironmentVar {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut environment_var = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(ENVIRONMENT_VAR_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement {
                    name: ref elem_name,
                    ref attributes,
                    ..
                } => match elem_name.local_name.as_str() {
                    ENVIRONMENT_VAR_TAG => {
                        let name = attribute_or_error(element_name, attributes, NAME_ATTR)?;
                        let value = read_simple_tag(event_reader, elem_name)?;
                        environment_var = Some(Self::Property { name, value });
                    }
                    VALUE_TAG => {
                        let value = read_simple_tag(event_reader, elem_name)?;
                        environment_var = Some(Self::Value(value));
                    }
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

        environment_var.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: ENVIRONMENT_VAR_TAG.into(),
            element: element_name.local_name.to_string(),
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Type {
    Artifact,
    Attestation,
    Log,
    Evidence,
    Metrics,
    Other,
}

const TYPE_TAG: &str = "type";

impl ToXml for Type {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let content = match self {
            Self::Artifact => "artifact",
            Self::Attestation => "attestation",
            Self::Log => "log",
            Self::Evidence => "evidence",
            Self::Metrics => "metrics",
            Self::Other => "other",
        };

        write_simple_tag(writer, TYPE_TAG, content)
    }
}

impl FromXml for Type {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        match read_simple_tag(event_reader, &element_name)?.as_str() {
            "artifact" => Ok(Self::Artifact),
            "attestation" => Ok(Self::Attestation),
            "log" => Ok(Self::Log),
            "evidence" => Ok(Self::Evidence),
            "metrics" => Ok(Self::Metrics),
            "other" => Ok(Self::Other),
            unexpected => Err(XmlReadError::UnexpectedElementReadError {
                error: format!("Got unexpected element {:?}", unexpected),
                element: TYPE_TAG.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;

    fn example_outputs() -> Outputs {
        Outputs(vec![Output {
            required: RequiredOutputField::Resource {
                resource: ResourceReference::Ref {
                    r#ref: "component-14".into(),
                },
            },
            r#type: Some(Type::Artifact),
            source: Some(ResourceReference::Ref {
                r#ref: "component-15".into(),
            }),
            target: Some(ResourceReference::Ref {
                r#ref: "component-16".into(),
            }),
            properties: None,
        }])
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_outputs());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<outputs>
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
</outputs>
"#;
        let actual: Outputs = read_element_from_string(input);
        let expected = example_outputs();
        assert_eq!(actual, expected);
    }
}
