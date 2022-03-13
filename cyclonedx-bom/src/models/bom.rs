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

use crate::models::component::Components;
use crate::models::composition::Compositions;
use crate::models::dependency::Dependencies;
use crate::models::external_reference::ExternalReferences;
use crate::models::metadata::Metadata;
use crate::models::property::Properties;
use crate::models::service::Services;

#[derive(Debug, PartialEq)]
pub struct Bom {
    pub version: u32,
    pub serial_number: Option<UrnUuid>,
    pub metadata: Option<Metadata>,
    pub components: Option<Components>,
    pub services: Option<Services>,
    pub external_references: Option<ExternalReferences>,
    pub dependencies: Option<Dependencies>,
    pub compositions: Option<Compositions>,
    pub properties: Option<Properties>,
}

impl Default for Bom {
    fn default() -> Self {
        Self {
            version: 1,
            serial_number: Some(UrnUuid(format!("urn:uuid:{}", uuid::Uuid::new_v4()))),
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: None,
            properties: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct UrnUuid(pub(crate) String);
