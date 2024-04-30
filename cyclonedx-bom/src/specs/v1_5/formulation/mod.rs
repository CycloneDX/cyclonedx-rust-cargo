mod runtime_topology;
mod workflow;

// #definitions/formula
pub(crate) struct Formulation {
    // TODO(@divma): adjust type
    bom_ref: u8,
    // TODO(@divma): adjust type
    components: u8,
    // TODO(@divma): adjust type
    services: u8,
    // TODO(@divma): adjust type
    workflows: u8,
    // TODO(@divma): adjust type
    properties: u8,
}
