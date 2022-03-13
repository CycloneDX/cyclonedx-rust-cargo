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

use crate::external_models::{normalized_string::NormalizedString, uri::Uri};
use crate::models::external_reference::ExternalReferences;
use crate::models::license::Licenses;
use crate::models::organization::OrganizationalEntity;
use crate::models::property::Properties;

#[derive(Debug, PartialEq)]
pub struct Service {
    pub bom_ref: Option<String>,
    pub provider: Option<OrganizationalEntity>,
    pub group: Option<NormalizedString>,
    pub name: NormalizedString,
    pub version: Option<NormalizedString>,
    pub description: Option<NormalizedString>,
    pub endpoints: Option<Vec<Uri>>,
    pub authenticated: Option<bool>,
    pub x_trust_boundary: Option<bool>,
    pub data: Option<Vec<DataClassification>>,
    pub licenses: Option<Licenses>,
    pub external_references: Option<ExternalReferences>,
    pub properties: Option<Properties>,
    pub services: Option<Services>,
}

#[derive(Debug, PartialEq)]
pub struct Services(pub Vec<Service>);

#[derive(Debug, PartialEq)]
pub struct DataClassification {
    pub flow: DataFlowType,
    pub classification: NormalizedString,
}

#[derive(Debug, PartialEq)]
pub enum DataFlowType {
    Inbound,
    Outbound,
    BiDirectional,
    Unknown,
    #[doc(hidden)]
    UnknownDataFlow(String),
}

impl ToString for DataFlowType {
    fn to_string(&self) -> String {
        match self {
            DataFlowType::Inbound => "inbound",
            DataFlowType::Outbound => "outbound",
            DataFlowType::BiDirectional => "bi-directional",
            DataFlowType::Unknown => "unknown",
            DataFlowType::UnknownDataFlow(df) => df,
        }
        .to_string()
    }
}

impl DataFlowType {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "inbound" => Self::Inbound,
            "outbound" => Self::Outbound,
            "bi-directional" => Self::BiDirectional,
            "unknown" => Self::Unknown,
            unknown => Self::UnknownDataFlow(unknown.to_string()),
        }
    }
}
