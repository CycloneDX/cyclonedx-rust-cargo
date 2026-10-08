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
        date_time::{validate_date_time, DateTime},
        uri::{validate_uri, Uri},
    },
    models::{
        attachment::Attachment,
        bom::{BomReference, SpecVersion},
        component::{validate_confidence, Components, ConfidenceScore},
        data_governance::DataGovernance,
        external_reference::{ExternalReference, ExternalReferences},
        organization::{OrganizationalContact, OrganizationalEntity},
        service::Services,
        signature::Signature,
    },
    validation::{Validate, ValidationContext, ValidationResult},
};

/// The list of assessors, attestations, claims, evidence, targets and the affirmation that
/// together form the declarations of a BOM.
///
/// Added in version 1.6, see bom-1.6.schema.json #properties/declarations
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Declarations {
    pub assessors: Option<Vec<Assessor>>,
    pub attestations: Option<Vec<Attestation>>,
    pub claims: Option<Vec<Claim>>,
    pub evidence: Option<Vec<DeclarationEvidence>>,
    pub targets: Option<Targets>,
    pub affirmation: Option<Affirmation>,
    pub signature: Option<Signature>,
}

impl Validate for Declarations {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("assessors", self.assessors.as_ref(), |assessor| {
                assessor.validate_version(version)
            })
            .add_list_option("attestations", self.attestations.as_ref(), |attestation| {
                attestation.validate_version(version)
            })
            .add_list_option("claims", self.claims.as_ref(), |claim| {
                claim.validate_version(version)
            })
            .add_list_option("evidence", self.evidence.as_ref(), |evidence| {
                evidence.validate_version(version)
            })
            .add_struct_option("targets", self.targets.as_ref(), version)
            .add_struct_option("affirmation", self.affirmation.as_ref(), version)
            .into()
    }
}

/// The assessor who evaluates claims and determines conformance to requirements.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Assessor {
    pub bom_ref: Option<BomReference>,
    pub third_party: Option<bool>,
    pub organization: Option<OrganizationalEntity>,
}

impl Validate for Assessor {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("organization", self.organization.as_ref(), version)
            .into()
    }
}

/// A collection of requirements, claims and the conformance and confidence assigned by an
/// assessor.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Attestation {
    pub summary: Option<String>,
    /// Reference to an [`Assessor`]
    pub assessor: Option<BomReference>,
    pub map: Option<Vec<AttestationMap>>,
    pub signature: Option<Signature>,
}

impl Validate for Attestation {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("map", self.map.as_ref(), |map| {
                map.validate_version(version)
            })
            .into()
    }
}

/// Maps a requirement to the claims and counter claims supporting it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttestationMap {
    pub requirement: Option<BomReference>,
    pub claims: Option<Vec<BomReference>>,
    pub counter_claims: Option<Vec<BomReference>>,
    pub conformance: Option<Conformance>,
    pub confidence: Option<Confidence>,
}

impl Validate for AttestationMap {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("conformance", self.conformance.as_ref(), version)
            .add_struct_option("confidence", self.confidence.as_ref(), version)
            .into()
    }
}

/// The conformance of the claims meeting a requirement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Conformance {
    /// Score between 0.0 and 1.0
    pub score: Option<ConfidenceScore>,
    pub rationale: Option<String>,
    pub mitigation_strategies: Option<Vec<BomReference>>,
}

impl Validate for Conformance {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("score", self.score.as_ref(), validate_confidence)
            .into()
    }
}

/// The confidence of the claims meeting a requirement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Confidence {
    /// Score between 0.0 and 1.0
    pub score: Option<ConfidenceScore>,
    pub rationale: Option<String>,
}

impl Validate for Confidence {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("score", self.score.as_ref(), validate_confidence)
            .into()
    }
}

/// A claim about a target, backed by evidence.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Claim {
    pub bom_ref: Option<BomReference>,
    /// Reference to the object the claim is about (organization, component or service)
    pub target: Option<BomReference>,
    pub predicate: Option<String>,
    pub mitigation_strategies: Option<Vec<BomReference>>,
    pub reasoning: Option<String>,
    pub evidence: Option<Vec<BomReference>>,
    pub counter_evidence: Option<Vec<BomReference>>,
    pub external_references: Option<ExternalReferences>,
    pub signature: Option<Signature>,
}

impl Validate for Claim {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option(
                "external_references",
                self.external_references.as_ref(),
                version,
            )
            .into()
    }
}

/// Evidence that supports or refutes a claim.
///
/// Named `DeclarationEvidence` to avoid confusion with the component `Evidence`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeclarationEvidence {
    pub bom_ref: Option<BomReference>,
    pub property_name: Option<String>,
    pub description: Option<String>,
    pub data: Option<Vec<EvidenceData>>,
    pub created: Option<DateTime>,
    pub expires: Option<DateTime>,
    pub author: Option<OrganizationalContact>,
    pub reviewer: Option<OrganizationalContact>,
    pub signature: Option<Signature>,
}

impl Validate for DeclarationEvidence {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("data", self.data.as_ref(), |data| {
                data.validate_version(version)
            })
            .add_field_option("created", self.created.as_ref(), validate_date_time)
            .add_field_option("expires", self.expires.as_ref(), validate_date_time)
            .add_struct_option("author", self.author.as_ref(), version)
            .add_struct_option("reviewer", self.reviewer.as_ref(), version)
            .into()
    }
}

/// The data the evidence consists of.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EvidenceData {
    pub name: Option<String>,
    pub contents: Option<EvidenceDataContents>,
    /// Data classification, e.g. "PII" or "Company Confidential"
    pub classification: Option<String>,
    pub sensitive_data: Option<Vec<String>>,
    pub governance: Option<DataGovernance>,
}

impl Validate for EvidenceData {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("contents", self.contents.as_ref(), version)
            .add_struct_option("governance", self.governance.as_ref(), version)
            .into()
    }
}

/// The contents of [`EvidenceData`], inline or by URL.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EvidenceDataContents {
    pub attachment: Option<Attachment>,
    pub url: Option<Uri>,
}

impl Validate for EvidenceDataContents {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("attachment", self.attachment.as_ref(), version)
            .add_field_option("url", self.url.as_ref(), validate_uri)
            .into()
    }
}

/// The organizations, components and services the claims are about.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Targets {
    pub organizations: Option<Vec<OrganizationalEntity>>,
    pub components: Option<Components>,
    pub services: Option<Services>,
}

impl Validate for Targets {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("organizations", self.organizations.as_ref(), |org| {
                org.validate_version(version)
            })
            .add_struct_option("components", self.components.as_ref(), version)
            .add_struct_option("services", self.services.as_ref(), version)
            .into()
    }
}

/// A statement that the declarations are accurate, signed by the signatories.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Affirmation {
    pub statement: Option<String>,
    pub signatories: Option<Vec<Signatory>>,
    pub signature: Option<Signature>,
}

impl Validate for Affirmation {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("signatories", self.signatories.as_ref(), |signatory| {
                signatory.validate_version(version)
            })
            .into()
    }
}

/// A person who signs the [`Affirmation`], either digitally with a signature or by an
/// organization plus an external reference to the signature.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Signatory {
    pub name: Option<String>,
    pub role: Option<String>,
    pub signature: Option<Signature>,
    pub organization: Option<OrganizationalEntity>,
    pub external_reference: Option<ExternalReference>,
}

impl Validate for Signatory {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        // The JSON schema's oneOf (signature, or organization plus external reference) is not
        // enforced: XML carries the signature as XML-DSig, which this crate skips when reading,
        // so a valid XML signatory would always fail.
        ValidationContext::new()
            .add_struct_option("organization", self.organization.as_ref(), version)
            .add_struct_option(
                "external_reference",
                self.external_reference.as_ref(),
                version,
            )
            .into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        external_models::normalized_string::NormalizedString,
        models::external_reference::{ExternalReferenceType, Uri as ExternalUri},
        validation,
    };
    use pretty_assertions::assert_eq;

    fn signatory() -> Signatory {
        Signatory {
            name: Some("Jerry".to_string()),
            role: Some("COO".to_string()),
            signature: None,
            organization: Some(OrganizationalEntity {
                bom_ref: None,
                name: Some(NormalizedString::new("Acme Inc")),
                url: None,
                contact: None,
            }),
            external_reference: Some(ExternalReference {
                properties: None,
                external_reference_type: ExternalReferenceType::ElectronicSignature,
                url: ExternalUri::Url(Uri("https://example.com/coo-sig.png".to_string())),
                comment: None,
                hashes: None,
            }),
        }
    }

    #[test]
    fn valid_declarations_should_pass_validation() {
        let declarations = Declarations {
            assessors: None,
            attestations: Some(vec![Attestation {
                summary: None,
                assessor: None,
                map: Some(vec![AttestationMap {
                    requirement: None,
                    claims: None,
                    counter_claims: None,
                    conformance: Some(Conformance {
                        score: Some(ConfidenceScore::new(0.8)),
                        rationale: None,
                        mitigation_strategies: None,
                    }),
                    confidence: None,
                }]),
                signature: None,
            }]),
            claims: None,
            evidence: None,
            targets: None,
            affirmation: Some(Affirmation {
                statement: None,
                signatories: Some(vec![signatory()]),
                signature: None,
            }),
            signature: None,
        };

        assert!(declarations.validate_version(SpecVersion::V1_6).passed());
    }

    #[test]
    fn invalid_declarations_should_fail_validation() {
        let declarations = Declarations {
            assessors: None,
            attestations: Some(vec![Attestation {
                summary: None,
                assessor: None,
                map: Some(vec![AttestationMap {
                    requirement: None,
                    claims: None,
                    counter_claims: None,
                    conformance: None,
                    confidence: Some(Confidence {
                        score: Some(ConfidenceScore::new(1.5)),
                        rationale: None,
                    }),
                }]),
                signature: None,
            }]),
            claims: None,
            evidence: Some(vec![DeclarationEvidence {
                bom_ref: None,
                property_name: None,
                description: None,
                data: None,
                created: Some(DateTime("invalid".to_string())),
                expires: None,
                author: None,
                reviewer: None,
                signature: None,
            }]),
            targets: None,
            affirmation: None,
            signature: None,
        };

        let actual = declarations.validate_version(SpecVersion::V1_6);

        assert_eq!(
            actual,
            vec![
                validation::list(
                    "attestations",
                    [(
                        0,
                        validation::list(
                            "map",
                            [(
                                0,
                                validation::r#struct(
                                    "confidence",
                                    validation::field(
                                        "score",
                                        "Confidence score outside range 0.0 - 1.0"
                                    )
                                )
                            )]
                        )
                    )]
                ),
                validation::list(
                    "evidence",
                    [(
                        0,
                        validation::field("created", "DateTime does not conform to ISO 8601")
                    )]
                ),
            ]
            .into()
        );
    }
}
