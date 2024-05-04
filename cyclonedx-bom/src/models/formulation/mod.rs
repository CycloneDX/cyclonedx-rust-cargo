pub mod workflow;

use crate::{
    prelude::{SpecVersion, Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

use self::workflow::Workflow;

use super::{bom::BomReference, component::Components, property::Properties, service::Services};

pub(crate) struct Formula {
    bom_ref: Option<BomReference>,
    components: Option<Components>,
    services: Option<Services>,
    workflows: Option<Vec<Workflow>>,
    properties: Option<Properties>,
}

impl Validate for Formula {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match version {
            SpecVersion::V1_3 | SpecVersion::V1_4 => Err(ValidationError::new(
                "Formula is not defined for versions 1.3 and 1.4",
            ))
            .into(),
            version @ SpecVersion::V1_5 => ValidationContext::new()
                .add_unique_list_option(
                    "components",
                    self.components.as_ref().map(|wrapper| wrapper.0.iter()),
                    |component| component.validate(),
                )
                .into(),
        }
    }
}
