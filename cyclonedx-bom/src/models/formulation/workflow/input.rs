use crate::{
    models::{attachment::Attachment, property::Properties},
    prelude::Validate,
    validation::ValidationContext,
};

use super::{resource_reference::ResourceReference, EnvironmentVar};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Input {
    pub required: RequiredInputField,
    pub source: Option<ResourceReference>,
    pub target: Option<ResourceReference>,
    pub properties: Option<Properties>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequiredInputField {
    Resource(ResourceReference),
    Parameters(Vec<Parameter>),
    EnvironmentVars(Vec<EnvironmentVar>),
    Data(Attachment),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameter {
    pub name: Option<String>,
    pub value: Option<String>,
    pub data_type: Option<String>,
}
