/*
 * This file is part of CycloneDX Rust Cargo.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{
    external_models::uri::validate_uri,
    models::{attachment::Attachment, data_governance::DataGovernance},
    prelude::{Uri, Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

use super::{
    bom::{BomReference, SpecVersion},
    property::Properties,
};

/// Inline Component Data
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentData {
    pub bom_ref: Option<BomReference>,
    /// 'type' field
    pub data_type: ComponentDataType,
    pub name: Option<String>,
    pub contents: Option<DataContents>,
    pub classification: Option<String>,
    pub sensitive_data: Option<String>,
    pub graphics: Option<GraphicsCollection>,
    pub description: Option<String>,
    pub governance: Option<DataGovernance>,
}

impl Validate for ComponentData {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("type", &self.data_type, validate_datatype)
            .add_struct_option("contents", self.contents.as_ref(), version)
            .add_struct_option("graphics", self.graphics.as_ref(), version)
            .add_struct_option("governance", self.governance.as_ref(), version)
            .into()
    }
}

fn validate_datatype(datatype: &ComponentDataType) -> Result<(), ValidationError> {
    if matches!(datatype, ComponentDataType::Unknown(_)) {
        return Err("Unknown component data type found".into());
    }
    Ok(())
}

/// Type of data
#[derive(Clone, Debug, PartialEq, Eq, strum::Display, strum::EnumString, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum ComponentDataType {
    SourceCode,
    Configuration,
    Dataset,
    Definition,
    Other,
    #[doc(hidden)]
    #[strum(default)]
    Unknown(String),
}

impl From<String> for ComponentDataType {
    fn from(value: String) -> Self {
        std::str::FromStr::from_str(&value).expect("infallible")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataContents {
    pub attachment: Option<Attachment>,
    pub url: Option<Uri>,
    pub properties: Option<Properties>,
}

impl Validate for DataContents {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("attachment", self.attachment.as_ref(), version)
            .add_field_option("url", self.url.as_ref(), validate_uri)
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}

/// bom-1.5.schema.json #definitions/graphicsCollection
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GraphicsCollection {
    pub description: Option<String>,
    pub collection: Option<Vec<Graphic>>,
}

impl Validate for GraphicsCollection {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("collection", self.collection.as_ref(), |graphic| {
                graphic.validate_version(version)
            })
            .into()
    }
}

/// bom-1.5.schema.json #definitions/graphic
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Graphic {
    pub name: Option<String>,
    pub image: Option<Attachment>,
}

impl Validate for Graphic {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("image", self.image.as_ref(), version)
            .into()
    }
}
