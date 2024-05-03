use crate::models::{attachment::Attachment, property::Properties};

use super::resource_reference::ResourceReference;

pub(crate) struct Output {
    pub(crate) required: RequiredOutputField,
    pub(crate) r#type: Option<Type>,
    pub(crate) source: Option<ResourceReference>,
    pub(crate) target: Option<ResourceReference>,
    pub(crate) properties: Option<Properties>,
}

pub(crate) enum RequiredOutputField {
    Resource { resource: ResourceReference },
    EnvironmentVars { environment_vars: Vec< EnvironmentVar > },
    Data { data: Attachment },
}

pub(crate) enum Type {
    Artifact,
    Attestation,
    Log,
    Evidence,
    Metrics,
    Other,
}

pub(crate) enum EnvironmentVar {
    Property { name: String, value: String },
    Value(String),
}
