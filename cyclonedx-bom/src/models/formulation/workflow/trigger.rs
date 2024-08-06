use crate::{
    external_models::validate_date_time,
    models::{
        attachment::Attachment,
        bom::{validate_bom_ref, BomReference},
        property::Properties,
    },
    prelude::{DateTime, Validate},
    validation::{ValidationContext, ValidationError},
};

use super::{input::Input, output::Output, resource_reference::ResourceReference};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Trigger {
    pub bom_ref: BomReference,
    pub uid: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub resource_references: Option<Vec<ResourceReference>>,
    pub r#type: Type,
    pub event: Option<Event>,
    pub conditions: Option<Vec<Condition>>,
    pub time_activated: Option<DateTime>,
    pub inputs: Option<Vec<Input>>,
    pub outputs: Option<Vec<Output>>,
    pub properties: Option<Properties>,
}

impl Validate for Trigger {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        ValidationContext::new()
            .add_field("bom_ref", &self.bom_ref, |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .add_list_option(
                "resource_reference",
                self.resource_references.as_ref(),
                |resource_reference| resource_reference.validate_version(version),
            )
            .add_struct("type", &self.r#type, version)
            .add_struct_option("event", self.event.as_ref(), version)
            .add_unique_list_option("conditions", self.conditions.as_ref(), |condition| {
                condition.validate_version(version)
            })
            .add_field_option(
                "time_activated",
                self.time_activated.as_ref(),
                validate_date_time,
            )
            .add_unique_list_option("inputs", self.inputs.as_ref(), |input| {
                input.validate_version(version)
            })
            .add_unique_list_option("outputs", self.outputs.as_ref(), |output| {
                output.validate_version(version)
            })
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}

#[derive(Debug, Clone, strum::Display, PartialEq, Eq, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum Type {
    Manual,
    Api,
    Webhook,
    Scheduled,
    #[strum(default)]
    #[doc(hidden)]
    UnknownType(String),
}

impl Type {
    pub fn new_unchecked<S: AsRef<str>>(s: S) -> Self {
        match s.as_ref() {
            "manual" => Self::Manual,
            "api" => Self::Api,
            "webhook" => Self::Webhook,
            "scheduled" => Self::Scheduled,
            unknown => Self::UnknownType(unknown.to_string()),
        }
    }
}

impl Validate for Type {
    fn validate_version(
        &self,
        _version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        match self {
            Self::UnknownType(_) => Err(ValidationError::new("unknown trigger type")),
            _ => Ok(()),
        }
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Event {
    pub uid: Option<String>,
    pub description: Option<String>,
    pub time_received: Option<DateTime>,
    pub data: Option<Attachment>,
    pub source: Option<ResourceReference>,
    pub target: Option<ResourceReference>,
    pub properties: Option<Properties>,
}

impl Validate for Event {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        ValidationContext::new()
            .add_field_option(
                "time_received",
                self.time_received.as_ref(),
                validate_date_time,
            )
            .add_struct_option("data", self.data.as_ref(), version)
            .add_struct_option("source", self.source.as_ref(), version)
            .add_struct_option("target", self.target.as_ref(), version)
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Condition {
    pub description: Option<String>,
    pub expression: Option<String>,
    pub properties: Option<Properties>,
}

impl Validate for Condition {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        ValidationContext::new()
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}
