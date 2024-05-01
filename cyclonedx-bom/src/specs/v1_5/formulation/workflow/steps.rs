use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    specs::common::property::Properties,
    xml::{
        read_lax_validation_list_tag, read_lax_validation_tag, read_list_tag, read_simple_tag,
        to_xml_read_error, unexpected_element_error, write_close_tag, write_list_tag,
        write_simple_option_tag, write_start_tag, FromXml, ToXml,
    },
};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct Steps(Vec<Step>);

const STEPS_TAG: &str = "steps";

impl ToXml for Steps {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_list_tag(writer, STEPS_TAG, &self.0)
    }
}

impl FromXml for Steps {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        read_lax_validation_list_tag(event_reader, element_name, STEP_TAG).map(Self)
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct Step {
    #[serde(skip_serializing_if = "Option::is_none")]
    commands: Option<Commands>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
}

const STEP_TAG: &str = "step";
const DESCRIPTION_TAG: &str = "description";
const NAME_TAG: &str = "name";

impl ToXml for Step {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, STEP_TAG)?;

        self.commands.write_xml_element(writer)?;

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
                        commands = Some(Commands::read_xml_element(
                            event_reader,
                            &elem_name,
                            &attributes,
                        )?)
                    }
                    DESCRIPTION_TAG => {
                        description = Some(read_simple_tag(event_reader, &elem_name)?)
                    }
                    NAME_TAG => name = Some(read_simple_tag(event_reader, &elem_name)?),
                    PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            &elem_name,
                            &attributes,
                        )?)
                    }
                    _ => read_lax_validation_tag(event_reader, &elem_name)?,
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
pub(crate) struct Commands(Vec<Command>);

const COMMANDS_TAG: &str = "commands";

impl ToXml for Commands {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_list_tag(writer, COMMANDS_TAG, &self.0)
    }
}

impl FromXml for Commands {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, COMMAND_TAG).map(Self)
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct Command {
    #[serde(skip_serializing_if = "Option::is_none")]
    executed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
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
                    EXECUTED_TAG => executed = Some(read_simple_tag(event_reader, &name)?),
                    PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
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

    fn example_steps() -> Steps {
        Steps(vec![Step {
            commands: Some(Commands(vec![Command {
                executed: Some("ls -las".into()),
                properties: Some(Properties(vec![Property {
                    name: "Foo".into(),
                    value: "Bar".into(),
                }])),
            }])),
            description: Some("Description here".into()),
            name: Some("My step".into()),
            properties: Some(Properties(vec![Property {
                name: "Foo".into(),
                value: "Bar".into(),
            }])),
        }])
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_steps());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<steps>
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
</steps>
"#;
        let actual: Steps = read_element_from_string(input);
        let expected = example_steps();
        assert_eq!(actual, expected);
    }
}
