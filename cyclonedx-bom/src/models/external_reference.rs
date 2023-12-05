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
use crate::validation::{
    FailureReason, Validate, ValidationContext, ValidationError, ValidationPathComponent,
    ValidationResult,
};

/// Represents a way to document systems, sites, and information that may be relevant but which are not included with the BOM.
///
/// Please see the [CycloneDX use case](https://cyclonedx.org/use-cases/#external-references) for more information and examples.
#[derive(Debug, PartialEq, Eq)]
pub struct ExternalReference {
    pub external_reference_type: ExternalReferenceType,
    pub url: Uri,
    pub comment: Option<String>,
    pub hashes: Option<Hashes>,
}

impl ExternalReference {
    /// Constructs a new `ExternalReference` with the reference type and url
    /// ```
    /// use cyclonedx_bom::models::external_reference::{ExternalReference, ExternalReferenceType};
    /// use cyclonedx_bom::external_models::uri::{Uri, UriError};
    /// use std::convert::TryFrom;
    ///
    /// let url = Uri::try_from("https://example.org/support/sbom/portal-server/1.0.0".to_string())?;
    /// let external_reference = ExternalReference::new(ExternalReferenceType::Bom, url);
    /// # Ok::<(), UriError>(())
    /// ```
    pub fn new(external_reference_type: ExternalReferenceType, url: Uri) -> Self {
        Self {
            external_reference_type,
            url,
            comment: None,
            hashes: None,
        }
    }
}

impl Validate for ExternalReference {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        let external_reference_type_context = context
            .extend_context_with_struct_field("ExternalReference", "external_reference_type");

        results.push(
            self.external_reference_type
                .validate_with_context(external_reference_type_context)?,
        );

        let url_context = context.extend_context_with_struct_field("ExternalReference", "url");

        results.push(self.url.validate_with_context(url_context)?);

        if let Some(hashes) = &self.hashes {
            let context = context.extend_context_with_struct_field("ExternalReference", "hashes");

            results.push(hashes.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExternalReferences(pub Vec<ExternalReference>);

impl Validate for ExternalReferences {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, external_reference) in self.0.iter().enumerate() {
            let context = context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(external_reference.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_externalReferenceType).
#[derive(Debug, PartialEq, Eq)]
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

impl Validate for ExternalReferenceType {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match self {
            ExternalReferenceType::UnknownExternalReferenceType(_) => {
                Ok(ValidationResult::Failed {
                    reasons: vec![FailureReason {
                        message: "Unknown external reference type".to_string(),
                        context,
                    }],
                })
            }
            _ => Ok(ValidationResult::Passed),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::models::hash::{Hash, HashValue};
    use crate::validation::{FailureReason, ValidationPathComponent};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = ExternalReferences(vec![ExternalReference {
            external_reference_type: ExternalReferenceType::Bom,
            url: Uri("https://example.com".to_string()),
            comment: Some("Comment".to_string()),
            hashes: Some(Hashes(vec![])),
        }])
        .validate()
        .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = ExternalReferences(vec![ExternalReference {
            external_reference_type: ExternalReferenceType::UnknownExternalReferenceType(
                "unknown reference type".to_string(),
            ),
            url: Uri("invalid uri".to_string()),
            comment: Some("Comment".to_string()),
            hashes: Some(Hashes(vec![Hash {
                alg: crate::models::hash::HashAlgorithm::MD5,
                content: HashValue("invalid hash".to_string()),
            }])),
        }])
        .validate()
        .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message: "Unknown external reference type".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "ExternalReference".to_string(),
                                field_name: "external_reference_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Uri does not conform to RFC 3986".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "ExternalReference".to_string(),
                                field_name: "url".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "HashValue does not match regular expression".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "ExternalReference".to_string(),
                                field_name: "hashes".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Hash".to_string(),
                                field_name: "content".to_string()
                            },
                        ])
                    },
                ]
            }
        );
    }
}
