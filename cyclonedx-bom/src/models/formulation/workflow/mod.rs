pub mod input;
pub mod output;
pub mod resource_reference;
pub mod step;
pub mod trigger;
pub mod workspace;

use crate::{
    external_models::validate_date_time,
    models::{bom::BomReference, dependency::Dependency, property::Properties},
    prelude::{DateTime, Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

use self::{
    input::Input, output::Output, resource_reference::ResourceReference, step::Step,
    trigger::Trigger, workspace::Workspace,
};

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct Workflow {
    pub(crate) bom_ref: BomReference,
    pub(crate) uid: String,
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) resource_references: Option<Vec<ResourceReference>>,
    pub(crate) tasks: Option<Vec<Task>>,
    pub(crate) task_dependencies: Option<Vec<Dependency>>,
    pub(crate) task_types: Vec<TaskType>,
    pub(crate) trigger: Option<Trigger>,
    pub(crate) steps: Option<Vec<Step>>,
    pub(crate) inputs: Option<Vec<Input>>,
    pub(crate) outputs: Option<Vec<Output>>,
    pub(crate) time_start: Option<DateTime>,
    pub(crate) time_end: Option<DateTime>,
    pub(crate) workspaces: Option<Vec<Workspace>>,
    pub(crate) runtime_topology: Option<Vec<Dependency>>,
    pub(crate) properties: Option<Properties>,
}

impl Validate for Workflow {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        ValidationContext::new()
            .add_list_option(
                "resource_references",
                self.resource_references.as_ref(),
                |resource_reference| resource_reference.validate_version(version),
            )
            .add_unique_list_option("tasks", self.tasks.as_ref(), |task| {
                task.validate_version(version)
            })
            .add_unique_list_option("task_dependencies", self.task_dependencies.as_ref(), |_| {
                ValidationResult::new()
            })
            .add_list("task_types", &self.task_types, |task_type| {
                task_type.validate_version(version)
            })
            .add_struct_option("trigger", self.trigger.as_ref(), version)
            .add_unique_list_option("steps", self.steps.as_ref(), |step| {
                step.validate_version(version)
            })
            .add_unique_list_option("inputs", self.inputs.as_ref(), |input| {
                input.validate_version(version)
            })
            .add_unique_list_option("outputs", self.outputs.as_ref(), |output| {
                output.validate_version(version)
            })
            .add_field_option("time_start", self.time_start.as_ref(), validate_date_time)
            .add_field_option("time_end", self.time_end.as_ref(), validate_date_time)
            .add_unique_list_option("workspaces", self.workspaces.as_ref(), |workspace| {
                workspace.validate_version(version)
            })
            .add_unique_list_option("runtime_topology", self.runtime_topology.as_ref(), |_| {
                ValidationResult::new()
            })
            .into()
    }
}
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct Task {
    pub(crate) bom_ref: BomReference,
    pub(crate) uid: String,
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) resource_references: Option<Vec<ResourceReference>>,
    pub(crate) task_types: Vec<TaskType>,
    pub(crate) trigger: Option<Trigger>,
    pub(crate) steps: Option<Vec<Step>>,
    pub(crate) inputs: Option<Vec<Input>>,
    pub(crate) outputs: Option<Vec<Output>>,
    pub(crate) time_start: Option<DateTime>,
    pub(crate) time_end: Option<DateTime>,
    pub(crate) workspaces: Option<Vec<Workspace>>,
    pub(crate) runtime_topology: Option<Vec<Dependency>>,
    pub(crate) properties: Option<Properties>,
}

impl Validate for Task {
    fn validate_version(
        &self,
        version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        ValidationContext::new()
            .add_list_option(
                "resource_references",
                self.resource_references.as_ref(),
                |resource_reference| resource_reference.validate_version(version),
            )
            .add_list("task_types", &self.task_types, |task_type| {
                task_type.validate_version(version)
            })
            .add_struct_option("trigger", self.trigger.as_ref(), version)
            .add_unique_list_option("steps", self.steps.as_ref(), |step| {
                step.validate_version(version)
            })
            .add_unique_list_option("inputs", self.inputs.as_ref(), |input| {
                input.validate_version(version)
            })
            .add_unique_list_option("outputs", self.outputs.as_ref(), |output| {
                output.validate_version(version)
            })
            .add_field_option("time_start", self.time_start.as_ref(), validate_date_time)
            .add_field_option("time_end", self.time_end.as_ref(), validate_date_time)
            .add_unique_list_option("workspaces", self.workspaces.as_ref(), |workspace| {
                workspace.validate_version(version)
            })
            .add_unique_list_option("runtime_topology", self.runtime_topology.as_ref(), |_| {
                ValidationResult::new()
            })
            .into()
    }
}

#[derive(PartialEq, Eq, Hash, strum::Display)]
#[strum(serialize_all = "kebab-case")]
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
    #[strum(default)]
    UnknownTaskType(String),
}

impl TaskType {
    pub(crate) fn new_unchecked<S: AsRef<str>>(s: S) -> Self {
        match s.as_ref() {
            "copy" => Self::Copy,
            "clone" => Self::Clone,
            "lint" => Self::Lint,
            "scan" => Self::Scan,
            "merge" => Self::Merge,
            "build" => Self::Build,
            "test" => Self::Test,
            "deliver" => Self::Deliver,
            "deploy" => Self::Deploy,
            "release" => Self::Release,
            "clean" => Self::Clean,
            "other" => Self::Other,
            unknown => Self::UnknownTaskType(unknown.to_owned()),
        }
    }
}

impl Validate for TaskType {
    fn validate_version(
        &self,
        _version: crate::prelude::SpecVersion,
    ) -> crate::prelude::ValidationResult {
        match self {
            Self::UnknownTaskType(_) => Err(ValidationError::new("unknown task type")),
            _ => Ok(()),
        }
        .into()
    }
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum EnvironmentVar {
    Property { name: String, value: String },
    Value(String),
}
