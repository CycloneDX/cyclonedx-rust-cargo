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

use crate::external_models::{normalized_string::NormalizedString, spdx::SpdxIdentifier, uri::Uri};
use crate::models::attached_text::AttachedText;

#[derive(Debug, PartialEq)]
pub enum LicenseChoice {
    License(License),
    Expression(NormalizedString),
}

#[derive(Debug, PartialEq)]
pub struct License {
    pub license_identifier: LicenseIdentifier,
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
}

#[derive(Debug, PartialEq)]
pub struct Licenses(pub Vec<LicenseChoice>);

#[derive(Debug, PartialEq)]
pub enum LicenseIdentifier {
    SpdxId(SpdxIdentifier),
    Name(NormalizedString),
}
