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

use crate::external_models::{
    normalized_string::NormalizedString,
    uri::{Purl, Uri},
};
use crate::models::attached_text::AttachedText;
use crate::models::code::{Commits, Patches};
use crate::models::external_reference::ExternalReferences;
use crate::models::hash::Hashes;
use crate::models::license::Licenses;
use crate::models::organization::OrganizationalEntity;
use crate::models::property::Properties;

#[derive(Debug, PartialEq)]
pub struct Component {
    pub component_type: Classification,
    pub mime_type: Option<MimeType>,
    pub bom_ref: Option<String>,
    pub supplier: Option<OrganizationalEntity>,
    pub author: Option<NormalizedString>,
    pub publisher: Option<NormalizedString>,
    pub group: Option<NormalizedString>,
    pub name: NormalizedString,
    pub version: NormalizedString,
    pub description: Option<NormalizedString>,
    pub scope: Option<Scope>,
    pub hashes: Option<Hashes>,
    pub licenses: Option<Licenses>,
    pub copyright: Option<NormalizedString>,
    pub cpe: Option<Cpe>,
    pub purl: Option<Purl>,
    pub swid: Option<Swid>,
    pub modified: Option<bool>,
    pub pedigree: Option<Pedigree>,
    pub external_references: Option<ExternalReferences>,
    pub properties: Option<Properties>,
    pub components: Option<Components>,
    pub evidence: Option<ComponentEvidence>,
}

#[derive(Debug, PartialEq)]
pub struct Components(pub Vec<Component>);

#[derive(Debug, PartialEq)]
pub enum Classification {
    Application,
    Framework,
    Library,
    Container,
    OperatingSystem,
    Device,
    Firmware,
    File,
    #[doc(hidden)]
    UnknownClassification(String),
}

impl ToString for Classification {
    fn to_string(&self) -> String {
        match self {
            Classification::Application => "application",
            Classification::Framework => "framework",
            Classification::Library => "library",
            Classification::Container => "container",
            Classification::OperatingSystem => "operating-system",
            Classification::Device => "device",
            Classification::Firmware => "firmware",
            Classification::File => "file",
            Classification::UnknownClassification(uc) => uc,
        }
        .to_string()
    }
}

impl Classification {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "application" => Self::Application,
            "framework" => Self::Framework,
            "library" => Self::Library,
            "container" => Self::Container,
            "operating-system" => Self::OperatingSystem,
            "device" => Self::Device,
            "firmware" => Self::Firmware,
            "file" => Self::File,
            unknown => Self::UnknownClassification(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Scope {
    Required,
    Optional,
    Excluded,
    #[doc(hidden)]
    UnknownScope(String),
}

impl ToString for Scope {
    fn to_string(&self) -> String {
        match self {
            Scope::Required => "required",
            Scope::Optional => "optional",
            Scope::Excluded => "excluded",
            Scope::UnknownScope(us) => us,
        }
        .to_string()
    }
}

impl Scope {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "required" => Self::Required,
            "optional" => Self::Optional,
            "excluded" => Self::Excluded,
            unknown => Self::UnknownScope(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MimeType(pub(crate) String); // TODO: validate regex

#[derive(Debug, PartialEq)]
pub struct Swid {
    pub tag_id: String,
    pub name: String,
    pub version: Option<String>,
    pub tag_version: Option<u32>,
    pub patch: Option<bool>,
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
}

#[derive(Debug, PartialEq)]
pub struct Cpe(pub(crate) String); // TODO: validate

#[derive(Debug, PartialEq)]
pub struct ComponentEvidence {
    pub licenses: Option<Licenses>,
    pub copyright: Option<CopyrightTexts>,
}

#[derive(Debug, PartialEq)]
pub struct Pedigree {
    pub ancestors: Option<Components>,
    pub descendants: Option<Components>,
    pub variants: Option<Components>,
    pub commits: Option<Commits>,
    pub patches: Option<Patches>,
    pub notes: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Copyright(pub String);

#[derive(Debug, PartialEq)]
pub struct CopyrightTexts(pub(crate) Vec<Copyright>);
