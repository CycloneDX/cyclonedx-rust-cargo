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

use once_cell::sync::Lazy;
use regex::Regex;

use crate::external_models::uri::{validate_uri as validate_url, Uri as Url};
use crate::models::hash::Hashes;
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

use super::bom::SpecVersion;

/// Represents a way to document systems, sites, and information that may be relevant but which are not included with the BOM.
///
/// Please see the [CycloneDX use case](https://cyclonedx.org/use-cases/#external-references) for more information and examples.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    /// use cyclonedx_bom::external_models::uri::Uri;
    ///
    /// let url = Uri::new("https://example.org/support/sbom/portal-server/1.0.0");
    /// let external_reference = ExternalReference::new(ExternalReferenceType::Bom, url);
    /// ```
    pub fn new(external_reference_type: ExternalReferenceType, url: impl Into<Uri>) -> Self {
        Self {
            external_reference_type,
            url: url.into(),
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
            .add_field("url", &self.url, |uri| validate_reference_uri(uri, version))
            .add_list("hashes", &self.hashes, |hash| {
                hash.validate_version(version)
            })
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExternalReferences(pub Vec<ExternalReference>);

impl Validate for ExternalReferences {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |reference| {
                reference.validate_version(version)
            })
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, strum::Display)]
#[strum(serialize_all = "kebab-case")]
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
    DistributionIntake,
    License,
    BuildMeta,
    BuildSystem,
    Other,
    #[doc(hidden)]
    #[strum(default)]
    UnknownExternalReferenceType(String),
    ReleaseNotes,
    SecurityContact,
    ModelCard,
    Log,
    Configuration,
    Evidence,
    Formulation,
    Attestation,
    ThreatModel,
    AdversaryModel,
    RiskAssessment,
    VulnerabilityAssertion,
    ExploitabilityStatement,
    PentestReport,
    StaticAnalysisReport,
    DynamicAnalysisReport,
    RuntimeAnalysisReport,
    ComponentAnalysisReport,
    MaturityReport,
    CertificationReport,
    CondifiedInfrastructure,
    QualityMetrics,
    Poam,
}

impl ExternalReferenceType {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
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
            "distribution-intake" => Self::DistributionIntake,
            "license" => Self::License,
            "build-meta" => Self::BuildMeta,
            "build-system" => Self::BuildSystem,
            "release-notes" => Self::ReleaseNotes,
            "security-contact" => Self::SecurityContact,
            "model-card" => Self::ModelCard,
            "log" => Self::Log,
            "configuration" => Self::Configuration,
            "evidence" => Self::Evidence,
            "formulation" => Self::Formulation,
            "attestation" => Self::Attestation,
            "threat-model" => Self::ThreatModel,
            "adversary-model" => Self::AdversaryModel,
            "risk-assessment" => Self::RiskAssessment,
            "vulnerability-assertion" => Self::VulnerabilityAssertion,
            "exploitability-statement" => Self::ExploitabilityStatement,
            "pentest-report" => Self::PentestReport,
            "static-analysis-report" => Self::StaticAnalysisReport,
            "dynamic-analysis-report" => Self::DynamicAnalysisReport,
            "runtime-analysis-report" => Self::RuntimeAnalysisReport,
            "component-analysis-report" => Self::ComponentAnalysisReport,
            "maturity-report" => Self::MaturityReport,
            "certification-report" => Self::CertificationReport,
            "codified-infrastructure" => Self::CondifiedInfrastructure,
            "quality-metrics" => Self::QualityMetrics,
            "poam" => Self::Poam,
            "other" => Self::Other,
            unknown => Self::UnknownExternalReferenceType(unknown.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Uri {
    Url(Url),
    BomLink(BomLink),
}

/// Validates an [`Uri`], the [`Uri::BomLink`] variant was added in 1.5 only.
fn validate_reference_uri(uri: &Uri, version: SpecVersion) -> Result<(), ValidationError> {
    match uri {
        Uri::Url(url) => validate_url(url),
        Uri::BomLink(bom_link) => validate_bom_link(bom_link, version),
    }
}

impl From<Url> for Uri {
    fn from(url: Url) -> Self {
        if url.is_bomlink() {
            Self::BomLink(BomLink(url.to_string()))
        } else {
            Self::Url(url)
        }
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Uri::Url(uri) => uri.to_string(),
            Uri::BomLink(link) => link.0.to_string(),
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BomLink(pub String);

fn validate_bom_link(bom_link: &BomLink, version: SpecVersion) -> Result<(), ValidationError> {
    if version < SpecVersion::V1_5 {
        return Err("BOM-Link not supported before version 1.5".into());
    }

    static BOM_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^urn:cdx:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/[1-9][0-9]*(#.+)?$").unwrap()
    });

    if !BOM_LINK_REGEX.is_match(&bom_link.0) {
        return Err(ValidationError::new("Invalid BOM-Link"));
    }

    Ok(())
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
    fn it_should_convert_url_into_uri() {
        let url = Url("https://example.com".to_string());
        assert_eq!(Uri::Url(Url("https://example.com".to_string())), url.into());
    }

    #[test]
    fn it_should_convert_url_into_bomlink() {
        let url = Url("urn:cdx:f08a6ccd-4dce-4759-bd84-c626675d60a7/1".to_string());
        assert_eq!(
            Uri::BomLink(BomLink(
                "urn:cdx:f08a6ccd-4dce-4759-bd84-c626675d60a7/1".to_string()
            )),
            url.into()
        );
    }

    #[test]
    fn it_should_validate_external_reference_with_bomlink_correctly() {
        let url = Uri::BomLink(BomLink(
            "urn:cdx:f08a6ccd-4dce-4759-bd84-c626675d60a7/1".to_string(),
        ));

        let external_reference = ExternalReference {
            external_reference_type: ExternalReferenceType::Bom,
            url,
            comment: Some("Comment".to_string()),
            hashes: Some(Hashes(vec![])),
        };

        assert!(external_reference
            .validate_version(SpecVersion::V1_5)
            .passed());
        assert!(external_reference
            .validate_version(SpecVersion::V1_4)
            .has_errors());
        assert!(external_reference
            .validate_version(SpecVersion::V1_3)
            .has_errors());
    }

    #[test]
    fn it_should_pass_validation() {
        let validation_result = ExternalReferences(vec![
            ExternalReference {
                external_reference_type: ExternalReferenceType::Bom,
                url: Uri::Url(Url("https://example.com".to_string())),
                comment: Some("Comment".to_string()),
                hashes: Some(Hashes(vec![])),
            },
            ExternalReference {
                external_reference_type: ExternalReferenceType::Bom,
                url: Uri::BomLink(BomLink(
                    "urn:cdx:f08a6ccd-4dce-4759-bd84-c626675d60a7/1".to_string(),
                )),
                comment: Some("Comment".to_string()),
                hashes: Some(Hashes(vec![])),
            },
            ExternalReference {
                external_reference_type: ExternalReferenceType::Bom,
                url: Uri::BomLink(BomLink(
                    "urn:cdx:f08a6ccd-4dce-4759-bd84-c626675d60a7/1#componentA".to_string(),
                )),
                comment: Some("Comment".to_string()),
                hashes: Some(Hashes(vec![])),
            },
        ])
        .validate_version(SpecVersion::V1_5);

        assert!(validation_result.passed());
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = ExternalReferences(vec![
            ExternalReference {
                external_reference_type: ExternalReferenceType::UnknownExternalReferenceType(
                    "unknown reference type".to_string(),
                ),
                url: Uri::Url(Url("invalid uri".to_string())),
                comment: Some("Comment".to_string()),
                hashes: Some(Hashes(vec![Hash {
                    alg: crate::models::hash::HashAlgorithm::MD5,
                    content: HashValue("invalid hash".to_string()),
                }])),
            },
            ExternalReference {
                external_reference_type: ExternalReferenceType::UnknownExternalReferenceType(
                    "unknown reference type".to_string(),
                ),
                url: Uri::BomLink(BomLink("invalid bom-link".to_string())),
                comment: Some("Comment".to_string()),
                hashes: Some(Hashes(vec![Hash {
                    alg: crate::models::hash::HashAlgorithm::MD5,
                    content: HashValue("invalid hash".to_string()),
                }])),
            },
        ])
        .validate_version(SpecVersion::V1_5);

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [
                    (
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
                    ),
                    (
                        1,
                        vec![
                            validation::field(
                                "external_reference_type",
                                "Unknown external reference type"
                            ),
                            validation::field("url", "Invalid BOM-Link"),
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
                    )
                ]
            )
        );
    }
}
