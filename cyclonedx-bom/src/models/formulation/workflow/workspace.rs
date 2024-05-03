use crate::models::{bom::BomReference, property::Properties};

use super::resource_reference::ResourceReference;

pub(crate) struct Workspace {
    pub(crate) bom_ref: BomReference,
    pub(crate) uid: String,
    pub(crate) name: Option<String>,
    pub(crate) aliases: Option<Vec<String>>,
    pub(crate) description: Option<String>,
    pub(crate) resource_references: Option<Vec<ResourceReference>>,
    pub(crate) access_mode: Option<String>,
    pub(crate) mount_path: Option<String>,
    pub(crate) managed_data_type: Option<String>,
    pub(crate) volume_request: Option<String>,
    pub(crate) volume: Option<Volume>,
    pub(crate) properties: Option<Properties>,
}

pub(crate) struct Volume {
    pub(crate) uid: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) mode: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) size_allocated: Option<String>,
    pub(crate) persistent: Option<bool>,
    pub(crate) remote: Option<bool>,
    pub(crate) properties: Option<Properties>,
}
