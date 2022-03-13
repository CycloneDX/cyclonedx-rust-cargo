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

use crate::external_models::uri::Uri;
use crate::models::hash::Hashes;

#[derive(Debug, PartialEq)]
pub struct ExternalReference {
    pub external_reference_type: ExternalReferenceType,
    pub url: Uri,
    pub comment: Option<String>,
    pub hashes: Option<Hashes>,
}

#[derive(Debug, PartialEq)]
pub struct ExternalReferences(pub Vec<ExternalReference>);

#[derive(Debug, PartialEq)]
pub enum ExternalReferenceType {
    Vcs,
    IssueTracker,
    Website,
    Advisories,
    Bom,
    MailingList,
    Social,
    Chat,
    Documentation,
    Support,
    Distribution,
    License,
    BuildMeta,
    BuildSystem,
    Other,
    #[doc(hidden)]
    UnknownExternalReferenceType(String),
}

impl ToString for ExternalReferenceType {
    fn to_string(&self) -> String {
        match self {
            ExternalReferenceType::Vcs => "vcs",
            ExternalReferenceType::IssueTracker => "issue-tracker",
            ExternalReferenceType::Website => "website",
            ExternalReferenceType::Advisories => "advisories",
            ExternalReferenceType::Bom => "bom",
            ExternalReferenceType::MailingList => "mailing-list",
            ExternalReferenceType::Social => "social",
            ExternalReferenceType::Chat => "chat",
            ExternalReferenceType::Documentation => "documentation",
            ExternalReferenceType::Support => "support",
            ExternalReferenceType::Distribution => "distribution",
            ExternalReferenceType::License => "license",
            ExternalReferenceType::BuildMeta => "build-meta",
            ExternalReferenceType::BuildSystem => "build-system",
            ExternalReferenceType::Other => "other",
            ExternalReferenceType::UnknownExternalReferenceType(un) => un,
        }
        .to_string()
    }
}

impl ExternalReferenceType {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "vcs" => Self::Vcs,
            "issue-tracker" => Self::IssueTracker,
            "website" => Self::Website,
            "advisories" => Self::Advisories,
            "bom" => Self::Bom,
            "mailing-list" => Self::MailingList,
            "social" => Self::Social,
            "chat" => Self::Chat,
            "documentation" => Self::Documentation,
            "support" => Self::Support,
            "distribution" => Self::Distribution,
            "license" => Self::License,
            "build-meta" => Self::BuildMeta,
            "build-system" => Self::BuildSystem,
            "other" => Self::Other,
            unknown => Self::UnknownExternalReferenceType(unknown.to_string()),
        }
    }
}
