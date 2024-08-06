use crate::{
    models::{bom::BomReference, property::Properties},
    prelude::{SpecVersion, Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

use super::resource_reference::ResourceReference;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Workspace {
    pub bom_ref: BomReference,
    pub uid: String,
    pub name: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub description: Option<String>,
    pub resource_references: Option<Vec<ResourceReference>>,
    pub access_mode: Option<AccessMode>,
    pub mount_path: Option<String>,
    pub managed_data_type: Option<String>,
    pub volume_request: Option<String>,
    pub volume: Option<Volume>,
    pub properties: Option<Properties>,
}

impl Validate for Workspace {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_unique_list_option(
                "resource_references",
                self.resource_references.as_ref(),
                |resource_reference| resource_reference.validate_version(version),
            )
            .add_field_option(
                "access_mode",
                self.access_mode.as_ref(),
                validate_access_mode,
            )
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum AccessMode {
    ReadOnly,
    ReadWrite,
    ReadWriteOnce,
    WriteOnce,
    WriteOnly,
    #[strum(default)]
    #[doc(hidden)]
    UnknownAccessMode(String),
}

impl AccessMode {
    pub fn new_unchecked<S: AsRef<str>>(s: S) -> Self {
        match s.as_ref() {
            "read-only" => Self::ReadOnly,
            "read-write" => Self::ReadWrite,
            "read-write-once" => Self::ReadWriteOnce,
            "write-once" => Self::WriteOnce,
            "write-only" => Self::WriteOnly,
            unknown => Self::UnknownAccessMode(unknown.to_owned()),
        }
    }
}

pub fn validate_access_mode(access_mode: &AccessMode) -> Result<(), ValidationError> {
    match access_mode {
        AccessMode::UnknownAccessMode(_) => Err(ValidationError::new("Unknown access mode")),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Volume {
    pub uid: Option<String>,
    pub name: Option<String>,
    pub mode: Mode,
    pub path: Option<String>,
    pub size_allocated: Option<String>,
    pub persistent: Option<bool>,
    pub remote: Option<bool>,
    pub properties: Option<Properties>,
}

impl Validate for Volume {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("mode", &self.mode, validate_mode)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, strum::Display)]
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
    pub fn new_unchecked<S: AsRef<str>>(s: S) -> Self {
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
