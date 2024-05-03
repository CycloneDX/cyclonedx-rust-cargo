use crate::models::{bom::BomReference, external_reference::{BomLink, ExternalReference}};

pub(crate) enum ResourceReference {
    Ref {
        r#ref: Option<BomReference>,
        bom_link: Option<BomLink>,
    },
    ExternalReference(ExternalReference),
}
