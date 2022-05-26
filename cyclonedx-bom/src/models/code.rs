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
    external_models::{date_time::DateTime, normalized_string::NormalizedString, uri::Uri},
    validation::{Validate, ValidationContext, ValidationError, ValidationResult},
};

use super::attached_text::AttachedText;

#[derive(Debug, PartialEq)]
pub struct Commit {
    pub uid: Option<NormalizedString>,
    pub url: Option<Uri>,
    pub author: Option<IdentifiableAction>,
    pub committer: Option<IdentifiableAction>,
    pub message: Option<NormalizedString>,
}

#[derive(Debug, PartialEq)]
pub struct Commits(pub Vec<Commit>);

#[derive(Debug, PartialEq)]
pub struct Diff {
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
}

#[derive(Debug, PartialEq)]
pub struct IdentifiableAction {
    pub timestamp: Option<DateTime>,
    pub name: Option<NormalizedString>,
    pub email: Option<NormalizedString>,
}

#[derive(Debug, PartialEq)]
pub struct Issue {
    pub issue_type: IssueClassification,
    pub id: Option<NormalizedString>,
    pub name: Option<NormalizedString>,
    pub description: Option<NormalizedString>,
    pub source: Option<Source>,
    pub references: Option<Vec<Uri>>,
}

#[derive(Debug, PartialEq)]
pub enum IssueClassification {
    Defect,
    Enhancement,
    Security,
    #[doc(hidden)]
    UnknownIssueClassification(String),
}

impl ToString for IssueClassification {
    fn to_string(&self) -> String {
        match self {
            IssueClassification::Defect => "defect",
            IssueClassification::Enhancement => "enhancement",
            IssueClassification::Security => "security",
            IssueClassification::UnknownIssueClassification(uic) => uic,
        }
        .to_string()
    }
}

impl IssueClassification {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "defect" => Self::Defect,
            "enhancement" => Self::Enhancement,
            "security" => Self::Security,
            unknown => Self::UnknownIssueClassification(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Patch {
    pub patch_type: PatchClassification,
    pub diff: Option<Diff>,
    pub resolves: Option<Vec<Issue>>,
}

#[derive(Debug, PartialEq)]
pub struct Patches(pub Vec<Patch>);

#[derive(Debug, PartialEq)]
pub enum PatchClassification {
    Unofficial,
    Monkey,
    Backport,
    CherryPick,
    #[doc(hidden)]
    UnknownPatchClassification(String),
}

impl ToString for PatchClassification {
    fn to_string(&self) -> String {
        match self {
            PatchClassification::Unofficial => "unofficial",
            PatchClassification::Monkey => "monkey",
            PatchClassification::Backport => "backport",
            PatchClassification::CherryPick => "cherry-pick",
            PatchClassification::UnknownPatchClassification(upc) => upc,
        }
        .to_string()
    }
}

impl PatchClassification {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "unofficial" => Self::Unofficial,
            "monkey" => Self::Monkey,
            "backport" => Self::Backport,
            "cherry-pick" => Self::CherryPick,
            unknown => Self::UnknownPatchClassification(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Source {
    pub name: Option<NormalizedString>,
    pub url: Option<Uri>,
}

impl Validate for Source {
    fn validate_with_context(
        &self,
        _context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        todo!()
    }
}
