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

pub(crate) mod bom;

pub(crate) mod vulnerability;
pub(crate) mod vulnerability_analysis;
pub(crate) mod vulnerability_credits;
pub(crate) mod vulnerability_rating;
pub(crate) mod vulnerability_reference;
pub(crate) mod vulnerability_source;
pub(crate) mod vulnerability_target;

pub(crate) use crate::specs::common::component::v1_4 as component;
pub(crate) use crate::specs::common::composition::v1_4 as composition;
pub(crate) use crate::specs::common::external_reference::v1_4 as external_reference;
pub(crate) use crate::specs::common::metadata::v1_4 as metadata;
pub(crate) use crate::specs::common::service::v1_4 as service;
pub(crate) use crate::specs::common::tool::v1_4 as tool;
