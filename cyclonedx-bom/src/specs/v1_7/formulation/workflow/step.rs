use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    models::formulation::workflow::step as models,
    specs::common::property::Properties,
    utilities::{convert_optional, convert_optional_vec},
    xml::{
        read_lax_validation_tag, read_list_tag, read_simple_tag, to_xml_read_error,
        unexpected_element_error, write_close_tag, write_list_tag, write_simple_option_tag,
        write_start_tag, FromXml, ToXml,
    },
};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct Step {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) commands: Option<Vec<Command>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::Step> for Step {
    fn from(step: models::Step) -> Self {
        Self {
            commands: convert_optional_vec(step.commands),
            description: step.description,
            name: step.name,
            properties: convert_optional(step.properties),
        }
    }
}

impl From<Step> for models::Step {
    fn from(step: Step) -> Self {
        Self {
            commands: convert_optional_vec(step.commands),
            description: step.description,
            name: step.name,
            properties: convert_optional(step.properties),
        }
    }
}

const COMMANDS_TAG: &str = "commands";
const STEP_TAG: &str = "step";
const DESCRIPTION_TAG: &str = "description";
const NAME_TAG: &str = "name";

impl ToXml for Step {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, STEP_TAG)?;

        if let Some(commands) = &self.commands {
            write_list_tag(writer, COMMANDS_TAG, commands)?;
        }

        write_simple_option_tag(writer, DESCRIPTION_TAG, &self.description)?;

        write_simple_option_tag(writer, NAME_TAG, &self.name)?;

        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, STEP_TAG)
    }
}

impl FromXml for Step {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut commands = None;
        let mut description = None;
        let mut name = None;
        let mut properties = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name: ref elem_name,
                    ref attributes,
                    ..
                } => match elem_name.local_name.as_str() {
                    COMMANDS_TAG => {
                        commands = Some(read_list_tag(event_reader, elem_name, COMMAND_TAG)?)
                    }
                    DESCRIPTION_TAG => {
                        description = Some(read_simple_tag(event_reader, elem_name)?)
                    }
                    NAME_TAG => name = Some(read_simple_tag(event_reader, elem_name)?),
                    PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            elem_name,
                            attributes,
                        )?)
                    }
                    _ => read_lax_validation_tag(event_reader, elem_name)?,
                },
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            commands,
            description,
            name,
            properties,
        })
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct Command {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) executed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::Command> for Command {
    fn from(command: models::Command) -> Self {
        Self {
            executed: command.executed,
            properties: convert_optional(command.properties),
        }
    }
}

impl From<Command> for models::Command {
    fn from(command: Command) -> Self {
        Self {
            executed: command.executed,
            properties: convert_optional(command.properties),
        }
    }
}

const COMMAND_TAG: &str = "command";
const EXECUTED_TAG: &str = "executed";
const PROPERTIES_TAG: &str = "properties";

impl ToXml for Command {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, COMMAND_TAG)?;

        write_simple_option_tag(writer, EXECUTED_TAG, &self.executed)?;

        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, COMMAND_TAG)
    }
}

impl FromXml for Command {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut executed = None;
        let mut properties = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(COMMAND_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                } => match name.local_name.as_str() {
                    EXECUTED_TAG => executed = Some(read_simple_tag(event_reader, name)?),
                    PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            name,
                            attributes,
                        )?)
                    }
                    _ => return Err(unexpected_element_error(element_name, next_element)),
                },
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            executed,
            properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        specs::common::property::Property,
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::*;

    fn example_step() -> Step {
        Step {
            commands: Some(vec![Command {
                executed: Some("ls -las".into()),
                properties: Some(Properties(vec![Property {
                    name: "Foo".into(),
                    value: "Bar".into(),
                }])),
            }]),
            description: Some("Description here".into()),
            name: Some("My step".into()),
            properties: Some(Properties(vec![Property {
                name: "Foo".into(),
                value: "Bar".into(),
            }])),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_step());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<step>
    <name>My step</name>
    <description>Description here</description>
    <commands>
        <command>
            <executed>ls -las</executed>
            <properties>
                <property name="Foo">Bar</property>
            </properties>
        </command>
    </commands>
    <properties>
        <property name="Foo">Bar</property>
    </properties>
</step>
"#;
        let actual: Step = read_element_from_string(input);
        let expected = example_step();
        assert_eq!(actual, expected);
    }
}
