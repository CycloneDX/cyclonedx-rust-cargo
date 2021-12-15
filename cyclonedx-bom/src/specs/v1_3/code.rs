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
    models,
    specs::v1_3::attached_text::AttachedText,
    utilities::{convert_optional, convert_optional_vec},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Commit {
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<IdentifiableAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    committer: Option<IdentifiableAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl From<models::Commit> for Commit {
    fn from(other: models::Commit) -> Self {
        Self {
            uid: other.uid.map(|uid| uid.to_string()),
            url: other.url.map(|url| url.to_string()),
            author: convert_optional(other.author),
            committer: convert_optional(other.committer),
            message: other.message.map(|m| m.to_string()),
        }
    }
}

impl From<Commit> for models::Commit {
    fn from(other: Commit) -> Self {
        Self {
            uid: other.uid.map(NormalizedString::new_unchecked),
            url: other.url.map(Uri),
            author: convert_optional(other.author),
            committer: convert_optional(other.committer),
            message: other.message.map(NormalizedString::new_unchecked),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct IdentifiableAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
}

impl From<models::IdentifiableAction> for IdentifiableAction {
    fn from(other: models::IdentifiableAction) -> Self {
        Self {
            timestamp: other.timestamp.map(|t| t.to_string()),
            name: other.name.map(|n| n.to_string()),
            email: other.email.map(|e| e.to_string()),
        }
    }
}

impl From<IdentifiableAction> for models::IdentifiableAction {
    fn from(other: IdentifiableAction) -> Self {
        Self {
            timestamp: other.timestamp.map(DateTime),
            name: other.name.map(NormalizedString::new_unchecked),
            email: other.email.map(NormalizedString::new_unchecked),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Patch {
    #[serde(rename = "type")]
    patch_type: String,
    diff: Diff,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolves: Option<Vec<Issue>>,
}

impl From<models::Patch> for Patch {
    fn from(other: models::Patch) -> Self {
        Self {
            patch_type: other.patch_type.to_string(),
            diff: other.diff.into(),
            resolves: convert_optional_vec(other.resolves),
        }
    }
}

impl From<Patch> for models::Patch {
    fn from(other: Patch) -> Self {
        Self {
            patch_type: models::PatchClassification::new_unchecked(other.patch_type),
            diff: other.diff.into(),
            resolves: convert_optional_vec(other.resolves),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Diff {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<AttachedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl From<models::Diff> for Diff {
    fn from(other: models::Diff) -> Self {
        Self {
            text: convert_optional(other.text),
            url: other.url.map(|u| u.to_string()),
        }
    }
}

impl From<Diff> for models::Diff {
    fn from(other: Diff) -> Self {
        Self {
            text: convert_optional(other.text),
            url: other.url.map(Uri),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Issue {
    #[serde(rename = "type")]
    issue_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    references: Option<Vec<String>>,
}

impl From<models::Issue> for Issue {
    fn from(other: models::Issue) -> Self {
        Self {
            issue_type: other.issue_type.to_string(),
            id: other.id.map(|i| i.to_string()),
            name: other.name.map(|n| n.to_string()),
            description: other.description.map(|d| d.to_string()),
            source: convert_optional(other.source),
            references: other
                .references
                .map(|references| references.into_iter().map(|r| r.to_string()).collect()),
        }
    }
}

impl From<Issue> for models::Issue {
    fn from(other: Issue) -> Self {
        Self {
            issue_type: models::IssueClassification::new_unchecked(other.issue_type),
            id: other.id.map(NormalizedString::new_unchecked),
            name: other.name.map(NormalizedString::new_unchecked),
            description: other.description.map(NormalizedString::new_unchecked),
            source: convert_optional(other.source),
            references: other
                .references
                .map(|references| references.into_iter().map(Uri).collect()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl From<models::Source> for Source {
    fn from(other: models::Source) -> Self {
        Self {
            name: other.name.map(|n| n.to_string()),
            url: other.url.map(|u| u.to_string()),
        }
    }
}

impl From<Source> for models::Source {
    fn from(other: Source) -> Self {
        Self {
            name: other.name.map(NormalizedString::new_unchecked),
            url: other.url.map(Uri),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::specs::v1_3::attached_text::test::{
        corresponding_attached_text, example_attached_text,
    };

    use super::*;

    pub(crate) fn example_commit() -> Commit {
        Commit {
            uid: Some("uid".to_string()),
            url: Some("url".to_string()),
            author: Some(example_identifiable_action()),
            committer: Some(example_identifiable_action()),
            message: Some("message".to_string()),
        }
    }

    pub(crate) fn corresponding_commit() -> models::Commit {
        models::Commit {
            uid: Some(NormalizedString::new_unchecked("uid".to_string())),
            url: Some(Uri("url".to_string())),
            author: Some(corresponding_identifiable_action()),
            committer: Some(corresponding_identifiable_action()),
            message: Some(NormalizedString::new_unchecked("message".to_string())),
        }
    }

    fn example_identifiable_action() -> IdentifiableAction {
        IdentifiableAction {
            timestamp: Some("timestamp".to_string()),
            name: Some("name".to_string()),
            email: Some("email".to_string()),
        }
    }

    fn corresponding_identifiable_action() -> models::IdentifiableAction {
        models::IdentifiableAction {
            timestamp: Some(DateTime("timestamp".to_string())),
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            email: Some(NormalizedString::new_unchecked("email".to_string())),
        }
    }

    pub(crate) fn example_patch() -> Patch {
        Patch {
            patch_type: "patch type".to_string(),
            diff: example_diff(),
            resolves: Some(vec![example_issue()]),
        }
    }

    pub(crate) fn corresponding_patch() -> models::Patch {
        models::Patch {
            patch_type: models::PatchClassification::UnknownPatchClassification(
                "patch type".to_string(),
            ),
            diff: corresponding_diff(),
            resolves: Some(vec![corresponding_issue()]),
        }
    }

    fn example_diff() -> Diff {
        Diff {
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        }
    }

    fn corresponding_diff() -> models::Diff {
        models::Diff {
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        }
    }

    fn example_issue() -> Issue {
        Issue {
            issue_type: "issue type".to_string(),
            id: Some("id".to_string()),
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            source: Some(example_source()),
            references: Some(vec!["reference".to_string()]),
        }
    }

    fn corresponding_issue() -> models::Issue {
        models::Issue {
            issue_type: models::IssueClassification::UnknownIssueClassification(
                "issue type".to_string(),
            ),
            id: Some(NormalizedString::new_unchecked("id".to_string())),
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            description: Some(NormalizedString::new_unchecked("description".to_string())),
            source: Some(corresponding_source()),
            references: Some(vec![Uri("reference".to_string())]),
        }
    }

    fn example_source() -> Source {
        Source {
            name: Some("name".to_string()),
            url: Some("url".to_string()),
        }
    }

    fn corresponding_source() -> models::Source {
        models::Source {
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            url: Some(Uri("url".to_string())),
        }
    }
}
