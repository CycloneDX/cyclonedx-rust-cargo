use crate::specs::common::{self, property::Properties};

use super::{component::Components, service::Services};

mod runtime_topology;
mod workflow;

// #definitions/formula
pub(crate) struct Formulation {
    bom_ref: Option<common::bom_reference::BomReference>,
    components: Option<Components>,
    services: Option<Services>,
    workflows: u8,
    properties: Option<Properties>,
}
