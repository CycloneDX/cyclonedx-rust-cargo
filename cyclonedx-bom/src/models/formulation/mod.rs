pub mod workflow;

use self::workflow::Workflow;

use super::{bom::BomReference, component::Components, property::Properties, service::Services};

pub(crate) struct Formula {
    bom_ref: Option<BomReference>,
    components: Option<Components>,
    services: Option<Services>,
    workflows: Option<Vec<Workflow>>,
    properties: Option<Properties>,
}
