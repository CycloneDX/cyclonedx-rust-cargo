pub mod input;
pub mod output;
pub mod resource_reference;
pub mod step;
pub mod trigger;
pub mod workspace;

use crate::{
    models::{bom::BomReference, dependency::Dependency, property::Properties},
    prelude::DateTime,
};

use self::{
    input::Input, output::Output, resource_reference::ResourceReference, step::Step,
    trigger::Trigger, workspace::Workspace,
};

pub(crate) struct Workflow {
    bom_ref: BomReference,
    name: Option<String>,
    description: Option<String>,
    resource_references: Option<Vec<ResourceReference>>,
    tasks: Option<Vec<Task>>,
    task_dependencies: Option<Vec<Dependency>>,
    task_types: Vec<TaskType>,
    trigger: Option<Trigger>,
    inputs: Option<Vec<Input>>,
    outputs: Option<Vec<Output>>,
    time_start: Option<DateTime>,
    time_end: Option<DateTime>,
    workspaces: Option<Vec<Workspace>>,
    runtime_topology: Option<Vec<Dependency>>,
    properties: Option<Properties>,
}

pub(crate) enum TaskType {
    Copy,
    Clone,
    Lint,
    Scan,
    Merge,
    Build,
    Test,
    Deliver,
    Deploy,
    Release,
    Clean,
    Other,
}

pub(crate) struct Task {
    bom_ref: BomReference,
    uid: String,
    name: Option<String>,
    description: Option<String>,
    resource_references: Option<Vec<ResourceReference>>,
    task_types: Vec<TaskType>,
    trigger: Option<Trigger>,
    steps: Option<Vec<Step>>,
    inputs: Option<Vec<Input>>,
    outputs: Option<Vec<Output>>,
    time_start: Option<DateTime>,
    time_end: Option<DateTime>,
    workspaces: Option<Vec<Workspace>>,
    runtime_topology: Option<Vec<Dependency>>,
    properties: Option<Properties>,
}

pub(crate) enum EnvironmentVar {
    Property { name: String, value: String },
    Value(String),
}
