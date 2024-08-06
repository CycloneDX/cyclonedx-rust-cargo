use crate::{
    models::{attachment::Attachment, property::Properties},
    prelude::Validate,
    validation::{ValidationContext, ValidationError},
};

use super::{resource_reference::ResourceReference, EnvironmentVar};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Output {
    pub required: RequiredOutputField,
    pub r#type: Option<Type>,
    pub source: Option<ResourceReference>,
    pub target: Option<ResourceReference>,
    pub properties: Option<Properties>,
}

impl Validate for Output {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        let mut ctx = ValidationContext::new();

        match &self.required {
            RequiredOutputField::Resource(resource) => {
                ctx.add_struct("resource", resource, version);
            }
            RequiredOutputField::Data(data) => {
                ctx.add_struct("data", data, version);
            }
            _ => {}
        }

        ctx.add_struct_option("type", self.r#type.as_ref(), version)
            .add_struct_option("source", self.source.as_ref(), version)
            .add_struct_option("target", self.target.as_ref(), version)
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequiredOutputField {
    Resource(ResourceReference),
    EnvironmentVars(Vec<EnvironmentVar>),
    Data(Attachment),
}

#[derive(Debug, Clone, strum::Display, PartialEq, Eq, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum Type {
    Artifact,
    Attestation,
    Log,
    Evidence,
    Metrics,
    Other,
    #[strum(default)]
    #[doc(hidden)]
    Unknown(String),
}

impl Type {
    pub fn new_unchecked<S: AsRef<str>>(s: S) -> Self {
        match s.as_ref() {
            "artifact" => Self::Artifact,
            "attestation" => Self::Attestation,
            "log" => Self::Log,
            "evidence" => Self::Evidence,
            "metrics" => Self::Metrics,
            "other" => Self::Other,
            unknown => Self::Unknown(unknown.to_owned()),
        }
    }
}

impl Validate for Type {
    fn validate_version(
        &self,
        _version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        match self {
            Self::Unknown(_) => Err(ValidationError::new("unknown output type")),
            _ => Ok(()),
        }
        .into()
    }
}
