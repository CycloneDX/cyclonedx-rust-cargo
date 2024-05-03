use crate::{
    models::{bom::BomReference, property::Properties},
    prelude::{SpecVersion, Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

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
    pub(crate) mode: Mode,
    pub(crate) path: Option<String>,
    pub(crate) size_allocated: Option<String>,
    pub(crate) persistent: Option<bool>,
    pub(crate) remote: Option<bool>,
    pub(crate) properties: Option<Properties>,
}

impl Validate for Volume {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("mode", &self.mode, validate_mode)
            .into()
    }
}

#[derive(Default, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum Mode {
    #[default]
    Filesystem,
    Block,
    #[strum(default)]
    #[doc(hidden)]
    UnknownMode(String),
}

impl Mode {
    pub(crate) fn new_unchecked<S: AsRef<str>>(s: S) -> Self {
        match s.as_ref() {
            "filesystem" => Self::Filesystem,
            "block" => Self::Block,
            unknown => Self::UnknownMode(unknown.to_owned()),
        }
    }
}

pub fn validate_mode(mode: &Mode) -> Result<(), ValidationError> {
    match mode {
        Mode::UnknownMode(_) => Err(ValidationError::new("Unknown mode")),
        _ => Ok(()),
    }
}
