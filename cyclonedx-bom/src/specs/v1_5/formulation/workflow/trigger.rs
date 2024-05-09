use serde::{Deserialize, Serialize};
use xml::{reader, writer};

use crate::{
    errors::XmlReadError,
    models::formulation::workflow::trigger as models,
    specs::{common::property::Properties, v1_5::attachment::Attachment},
    utilities::{convert_optional, convert_optional_vec},
    xml::{
        optional_attribute, read_lax_validation_tag, read_list_tag, read_simple_tag,
        to_xml_read_error, to_xml_write_error, unexpected_element_error, write_close_tag,
        write_list_tag, write_simple_option_tag, write_simple_tag, write_start_tag, FromXml,
        ToInnerXml, ToXml,
    },
};

use super::{
    input::Input,
    output::Output,
    resource_reference::{ResourceReference, ResourceReferences},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Trigger {
    #[serde(rename = "bom-ref")]
    pub(crate) bom_ref: String,
    pub(crate) uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resource_references: Option<ResourceReferences>,
    pub(crate) r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) event: Option<Event>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) conditions: Option<Vec<Condition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) time_activated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) inputs: Option<Vec<Input>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) outputs: Option<Vec<Output>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::Trigger> for Trigger {
    fn from(trigger: models::Trigger) -> Self {
        Self {
            bom_ref: trigger.bom_ref.0,
            uid: trigger.uid,
            name: trigger.name,
            description: trigger.description,
            resource_references: convert_optional_vec(trigger.resource_references)
                .map(ResourceReferences),
            r#type: trigger.r#type.to_string(),
            event: convert_optional(trigger.event),
            conditions: convert_optional_vec(trigger.conditions),
            time_activated: trigger.time_activated.map(|dt| dt.0),
            inputs: convert_optional_vec(trigger.inputs),
            outputs: convert_optional_vec(trigger.outputs),
            properties: convert_optional(trigger.properties),
        }
    }
}

impl From<Trigger> for models::Trigger {
    fn from(trigger: Trigger) -> Self {
        Self {
            bom_ref: crate::models::bom::BomReference::new(trigger.bom_ref),
            uid: trigger.uid,
            name: trigger.name,
            description: trigger.description,
            resource_references: convert_optional_vec(trigger.resource_references.map(|rs| rs.0)),
            r#type: models::Type::new_unchecked(trigger.r#type),
            event: convert_optional(trigger.event),
            conditions: convert_optional_vec(trigger.conditions),
            time_activated: trigger.time_activated.map(crate::prelude::DateTime),
            inputs: convert_optional_vec(trigger.inputs),
            outputs: convert_optional_vec(trigger.outputs),
            properties: convert_optional(trigger.properties),
        }
    }
}

const TRIGGER_TAG: &str = "trigger";
const BOM_REF_ATTR: &str = "bom-ref";
const NAME_TAG: &str = "name";
const RESOURCE_REFERENCES_TAG: &str = "resourceReferences";
const TYPE_TAG: &str = "type";
const TIME_ACTIVATED_TAG: &str = "timeActivated";
const CONDITIONS_TAG: &str = "conditions";
const INPUTS_TAG: &str = "inputs";
const INPUT_TAG: &str = "input";
const OUTPUTS_TAG: &str = "outputs";
const OUTPUT_TAG: &str = "output";

impl ToXml for Trigger {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(TRIGGER_TAG).attr(BOM_REF_ATTR, &self.bom_ref))
            .map_err(to_xml_write_error(TRIGGER_TAG))?;

        write_simple_tag(writer, UID_TAG, &self.uid)?;
        write_simple_option_tag(writer, NAME_TAG, &self.name)?;
        write_simple_option_tag(writer, DESCRIPTION_TAG, &self.description)?;
        self.resource_references.write_xml_element(writer)?;
        write_simple_tag(writer, TYPE_TAG, &self.r#type)?;
        self.event.write_xml_element(writer)?;
        if let Some(conditions) = &self.conditions {
            write_list_tag(writer, CONDITIONS_TAG, conditions)?;
        }
        write_simple_option_tag(writer, TIME_ACTIVATED_TAG, &self.time_activated)?;
        if let Some(inputs) = &self.inputs {
            write_list_tag(writer, INPUTS_TAG, inputs)?;
        }
        if let Some(outputs) = &self.outputs {
            write_list_tag(writer, OUTPUTS_TAG, outputs)?;
        }
        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, TRIGGER_TAG)
    }
}

impl FromXml for Trigger {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR)
            .ok_or_else(|| XmlReadError::required_data_missing(BOM_REF_ATTR, element_name))?;

        let mut uid = None;
        let mut name = None;
        let mut description = None;
        let mut resource_references = None;
        let mut r#type = None;
        let mut event = None;
        let mut conditions = None;
        let mut time_activated = None;
        let mut inputs = None;
        let mut outputs = None;
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
                    UID_TAG => uid = Some(read_simple_tag(event_reader, elem_name)?),
                    NAME_TAG => name = Some(read_simple_tag(event_reader, elem_name)?),
                    DESCRIPTION_TAG => {
                        description = Some(read_simple_tag(event_reader, elem_name)?)
                    }
                    RESOURCE_REFERENCES_TAG => {
                        resource_references = Some(ResourceReferences::read_xml_element(
                            event_reader,
                            elem_name,
                            attributes,
                        )?)
                    }
                    TYPE_TAG => r#type = Some(read_simple_tag(event_reader, elem_name)?),
                    EVENT_TAG => {
                        event = Some(Event::read_xml_element(
                            event_reader,
                            elem_name,
                            attributes,
                        )?)
                    }
                    CONDITIONS_TAG => {
                        conditions = Some(read_list_tag(event_reader, elem_name, CONDITION_TAG)?)
                    }
                    TIME_ACTIVATED_TAG => {
                        time_activated = Some(read_simple_tag(event_reader, elem_name)?)
                    }
                    INPUTS_TAG => {
                        inputs = Some(read_list_tag(event_reader, elem_name, INPUT_TAG)?);
                    }
                    OUTPUTS_TAG => {
                        outputs = Some(read_list_tag(event_reader, elem_name, OUTPUT_TAG)?)
                    }
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

        let uid = uid.ok_or_else(|| XmlReadError::required_data_missing(UID_TAG, element_name))?;
        let r#type =
            r#type.ok_or_else(|| XmlReadError::required_data_missing(TYPE_TAG, element_name))?;

        Ok(Self {
            bom_ref,
            uid,
            name,
            description,
            resource_references,
            r#type,
            event,
            conditions,
            time_activated,
            inputs,
            outputs,
            properties,
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Event {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) time_received: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<ResourceReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) target: Option<ResourceReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::Event> for Event {
    fn from(event: models::Event) -> Self {
        Self {
            uid: event.uid,
            description: event.description,
            time_received: event.time_received.map(|dt| dt.0),
            data: convert_optional(event.data),
            source: convert_optional(event.source),
            target: convert_optional(event.target),
            properties: convert_optional(event.properties),
        }
    }
}

impl From<Event> for models::Event {
    fn from(event: Event) -> Self {
        Self {
            uid: event.uid,
            description: event.description,
            time_received: event.time_received.map(crate::prelude::DateTime),
            data: convert_optional(event.data),
            source: convert_optional(event.source),
            target: convert_optional(event.target),
            properties: convert_optional(event.properties),
        }
    }
}

const EVENT_TAG: &str = "event";
const UID_TAG: &str = "uid";
const DESCRIPTION_TAG: &str = "description";
const TIME_RECEIVED_TAG: &str = "timeReceived";
const DATA_TAG: &str = "data";
const SOURCE_TAG: &str = "source";
const TARGET_TAG: &str = "target";

impl ToXml for Event {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, EVENT_TAG)?;

        write_simple_option_tag(writer, UID_TAG, &self.uid)?;
        write_simple_option_tag(writer, DESCRIPTION_TAG, &self.description)?;
        write_simple_option_tag(writer, TIME_RECEIVED_TAG, &self.time_received)?;
        self.data.write_xml_named_element(writer, DATA_TAG)?;
        self.source.write_xml_named_element(writer, SOURCE_TAG)?;
        self.target.write_xml_named_element(writer, TARGET_TAG)?;
        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, EVENT_TAG)
    }
}

impl FromXml for Event {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut uid = None;
        let mut description = None;
        let mut time_received = None;
        let mut data = None;
        let mut source = None;
        let mut target = None;
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
                    UID_TAG => uid = Some(read_simple_tag(event_reader, elem_name)?),
                    DESCRIPTION_TAG => {
                        description = Some(read_simple_tag(event_reader, elem_name)?)
                    }
                    TIME_RECEIVED_TAG => {
                        time_received = Some(read_simple_tag(event_reader, elem_name)?)
                    }
                    DATA_TAG => {
                        data = Some(Attachment::read_xml_element(
                            event_reader,
                            elem_name,
                            attributes,
                        )?)
                    }
                    SOURCE_TAG => {
                        source = Some(ResourceReference::read_xml_element(
                            event_reader,
                            elem_name,
                            attributes,
                        )?)
                    }
                    TARGET_TAG => {
                        target = Some(ResourceReference::read_xml_element(
                            event_reader,
                            elem_name,
                            attributes,
                        )?)
                    }
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
            uid,
            description,
            time_received,
            data,
            source,
            target,
            properties,
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Condition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) expression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::Condition> for Condition {
    fn from(condition: models::Condition) -> Self {
        Self {
            description: condition.description,
            expression: condition.expression,
            properties: convert_optional(condition.properties),
        }
    }
}

impl From<Condition> for models::Condition {
    fn from(condition: Condition) -> Self {
        Self {
            description: condition.description,
            expression: condition.expression,
            properties: convert_optional(condition.properties),
        }
    }
}

const CONDITION_TAG: &str = "condition";
const EXPRESSION_TAG: &str = "expression";
const PROPERTIES_TAG: &str = "properties";

impl ToXml for Condition {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, CONDITION_TAG)?;

        write_simple_option_tag(writer, DESCRIPTION_TAG, &self.description)?;
        write_simple_option_tag(writer, EXPRESSION_TAG, &self.expression)?;
        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, CONDITION_TAG)
    }
}

impl FromXml for Condition {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut description = None;
        let mut expression = None;
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
                    DESCRIPTION_TAG => {
                        description = Some(read_simple_tag(event_reader, elem_name)?)
                    }
                    EXPRESSION_TAG => expression = Some(read_simple_tag(event_reader, elem_name)?),
                    PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            elem_name,
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
            description,
            expression,
            properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        specs::{
            common::property::Property,
            v1_5::formulation::workflow::{
                input::{Input, RequiredInputField},
                output::{Output, RequiredOutputField},
            },
        },
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::*;

    fn example_trigger() -> Trigger {
        Trigger {
            bom_ref: "trigger-2".into(),
            uid: "trigger-uid-1".into(),
            name: Some("My trigger".into()),
            description: Some("Description here".into()),
            resource_references: Some(ResourceReferences(vec![ResourceReference::Ref {
                r#ref: "component-a".into(),
            }])),
            r#type: "api".into(),
            event: Some(Event {
                uid: Some("event-1".into()),
                description: Some("Description here".into()),
                time_received: Some("2023-01-01T00:00:00+00:00".into()),
                data: Some(Attachment {
                    content: "FooBar".into(),
                    content_type: None,
                    encoding: None,
                }),
                source: Some(ResourceReference::Ref {
                    r#ref: "component-g".into(),
                }),
                target: Some(ResourceReference::Ref {
                    r#ref: "component-h".into(),
                }),
                properties: Some(Properties(vec![Property {
                    name: "Foo".into(),
                    value: "Bar".into(),
                }])),
            }),
            conditions: Some(vec![Condition {
                description: Some("Description here".into()),
                expression: Some("1 == 1".into()),
                properties: Some(Properties(vec![Property {
                    name: "Foo".into(),
                    value: "Bar".into(),
                }])),
            }]),
            time_activated: Some("2023-01-01T00:00:00+00:00".into()),
            inputs: Some(vec![Input {
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
            }]),
            outputs: Some(vec![Output {
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
            }]),
            properties: Some(Properties(vec![Property {
                name: "Foo".into(),
                value: "Bar".into(),
            }])),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_trigger());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<trigger bom-ref="trigger-2">
    <uid>trigger-uid-1</uid>
    <name>My trigger</name>
    <description>Description here</description>
    <resourceReferences>
        <resourceReference>
            <ref>component-a</ref>
        </resourceReference>
    </resourceReferences>
    <type>api</type>
    <event>
        <uid>event-1</uid>
        <description>Description here</description>
        <timeReceived>2023-01-01T00:00:00+00:00</timeReceived>
        <data>FooBar</data>
        <source>
            <ref>component-g</ref>
        </source>
        <target>
            <ref>component-h</ref>
        </target>
        <properties>
            <property name="Foo">Bar</property>
        </properties>
    </event>
    <conditions>
        <condition>
            <description>Description here</description>
            <expression>1 == 1</expression>
            <properties>
                <property name="Foo">Bar</property>
            </properties>
        </condition>
    </conditions>
    <timeActivated>2023-01-01T00:00:00+00:00</timeActivated>
    <inputs>
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
    </inputs>
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
    <properties>
        <property name="Foo">Bar</property>
    </properties>
</trigger>
"#;
        let actual: Trigger = read_element_from_string(input);
        let expected = example_trigger();
        pretty_assertions::assert_eq!(actual, expected);
    }
}
