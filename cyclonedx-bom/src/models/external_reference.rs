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

use crate::external_models::uri::{validate_uri, Uri};
use crate::models::hash::Hashes;
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

use super::bom::SpecVersion;

/// Represents a way to document systems, sites, and information that may be relevant but which are not included with the BOM.
///
/// Please see the [CycloneDX use case](https://cyclonedx.org/use-cases/#external-references) for more information and examples.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field(
                "external_reference_type",
                &self.external_reference_type,
                validate_external_reference_type,
            )
            .add_field("url", &self.url, validate_uri)
            .add_list("hashes", &self.hashes, |hash| hash.validate_version(version))
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalReferences(pub Vec<ExternalReference>);

impl Validate for ExternalReferences {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |reference| reference.validate_version(version))
            .into()
    }
}

pub fn validate_external_reference_type(
    reference_type: &ExternalReferenceType,
) -> Result<(), ValidationError> {
    if matches!(
        reference_type,
        ExternalReferenceType::UnknownExternalReferenceType(_)
    ) {
        return Err("Unknown external reference type".into());
    }
    Ok(())
}

/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_externalReferenceType).
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod test {
    use crate::{
        models::hash::{Hash, HashValue},
        validation,
    };

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
        .validate();

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
        .validate();

        assert_eq!(
            validation_result.errors(),
            Some(validation::list(
                "inner",
                [(
                    0,
                    vec![
                        validation::field(
                            "external_reference_type",
                            "Unknown external reference type"
                        ),
                        validation::field("url", "Uri does not conform to RFC 3986"),
                        validation::list(
                            "hashes",
                            [(
                                0,
                                validation::list(
                                    "inner",
                                    [(
                                        0,
                                        validation::field(
                                            "content",
                                            "HashValue does not match regular expression"
                                        )
                                    )]
                                )
                            )]
                        )
                    ]
                )]
            ))
        );
    }
}
