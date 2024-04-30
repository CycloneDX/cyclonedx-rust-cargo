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

pub(crate) mod annotation;
pub(crate) mod attachment;
pub(crate) mod component_data;
pub(crate) mod data_governance;
pub(crate) mod evidence;
pub(crate) mod formulation;
pub(crate) mod licensing;
pub(crate) mod lifecycles;
pub(crate) mod modelcard;
pub(crate) mod proof_of_concept;
pub(crate) mod service_data;

pub(crate) use crate::specs::common::bom::v1_5 as bom;
pub(crate) use crate::specs::common::component::v1_5 as component;
pub(crate) use crate::specs::common::composition::v1_5 as composition;
pub(crate) use crate::specs::common::external_reference::v1_5 as external_reference;
pub(crate) use crate::specs::common::license::v1_5 as license;
pub(crate) use crate::specs::common::metadata::v1_5 as metadata;
pub(crate) use crate::specs::common::service::v1_5 as service;
pub(crate) use crate::specs::common::tool::v1_5 as tool;
pub(crate) use crate::specs::common::vulnerability::v1_5 as vulnerability;
pub(crate) use crate::specs::common::vulnerability_analysis::v1_5 as vulnerability_analysis;
