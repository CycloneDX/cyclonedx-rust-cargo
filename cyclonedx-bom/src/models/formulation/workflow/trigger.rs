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
pub(crate) struct Trigger {
    pub(crate) bom_ref: BomReference,
    pub(crate) uid: String,
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) resource_references: Option<Vec<ResourceReference>>,
    pub(crate) r#type: Type,
    pub(crate) event: Option<Event>,
    pub(crate) conditions: Option<Vec<Condition>>,
    pub(crate) time_activated: Option<DateTime>,
    pub(crate) inputs: Option<Vec<Input>>,
    pub(crate) outputs: Option<Vec<Output>>,
    pub(crate) properties: Option<Properties>,
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
pub(crate) struct Event {
    pub(crate) uid: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) time_received: Option<DateTime>,
    pub(crate) data: Option<Attachment>,
    pub(crate) source: Option<ResourceReference>,
    pub(crate) target: Option<ResourceReference>,
    pub(crate) properties: Option<Properties>,
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
pub(crate) struct Condition {
    pub(crate) description: Option<String>,
    pub(crate) expression: Option<String>,
    pub(crate) properties: Option<Properties>,
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
