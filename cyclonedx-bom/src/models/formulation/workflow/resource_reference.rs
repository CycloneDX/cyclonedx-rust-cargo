use crate::{
    models::external_reference::ExternalReference, prelude::Validate, validation::ValidationContext,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceReference {
    Ref(String),
    ExternalReference(ExternalReference),
}

impl Validate for ResourceReference {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        let mut ctx = ValidationContext::new();

        if let ResourceReference::ExternalReference(external_reference) = self {
            ctx.add_struct("external_reference", external_reference, version);
        }

        ctx.into()
    }
}
