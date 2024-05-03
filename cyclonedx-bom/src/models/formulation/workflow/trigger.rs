use crate::models::{attachment::Attachment, property::Properties};

use super::{input::Input, output::Output, resource_reference::ResourceReference};

pub(crate) struct Trigger {
    pub(crate) bom_ref: String,
    pub(crate) uid: String,
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) resource_references: Option<Vec<ResourceReference>>,
    pub(crate) r#type: String,
    pub(crate) event: Option<Event>,
    pub(crate) conditions: Option<Vec<Condition>>,
    pub(crate) time_activated: Option<String>,
    pub(crate) inputs: Option<Vec<Input>>,
    pub(crate) outputs: Option<Vec<Output>>,
    pub(crate) properties: Option<Properties>,
}

pub(crate) struct Event {
    pub(crate) uid: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) time_received: Option<String>,
    pub(crate) data: Option<Attachment>,
    pub(crate) source: Option<ResourceReference>,
    pub(crate) target: Option<ResourceReference>,
    pub(crate) properties: Option<Properties>,
}

pub(crate) struct Condition {
    pub(crate) description: Option<String>,
    pub(crate) expression: Option<String>,
    pub(crate) properties: Option<Properties>,
}
