use crate::{models::property::Properties, prelude::Validate, validation::ValidationContext};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Step {
    pub commands: Option<Vec<Command>>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub properties: Option<Properties>,
}

impl Validate for Step {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        ValidationContext::new()
            .add_list_option("commands", self.commands.as_ref(), |command| {
                command.validate_version(version)
            })
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Command {
    pub executed: Option<String>,
    pub properties: Option<Properties>,
}

impl Validate for Command {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        ValidationContext::new()
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}
