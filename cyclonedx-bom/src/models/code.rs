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
    external_models::{
        date_time::DateTime,
        normalized_string::{validate_normalized_string, NormalizedString},
        uri::{validate_uri, Uri},
        validate_date_time,
    },
    validation::{Validate, ValidationContext, ValidationError, ValidationResult},
};

use super::{attached_text::AttachedText, bom::SpecVersion};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Commit {
    pub uid: Option<NormalizedString>,
    pub url: Option<Uri>,
    pub author: Option<IdentifiableAction>,
    pub committer: Option<IdentifiableAction>,
    pub message: Option<NormalizedString>,
}

impl Validate for Commit {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("uid", self.uid.as_ref(), validate_normalized_string)
            .add_field_option("url", self.url.as_ref(), validate_uri)
            .add_struct_option("author", self.author.as_ref(), version)
            .add_struct_option("committer", self.committer.as_ref(), version)
            .add_field_option("message", self.message.as_ref(), validate_normalized_string)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Commits(pub Vec<Commit>);

impl Validate for Commits {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |commit| commit.validate_version(version))
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diff {
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
}

impl Validate for Diff {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("text", self.text.as_ref(), version)
            .add_field_option("url", self.url.as_ref(), validate_uri)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdentifiableAction {
    pub timestamp: Option<DateTime>,
    pub name: Option<NormalizedString>,
    pub email: Option<NormalizedString>,
}

impl Validate for IdentifiableAction {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("timestamp", self.timestamp.as_ref(), validate_date_time)
            .add_field_option("name", self.name.as_ref(), validate_normalized_string)
            .add_field_option("email", self.email.as_ref(), validate_normalized_string)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Issue {
    pub issue_type: IssueClassification,
    pub id: Option<NormalizedString>,
    pub name: Option<NormalizedString>,
    pub description: Option<NormalizedString>,
    pub source: Option<Source>,
    pub references: Option<Vec<Uri>>,
}

impl Validate for Issue {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field(
                "issue_type",
                &self.issue_type,
                validate_issue_classification,
            )
            .add_field_option("id", self.id.as_ref(), validate_normalized_string)
            .add_field_option("name", self.name.as_ref(), validate_normalized_string)
            .add_field_option(
                "description",
                self.description.as_ref(),
                validate_normalized_string,
            )
            .add_struct_option("source", self.source.as_ref(), version)
            .add_list_option("references", self.references.as_ref(), validate_uri)
            .into()
    }
}

pub fn validate_issue_classification(
    classification: &IssueClassification,
) -> Result<(), ValidationError> {
    if matches!(
        classification,
        IssueClassification::UnknownIssueClassification(_)
    ) {
        return Err(ValidationError::new("Unknown issue classification"));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, strum::Display, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum IssueClassification {
    Defect,
    Enhancement,
    Security,
    #[doc(hidden)]
    #[strum(default)]
    UnknownIssueClassification(String),
}

impl IssueClassification {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "defect" => Self::Defect,
            "enhancement" => Self::Enhancement,
            "security" => Self::Security,
            unknown => Self::UnknownIssueClassification(unknown.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Patch {
    pub patch_type: PatchClassification,
    pub diff: Option<Diff>,
    pub resolves: Option<Vec<Issue>>,
}

impl Validate for Patch {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_enum(
                "patch_type",
                &self.patch_type,
                validate_patch_classification,
            )
            .add_struct_option("diff", self.diff.as_ref(), version)
            .add_list_option("resolves", self.resolves.as_ref(), |issue| {
                issue.validate_version(version)
            })
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Patches(pub Vec<Patch>);

impl Validate for Patches {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |patch| patch.validate_version(version))
            .into()
    }
}

pub fn validate_patch_classification(
    classification: &PatchClassification,
) -> Result<(), ValidationError> {
    if matches!(
        classification,
        PatchClassification::UnknownPatchClassification(_)
    ) {
        return Err("Unknown patch classification".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, strum::Display, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum PatchClassification {
    Unofficial,
    Monkey,
    Backport,
    CherryPick,
    #[doc(hidden)]
    #[strum(default)]
    UnknownPatchClassification(String),
}

impl PatchClassification {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "unofficial" => Self::Unofficial,
            "monkey" => Self::Monkey,
            "backport" => Self::Backport,
            "cherry-pick" => Self::CherryPick,
            unknown => Self::UnknownPatchClassification(unknown.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Source {
    pub name: Option<NormalizedString>,
    pub url: Option<Uri>,
}

impl Validate for Source {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("name", self.name.as_ref(), validate_normalized_string)
            .add_field_option("url", self.url.as_ref(), validate_uri)
            .into()
    }
}

#[cfg(test)]
mod test {
    use crate::validation;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_commits_should_pass_validation() {
        let validation_result = Commits(vec![Commit {
            uid: Some(NormalizedString("no_whitespace".to_string())),
            url: Some(Uri("https://www.example.com".to_string())),
            author: Some(IdentifiableAction {
                timestamp: Some(DateTime("1969-06-28T01:20:00.00-04:00".to_string())),
                name: Some(NormalizedString("Name".to_string())),
                email: Some(NormalizedString("email@example.com".to_string())),
            }),
            committer: Some(IdentifiableAction {
                timestamp: Some(DateTime("1969-06-28T01:20:00.00-04:00".to_string())),
                name: Some(NormalizedString("Name".to_string())),
                email: Some(NormalizedString("email@example.com".to_string())),
            }),
            message: Some(NormalizedString("no_whitespace".to_string())),
        }])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn invalid_commits_should_fail_validation() {
        let validation_result = Commits(vec![Commit {
            uid: Some(NormalizedString("spaces and\ttabs".to_string())),
            url: Some(Uri("invalid uri".to_string())),
            author: Some(IdentifiableAction {
                timestamp: Some(DateTime("Thursday".to_string())),
                name: Some(NormalizedString("spaces and\ttabs".to_string())),
                email: Some(NormalizedString("spaces and\ttabs".to_string())),
            }),
            committer: Some(IdentifiableAction {
                timestamp: Some(DateTime("1970-01-01".to_string())),
                name: Some(NormalizedString("spaces and\ttabs".to_string())),
                email: Some(NormalizedString("spaces and\ttabs".to_string())),
            }),
            message: Some(NormalizedString("spaces and\ttabs".to_string())),
        }])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    vec![
                        validation::field(
                            "uid",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::field("url", "Uri does not conform to RFC 3986"),
                        validation::r#struct(
                            "author",
                            vec![
                                validation::field("timestamp", "DateTime does not conform to ISO 8601"),
                                validation::field("name", "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"),
                                validation::field("email", "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n")
                            ]
                        ),
                        validation::r#struct(
                            "committer",
                            vec![
                                validation::field("timestamp", "DateTime does not conform to ISO 8601"),
                                validation::field("name", "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"),
                                validation::field("email", "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"),
                            ]
                        ),
                        validation::field("message", "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n")
                    ]
                )]
            )
        );
    }

    #[test]
    fn valid_patches_should_pass_validation() {
        let validation_result = Patches(vec![Patch {
            patch_type: PatchClassification::Backport,
            diff: Some(Diff {
                text: Some(AttachedText {
                    content_type: None,
                    encoding: None,
                    content: "content".to_string(),
                }),
                url: Some(Uri("https://www.example.com".to_string())),
            }),
            resolves: Some(vec![Issue {
                issue_type: IssueClassification::Defect,
                id: Some(NormalizedString("issue_id".to_string())),
                name: Some(NormalizedString("issue_name".to_string())),
                description: Some(NormalizedString("issue_description".to_string())),
                source: Some(Source {
                    name: Some(NormalizedString("source_name".to_string())),
                    url: Some(Uri("https://example.com".to_string())),
                }),
                references: Some(vec![Uri("https://example.com".to_string())]),
            }]),
        }])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn invalid_patches_should_fail_validation() {
        let validation_result = Patches(vec![Patch {
            patch_type: PatchClassification::UnknownPatchClassification("unknown".to_string()),
            diff: Some(Diff {
                text: Some(AttachedText {
                    content_type: Some(NormalizedString("spaces and \ttabs".to_string())),
                    encoding: None,
                    content: "content".to_string(),
                }),
                url: Some(Uri("invalid uri".to_string())),
            }),
            resolves: Some(vec![Issue {
                issue_type: IssueClassification::UnknownIssueClassification("unknown".to_string()),
                id: Some(NormalizedString("spaces and \ttabs".to_string())),
                name: Some(NormalizedString("spaces and \ttabs".to_string())),
                description: Some(NormalizedString("spaces and \ttabs".to_string())),
                source: Some(Source {
                    name: Some(NormalizedString("spaces and \ttabs".to_string())),
                    url: Some(Uri("invalid uri".to_string())),
                }),
                references: Some(vec![Uri("invalid uri".to_string())]),
            }]),
        }])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    vec![
                        validation::r#enum("patch_type", "Unknown patch classification"),
                        validation::r#struct(
                            "diff",
                            vec![
                                validation::r#struct(
                                    "text",
                                    vec![
                                        validation::field(
                                            "content_type",
                                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                        )
                                    ]
                                ),
                                validation::field("url", "Uri does not conform to RFC 3986")
                            ]
                        ),
                        validation::list(
                            "resolves",
                            [(
                                0,
                                vec![
                                    validation::field("issue_type", "Unknown issue classification"),
                                    validation::field("id", "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"),
                                    validation::field("name", "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"),
                                    validation::field("description", "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"),
                                    validation::r#struct(
                                        "source",
                                        vec![
                                            validation::field(
                                                "name",
                                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n",
                                            ),
                                            validation::field(
                                                "url",
                                                "Uri does not conform to RFC 3986",
                                            )
                                        ]
                                    ),
                                    validation::list("references", [(0, validation::custom("", ["Uri does not conform to RFC 3986"]))])
                                ]
                            )]
                        )
                    ]
                )]
            )
        );
    }
}
