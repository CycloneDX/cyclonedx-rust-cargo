mod volume;

use serde::{Deserialize, Serialize};

use crate::specs::common::property::Properties;

use super::resource_reference::ResourceReferences;
use volume::Volume;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Workspace {
    #[serde(rename = "kebab-case")]
    bom_ref: String,
    uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_references: Option<ResourceReferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mount_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    managed_data_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume_request: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume: Option<Volume>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties:Option<Properties>,
}
