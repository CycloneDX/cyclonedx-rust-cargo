use crate::specs::common::{
    bom_reference::BomReference, dependency::Dependencies, property::Properties,
};

use self::{
    input::Inputs, output::Outputs, resource_reference::ResourceReferences, steps::Steps,
    trigger::Trigger,
};

mod input;
mod output;
mod resource_reference;
mod steps;
mod trigger;
mod workspace;

use serde::{Deserialize, Serialize};

use super::runtime_topology::RuntimeTopology;

/// bom-1.5.schema.json #definitions/workflow
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    bom_ref: BomReference,
    uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "ResourceReferences::is_empty", default)]
    resource_references: ResourceReferences,
    tasks: Tasks,
    #[serde(skip_serializing_if = "Dependencies::is_empty", default)]
    task_dependencies: Dependencies,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    task_types: Vec<TaskType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trigger: Option<Trigger>,
    #[serde(skip_serializing_if = "Steps::is_empty", default)]
    steps: Steps,
    inputs: Inputs,
    outputs: Outputs,
    time_start: Option<String>,
    time_end: Option<String>,
    // FIXME: missing Workspaces definition
    workspaces: Option<u8>,
    #[serde(skip_serializing_if = "RuntimeTopology::is_empty", default)]
    runtime_topology: RuntimeTopology,
    #[serde(skip_serializing_if = "Properties::is_empty", default)]
    properties: Properties,
}

/// bom-1.5.schema.json Vec of tasks #definitions/task FIXME: implement
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tasks;

impl Tasks {
    pub fn is_empty() -> bool {
        true
    }
}

/// bom-1.5.schema.json Vec of tasks #definitions/taskTypes FIXME: implement
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TaskType;
