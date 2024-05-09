mod input;
mod output;
mod resource_reference;
mod step;
mod trigger;
mod workspace;

use crate::{
    elem_tag,
    errors::XmlReadError,
    get_elements_lax,
    models::formulation::workflow as models,
    specs::common::{dependency::Dependency, property::Properties},
    utilities::{convert_optional, convert_optional_vec},
    xml::{
        attribute_or_error, read_lax_validation_tag, read_simple_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_close_tag, write_list_tag,
        write_simple_option_tag, write_simple_tag, write_start_tag, FromXml, ToXml, VecElemTag,
        VecXmlReader,
    },
};

use self::{
    input::Input, output::Output, resource_reference::ResourceReferences, step::Step,
    trigger::Trigger, workspace::Workspace,
};

use serde::{Deserialize, Serialize};
use xml::{reader, writer};

/// bom-1.5.schema.json #definitions/workflow
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Workflow {
    #[serde(rename = "bom-ref")]
    bom_ref: String,
    uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_references: Option<ResourceReferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tasks: Option<Vec<Task>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_dependencies: Option<Vec<Dependency>>,
    task_types: Vec<TaskType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trigger: Option<Trigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    steps: Option<Vec<Step>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inputs: Option<Vec<Input>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outputs: Option<Vec<Output>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspaces: Option<Vec<Workspace>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_topology: Option<Vec<Dependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
}

impl From<models::Workflow> for Workflow {
    fn from(workflow: models::Workflow) -> Self {
        Self {
            bom_ref: workflow.bom_ref.0,
            uid: workflow.uid,
            name: workflow.name,
            description: workflow.description,
            resource_references: convert_optional_vec(workflow.resource_references)
                .map(ResourceReferences),
            tasks: convert_optional_vec(workflow.tasks),
            task_dependencies: convert_optional_vec(workflow.task_dependencies),
            task_types: workflow
                .task_types
                .into_iter()
                .map(|tt| TaskType(tt.to_string()))
                .collect(),
            trigger: convert_optional(workflow.trigger),
            steps: convert_optional_vec(workflow.steps),
            inputs: convert_optional_vec(workflow.inputs),
            outputs: convert_optional_vec(workflow.outputs),
            time_start: workflow.time_start.map(|dt| dt.0),
            time_end: workflow.time_end.map(|dt| dt.0),
            workspaces: convert_optional_vec(workflow.workspaces),
            runtime_topology: convert_optional_vec(workflow.runtime_topology),
            properties: convert_optional(workflow.properties),
        }
    }
}

impl From<Workflow> for models::Workflow {
    fn from(workflow: Workflow) -> Self {
        Self {
            bom_ref: crate::models::bom::BomReference(workflow.bom_ref),
            uid: workflow.uid,
            name: workflow.name,
            description: workflow.description,
            resource_references: convert_optional_vec(workflow.resource_references.map(|rs| rs.0)),
            tasks: convert_optional_vec(workflow.tasks),
            task_dependencies: convert_optional_vec(workflow.task_dependencies),
            task_types: workflow
                .task_types
                .into_iter()
                .map(|tt| models::TaskType::new_unchecked(tt.0))
                .collect(),
            trigger: convert_optional(workflow.trigger),
            steps: convert_optional_vec(workflow.steps),
            inputs: convert_optional_vec(workflow.inputs),
            outputs: convert_optional_vec(workflow.outputs),
            time_start: workflow.time_start.map(crate::prelude::DateTime),
            time_end: workflow.time_end.map(crate::prelude::DateTime),
            workspaces: convert_optional_vec(workflow.workspaces),
            runtime_topology: convert_optional_vec(workflow.runtime_topology),
            properties: convert_optional(workflow.properties),
        }
    }
}

const WORKFLOW_TAG: &str = "workflow";
const BOM_REF_ATTR: &str = "bom-ref";
const UID_TAG: &str = "uid";
const NAME_TAG: &str = "name";
const DESCRIPTION_TAG: &str = "description";
const RESOURCE_REFERENCES_TAG: &str = "resourceReferences";
const TASKS_TAG: &str = "tasks";
const TASK_DEPENDENCIES_TAG: &str = "taskDependencies";
const TASK_TYPES_TAG: &str = "taskTypes";
const TRIGGER_TAG: &str = "trigger";
const STEPS_TAG: &str = "steps";
const INPUTS_TAG: &str = "inputs";
const OUTPUTS_TAG: &str = "outputs";
const TIME_START_TAG: &str = "timeStart";
const TIME_END_TAG: &str = "timeEnd";
const WORKSPACES_TAG: &str = "workspaces";
const RUNTIME_TOPOLOGY_TAG: &str = "runtimeTopology";
const PROPERTIES_TAG: &str = "properties";

elem_tag!(TaskTag = "task");
elem_tag!(TaskTypeTag = "taskType");
elem_tag!(TaskDependencyTag = "dependency");
elem_tag!(StepTag = "step");
elem_tag!(InputTag = "input");
elem_tag!(OutputTag = "output");
elem_tag!(WorkspaceTag = "workspace");
elem_tag!(DependencyTag = "dependency");

impl ToXml for Workflow {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(
                writer::XmlEvent::start_element(WORKFLOW_TAG)
                    .attr(BOM_REF_ATTR, self.bom_ref.as_ref()),
            )
            .map_err(to_xml_write_error(WORKFLOW_TAG))?;

        write_simple_tag(writer, UID_TAG, &self.uid)?;
        write_simple_option_tag(writer, NAME_TAG, &self.name)?;
        write_simple_option_tag(writer, DESCRIPTION_TAG, &self.description)?;
        self.resource_references.write_xml_element(writer)?;
        if let Some(tasks) = &self.tasks {
            write_list_tag(writer, TASKS_TAG, tasks)?;
        }
        if let Some(task_dependencies) = &self.task_dependencies {
            write_list_tag(writer, TASK_DEPENDENCIES_TAG, task_dependencies)?;
        }
        write_list_tag(writer, TASK_TYPES_TAG, &self.task_types)?;
        self.trigger.write_xml_element(writer)?;
        if let Some(steps) = &self.steps {
            write_list_tag(writer, STEPS_TAG, steps)?;
        }
        if let Some(inputs) = &self.inputs {
            write_list_tag(writer, INPUTS_TAG, inputs)?;
        }
        if let Some(outputs) = &self.outputs {
            write_list_tag(writer, OUTPUTS_TAG, outputs)?;
        }
        write_simple_option_tag(writer, TIME_START_TAG, &self.time_start)?;
        write_simple_option_tag(writer, TIME_END_TAG, &self.time_end)?;
        if let Some(workspaces) = &self.workspaces {
            write_list_tag(writer, WORKSPACES_TAG, workspaces)?;
        }
        if let Some(runtime_topology) = &self.runtime_topology {
            write_list_tag(writer, RUNTIME_TOPOLOGY_TAG, runtime_topology)?;
        }
        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, WORKFLOW_TAG)
    }
}

impl FromXml for Workflow {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = attribute_or_error(element_name, attributes, BOM_REF_ATTR)?;

        get_elements_lax! {
            event_reader, element_name,
            UID_TAG => uid: String,
            NAME_TAG => name: String,
            DESCRIPTION_TAG => description: String,
            RESOURCE_REFERENCES_TAG => resource_references: ResourceReferences,
            TASKS_TAG => tasks: VecXmlReader<Task, TaskTag>,
            TASK_DEPENDENCIES_TAG => task_dependencies: VecXmlReader<Dependency, TaskDependencyTag>,
            TASK_TYPES_TAG => task_types: VecXmlReader<TaskType, TaskTypeTag>,
            TRIGGER_TAG => trigger: Trigger,
            STEPS_TAG => steps: VecXmlReader<Step, StepTag>,
            INPUTS_TAG => inputs: VecXmlReader<Input, InputTag>,
            OUTPUTS_TAG => outputs: VecXmlReader<Output, OutputTag>,
            TIME_START_TAG => time_start: String,
            TIME_END_TAG => time_end: String,
            WORKSPACES_TAG => workspaces: VecXmlReader<Workspace, WorkspaceTag>,
            RUNTIME_TOPOLOGY_TAG => runtime_topology: VecXmlReader<Dependency, DependencyTag>,
            PROPERTIES_TAG => properties: Properties,
        };

        Ok(Self {
            bom_ref,
            uid: uid.ok_or_else(|| XmlReadError::required_data_missing(UID_TAG, element_name))?,
            name,
            description,
            resource_references,
            tasks: tasks.map(Vec::from),
            task_dependencies: task_dependencies.map(Vec::from),
            task_types: task_types
                .map(Vec::from)
                .ok_or_else(|| XmlReadError::required_data_missing(TASK_TYPES_TAG, element_name))?,
            trigger,
            steps: steps.map(Vec::from),
            inputs: inputs.map(Vec::from),
            outputs: outputs.map(Vec::from),
            time_start,
            time_end,
            workspaces: workspaces.map(Vec::from),
            runtime_topology: runtime_topology.map(Vec::from),
            properties,
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct TaskType(String);

impl ToXml for TaskType {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_simple_tag(writer, TaskTypeTag::VALUE, &self.0)
    }
}

impl FromXml for TaskType {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_simple_tag(event_reader, element_name).map(Self)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(rename = "bom-ref")]
    bom_ref: String,
    uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_references: Option<ResourceReferences>,
    task_types: Vec<TaskType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trigger: Option<Trigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    steps: Option<Vec<Step>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inputs: Option<Vec<Input>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outputs: Option<Vec<Output>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspaces: Option<Vec<Workspace>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_topology: Option<Vec<Dependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
}

impl From<models::Task> for Task {
    fn from(task: models::Task) -> Self {
        Self {
            bom_ref: task.bom_ref.0,
            uid: task.uid,
            name: task.name,
            description: task.description,
            resource_references: convert_optional_vec(task.resource_references)
                .map(ResourceReferences),
            task_types: task
                .task_types
                .into_iter()
                .map(|tt| TaskType(tt.to_string()))
                .collect(),
            trigger: convert_optional(task.trigger),
            steps: convert_optional_vec(task.steps),
            inputs: convert_optional_vec(task.inputs),
            outputs: convert_optional_vec(task.outputs),
            time_start: task.time_start.map(|dt| dt.0),
            time_end: task.time_end.map(|dt| dt.0),
            workspaces: convert_optional_vec(task.workspaces),
            runtime_topology: convert_optional_vec(task.runtime_topology),
            properties: convert_optional(task.properties),
        }
    }
}

impl From<Task> for models::Task {
    fn from(task: Task) -> Self {
        Self {
            bom_ref: crate::models::bom::BomReference(task.bom_ref),
            uid: task.uid,
            name: task.name,
            description: task.description,
            resource_references: convert_optional_vec(task.resource_references.map(|rs| rs.0)),
            task_types: task
                .task_types
                .into_iter()
                .map(|tt| models::TaskType::new_unchecked(tt.0))
                .collect(),
            trigger: convert_optional(task.trigger),
            steps: convert_optional_vec(task.steps),
            inputs: convert_optional_vec(task.inputs),
            outputs: convert_optional_vec(task.outputs),
            time_start: task.time_start.map(crate::prelude::DateTime),
            time_end: task.time_end.map(crate::prelude::DateTime),
            workspaces: convert_optional_vec(task.workspaces),
            runtime_topology: convert_optional_vec(task.runtime_topology),
            properties: convert_optional(task.properties),
        }
    }
}

impl ToXml for Task {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(
                writer::XmlEvent::start_element(TaskTag::VALUE)
                    .attr(BOM_REF_ATTR, self.bom_ref.as_ref()),
            )
            .map_err(to_xml_write_error(TaskTag::VALUE))?;

        write_simple_tag(writer, UID_TAG, &self.uid)?;
        write_simple_option_tag(writer, NAME_TAG, &self.name)?;
        write_simple_option_tag(writer, DESCRIPTION_TAG, &self.description)?;
        self.resource_references.write_xml_element(writer)?;
        self.trigger.write_xml_element(writer)?;
        if let Some(steps) = &self.steps {
            write_list_tag(writer, STEPS_TAG, steps)?;
        }
        if let Some(inputs) = &self.inputs {
            write_list_tag(writer, INPUTS_TAG, inputs)?;
        }
        if let Some(outputs) = &self.outputs {
            write_list_tag(writer, OUTPUTS_TAG, outputs)?;
        }
        write_simple_option_tag(writer, TIME_START_TAG, &self.time_start)?;
        write_simple_option_tag(writer, TIME_END_TAG, &self.time_end)?;
        if let Some(workspaces) = &self.workspaces {
            write_list_tag(writer, WORKSPACES_TAG, workspaces)?;
        }
        if let Some(runtime_topology) = &self.runtime_topology {
            write_list_tag(writer, RUNTIME_TOPOLOGY_TAG, runtime_topology)?;
        }
        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, WORKFLOW_TAG)
    }
}

impl FromXml for Task {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = attribute_or_error(element_name, attributes, BOM_REF_ATTR)?;

        get_elements_lax! {
            event_reader, element_name,
            UID_TAG => uid: String,
            NAME_TAG => name: String,
            DESCRIPTION_TAG => description: String,
            RESOURCE_REFERENCES_TAG => resource_references: ResourceReferences,
            TASK_TYPES_TAG => task_types: VecXmlReader<TaskType, TaskTypeTag>,
            TRIGGER_TAG => trigger: Trigger,
            STEPS_TAG => steps: VecXmlReader<Step, StepTag>,
            INPUTS_TAG => inputs: VecXmlReader<Input, InputTag>,
            OUTPUTS_TAG => outputs: VecXmlReader<Output, OutputTag>,
            TIME_START_TAG => time_start: String,
            TIME_END_TAG => time_end: String,
            WORKSPACES_TAG => workspaces: VecXmlReader<Workspace, WorkspaceTag>,
            RUNTIME_TOPOLOGY_TAG => runtime_topology: VecXmlReader<Dependency, DependencyTag>,
            PROPERTIES_TAG => properties: Properties,
        };

        Ok(Self {
            bom_ref,
            uid: uid.ok_or_else(|| XmlReadError::required_data_missing(UID_TAG, element_name))?,
            name,
            description,
            resource_references,
            task_types: task_types
                .map(Vec::from)
                .ok_or_else(|| XmlReadError::required_data_missing(TASK_TYPES_TAG, element_name))?,
            trigger,
            steps: steps.map(Vec::from),
            inputs: inputs.map(Vec::from),
            outputs: outputs.map(Vec::from),
            time_start,
            time_end,
            workspaces: workspaces.map(Vec::from),
            runtime_topology: runtime_topology.map(Vec::from),
            properties,
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct EnvironmentVars(pub(crate) Vec<EnvironmentVar>);

const ENVIRONMENT_VARS_TAG: &str = "environmentVars";

impl ToXml for EnvironmentVars {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, ENVIRONMENT_VARS_TAG)?;

        for environment_var in &self.0 {
            environment_var.write_xml_element(writer)?;
        }

        write_close_tag(writer, ENVIRONMENT_VARS_TAG)
    }
}

impl FromXml for EnvironmentVars {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut environment_vars = vec![];

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(ENVIRONMENT_VARS_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ENVIRONMENT_VAR_TAG => {
                    let value = read_simple_tag(event_reader, &name)?;
                    let name = attribute_or_error(&name, &attributes, NAME_ATTR)?;
                    environment_vars.push(EnvironmentVar::Property { name, value });
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == VALUE_TAG => {
                    let value = read_simple_tag(event_reader, &name)?;
                    environment_vars.push(EnvironmentVar::Value(value));
                }
                // lax validation of any elements from a different schema
                reader::XmlEvent::StartElement { name, .. } => {
                    read_lax_validation_tag(event_reader, &name)?
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self(environment_vars))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, rename_all_fields = "camelCase")]
pub(crate) enum EnvironmentVar {
    Property { name: String, value: String },
    Value(String),
}

impl From<models::EnvironmentVar> for EnvironmentVar {
    fn from(environment_var: models::EnvironmentVar) -> Self {
        match environment_var {
            models::EnvironmentVar::Property { name, value } => Self::Property { name, value },
            models::EnvironmentVar::Value(value) => Self::Value(value),
        }
    }
}

impl From<EnvironmentVar> for models::EnvironmentVar {
    fn from(environment_var: EnvironmentVar) -> Self {
        match environment_var {
            EnvironmentVar::Property { name, value } => Self::Property { name, value },
            EnvironmentVar::Value(value) => Self::Value(value),
        }
    }
}

const ENVIRONMENT_VAR_TAG: &str = "environmentVar";
const VALUE_TAG: &str = "value";
const NAME_ATTR: &str = "name";

impl ToXml for EnvironmentVar {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        match self {
            Self::Property { name, value } => {
                writer
                    .write(writer::XmlEvent::start_element(ENVIRONMENT_VAR_TAG).attr("name", name))
                    .map_err(to_xml_write_error(ENVIRONMENT_VAR_TAG))?;

                writer
                    .write(writer::XmlEvent::characters(value))
                    .map_err(to_xml_write_error(ENVIRONMENT_VAR_TAG))?;

                write_close_tag(writer, ENVIRONMENT_VAR_TAG)?;
            }
            Self::Value(value) => {
                write_simple_tag(writer, VALUE_TAG, value)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        specs::{common::property::Property, v1_5::attachment::Attachment},
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use self::{
        input::{self, Input, RequiredInputField},
        output::{self, Output, RequiredOutputField},
        resource_reference::ResourceReference,
        step::{Command, Step},
        trigger::{Condition, Event},
        workspace::volume::Volume,
    };

    use super::*;

    fn example_workflow() -> Workflow {
        Workflow {
            bom_ref: "workflow-1".into(),
            uid: "8edb2b08-e2c7-11ed-b5ea-0242ac120002".into(),
            name: Some("My workflow".into()),
            description: Some("Workflow description here".into()),
            resource_references: Some(ResourceReferences(vec![ResourceReference::Ref {
                r#ref: "component-a".into(),
            }])),
            tasks: Some(vec![Task {
                bom_ref: "task-1".into(),
                uid: "task-uid-1".into(),
                name: Some("fetch-repository".into()),
                description: Some("Description here".into()),
                resource_references: Some(ResourceReferences(vec![ResourceReference::Ref {
                    r#ref: "component-a".into(),
                }])),
                task_types: vec![TaskType("clone".into()), TaskType("build".into())],
                trigger: Some(Trigger {
                    bom_ref: "trigger-1".into(),
                    uid: "trigger-1".into(),
                    name: None,
                    description: None,
                    resource_references: None,
                    r#type: "api".into(),
                    event: None,
                    conditions: None,
                    time_activated: None,
                    inputs: None,
                    outputs: None,
                    properties: None,
                }),
                steps: Some(vec![Step {
                    commands: None,
                    description: None,
                    name: Some("My step".into()),
                    properties: None,
                }]),
                inputs: Some(vec![Input {
                    required: RequiredInputField::Resource {
                        resource: ResourceReference::Ref {
                            r#ref: "component-a".into(),
                        },
                    },
                    source: None,
                    target: None,
                    properties: None,
                }]),
                outputs: Some(vec![Output {
                    required: RequiredOutputField::Resource {
                        resource: ResourceReference::Ref {
                            r#ref: "component-b".into(),
                        },
                    },
                    r#type: None,
                    source: None,
                    target: None,
                    properties: None,
                }]),
                time_start: Some("2023-01-01T00:00:00+00:00".into()),
                time_end: Some("2023-01-01T00:00:00+00:00".into()),
                workspaces: Some(vec![Workspace {
                    bom_ref: "workspace-1".into(),
                    uid: "workspace-uid-1".into(),
                    name: Some("workspace".into()),
                    aliases: None,
                    description: None,
                    resource_references: None,
                    access_mode: None,
                    mount_path: None,
                    managed_data_type: None,
                    volume_request: None,
                    volume: None,
                    properties: None,
                }]),
                runtime_topology: Some(vec![Dependency {
                    dependency_ref: "component-1".into(),
                    depends_on: vec![],
                }]),
                properties: None,
            }]),
            task_dependencies: Some(vec![Dependency {
                dependency_ref: "task-1".into(),
                depends_on: vec![],
            }]),
            task_types: vec![TaskType("clean".into()), TaskType("build".into())],
            trigger: Some(Trigger {
                bom_ref: "trigger-2".into(),
                uid: "trigger-uid-1".into(),
                name: Some("My trigger".into()),
                description: Some("Description here".into()),
                resource_references: Some(ResourceReferences(vec![ResourceReference::Ref {
                    r#ref: "component-a".into(),
                }])),
                r#type: "api".into(),
                event: Some(Event {
                    uid: Some("event-1".into()),
                    description: Some("Description here".into()),
                    time_received: Some("2023-01-01T00:00:00+00:00".into()),
                    data: Some(Attachment {
                        content: "FooBar".into(),
                        content_type: None,
                        encoding: None,
                    }),
                    source: Some(ResourceReference::Ref {
                        r#ref: "component-g".into(),
                    }),
                    target: Some(ResourceReference::Ref {
                        r#ref: "component-h".into(),
                    }),
                    properties: Some(Properties(vec![Property {
                        name: "Foo".into(),
                        value: "Bar".into(),
                    }])),
                }),
                conditions: Some(vec![Condition {
                    description: Some("Description here".into()),
                    expression: Some("1 == 1".into()),
                    properties: Some(Properties(vec![Property {
                        name: "Foo".into(),
                        value: "Bar".into(),
                    }])),
                }]),
                time_activated: Some("2023-01-01T00:00:00+00:00".into()),
                inputs: Some(vec![Input {
                    required: RequiredInputField::Resource {
                        resource: ResourceReference::Ref {
                            r#ref: "component-10".into(),
                        },
                    },
                    source: Some(ResourceReference::Ref {
                        r#ref: "component-11".into(),
                    }),
                    target: Some(ResourceReference::Ref {
                        r#ref: "component-12".into(),
                    }),
                    properties: None,
                }]),
                outputs: Some(vec![Output {
                    required: RequiredOutputField::Resource {
                        resource: ResourceReference::Ref {
                            r#ref: "component-14".into(),
                        },
                    },
                    r#type: Some("artifact".into()),
                    source: Some(ResourceReference::Ref {
                        r#ref: "component-15".into(),
                    }),
                    target: Some(ResourceReference::Ref {
                        r#ref: "component-16".into(),
                    }),
                    properties: None,
                }]),
                properties: Some(Properties(vec![Property {
                    name: "Foo".into(),
                    value: "Bar".into(),
                }])),
            }),
            steps: Some(vec![Step {
                commands: Some(vec![Command {
                    executed: Some("ls -las".into()),
                    properties: Some(Properties(vec![Property {
                        name: "Foo".into(),
                        value: "Bar".into(),
                    }])),
                }]),
                description: Some("Description here".into()),
                name: Some("My step".into()),
                properties: Some(Properties(vec![Property {
                    name: "Foo".into(),
                    value: "Bar".into(),
                }])),
            }]),
            inputs: Some(vec![
                Input {
                    required: RequiredInputField::EnvironmentVars {
                        environment_vars: EnvironmentVars(vec![EnvironmentVar::Property {
                            name: "Foo".into(),
                            value: "Bar".into(),
                        }]),
                    },
                    source: None,
                    target: None,
                    properties: None,
                },
                Input {
                    required: RequiredInputField::EnvironmentVars {
                        environment_vars: EnvironmentVars(vec![EnvironmentVar::Value(
                            "FooBar".into(),
                        )]),
                    },
                    source: None,
                    target: None,
                    properties: None,
                },
                Input {
                    required: RequiredInputField::EnvironmentVars {
                        environment_vars: EnvironmentVars(vec![
                            EnvironmentVar::Property {
                                name: "Foo".into(),
                                value: "Bar".into(),
                            },
                            EnvironmentVar::Value("FooBar".into()),
                        ]),
                    },
                    source: None,
                    target: None,
                    properties: None,
                },
            ]),
            outputs: Some(vec![
                Output {
                    required: RequiredOutputField::EnvironmentVars {
                        environment_vars: EnvironmentVars(vec![EnvironmentVar::Property {
                            name: "Foo".into(),
                            value: "Bar".into(),
                        }]),
                    },
                    r#type: None,
                    source: None,
                    target: None,
                    properties: None,
                },
                Output {
                    required: RequiredOutputField::EnvironmentVars {
                        environment_vars: EnvironmentVars(vec![EnvironmentVar::Value(
                            "FooBar".into(),
                        )]),
                    },
                    r#type: None,
                    source: None,
                    target: None,
                    properties: None,
                },
                Output {
                    required: RequiredOutputField::EnvironmentVars {
                        environment_vars: EnvironmentVars(vec![
                            EnvironmentVar::Property {
                                name: "Foo".into(),
                                value: "Bar".into(),
                            },
                            EnvironmentVar::Value("FooBar".into()),
                        ]),
                    },
                    r#type: None,
                    source: None,
                    target: None,
                    properties: None,
                },
            ]),
            time_start: Some("2023-01-01T00:00:00+00:00".into()),
            time_end: Some("2023-01-01T00:00:00+00:00".into()),
            workspaces: Some(vec![Workspace {
                bom_ref: "workspace-2".into(),
                uid: "workspace-1".into(),
                name: Some("My workspace".into()),
                aliases: Some(vec!["default-workspace".into()]),
                description: Some("Description here".into()),
                resource_references: Some(ResourceReferences(vec![ResourceReference::Ref {
                    r#ref: "component-t".into(),
                }])),
                access_mode: Some("read-write".into()),
                mount_path: Some("/tmp/workspace".into()),
                managed_data_type: Some("ConfigMap".into()),
                volume_request: Some("requestedVolumeClaim".into()),
                volume: Some(Volume {
                    uid: Some("volume-1".into()),
                    name: Some("My volume".into()),
                    mode: Some("filesystem".into()),
                    path: Some("/".into()),
                    size_allocated: Some("10GB".into()),
                    persistent: Some(true),
                    remote: Some(false),
                    properties: None,
                }),
                properties: None,
            }]),
            runtime_topology: Some(vec![Dependency {
                dependency_ref: "component-r".into(),
                depends_on: vec![],
            }]),
            properties: Some(Properties(vec![Property {
                name: "Foo".into(),
                value: "Bar".into(),
            }])),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_workflow());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<workflow bom-ref="workflow-1">
    <uid>8edb2b08-e2c7-11ed-b5ea-0242ac120002</uid>
    <name>My workflow</name>
    <description>Workflow description here</description>
    <resourceReferences>
        <resourceReference>
            <ref>component-a</ref>
        </resourceReference>
    </resourceReferences>
    <tasks>
        <task bom-ref="task-1">
            <uid>task-uid-1</uid>
            <name>fetch-repository</name>
            <description>Description here</description>
            <resourceReferences>
                <resourceReference>
                    <ref>component-a</ref>
                </resourceReference>
            </resourceReferences>
            <taskTypes>
                <taskType>clone</taskType>
                <taskType>build</taskType>
            </taskTypes>
            <trigger bom-ref="trigger-1">
                <uid>trigger-1</uid>
                <type>api</type>
            </trigger>
            <steps>
                <step>
                    <name>My step</name>
                </step>
            </steps>
            <inputs>
                <input>
                    <resource>
                        <ref>component-a</ref>
                    </resource>
                </input>
            </inputs>
            <outputs>
                <output>
                    <resource>
                        <ref>component-b</ref>
                    </resource>
                </output>
            </outputs>
            <timeStart>2023-01-01T00:00:00+00:00</timeStart>
            <timeEnd>2023-01-01T00:00:00+00:00</timeEnd>
            <workspaces>
                <workspace bom-ref="workspace-1">
                    <uid>workspace-uid-1</uid>
                    <name>workspace</name>
                </workspace>
            </workspaces>
            <runtimeTopology>
                <dependency ref="component-1" />
            </runtimeTopology>
        </task>
    </tasks>
    <taskDependencies>
        <dependency ref="task-1" />
    </taskDependencies>
    <taskTypes>
        <taskType>clean</taskType>
        <taskType>build</taskType>
    </taskTypes>
    <trigger bom-ref="trigger-2">
        <uid>trigger-uid-1</uid>
        <name>My trigger</name>
        <description>Description here</description>
        <resourceReferences>
            <resourceReference>
                <ref>component-a</ref>
            </resourceReference>
        </resourceReferences>
        <type>api</type>
        <event>
            <uid>event-1</uid>
            <description>Description here</description>
            <timeReceived>2023-01-01T00:00:00+00:00</timeReceived>
            <data>FooBar</data>
            <source>
                <ref>component-g</ref>
            </source>
            <target>
                <ref>component-h</ref>
            </target>
            <properties>
                <property name="Foo">Bar</property>
            </properties>
        </event>
        <conditions>
            <condition>
                <description>Description here</description>
                <expression>1 == 1</expression>
                <properties>
                    <property name="Foo">Bar</property>
                </properties>
            </condition>
        </conditions>
        <timeActivated>2023-01-01T00:00:00+00:00</timeActivated>
        <inputs>
            <input>
                <resource>
                    <ref>component-10</ref>
                </resource>
                <source>
                    <ref>component-11</ref>
                </source>
                <target>
                    <ref>component-12</ref>
                </target>
            </input>
        </inputs>
        <outputs>
            <output>
                <resource>
                    <ref>component-14</ref>
                </resource>
                <type>artifact</type>
                <source>
                    <ref>component-15</ref>
                </source>
                <target>
                    <ref>component-16</ref>
                </target>
            </output>
        </outputs>
        <properties>
            <property name="Foo">Bar</property>
        </properties>
    </trigger>
    <steps>
        <step>
            <name>My step</name>
            <description>Description here</description>
            <commands>
                <command>
                    <executed>ls -las</executed>
                    <properties>
                        <property name="Foo">Bar</property>
                    </properties>
                </command>
            </commands>
            <properties>
                <property name="Foo">Bar</property>
            </properties>
        </step>
    </steps>
    <inputs>
        <input>
            <environmentVars>
                <environmentVar name="Foo">Bar</environmentVar>
            </environmentVars>
        </input>
        <input>
            <environmentVars>
                <value>FooBar</value>
            </environmentVars>
        </input>
        <input>
            <environmentVars>
                <environmentVar name="Foo">Bar</environmentVar>
                <value>FooBar</value>
            </environmentVars>
        </input>
    </inputs>
    <outputs>
        <output>
            <environmentVars>
                <environmentVar name="Foo">Bar</environmentVar>
            </environmentVars>
        </output>
        <output>
            <environmentVars>
                <value>FooBar</value>
            </environmentVars>
        </output>
        <output>
            <environmentVars>
                <environmentVar name="Foo">Bar</environmentVar>
                <value>FooBar</value>
            </environmentVars>
        </output>
    </outputs>
    <timeStart>2023-01-01T00:00:00+00:00</timeStart>
    <timeEnd>2023-01-01T00:00:00+00:00</timeEnd>
    <workspaces>
        <workspace bom-ref="workspace-2">
            <uid>workspace-1</uid>
            <name>My workspace</name>
            <aliases>
                <alias>default-workspace</alias>
            </aliases>
            <description>Description here</description>
            <resourceReferences>
                <resourceReference>
                    <ref>component-t</ref>
                </resourceReference>
            </resourceReferences>
            <accessMode>read-write</accessMode>
            <mountPath>/tmp/workspace</mountPath>
            <managedDataType>ConfigMap</managedDataType>
            <volumeRequest>requestedVolumeClaim</volumeRequest>
            <volume>
                <uid>volume-1</uid>
                <name>My volume</name>
                <mode>filesystem</mode>
                <path>/</path>
                <sizeAllocated>10GB</sizeAllocated>
                <persistent>true</persistent>
                <remote>false</remote>
            </volume>
        </workspace>
    </workspaces>
    <runtimeTopology>
        <dependency ref="component-r"/>
    </runtimeTopology>
    <properties>
        <property name="Foo">Bar</property>
    </properties>
</workflow>
"#;
        let actual: Workflow = read_element_from_string(input);
        let expected = example_workflow();
        pretty_assertions::assert_eq!(actual, expected);
    }
}
