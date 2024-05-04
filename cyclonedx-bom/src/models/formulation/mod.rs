pub mod workflow;

use crate::{
    prelude::{SpecVersion, Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

use self::workflow::Workflow;

use super::{bom::BomReference, component::Components, property::Properties, service::Services};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Formula {
    pub(crate) bom_ref: Option<BomReference>,
    pub(crate) components: Option<Components>,
    pub(crate) services: Option<Services>,
    pub(crate) workflows: Option<Vec<Workflow>>,
    pub(crate) properties: Option<Properties>,
}

impl Validate for Formula {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match version {
            SpecVersion::V1_3 | SpecVersion::V1_4 => Err(ValidationError::new(format!(
                "Formula is not defined for version {version}"
            )))
            .into(),
            SpecVersion::V1_5 => ValidationContext::new()
                .add_unique_list_option(
                    "components", // components is uniqueItems: true
                    self.components.as_ref().map(|wrapper| wrapper.0.iter()),
                    |component| component.validate_version(version),
                )
                .add_unique_list_option(
                    "services", // services is uniqueItems: true
                    self.services.as_ref().map(|wrapper| wrapper.0.iter()),
                    |service| service.validate_version(version),
                )
                .add_unique_list_option(
                    "workflows", // workflows is uniqueItems: true
                    self.workflows.as_ref(),
                    |workflow| workflow.validate_version(version),
                )
                .add_struct_option("properties", self.properties.as_ref(), version)
                .into(),
        }
    }
}
