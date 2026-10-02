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

use cyclonedx_bom_macros::versioned;

#[versioned("1.6", "1.7")]
pub(crate) mod base {
    #[versioned("1.6")]
    use crate::specs::v1_6::{
        attachment::Attachment,
        component::Components,
        data_governance::DataGovernance,
        external_reference::{ExternalReference, ExternalReferences},
        service::Services,
    };
    #[versioned("1.7")]
    use crate::specs::v1_7::{
        attachment::Attachment,
        component::Components,
        data_governance::DataGovernance,
        external_reference::{ExternalReference, ExternalReferences},
        service::Services,
    };
    use crate::{
        elem_tag,
        errors::{BomError, XmlReadError, XmlWriteError},
        external_models::{date_time::DateTime, uri::Uri},
        get_elements_lax,
        models::{self, bom::BomReference, component::ConfidenceScore},
        specs::common::{
            organization::{OrganizationalContact, OrganizationalEntity},
            signature::Signature,
        },
        utilities::{convert_optional, convert_optional_vec, try_convert_optional},
        xml::{
            optional_attribute, read_lax_validation_tag, read_simple_tag, to_xml_read_error,
            to_xml_write_error, unexpected_element_error, write_close_tag, write_list_string_tag,
            write_simple_option_tag, write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
            VecXmlReader,
        },
    };
    use serde::{Deserialize, Serialize};
    use xml::{attribute::OwnedAttribute, name::OwnedName, reader, writer::XmlEvent};

    const DECLARATIONS_TAG: &str = "declarations";
    const ASSESSORS_TAG: &str = "assessors";
    const ASSESSOR_TAG: &str = "assessor";
    const ATTESTATIONS_TAG: &str = "attestations";
    const ATTESTATION_TAG: &str = "attestation";
    const CLAIMS_TAG: &str = "claims";
    const CLAIM_TAG: &str = "claim";
    const COUNTER_CLAIMS_TAG: &str = "counterClaims";
    const COUNTER_CLAIM_TAG: &str = "counterClaim";
    const EVIDENCE_TAG: &str = "evidence";
    const COUNTER_EVIDENCE_TAG: &str = "counterEvidence";
    const TARGETS_TAG: &str = "targets";
    const AFFIRMATION_TAG: &str = "affirmation";
    const SIGNATURE_TAG: &str = "signature";
    const BOM_REF_ATTR: &str = "bom-ref";
    const THIRD_PARTY_TAG: &str = "thirdParty";
    const ORGANIZATION_TAG: &str = "organization";
    const ORGANIZATIONS_TAG: &str = "organizations";
    const COMPONENTS_TAG: &str = "components";
    const SERVICES_TAG: &str = "services";
    const SUMMARY_TAG: &str = "summary";
    const MAP_TAG: &str = "map";
    const REQUIREMENT_TAG: &str = "requirement";
    const CONFORMANCE_TAG: &str = "conformance";
    const CONFIDENCE_TAG: &str = "confidence";
    const SCORE_TAG: &str = "score";
    const RATIONALE_TAG: &str = "rationale";
    const MITIGATION_STRATEGIES_TAG: &str = "mitigationStrategies";
    const MITIGATION_STRATEGY_TAG: &str = "mitigationStrategy";
    const TARGET_TAG: &str = "target";
    const PREDICATE_TAG: &str = "predicate";
    const REASONING_TAG: &str = "reasoning";
    const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";
    const EXTERNAL_REFERENCE_TAG: &str = "externalReference";
    const PROPERTY_NAME_TAG: &str = "propertyName";
    const DESCRIPTION_TAG: &str = "description";
    const DATA_TAG: &str = "data";
    const CREATED_TAG: &str = "created";
    const EXPIRES_TAG: &str = "expires";
    const AUTHOR_TAG: &str = "author";
    const REVIEWER_TAG: &str = "reviewer";
    const NAME_TAG: &str = "name";
    const CONTENTS_TAG: &str = "contents";
    const ATTACHMENT_TAG: &str = "attachment";
    const URL_TAG: &str = "url";
    const CLASSIFICATION_TAG: &str = "classification";
    const SENSITIVE_DATA_TAG: &str = "sensitiveData";
    const GOVERNANCE_TAG: &str = "governance";
    const STATEMENT_TAG: &str = "statement";
    const SIGNATORIES_TAG: &str = "signatories";
    const SIGNATORY_TAG: &str = "signatory";
    const ROLE_TAG: &str = "role";

    elem_tag!(AssessorTag = "assessor");
    elem_tag!(AttestationTag = "attestation");
    elem_tag!(ClaimTag = "claim");
    elem_tag!(CounterClaimTag = "counterClaim");
    elem_tag!(EvidenceTag = "evidence");
    elem_tag!(MitigationStrategyTag = "mitigationStrategy");
    elem_tag!(OrganizationTag = "organization");
    elem_tag!(SignatoryTag = "signatory");

    fn into_bom_refs(refs: Option<Vec<String>>) -> Option<Vec<BomReference>> {
        refs.map(|refs| refs.into_iter().map(BomReference).collect())
    }

    fn from_bom_refs(refs: Option<Vec<BomReference>>) -> Option<Vec<String>> {
        refs.map(|refs| refs.into_iter().map(|r| r.0).collect())
    }

    fn write_start_tag_with_bom_ref<W: std::io::Write>(
        writer: &mut xml::EventWriter<W>,
        tag: &str,
        bom_ref: &Option<String>,
    ) -> Result<(), XmlWriteError> {
        let mut start_tag = XmlEvent::start_element(tag);
        if let Some(bom_ref) = bom_ref {
            start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
        }
        writer.write(start_tag).map_err(to_xml_write_error(tag))
    }

    fn write_list_option_tag<W: std::io::Write>(
        writer: &mut xml::EventWriter<W>,
        tag: &str,
        list: &Option<Vec<impl ToXml>>,
    ) -> Result<(), XmlWriteError> {
        if let Some(list) = list {
            crate::xml::write_list_tag(writer, tag, list)?;
        }
        Ok(())
    }

    fn write_string_list_option_tag<W: std::io::Write>(
        writer: &mut xml::EventWriter<W>,
        tag: &str,
        child_tag: &str,
        list: &Option<Vec<String>>,
    ) -> Result<(), XmlWriteError> {
        if let Some(list) = list {
            write_list_string_tag(writer, tag, child_tag, list)?;
        }
        Ok(())
    }

    /// Writes each item as a sibling element, for XSD elements with `maxOccurs="unbounded"`
    /// that have no wrapper element.
    fn write_repeated_string_tag<W: std::io::Write>(
        writer: &mut xml::EventWriter<W>,
        tag: &str,
        list: &Option<Vec<String>>,
    ) -> Result<(), XmlWriteError> {
        for item in list.iter().flatten() {
            write_simple_tag(writer, tag, item)?;
        }
        Ok(())
    }

    /// bom-1.6.schema.json #properties/declarations
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Declarations {
        #[serde(skip_serializing_if = "Option::is_none")]
        assessors: Option<Vec<Assessor>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attestations: Option<Vec<Attestation>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        claims: Option<Vec<Claim>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence: Option<Vec<DeclarationEvidence>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        targets: Option<Targets>,
        #[serde(skip_serializing_if = "Option::is_none")]
        affirmation: Option<Affirmation>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<Signature>,
    }

    impl TryFrom<models::declarations::Declarations> for Declarations {
        type Error = BomError;

        fn try_from(other: models::declarations::Declarations) -> Result<Self, Self::Error> {
            Ok(Self {
                assessors: convert_optional_vec(other.assessors),
                attestations: convert_optional_vec(other.attestations),
                claims: convert_optional_vec(other.claims),
                evidence: convert_optional_vec(other.evidence),
                targets: try_convert_optional(other.targets)?,
                affirmation: convert_optional(other.affirmation),
                signature: convert_optional(other.signature),
            })
        }
    }

    impl From<Declarations> for models::declarations::Declarations {
        fn from(other: Declarations) -> Self {
            Self {
                assessors: convert_optional_vec(other.assessors),
                attestations: convert_optional_vec(other.attestations),
                claims: convert_optional_vec(other.claims),
                evidence: convert_optional_vec(other.evidence),
                targets: convert_optional(other.targets),
                affirmation: convert_optional(other.affirmation),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl ToXml for Declarations {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, DECLARATIONS_TAG)?;
            write_list_option_tag(writer, ASSESSORS_TAG, &self.assessors)?;
            write_list_option_tag(writer, ATTESTATIONS_TAG, &self.attestations)?;
            write_list_option_tag(writer, CLAIMS_TAG, &self.claims)?;
            write_list_option_tag(writer, EVIDENCE_TAG, &self.evidence)?;
            self.targets.write_xml_element(writer)?;
            self.affirmation.write_xml_element(writer)?;
            self.signature.write_xml_element(writer)?;
            write_close_tag(writer, DECLARATIONS_TAG)
        }
    }

    impl FromXml for Declarations {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            get_elements_lax! {
                event_reader, element_name,
                ASSESSORS_TAG => assessors: VecXmlReader<Assessor, AssessorTag>,
                ATTESTATIONS_TAG => attestations: VecXmlReader<Attestation, AttestationTag>,
                CLAIMS_TAG => claims: VecXmlReader<Claim, ClaimTag>,
                EVIDENCE_TAG => evidence: VecXmlReader<DeclarationEvidence, EvidenceTag>,
                TARGETS_TAG => targets: Targets,
                AFFIRMATION_TAG => affirmation: Affirmation,
                SIGNATURE_TAG => signature: Signature,
            };

            Ok(Self {
                assessors: assessors.map(Vec::from),
                attestations: attestations.map(Vec::from),
                claims: claims.map(Vec::from),
                evidence: evidence.map(Vec::from),
                targets,
                affirmation,
                signature,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Assessor {
        #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
        bom_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        third_party: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        organization: Option<OrganizationalEntity>,
    }

    impl From<models::declarations::Assessor> for Assessor {
        fn from(other: models::declarations::Assessor) -> Self {
            Self {
                bom_ref: other.bom_ref.map(|r| r.0),
                third_party: other.third_party,
                organization: convert_optional(other.organization),
            }
        }
    }

    impl From<Assessor> for models::declarations::Assessor {
        fn from(other: Assessor) -> Self {
            Self {
                bom_ref: other.bom_ref.map(BomReference),
                third_party: other.third_party,
                organization: convert_optional(other.organization),
            }
        }
    }

    impl ToXml for Assessor {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag_with_bom_ref(writer, ASSESSOR_TAG, &self.bom_ref)?;
            if let Some(third_party) = self.third_party {
                write_simple_tag(writer, THIRD_PARTY_TAG, &third_party.to_string())?;
            }
            self.organization
                .write_xml_named_element(writer, ORGANIZATION_TAG)?;
            write_close_tag(writer, ASSESSOR_TAG)
        }
    }

    impl FromXml for Assessor {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
            get_elements_lax! {
                event_reader, element_name,
                THIRD_PARTY_TAG => third_party: bool,
                ORGANIZATION_TAG => organization: OrganizationalEntity,
            };

            Ok(Self {
                bom_ref,
                third_party,
                organization,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Attestation {
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        assessor: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        map: Option<Vec<AttestationMap>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<Signature>,
    }

    impl From<models::declarations::Attestation> for Attestation {
        fn from(other: models::declarations::Attestation) -> Self {
            Self {
                summary: other.summary,
                assessor: other.assessor.map(|r| r.0),
                map: convert_optional_vec(other.map),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl From<Attestation> for models::declarations::Attestation {
        fn from(other: Attestation) -> Self {
            Self {
                summary: other.summary,
                assessor: other.assessor.map(BomReference),
                map: convert_optional_vec(other.map),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl ToXml for Attestation {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, ATTESTATION_TAG)?;
            write_simple_option_tag(writer, SUMMARY_TAG, &self.summary)?;
            write_simple_option_tag(writer, ASSESSOR_TAG, &self.assessor)?;
            for map in self.map.iter().flatten() {
                map.write_xml_element(writer)?;
            }
            self.signature.write_xml_element(writer)?;
            write_close_tag(writer, ATTESTATION_TAG)
        }
    }

    impl FromXml for Attestation {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let mut summary: Option<String> = None;
            let mut assessor: Option<String> = None;
            let mut map: Option<Vec<AttestationMap>> = None;
            let mut signature: Option<Signature> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(ATTESTATION_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == SUMMARY_TAG =>
                    {
                        summary = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == ASSESSOR_TAG =>
                    {
                        assessor = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == MAP_TAG => {
                        map.get_or_insert_with(Vec::new)
                            .push(AttestationMap::read_xml_element(
                                event_reader,
                                &name,
                                &attributes,
                            )?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == SIGNATURE_TAG => {
                        signature = Some(Signature::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    // lax validation of any elements from a different schema
                    reader::XmlEvent::StartElement { name, .. } => {
                        read_lax_validation_tag(event_reader, &name)?
                    }
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            Ok(Self {
                summary,
                assessor,
                map,
                signature,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct AttestationMap {
        #[serde(skip_serializing_if = "Option::is_none")]
        requirement: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        claims: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        counter_claims: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        conformance: Option<Conformance>,
        #[serde(skip_serializing_if = "Option::is_none")]
        confidence: Option<Confidence>,
    }

    impl From<models::declarations::AttestationMap> for AttestationMap {
        fn from(other: models::declarations::AttestationMap) -> Self {
            Self {
                requirement: other.requirement.map(|r| r.0),
                claims: from_bom_refs(other.claims),
                counter_claims: from_bom_refs(other.counter_claims),
                conformance: convert_optional(other.conformance),
                confidence: convert_optional(other.confidence),
            }
        }
    }

    impl From<AttestationMap> for models::declarations::AttestationMap {
        fn from(other: AttestationMap) -> Self {
            Self {
                requirement: other.requirement.map(BomReference),
                claims: into_bom_refs(other.claims),
                counter_claims: into_bom_refs(other.counter_claims),
                conformance: convert_optional(other.conformance),
                confidence: convert_optional(other.confidence),
            }
        }
    }

    impl ToXml for AttestationMap {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, MAP_TAG)?;
            write_simple_option_tag(writer, REQUIREMENT_TAG, &self.requirement)?;
            write_string_list_option_tag(writer, CLAIMS_TAG, CLAIM_TAG, &self.claims)?;
            write_string_list_option_tag(
                writer,
                COUNTER_CLAIMS_TAG,
                COUNTER_CLAIM_TAG,
                &self.counter_claims,
            )?;
            self.conformance.write_xml_element(writer)?;
            self.confidence.write_xml_element(writer)?;
            write_close_tag(writer, MAP_TAG)
        }
    }

    impl FromXml for AttestationMap {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            get_elements_lax! {
                event_reader, element_name,
                REQUIREMENT_TAG => requirement: String,
                CLAIMS_TAG => claims: VecXmlReader<String, ClaimTag>,
                COUNTER_CLAIMS_TAG => counter_claims: VecXmlReader<String, CounterClaimTag>,
                CONFORMANCE_TAG => conformance: Conformance,
                CONFIDENCE_TAG => confidence: Confidence,
            };

            Ok(Self {
                requirement,
                claims: claims.map(Vec::from),
                counter_claims: counter_claims.map(Vec::from),
                conformance,
                confidence,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Conformance {
        #[serde(skip_serializing_if = "Option::is_none")]
        score: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rationale: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mitigation_strategies: Option<Vec<String>>,
    }

    impl From<models::declarations::Conformance> for Conformance {
        fn from(other: models::declarations::Conformance) -> Self {
            Self {
                score: other.score.map(|s| s.get()),
                rationale: other.rationale,
                mitigation_strategies: from_bom_refs(other.mitigation_strategies),
            }
        }
    }

    impl From<Conformance> for models::declarations::Conformance {
        fn from(other: Conformance) -> Self {
            Self {
                score: other.score.map(ConfidenceScore::new),
                rationale: other.rationale,
                mitigation_strategies: into_bom_refs(other.mitigation_strategies),
            }
        }
    }

    impl ToXml for Conformance {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, CONFORMANCE_TAG)?;
            if let Some(score) = self.score {
                write_simple_tag(writer, SCORE_TAG, &score.to_string())?;
            }
            write_simple_option_tag(writer, RATIONALE_TAG, &self.rationale)?;
            write_string_list_option_tag(
                writer,
                MITIGATION_STRATEGIES_TAG,
                MITIGATION_STRATEGY_TAG,
                &self.mitigation_strategies,
            )?;
            write_close_tag(writer, CONFORMANCE_TAG)
        }
    }

    impl FromXml for Conformance {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            get_elements_lax! {
                event_reader, element_name,
                SCORE_TAG => score: f32,
                RATIONALE_TAG => rationale: String,
                MITIGATION_STRATEGIES_TAG => mitigation_strategies: VecXmlReader<String, MitigationStrategyTag>,
            };

            Ok(Self {
                score,
                rationale,
                mitigation_strategies: mitigation_strategies.map(Vec::from),
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Confidence {
        #[serde(skip_serializing_if = "Option::is_none")]
        score: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rationale: Option<String>,
    }

    impl From<models::declarations::Confidence> for Confidence {
        fn from(other: models::declarations::Confidence) -> Self {
            Self {
                score: other.score.map(|s| s.get()),
                rationale: other.rationale,
            }
        }
    }

    impl From<Confidence> for models::declarations::Confidence {
        fn from(other: Confidence) -> Self {
            Self {
                score: other.score.map(ConfidenceScore::new),
                rationale: other.rationale,
            }
        }
    }

    impl ToXml for Confidence {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, CONFIDENCE_TAG)?;
            if let Some(score) = self.score {
                write_simple_tag(writer, SCORE_TAG, &score.to_string())?;
            }
            write_simple_option_tag(writer, RATIONALE_TAG, &self.rationale)?;
            write_close_tag(writer, CONFIDENCE_TAG)
        }
    }

    impl FromXml for Confidence {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            get_elements_lax! {
                event_reader, element_name,
                SCORE_TAG => score: f32,
                RATIONALE_TAG => rationale: String,
            };

            Ok(Self { score, rationale })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Claim {
        #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
        bom_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        predicate: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mitigation_strategies: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reasoning: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        counter_evidence: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        external_references: Option<ExternalReferences>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<Signature>,
    }

    impl From<models::declarations::Claim> for Claim {
        fn from(other: models::declarations::Claim) -> Self {
            Self {
                bom_ref: other.bom_ref.map(|r| r.0),
                target: other.target.map(|r| r.0),
                predicate: other.predicate,
                mitigation_strategies: from_bom_refs(other.mitigation_strategies),
                reasoning: other.reasoning,
                evidence: from_bom_refs(other.evidence),
                counter_evidence: from_bom_refs(other.counter_evidence),
                external_references: convert_optional(other.external_references),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl From<Claim> for models::declarations::Claim {
        fn from(other: Claim) -> Self {
            Self {
                bom_ref: other.bom_ref.map(BomReference),
                target: other.target.map(BomReference),
                predicate: other.predicate,
                mitigation_strategies: into_bom_refs(other.mitigation_strategies),
                reasoning: other.reasoning,
                evidence: into_bom_refs(other.evidence),
                counter_evidence: into_bom_refs(other.counter_evidence),
                external_references: convert_optional(other.external_references),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl ToXml for Claim {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag_with_bom_ref(writer, CLAIM_TAG, &self.bom_ref)?;
            write_simple_option_tag(writer, TARGET_TAG, &self.target)?;
            write_simple_option_tag(writer, PREDICATE_TAG, &self.predicate)?;
            write_string_list_option_tag(
                writer,
                MITIGATION_STRATEGIES_TAG,
                MITIGATION_STRATEGY_TAG,
                &self.mitigation_strategies,
            )?;
            write_simple_option_tag(writer, REASONING_TAG, &self.reasoning)?;
            write_repeated_string_tag(writer, EVIDENCE_TAG, &self.evidence)?;
            write_repeated_string_tag(writer, COUNTER_EVIDENCE_TAG, &self.counter_evidence)?;
            self.external_references.write_xml_element(writer)?;
            self.signature.write_xml_element(writer)?;
            write_close_tag(writer, CLAIM_TAG)
        }
    }

    impl FromXml for Claim {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
            let mut target: Option<String> = None;
            let mut predicate: Option<String> = None;
            let mut mitigation_strategies: Option<Vec<String>> = None;
            let mut reasoning: Option<String> = None;
            let mut evidence: Option<Vec<String>> = None;
            let mut counter_evidence: Option<Vec<String>> = None;
            let mut external_references: Option<ExternalReferences> = None;
            let mut signature: Option<Signature> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader.next().map_err(to_xml_read_error(CLAIM_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == TARGET_TAG =>
                    {
                        target = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == PREDICATE_TAG =>
                    {
                        predicate = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == MITIGATION_STRATEGIES_TAG =>
                    {
                        mitigation_strategies = Some(crate::xml::read_list_tag(
                            event_reader,
                            &name,
                            MITIGATION_STRATEGY_TAG,
                        )?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == REASONING_TAG =>
                    {
                        reasoning = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == EVIDENCE_TAG =>
                    {
                        evidence
                            .get_or_insert_with(Vec::new)
                            .push(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == COUNTER_EVIDENCE_TAG =>
                    {
                        counter_evidence
                            .get_or_insert_with(Vec::new)
                            .push(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == EXTERNAL_REFERENCES_TAG => {
                        external_references = Some(ExternalReferences::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == SIGNATURE_TAG => {
                        signature = Some(Signature::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    // lax validation of any elements from a different schema
                    reader::XmlEvent::StartElement { name, .. } => {
                        read_lax_validation_tag(event_reader, &name)?
                    }
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            Ok(Self {
                bom_ref,
                target,
                predicate,
                mitigation_strategies,
                reasoning,
                evidence,
                counter_evidence,
                external_references,
                signature,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct DeclarationEvidence {
        #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
        bom_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        property_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<Vec<EvidenceData>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        created: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expires: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        author: Option<OrganizationalContact>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reviewer: Option<OrganizationalContact>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<Signature>,
    }

    impl From<models::declarations::DeclarationEvidence> for DeclarationEvidence {
        fn from(other: models::declarations::DeclarationEvidence) -> Self {
            Self {
                bom_ref: other.bom_ref.map(|r| r.0),
                property_name: other.property_name,
                description: other.description,
                data: convert_optional_vec(other.data),
                created: other.created.map(|d| d.0),
                expires: other.expires.map(|d| d.0),
                author: convert_optional(other.author),
                reviewer: convert_optional(other.reviewer),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl From<DeclarationEvidence> for models::declarations::DeclarationEvidence {
        fn from(other: DeclarationEvidence) -> Self {
            Self {
                bom_ref: other.bom_ref.map(BomReference),
                property_name: other.property_name,
                description: other.description,
                data: convert_optional_vec(other.data),
                created: other.created.map(DateTime),
                expires: other.expires.map(DateTime),
                author: convert_optional(other.author),
                reviewer: convert_optional(other.reviewer),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl ToXml for DeclarationEvidence {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag_with_bom_ref(writer, EVIDENCE_TAG, &self.bom_ref)?;
            write_simple_option_tag(writer, PROPERTY_NAME_TAG, &self.property_name)?;
            write_simple_option_tag(writer, DESCRIPTION_TAG, &self.description)?;
            for data in self.data.iter().flatten() {
                data.write_xml_element(writer)?;
            }
            write_simple_option_tag(writer, CREATED_TAG, &self.created)?;
            write_simple_option_tag(writer, EXPIRES_TAG, &self.expires)?;
            self.author.write_xml_named_element(writer, AUTHOR_TAG)?;
            self.reviewer
                .write_xml_named_element(writer, REVIEWER_TAG)?;
            self.signature.write_xml_element(writer)?;
            write_close_tag(writer, EVIDENCE_TAG)
        }
    }

    impl FromXml for DeclarationEvidence {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
            let mut property_name: Option<String> = None;
            let mut description: Option<String> = None;
            let mut data: Option<Vec<EvidenceData>> = None;
            let mut created: Option<String> = None;
            let mut expires: Option<String> = None;
            let mut author: Option<OrganizationalContact> = None;
            let mut reviewer: Option<OrganizationalContact> = None;
            let mut signature: Option<Signature> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(EVIDENCE_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == PROPERTY_NAME_TAG =>
                    {
                        property_name = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == DESCRIPTION_TAG =>
                    {
                        description = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == DATA_TAG => {
                        data.get_or_insert_with(Vec::new)
                            .push(EvidenceData::read_xml_element(
                                event_reader,
                                &name,
                                &attributes,
                            )?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == CREATED_TAG =>
                    {
                        created = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == EXPIRES_TAG =>
                    {
                        expires = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == AUTHOR_TAG => {
                        author = Some(OrganizationalContact::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == REVIEWER_TAG => {
                        reviewer = Some(OrganizationalContact::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == SIGNATURE_TAG => {
                        signature = Some(Signature::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    // lax validation of any elements from a different schema
                    reader::XmlEvent::StartElement { name, .. } => {
                        read_lax_validation_tag(event_reader, &name)?
                    }
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            Ok(Self {
                bom_ref,
                property_name,
                description,
                data,
                created,
                expires,
                author,
                reviewer,
                signature,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct EvidenceData {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        contents: Option<EvidenceDataContents>,
        #[serde(skip_serializing_if = "Option::is_none")]
        classification: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sensitive_data: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        governance: Option<DataGovernance>,
    }

    impl From<models::declarations::EvidenceData> for EvidenceData {
        fn from(other: models::declarations::EvidenceData) -> Self {
            Self {
                name: other.name,
                contents: convert_optional(other.contents),
                classification: other.classification,
                sensitive_data: other.sensitive_data,
                governance: convert_optional(other.governance),
            }
        }
    }

    impl From<EvidenceData> for models::declarations::EvidenceData {
        fn from(other: EvidenceData) -> Self {
            Self {
                name: other.name,
                contents: convert_optional(other.contents),
                classification: other.classification,
                sensitive_data: other.sensitive_data,
                governance: convert_optional(other.governance),
            }
        }
    }

    impl ToXml for EvidenceData {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, DATA_TAG)?;
            write_simple_option_tag(writer, NAME_TAG, &self.name)?;
            self.contents.write_xml_element(writer)?;
            write_simple_option_tag(writer, CLASSIFICATION_TAG, &self.classification)?;
            write_repeated_string_tag(writer, SENSITIVE_DATA_TAG, &self.sensitive_data)?;
            self.governance
                .write_xml_named_element(writer, GOVERNANCE_TAG)?;
            write_close_tag(writer, DATA_TAG)
        }
    }

    impl FromXml for EvidenceData {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let mut data_name: Option<String> = None;
            let mut contents: Option<EvidenceDataContents> = None;
            let mut classification: Option<String> = None;
            let mut sensitive_data: Option<Vec<String>> = None;
            let mut governance: Option<DataGovernance> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader.next().map_err(to_xml_read_error(DATA_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                        data_name = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == CONTENTS_TAG => {
                        contents = Some(EvidenceDataContents::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == CLASSIFICATION_TAG =>
                    {
                        classification = Some(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == SENSITIVE_DATA_TAG =>
                    {
                        sensitive_data
                            .get_or_insert_with(Vec::new)
                            .push(read_simple_tag(event_reader, &name)?);
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == GOVERNANCE_TAG => {
                        governance = Some(DataGovernance::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    // lax validation of any elements from a different schema
                    reader::XmlEvent::StartElement { name, .. } => {
                        read_lax_validation_tag(event_reader, &name)?
                    }
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            Ok(Self {
                name: data_name,
                contents,
                classification,
                sensitive_data,
                governance,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct EvidenceDataContents {
        #[serde(skip_serializing_if = "Option::is_none")]
        attachment: Option<Attachment>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    }

    impl From<models::declarations::EvidenceDataContents> for EvidenceDataContents {
        fn from(other: models::declarations::EvidenceDataContents) -> Self {
            Self {
                attachment: convert_optional(other.attachment),
                url: other.url.map(|u| u.0),
            }
        }
    }

    impl From<EvidenceDataContents> for models::declarations::EvidenceDataContents {
        fn from(other: EvidenceDataContents) -> Self {
            Self {
                attachment: convert_optional(other.attachment),
                url: other.url.map(Uri),
            }
        }
    }

    impl ToXml for EvidenceDataContents {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, CONTENTS_TAG)?;
            self.attachment
                .write_xml_named_element(writer, ATTACHMENT_TAG)?;
            write_simple_option_tag(writer, URL_TAG, &self.url)?;
            write_close_tag(writer, CONTENTS_TAG)
        }
    }

    impl FromXml for EvidenceDataContents {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            get_elements_lax! {
                event_reader, element_name,
                ATTACHMENT_TAG => attachment: Attachment,
                URL_TAG => url: String,
            };

            Ok(Self { attachment, url })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Targets {
        #[serde(skip_serializing_if = "Option::is_none")]
        organizations: Option<Vec<OrganizationalEntity>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        components: Option<Components>,
        #[serde(skip_serializing_if = "Option::is_none")]
        services: Option<Services>,
    }

    impl TryFrom<models::declarations::Targets> for Targets {
        type Error = BomError;

        fn try_from(other: models::declarations::Targets) -> Result<Self, Self::Error> {
            Ok(Self {
                organizations: convert_optional_vec(other.organizations),
                components: try_convert_optional(other.components)?,
                services: try_convert_optional(other.services)?,
            })
        }
    }

    impl From<Targets> for models::declarations::Targets {
        fn from(other: Targets) -> Self {
            Self {
                organizations: convert_optional_vec(other.organizations),
                components: convert_optional(other.components),
                services: convert_optional(other.services),
            }
        }
    }

    impl ToXml for Targets {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, TARGETS_TAG)?;
            if let Some(organizations) = &self.organizations {
                write_start_tag(writer, ORGANIZATIONS_TAG)?;
                for organization in organizations {
                    organization.write_xml_named_element(writer, ORGANIZATION_TAG)?;
                }
                write_close_tag(writer, ORGANIZATIONS_TAG)?;
            }
            self.components.write_xml_element(writer)?;
            self.services.write_xml_element(writer)?;
            write_close_tag(writer, TARGETS_TAG)
        }
    }

    impl FromXml for Targets {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            get_elements_lax! {
                event_reader, element_name,
                ORGANIZATIONS_TAG => organizations: VecXmlReader<OrganizationalEntity, OrganizationTag>,
                COMPONENTS_TAG => components: Components,
                SERVICES_TAG => services: Services,
            };

            Ok(Self {
                organizations: organizations.map(Vec::from),
                components,
                services,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Affirmation {
        #[serde(skip_serializing_if = "Option::is_none")]
        statement: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signatories: Option<Vec<Signatory>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<Signature>,
    }

    impl From<models::declarations::Affirmation> for Affirmation {
        fn from(other: models::declarations::Affirmation) -> Self {
            Self {
                statement: other.statement,
                signatories: convert_optional_vec(other.signatories),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl From<Affirmation> for models::declarations::Affirmation {
        fn from(other: Affirmation) -> Self {
            Self {
                statement: other.statement,
                signatories: convert_optional_vec(other.signatories),
                signature: convert_optional(other.signature),
            }
        }
    }

    impl ToXml for Affirmation {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, AFFIRMATION_TAG)?;
            write_simple_option_tag(writer, STATEMENT_TAG, &self.statement)?;
            write_list_option_tag(writer, SIGNATORIES_TAG, &self.signatories)?;
            self.signature.write_xml_element(writer)?;
            write_close_tag(writer, AFFIRMATION_TAG)
        }
    }

    impl FromXml for Affirmation {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            get_elements_lax! {
                event_reader, element_name,
                STATEMENT_TAG => statement: String,
                SIGNATORIES_TAG => signatories: VecXmlReader<Signatory, SignatoryTag>,
                SIGNATURE_TAG => signature: Signature,
            };

            Ok(Self {
                statement,
                signatories: signatories.map(Vec::from),
                signature,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Signatory {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<Signature>,
        #[serde(skip_serializing_if = "Option::is_none")]
        organization: Option<OrganizationalEntity>,
        #[serde(skip_serializing_if = "Option::is_none")]
        external_reference: Option<ExternalReference>,
    }

    impl From<models::declarations::Signatory> for Signatory {
        fn from(other: models::declarations::Signatory) -> Self {
            Self {
                name: other.name,
                role: other.role,
                signature: convert_optional(other.signature),
                organization: convert_optional(other.organization),
                external_reference: convert_optional(other.external_reference),
            }
        }
    }

    impl From<Signatory> for models::declarations::Signatory {
        fn from(other: Signatory) -> Self {
            Self {
                name: other.name,
                role: other.role,
                signature: convert_optional(other.signature),
                organization: convert_optional(other.organization),
                external_reference: convert_optional(other.external_reference),
            }
        }
    }

    impl ToXml for Signatory {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, SIGNATORY_TAG)?;
            write_simple_option_tag(writer, NAME_TAG, &self.name)?;
            write_simple_option_tag(writer, ROLE_TAG, &self.role)?;
            self.organization
                .write_xml_named_element(writer, ORGANIZATION_TAG)?;
            self.external_reference
                .write_xml_named_element(writer, EXTERNAL_REFERENCE_TAG)?;
            // The XSD has no signature element; it sits in the trailing `xs:any` slot.
            self.signature.write_xml_element(writer)?;
            write_close_tag(writer, SIGNATORY_TAG)
        }
    }

    impl FromXml for Signatory {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            get_elements_lax! {
                event_reader, element_name,
                NAME_TAG => name: String,
                ROLE_TAG => role: String,
                ORGANIZATION_TAG => organization: OrganizationalEntity,
                EXTERNAL_REFERENCE_TAG => external_reference: ExternalReference,
                SIGNATURE_TAG => signature: Signature,
            };

            Ok(Self {
                name,
                role,
                signature,
                organization,
                external_reference,
            })
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;
        use crate::{
            external_models::normalized_string::NormalizedString,
            models::external_reference::{ExternalReferenceType, Uri as ExternalReferenceUri},
            specs::common::signature::test::{corresponding_signature, example_signature},
            xml::test::{read_element_from_string, write_element_to_string},
        };
        use pretty_assertions::assert_eq;

        pub(crate) fn example_declarations() -> Declarations {
            let signatory = Signatory {
                name: Some("Jerry".to_string()),
                role: Some("COO".to_string()),
                signature: None,
                organization: Some(OrganizationalEntity::new("Acme Inc")),
                external_reference: Some(
                    serde_json::from_str(
                        r#"{"type": "electronic-signature", "url": "https://example.com/sig.png"}"#,
                    )
                    .unwrap(),
                ),
            };
            Declarations {
                assessors: Some(vec![Assessor {
                    bom_ref: Some("assessor-1".to_string()),
                    third_party: Some(true),
                    organization: Some(OrganizationalEntity::new("Assessors Inc")),
                }]),
                attestations: Some(vec![Attestation {
                    summary: Some("summary".to_string()),
                    assessor: Some("assessor-1".to_string()),
                    map: Some(vec![AttestationMap {
                        requirement: Some("requirement-1".to_string()),
                        claims: Some(vec!["claim-1".to_string()]),
                        counter_claims: Some(vec!["counter-claim-1".to_string()]),
                        conformance: Some(Conformance {
                            score: Some(0.8),
                            rationale: Some("conformance rationale".to_string()),
                            mitigation_strategies: Some(vec!["evidence-1".to_string()]),
                        }),
                        confidence: Some(Confidence {
                            score: Some(1.0),
                            rationale: Some("confidence rationale".to_string()),
                        }),
                    }]),
                    signature: Some(example_signature()),
                }]),
                claims: Some(vec![Claim {
                    bom_ref: Some("claim-1".to_string()),
                    target: Some("acme-inc".to_string()),
                    predicate: Some("predicate".to_string()),
                    mitigation_strategies: Some(vec!["evidence-1".to_string()]),
                    reasoning: Some("reasoning".to_string()),
                    evidence: Some(vec!["evidence-1".to_string(), "evidence-2".to_string()]),
                    counter_evidence: Some(vec!["evidence-3".to_string()]),
                    external_references: None,
                    signature: None,
                }]),
                evidence: Some(vec![DeclarationEvidence {
                    bom_ref: Some("evidence-1".to_string()),
                    property_name: Some("internal.com.acme.someProperty".to_string()),
                    description: Some("description".to_string()),
                    data: Some(vec![EvidenceData {
                        name: Some("data name".to_string()),
                        contents: Some(EvidenceDataContents {
                            attachment: Some(Attachment {
                                content: "evidence".to_string(),
                                content_type: Some("text/plain".to_string()),
                                encoding: None,
                            }),
                            url: Some("https://example.com/evidence".to_string()),
                        }),
                        classification: Some("PII".to_string()),
                        sensitive_data: Some(vec!["first".to_string(), "second".to_string()]),
                        governance: None,
                    }]),
                    created: Some("2023-04-25T00:00:00+00:00".to_string()),
                    expires: Some("2023-05-25T00:00:00+00:00".to_string()),
                    author: Some(OrganizationalContact {
                        bom_ref: None,
                        name: Some("Mary".to_string()),
                        email: None,
                        phone: None,
                    }),
                    reviewer: None,
                    signature: None,
                }]),
                targets: Some(Targets {
                    organizations: Some(vec![OrganizationalEntity {
                        bom_ref: Some("acme-inc".to_string()),
                        ..OrganizationalEntity::new("Acme Inc")
                    }]),
                    components: None,
                    services: None,
                }),
                affirmation: Some(Affirmation {
                    statement: Some("statement".to_string()),
                    signatories: Some(vec![signatory]),
                    signature: None,
                }),
                signature: Some(example_signature()),
            }
        }

        pub(crate) fn corresponding_declarations() -> models::declarations::Declarations {
            use models::declarations::*;

            let entity =
                |bom_ref: Option<&str>, name: &str| models::organization::OrganizationalEntity {
                    bom_ref: bom_ref.map(BomReference::new),
                    name: Some(NormalizedString::new_unchecked(name.to_string())),
                    url: None,
                    contact: None,
                };
            let refs = |refs: &[&str]| Some(refs.iter().map(BomReference::new).collect());

            Declarations {
                assessors: Some(vec![Assessor {
                    bom_ref: Some(BomReference::new("assessor-1")),
                    third_party: Some(true),
                    organization: Some(entity(None, "Assessors Inc")),
                }]),
                attestations: Some(vec![Attestation {
                    summary: Some("summary".to_string()),
                    assessor: Some(BomReference::new("assessor-1")),
                    map: Some(vec![AttestationMap {
                        requirement: Some(BomReference::new("requirement-1")),
                        claims: refs(&["claim-1"]),
                        counter_claims: refs(&["counter-claim-1"]),
                        conformance: Some(Conformance {
                            score: Some(ConfidenceScore::new(0.8)),
                            rationale: Some("conformance rationale".to_string()),
                            mitigation_strategies: refs(&["evidence-1"]),
                        }),
                        confidence: Some(Confidence {
                            score: Some(ConfidenceScore::new(1.0)),
                            rationale: Some("confidence rationale".to_string()),
                        }),
                    }]),
                    signature: Some(corresponding_signature()),
                }]),
                claims: Some(vec![Claim {
                    bom_ref: Some(BomReference::new("claim-1")),
                    target: Some(BomReference::new("acme-inc")),
                    predicate: Some("predicate".to_string()),
                    mitigation_strategies: refs(&["evidence-1"]),
                    reasoning: Some("reasoning".to_string()),
                    evidence: refs(&["evidence-1", "evidence-2"]),
                    counter_evidence: refs(&["evidence-3"]),
                    external_references: None,
                    signature: None,
                }]),
                evidence: Some(vec![DeclarationEvidence {
                    bom_ref: Some(BomReference::new("evidence-1")),
                    property_name: Some("internal.com.acme.someProperty".to_string()),
                    description: Some("description".to_string()),
                    data: Some(vec![EvidenceData {
                        name: Some("data name".to_string()),
                        contents: Some(EvidenceDataContents {
                            attachment: Some(models::attachment::Attachment {
                                content: "evidence".to_string(),
                                content_type: Some("text/plain".to_string()),
                                encoding: None,
                            }),
                            url: Some(Uri("https://example.com/evidence".to_string())),
                        }),
                        classification: Some("PII".to_string()),
                        sensitive_data: Some(vec!["first".to_string(), "second".to_string()]),
                        governance: None,
                    }]),
                    created: Some(DateTime("2023-04-25T00:00:00+00:00".to_string())),
                    expires: Some(DateTime("2023-05-25T00:00:00+00:00".to_string())),
                    author: Some(models::organization::OrganizationalContact {
                        bom_ref: None,
                        name: Some(NormalizedString::new_unchecked("Mary".to_string())),
                        email: None,
                        phone: None,
                    }),
                    reviewer: None,
                    signature: None,
                }]),
                targets: Some(Targets {
                    organizations: Some(vec![entity(Some("acme-inc"), "Acme Inc")]),
                    components: None,
                    services: None,
                }),
                affirmation: Some(Affirmation {
                    statement: Some("statement".to_string()),
                    signatories: Some(vec![Signatory {
                        name: Some("Jerry".to_string()),
                        role: Some("COO".to_string()),
                        signature: None,
                        organization: Some(entity(None, "Acme Inc")),
                        external_reference: Some(models::external_reference::ExternalReference {
                            properties: None,
                            external_reference_type: ExternalReferenceType::ElectronicSignature,
                            url: ExternalReferenceUri::Url(Uri(
                                "https://example.com/sig.png".to_string()
                            )),
                            comment: None,
                            hashes: None,
                        }),
                    }]),
                    signature: None,
                }),
                signature: Some(corresponding_signature()),
            }
        }

        #[test]
        fn it_should_convert_to_and_from_the_model() {
            let model: models::declarations::Declarations = example_declarations().into();
            assert_eq!(model, corresponding_declarations());

            let spec: Declarations = corresponding_declarations()
                .try_into()
                .expect("Failed to convert declarations");
            assert_eq!(spec, example_declarations());
        }

        #[test]
        fn it_should_write_xml_full() {
            let xml_output = write_element_to_string(example_declarations());
            insta::assert_snapshot!(xml_output);
        }

        #[test]
        fn it_should_read_xml_full() {
            let input = r#"
<declarations>
  <assessors>
    <assessor bom-ref="assessor-1">
      <thirdParty>true</thirdParty>
      <organization>
        <name>Assessors Inc</name>
      </organization>
    </assessor>
  </assessors>
  <attestations>
    <attestation>
      <summary>summary</summary>
      <assessor>assessor-1</assessor>
      <map>
        <requirement>requirement-1</requirement>
        <claims>
          <claim>claim-1</claim>
        </claims>
        <counterClaims>
          <counterClaim>counter-claim-1</counterClaim>
        </counterClaims>
        <conformance>
          <score>0.8</score>
          <rationale>conformance rationale</rationale>
          <mitigationStrategies>
            <mitigationStrategy>evidence-1</mitigationStrategy>
          </mitigationStrategies>
        </conformance>
        <confidence>
          <score>1</score>
          <rationale>confidence rationale</rationale>
        </confidence>
      </map>
      <signature>
        <algorithm>HS512</algorithm>
        <value>1234567890</value>
      </signature>
    </attestation>
  </attestations>
  <claims>
    <claim bom-ref="claim-1">
      <target>acme-inc</target>
      <predicate>predicate</predicate>
      <mitigationStrategies>
        <mitigationStrategy>evidence-1</mitigationStrategy>
      </mitigationStrategies>
      <reasoning>reasoning</reasoning>
      <evidence>evidence-1</evidence>
      <evidence>evidence-2</evidence>
      <counterEvidence>evidence-3</counterEvidence>
      <ds:Signature xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
        <!-- XML signature here -->
      </ds:Signature>
    </claim>
  </claims>
  <evidence>
    <evidence bom-ref="evidence-1">
      <propertyName>internal.com.acme.someProperty</propertyName>
      <description>description</description>
      <data>
        <name>data name</name>
        <contents>
          <attachment content-type="text/plain">evidence</attachment>
          <url>https://example.com/evidence</url>
        </contents>
        <classification>PII</classification>
        <sensitiveData>first</sensitiveData>
        <sensitiveData>second</sensitiveData>
      </data>
      <created>2023-04-25T00:00:00+00:00</created>
      <expires>2023-05-25T00:00:00+00:00</expires>
      <author>
        <name>Mary</name>
      </author>
    </evidence>
  </evidence>
  <targets>
    <organizations>
      <organization bom-ref="acme-inc">
        <name>Acme Inc</name>
      </organization>
    </organizations>
  </targets>
  <affirmation>
    <statement>statement</statement>
    <signatories>
      <signatory>
        <name>Jerry</name>
        <role>COO</role>
        <organization>
          <name>Acme Inc</name>
        </organization>
        <externalReference type="electronic-signature">
          <url>https://example.com/sig.png</url>
        </externalReference>
      </signatory>
    </signatories>
  </affirmation>
  <signature>
    <algorithm>HS512</algorithm>
    <value>1234567890</value>
  </signature>
</declarations>
"#;
            let actual: Declarations = read_element_from_string(input);
            assert_eq!(actual, example_declarations());
        }

        #[test]
        fn it_should_round_trip_targets_with_components_and_services_through_xml() {
            let json = r#"{
                "organizations": [{ "name": "Acme Inc" }],
                "components": [{ "type": "library", "name": "acme-lib", "bom-ref": "lib-1" }],
                "services": [{ "name": "acme-service", "bom-ref": "service-1" }]
            }"#;
            let parse =
                || -> Targets { serde_json::from_str(json).expect("Failed to parse targets") };

            let xml_output = write_element_to_string(parse());
            let actual: Targets = read_element_from_string(xml_output);

            assert_eq!(actual, parse());
        }
    }
}
