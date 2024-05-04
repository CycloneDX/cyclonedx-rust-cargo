use crate::{
    models::{attachment::Attachment, property::Properties},
    prelude::Validate,
    validation::ValidationContext,
};

use super::{resource_reference::ResourceReference, EnvironmentVar};

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct Input {
    pub(crate) required: RequiredInputField,
    pub(crate) source: Option<ResourceReference>,
    pub(crate) target: Option<ResourceReference>,
    pub(crate) properties: Option<Properties>,
}

impl Validate for Input {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        let mut ctx = ValidationContext::new();

        match &self.required {
            RequiredInputField::Resource(resource) => {
                ctx.add_struct("resource", resource, version);
            }
            RequiredInputField::Data(data) => {
                ctx.add_struct("data", data, version);
            }
            _ => {}
        }

        ctx.add_struct_option("source", self.source.as_ref(), version)
            .add_struct_option("target", self.target.as_ref(), version)
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum RequiredInputField {
    Resource(ResourceReference),

    Parameters(Vec<Parameter>),
    EnvironmentVars(Vec<EnvironmentVar>),
    Data(Attachment),
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct Parameter {
    pub(crate) name: Option<String>,
    pub(crate) value: Option<String>,
    pub(crate) data_type: Option<String>,
}
