use crate::models::{attachment::Attachment, property::Properties};

use super::{resource_reference::ResourceReference, EnvironmentVar};

pub(crate) struct Input {
    pub(crate) required: RequiredInputField,
    pub(crate) source: Option<ResourceReference>,
    pub(crate) target: Option<ResourceReference>,
    pub(crate) properties: Option<Properties>,
}

pub(crate) enum RequiredInputField {
    Resource {
        resource: ResourceReference,
    },
    Parameters {
        parameters: Vec<Parameter>,
    },
    EnvironmentVars {
        environment_vars: Vec<EnvironmentVar>,
    },
    Data {
        data: Attachment,
    },
}
pub(crate) struct Parameter {
    name: Option<String>,
    value: Option<String>,
    data_type: Option<String>,
}

