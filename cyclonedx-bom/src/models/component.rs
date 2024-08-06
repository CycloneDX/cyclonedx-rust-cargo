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
use ordered_float::OrderedFloat;
use regex::Regex;
use std::fmt::Formatter;

use crate::external_models::normalized_string::validate_normalized_string;
use crate::external_models::uri::{validate_purl, validate_uri as validate_url};
use crate::models::attached_text::AttachedText;
use crate::models::bom::BomReference;
use crate::models::code::{Commits, Patches};
use crate::models::external_reference::ExternalReferences;
use crate::models::hash::Hashes;
use crate::models::license::Licenses;
use crate::models::organization::OrganizationalEntity;
use crate::models::property::Properties;
use crate::validation::ValidationError;
use crate::{
    external_models::{
        normalized_string::NormalizedString,
        uri::{Purl, Uri as Url},
    },
    validation::{Validate, ValidationContext, ValidationResult},
};

use super::bom::{validate_bom_ref, SpecVersion};
use super::component_data::ComponentData;
use super::modelcard::ModelCard;
use super::signature::Signature;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Component {
    pub component_type: Classification,
    pub mime_type: Option<MimeType>,
    pub bom_ref: Option<String>,
    pub supplier: Option<OrganizationalEntity>,
    pub author: Option<NormalizedString>,
    pub publisher: Option<NormalizedString>,
    pub group: Option<NormalizedString>,
    pub name: NormalizedString,
    pub version: Option<NormalizedString>,
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
    /// Added in version 1.4
    pub signature: Option<Signature>,
    /// Added in version 1.5
    pub model_card: Option<ModelCard>,
    /// Added in version 1.5
    pub data: Option<ComponentData>,
}

impl Component {
    pub fn new(
        component_type: Classification,
        name: &str,
        version: &str,
        bom_ref: Option<String>,
    ) -> Self {
        Self {
            component_type,
            name: NormalizedString::new(name),
            version: Some(NormalizedString::new(version)),
            bom_ref,
            mime_type: None,
            supplier: None,
            author: None,
            publisher: None,
            group: None,
            description: None,
            scope: None,
            hashes: None,
            licenses: None,
            copyright: None,
            cpe: None,
            purl: None,
            swid: None,
            modified: None,
            pedigree: None,
            external_references: None,
            properties: None,
            components: None,
            evidence: None,
            signature: None,
            model_card: None,
            data: None,
        }
    }
}

impl Validate for Component {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut ctx = ValidationContext::new();
        ctx.add_field("component_type", &self.component_type, |ct| {
            validate_classification(ct, version)
        });
        ctx.add_field_option("mime_type", self.mime_type.as_ref(), validate_mime_type);
        ctx.add_struct_option("supplier", self.supplier.as_ref(), version);
        ctx.add_field_option("author", self.author.as_ref(), validate_normalized_string);
        ctx.add_field_option(
            "publisher",
            self.publisher.as_ref(),
            validate_normalized_string,
        );
        ctx.add_field_option("group", self.group.as_ref(), validate_normalized_string);
        ctx.add_field("name", &self.name, validate_normalized_string);
        ctx.add_field_option("version", self.version.as_ref(), validate_normalized_string);
        ctx.add_field_option(
            "description",
            self.description.as_ref(),
            validate_normalized_string,
        );
        ctx.add_enum_option("scope", self.scope.as_ref(), validate_scope);
        ctx.add_struct_option("hashes", self.hashes.as_ref(), version);
        ctx.add_struct_option("licenses", self.licenses.as_ref(), version);
        ctx.add_field_option(
            "copyright",
            self.copyright.as_ref(),
            validate_normalized_string,
        );
        ctx.add_field_option("cpe", self.cpe.as_ref(), validate_cpe);
        ctx.add_field_option("purl", self.purl.as_ref(), validate_purl);
        ctx.add_struct_option("swid", self.swid.as_ref(), version);
        ctx.add_struct_option("pedigree", self.pedigree.as_ref(), version);
        ctx.add_struct_option(
            "external_references",
            self.external_references.as_ref(),
            version,
        );
        ctx.add_struct_option("properties", self.properties.as_ref(), version);
        ctx.add_struct_option("components", self.components.as_ref(), version);
        ctx.add_struct_option("evidence", self.evidence.as_ref(), version);
        ctx.into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Components(pub Vec<Component>);

impl Validate for Components {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |component| {
                component.validate_version(version)
            })
            .into()
    }
}

/// Checks the given [`Classification`] is valid.
pub fn validate_classification(
    classification: &Classification,
    version: SpecVersion,
) -> Result<(), ValidationError> {
    if SpecVersion::V1_3 <= version && version <= SpecVersion::V1_4 {
        if Classification::File < *classification {
            return Err(ValidationError::new("Unknown classification"));
        }
    } else if SpecVersion::V1_5 <= version
        && matches!(classification, Classification::UnknownClassification(_))
    {
        return Err(ValidationError::new("Unknown classification"));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, strum::Display, Hash)]
#[strum(serialize_all = "kebab-case")]
#[repr(u16)]
pub enum Classification {
    Application = 1,
    Framework = 2,
    Library = 3,
    Container = 4,
    OperatingSystem = 5,
    Device = 6,
    Firmware = 7,
    File = 8,
    /// Added in 1.5
    Platform = 9,
    /// Added in 1.5
    DeviceDriver = 10,
    /// Added in 1.5
    MachineLearningModel = 11,
    /// Added in 1.5
    Data = 12,
    #[doc(hidden)]
    #[strum(default)]
    UnknownClassification(String),
}

impl Classification {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "application" => Self::Application,
            "framework" => Self::Framework,
            "library" => Self::Library,
            "container" => Self::Container,
            "operating-system" => Self::OperatingSystem,
            "device" => Self::Device,
            "firmware" => Self::Firmware,
            "file" => Self::File,
            "platform" => Self::Platform,
            "device-driver" => Self::DeviceDriver,
            "machine-learning-model" => Self::MachineLearningModel,
            "data" => Self::Data,
            unknown => Self::UnknownClassification(unknown.to_string()),
        }
    }
}

pub fn validate_scope(scope: &Scope) -> Result<(), ValidationError> {
    if matches!(scope, Scope::UnknownScope(_)) {
        return Err(ValidationError::new("Unknown scope"));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, strum::Display, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum Scope {
    Required,
    Optional,
    Excluded,
    #[doc(hidden)]
    #[strum(default)]
    UnknownScope(String),
}

impl Scope {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "required" => Self::Required,
            "optional" => Self::Optional,
            "excluded" => Self::Excluded,
            unknown => Self::UnknownScope(unknown.to_string()),
        }
    }
}

/// Checks if given [`MimeType`] is valid / supported.
pub fn validate_mime_type(mime_type: &MimeType) -> Result<(), ValidationError> {
    static UUID_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[-+a-z0-9.]+/[-+a-z0-9.]+$").expect("Failed to compile regex."));

    if !UUID_REGEX.is_match(&mime_type.0) {
        return Err(ValidationError::new(
            "MimeType does not match regular expression",
        ));
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MimeType(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Swid {
    pub tag_id: String,
    pub name: String,
    pub version: Option<String>,
    pub tag_version: Option<u32>,
    pub patch: Option<bool>,
    pub text: Option<AttachedText>,
    pub url: Option<Url>,
}

impl Validate for Swid {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("text", self.text.as_ref(), version)
            .add_field_option("url", self.url.as_ref(), validate_url)
            .into()
    }
}

pub fn validate_cpe(cpe: &Cpe) -> Result<(), ValidationError> {
    static UUID_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r##"([c][pP][eE]:/[AHOaho]?(:[A-Za-z0-9\._\-~%]*){0,6})|(cpe:2\.3:[aho\*\-](:(((\?*|\*?)([a-zA-Z0-9\-\._]|(\\[\\\*\?!"#$$%&'\(\)\+,/:;<=>@\[\]\^`\{\|}~]))+(\?*|\*?))|[\*\-])){5}(:(([a-zA-Z]{2,3}(-([a-zA-Z]{2}|[0-9]{3}))?)|[\*\-]))(:(((\?*|\*?)([a-zA-Z0-9\-\._]|(\\[\\\*\?!"#$$%&'\(\)\+,/:;<=>@\[\]\^`\{\|}~]))+(\?*|\*?))|[\*\-])){4})"##,
        ).expect("Failed to compile regex.")
    });

    if !UUID_REGEX.is_match(&cpe.0) {
        return Err(ValidationError::new(
            "Cpe does not match regular expression",
        ));
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cpe(pub(crate) String);

impl Cpe {
    pub fn new(inner: &str) -> Self {
        Self(inner.to_string())
    }
}

impl From<String> for Cpe {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<String> for Cpe {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl AsRef<str> for Cpe {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Cpe> for String {
    fn from(value: Cpe) -> Self {
        value.0
    }
}

impl std::fmt::Display for Cpe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentEvidence {
    pub licenses: Option<Licenses>,
    pub copyright: Option<CopyrightTexts>,
    /// Added in version 1.5
    pub occurrences: Option<Occurrences>,
    /// Added in version 1.5
    pub callstack: Option<Callstack>,
    /// Added in version 1.5
    pub identity: Option<Identity>,
}

impl Validate for ComponentEvidence {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("licenses", self.licenses.as_ref(), version)
            .add_struct_option("copyright", self.copyright.as_ref(), version)
            .add_struct_option("occurrences", self.occurrences.as_ref(), version)
            .add_struct_option("callstack", self.callstack.as_ref(), version)
            .add_struct_option("identity", self.identity.as_ref(), version)
            .into()
    }
}

/// For more details see
/// https://cyclonedx.org/docs/1.5/json/#components_items_evidence_occurrences
/// Added in version 1.5
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Occurrences(pub Vec<Occurrence>);

impl Validate for Occurrences {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |occurrence| {
                occurrence.validate_version(version)
            })
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Occurrence {
    pub bom_ref: Option<BomReference>,
    pub location: String,
}

impl Occurrence {
    pub fn new(location: &str) -> Self {
        Self {
            bom_ref: None,
            location: location.to_string(),
        }
    }
}

impl Validate for Occurrence {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("bom-ref", self.bom_ref.as_ref(), |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Callstack {
    pub frames: Frames,
}

impl Callstack {
    pub fn new(frames: Frames) -> Self {
        Self { frames }
    }
}

impl Validate for Callstack {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        self.frames.validate_version(version)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Frames(pub Vec<Frame>);

impl Validate for Frames {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("frames", &self.0, |frame| frame.validate_version(version))
            .into()
    }
}

/// For more information see
/// https://cyclonedx.org/docs/1.5/json/#components_items_evidence_callstack
/// Added in version 1.5
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Frame {
    pub package: Option<NormalizedString>,
    pub module: NormalizedString,
    pub function: Option<NormalizedString>,
    pub parameters: Option<Vec<NormalizedString>>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub full_filename: Option<NormalizedString>,
}

impl Validate for Frame {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("package", self.package.as_ref(), validate_normalized_string)
            .add_field("module", self.module.as_ref(), validate_normalized_string)
            .add_field_option(
                "function",
                self.function.as_ref(),
                validate_normalized_string,
            )
            .add_list_option(
                "parameters",
                self.parameters.as_ref(),
                validate_normalized_string,
            )
            .add_field_option(
                "full_filename",
                self.full_filename.as_ref(),
                validate_normalized_string,
            )
            .into()
    }
}

pub fn validate_confidence(confidence: &ConfidenceScore) -> Result<(), ValidationError> {
    if confidence.get() < 0.0 && 1.0 > confidence.get() {
        return Err("Confidence score outside range 0.0 - 1.0".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConfidenceScore(pub OrderedFloat<f32>);

impl ConfidenceScore {
    pub fn new(value: f32) -> Self {
        Self(OrderedFloat(value))
    }

    pub fn get(&self) -> f32 {
        self.0 .0
    }
}

pub fn validate_identity_field(field: &IdentityField) -> Result<(), ValidationError> {
    if let IdentityField::Unknown(unknown) = field {
        return Err(format!("Unknown identity found '{}' given", unknown).into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, strum::Display, Hash)]
#[strum(serialize_all = "kebab-case")]
#[repr(u16)]
pub enum IdentityField {
    Group,
    Name,
    Version,
    Purl,
    Cpe,
    Swid,
    Hash,
    Unknown(String),
}

impl IdentityField {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "group" => Self::Group,
            "name" => Self::Name,
            "version" => Self::Version,
            "purl" => Self::Purl,
            "cpe" => Self::Cpe,
            "swid" => Self::Swid,
            "hash" => Self::Hash,
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

/// For more information see
/// https://cyclonedx.org/docs/1.5/json/#components_items_evidence_identity
/// Added in version 1.5
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identity {
    pub field: IdentityField,
    /// Level between 0.0-1.0 (where 1.0 is highest confidence)
    pub confidence: Option<ConfidenceScore>,
    pub methods: Option<Methods>,
    pub tools: Option<ToolsReferences>,
}

impl Validate for Identity {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("field", &self.field, validate_identity_field)
            .add_field_option("confidence", self.confidence.as_ref(), validate_confidence)
            .into()
    }
}

/// For more information see
/// https://cyclonedx.org/docs/1.5/json/#components_items_evidence_identity_methods
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Methods(pub Vec<Method>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Method {
    pub technique: String,
    pub confidence: ConfidenceScore,
    pub value: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ToolsReferences(pub Vec<String>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pedigree {
    pub ancestors: Option<Components>,
    pub descendants: Option<Components>,
    pub variants: Option<Components>,
    pub commits: Option<Commits>,
    pub patches: Option<Patches>,
    pub notes: Option<String>,
}

impl Validate for Pedigree {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();
        context.add_struct_option("ancestors", self.ancestors.as_ref(), version);
        context.add_struct_option("descendants", self.descendants.as_ref(), version);
        context.add_struct_option("variants", self.variants.as_ref(), version);
        context.add_struct_option("commits", self.commits.as_ref(), version);
        context.add_struct_option("patches", self.patches.as_ref(), version);
        context.into()
    }
}

pub fn validate_copyright(_copyright: &Copyright) -> Result<(), ValidationError> {
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Copyright(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CopyrightTexts(pub Vec<Copyright>);

impl Validate for CopyrightTexts {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, validate_copyright)
            .into()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        external_models::spdx::SpdxExpression,
        models::{
            attachment::Attachment,
            bom::BomReference,
            code::{Commit, Patch, PatchClassification},
            component_data::{
                ComponentData, ComponentDataType, DataContents, Graphic, GraphicsCollection,
            },
            data_governance::{DataGovernance, DataGovernanceResponsibleParty},
            external_reference::{ExternalReference, ExternalReferenceType, Uri},
            hash::{Hash, HashAlgorithm, HashValue},
            license::LicenseChoice,
            modelcard::{
                ApproachType, ConfidenceInterval, Considerations, Dataset, Datasets, Inputs,
                MLParameter, ModelParameters, ModelParametersApproach, Outputs, PerformanceMetric,
                PerformanceMetrics, QuantitativeAnalysis,
            },
            organization::OrganizationalContact,
            property::Property,
            signature::Algorithm,
        },
        validation,
    };

    use super::*;

    #[test]
    fn valid_components_should_pass_validation() {
        let vec = vec![Component {
            component_type: Classification::Application,
            mime_type: Some(MimeType("text/text".to_string())),
            bom_ref: Some("bom ref".to_string()),
            supplier: Some(OrganizationalEntity {
                bom_ref: Some(BomReference::new("Supplier 1")),
                name: Some(NormalizedString::new("name")),
                url: None,
                contact: None,
            }),
            author: Some(NormalizedString::new("author")),
            publisher: Some(NormalizedString::new("publisher")),
            group: Some(NormalizedString::new("group")),
            name: NormalizedString::new("name"),
            version: Some(NormalizedString::new("version")),
            description: Some(NormalizedString::new("description")),
            scope: Some(Scope::Required),
            hashes: Some(Hashes(vec![Hash {
                alg: HashAlgorithm::MD5,
                content: HashValue("a3bf1f3d584747e2569483783ddee45b".to_string()),
            }])),
            licenses: Some(Licenses(vec![LicenseChoice::Expression(
                SpdxExpression::new("MIT"),
            )])),
            copyright: Some(NormalizedString::new("copyright")),
            cpe: Some(Cpe("cpe:/a:example:mylibrary:1.0.0".to_string())),
            purl: Some(Purl("pkg:cargo/cyclonedx-bom@0.3.1".to_string())),
            swid: Some(Swid {
                tag_id: "tag ID".to_string(),
                name: "name".to_string(),
                version: Some("version".to_string()),
                tag_version: Some(1),
                patch: Some(true),
                text: Some(AttachedText {
                    content_type: None,
                    encoding: None,
                    content: "content".to_string(),
                }),
                url: Some(Url("https://example.com".to_string())),
            }),
            modified: Some(true),
            pedigree: Some(Pedigree {
                ancestors: Some(Components(vec![])),
                descendants: Some(Components(vec![])),
                variants: Some(Components(vec![])),
                commits: Some(Commits(vec![Commit {
                    uid: Some(NormalizedString::new("uid")),
                    url: None,
                    author: None,
                    committer: None,
                    message: None,
                }])),
                patches: Some(Patches(vec![Patch {
                    patch_type: PatchClassification::Backport,
                    diff: None,
                    resolves: None,
                }])),
                notes: Some("notes".to_string()),
            }),
            external_references: Some(ExternalReferences(vec![ExternalReference {
                external_reference_type: ExternalReferenceType::Bom,
                url: Uri::Url(Url("https://www.example.com".to_string())),
                comment: None,
                hashes: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString::new("value"),
            }])),
            components: Some(Components(vec![])),
            evidence: Some(ComponentEvidence {
                licenses: Some(Licenses(vec![LicenseChoice::Expression(
                    SpdxExpression::new("MIT"),
                )])),
                copyright: Some(CopyrightTexts(vec![Copyright("copyright".to_string())])),
                occurrences: Some(Occurrences(vec![Occurrence {
                    bom_ref: None,
                    location: "location".to_string(),
                }])),
                callstack: Some(Callstack::new(Frames(vec![Frame {
                    package: Some("package".into()),
                    module: "module".into(),
                    function: Some("function".into()),
                    parameters: None,
                    line: Some(10),
                    column: Some(20),
                    full_filename: Some("full_filename".into()),
                }]))),
                identity: Some(Identity {
                    field: IdentityField::Group,
                    confidence: Some(ConfidenceScore::new(0.8)),
                    methods: Some(Methods(vec![Method {
                        technique: "technique".to_string(),
                        confidence: ConfidenceScore::new(0.5),
                        value: Some("help".to_string()),
                    }])),
                    tools: None,
                }),
            }),
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
            model_card: Some(ModelCard {
                bom_ref: None,
                model_parameters: Some(ModelParameters {
                    approach: Some(ModelParametersApproach {
                        approach_type: Some(ApproachType::Supervised),
                    }),
                    task: Some("task".to_string()),
                    architecture_family: Some("architecture family".to_string()),
                    model_architecture: Some("model architecture".to_string()),
                    datasets: Some(Datasets(vec![Dataset::Component(ComponentData {
                        bom_ref: None,
                        data_type: ComponentDataType::SourceCode,
                        name: Some("dataset".to_string()),
                        contents: Some(DataContents {
                            attachment: Some(Attachment {
                                content: "data content".to_string(),
                                content_type: Some("text/plain".to_string()),
                                encoding: Some("base64".to_string()),
                            }),
                            url: Some(Url("https://example.com".to_string())),
                            properties: Some(Properties(vec![])),
                        }),
                        classification: Some("data classification".to_string()),
                        sensitive_data: Some("sensitive".to_string()),
                        graphics: Some(GraphicsCollection {
                            description: Some("All graphics".to_string()),
                            collection: Some(vec![Graphic {
                                name: Some("graphic-1".to_string()),
                                image: Some(Attachment {
                                    content_type: Some("image/jpeg".to_string()),
                                    encoding: Some("base64".to_string()),
                                    content: "imagebytes".to_string(),
                                }),
                            }]),
                        }),
                        description: Some("Component data description".to_string()),
                        governance: Some(DataGovernance {
                            custodians: Some(vec![DataGovernanceResponsibleParty::Contact(
                                OrganizationalContact {
                                    bom_ref: Some(BomReference::new("custodian-1")),
                                    name: Some("custodian".into()),
                                    email: None,
                                    phone: None,
                                },
                            )]),
                            stewards: None,
                            owners: None,
                        }),
                    })])),
                    inputs: Some(Inputs(vec![MLParameter::new("string")])),
                    outputs: Some(Outputs(vec![MLParameter::new("image")])),
                }),
                quantitative_analysis: Some(QuantitativeAnalysis {
                    performance_metrics: Some(PerformanceMetrics(vec![PerformanceMetric {
                        metric_type: Some("performance".to_string()),
                        value: Some("metric value".to_string()),
                        slice: None,
                        confidence_interval: Some(ConfidenceInterval {
                            lower_bound: Some("low".to_string()),
                            upper_bound: Some("high".to_string()),
                        }),
                    }])),
                    graphics: Some(GraphicsCollection {
                        description: Some("graphics".to_string()),
                        collection: None,
                    }),
                }),
                considerations: Some(Considerations {}),
                properties: Some(Properties(vec![Property {
                    name: "property".to_string(),
                    value: NormalizedString("value".to_string()),
                }])),
            }),
            data: Some(ComponentData {
                bom_ref: None,
                data_type: ComponentDataType::SourceCode,
                name: Some("github".into()),
                contents: Some(DataContents {
                    attachment: Some(Attachment {
                        content: "some pic".into(),
                        content_type: None,
                        encoding: Some("base64".into()),
                    }),
                    url: None,
                    properties: None,
                }),
                classification: None,
                sensitive_data: None,
                graphics: None,
                description: None,
                governance: None,
            }),
        }];
        let validation_result = Components(vec).validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn invalid_components_should_fail_validation() {
        let validation_result = Components(vec![Component {
            component_type: Classification::UnknownClassification("unknown".to_string()),
            mime_type: Some(MimeType("invalid mime type".to_string())),
            bom_ref: Some("bom ref".to_string()),
            supplier: Some(OrganizationalEntity {
                bom_ref: Some(BomReference::new("Supplier 1")),
                name: Some(NormalizedString("invalid\tname".to_string())),
                url: None,
                contact: None,
            }),
            author: Some(NormalizedString("invalid\tauthor".to_string())),
            publisher: Some(NormalizedString("invalid\tpublisher".to_string())),
            group: Some(NormalizedString("invalid\tgroup".to_string())),
            name: NormalizedString("invalid\tname".to_string()),
            version: Some(NormalizedString("invalid\tversion".to_string())),
            description: Some(NormalizedString("invalid\tdescription".to_string())),
            scope: Some(Scope::UnknownScope("unknown".to_string())),
            hashes: Some(Hashes(vec![Hash {
                alg: HashAlgorithm::MD5,
                content: HashValue("invalid hash content".to_string()),
            }])),
            licenses: Some(Licenses(vec![LicenseChoice::Expression(
                SpdxExpression::new("invalid license"),
            )])),
            copyright: Some(NormalizedString("invalid\tcopyright".to_string())),
            cpe: Some(Cpe("invalid cpe".to_string())),
            purl: Some(Purl("invalid purl".to_string())),
            swid: Some(Swid {
                tag_id: "tag ID".to_string(),
                name: "name".to_string(),
                version: Some("version".to_string()),
                tag_version: Some(1),
                patch: Some(true),
                text: Some(AttachedText {
                    content_type: Some(NormalizedString("invalid\tcontent_type".to_string())),
                    encoding: None,
                    content: "content".to_string(),
                }),
                url: Some(Url("invalid url".to_string())),
            }),
            modified: Some(true),
            pedigree: Some(Pedigree {
                ancestors: Some(Components(vec![invalid_component()])),
                descendants: Some(Components(vec![invalid_component()])),
                variants: Some(Components(vec![invalid_component()])),
                commits: Some(Commits(vec![Commit {
                    uid: Some(NormalizedString("invalid\tuid".to_string())),
                    url: None,
                    author: None,
                    committer: None,
                    message: None,
                }])),
                patches: Some(Patches(vec![Patch {
                    patch_type: PatchClassification::UnknownPatchClassification(
                        "unknown".to_string(),
                    ),
                    diff: None,
                    resolves: None,
                }])),
                notes: Some("notes".to_string()),
            }),
            external_references: Some(ExternalReferences(vec![ExternalReference {
                external_reference_type: ExternalReferenceType::UnknownExternalReferenceType(
                    "unknown".to_string(),
                ),
                url: Uri::Url(Url("https://www.example.com".to_string())),
                comment: None,
                hashes: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString("invalid\tvalue".to_string()),
            }])),
            components: Some(Components(vec![invalid_component()])),
            evidence: Some(ComponentEvidence {
                licenses: Some(Licenses(vec![LicenseChoice::Expression(
                    SpdxExpression::new("invalid license"),
                )])),
                copyright: Some(CopyrightTexts(vec![Copyright("copyright".to_string())])),
                occurrences: None,
                callstack: None,
                identity: None,
            }),
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
            model_card: None,
            data: None,
        }])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    vec![
                        validation::field("component_type", "Unknown classification"),
                        validation::field(
                            "mime_type",
                            "MimeType does not match regular expression"
                        ),
                        validation::r#struct(
                            "supplier",
                            validation::field(
                                "name",
                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                            )
                        ),
                        validation::field(
                            "author",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::field(
                            "publisher",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::field(
                            "group",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::field(
                            "name",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::field(
                            "version",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::field(
                            "description",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::r#enum(
                            "scope",
                            "Unknown scope"
                        ),
                        validation::r#struct(
                            "hashes",
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
                        ),
                        validation::r#struct(
                            "licenses",
                            validation::list(
                                "inner",
                                [(
                                    0,
                                    validation::r#enum(
                                        "expression",
                                        "SPDX expression is not valid"
                                    )
                                )]
                            )
                        ),
                        validation::field(
                            "copyright",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::field(
                            "cpe",
                            "Cpe does not match regular expression"
                        ),
                        validation::field(
                            "purl",
                            "Purl does not conform to Package URL spec: URL scheme must be pkg"
                        ),
                        validation::r#struct(
                            "swid",
                            vec![
                                validation::r#struct(
                                    "text",
                                    validation::field(
                                        "content_type",
                                        "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                    )
                                ),
                                validation::field(
                                    "url",
                                    "Uri does not conform to RFC 3986"
                                )
                            ]
                        ),
                        validation::r#struct(
                            "pedigree",
                            vec![
                                validation::r#struct(
                                    "ancestors",
                                    validation::list(
                                        "inner",
                                        [(
                                            0,
                                            validation::field("component_type", "Unknown classification")
                                        )]
                                    )
                                ),
                                validation::r#struct(
                                    "descendants",
                                    validation::list(
                                        "inner",
                                        [(
                                            0,
                                            validation::field("component_type", "Unknown classification")
                                        )]
                                    )
                                ),
                                validation::r#struct(
                                    "variants",
                                    validation::list(
                                        "inner",
                                        [(
                                            0,
                                            validation::field("component_type", "Unknown classification")
                                        )]
                                    )
                                ),
                                validation::r#struct(
                                    "commits",
                                    validation::list(
                                        "inner",
                                        [(
                                            0,
                                            validation::field(
                                                "uid",
                                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                            )
                                        )]
                                    )
                                ),
                                validation::r#struct(
                                    "patches",
                                    validation::list(
                                        "inner",
                                        [(
                                            0,
                                            validation::r#enum("patch_type", "Unknown patch classification")
                                        )]
                                    )
                                )
                            ]
                        ),
                        validation::r#struct(
                            "external_references",
                            validation::list(
                                "inner",
                                [(
                                    0,
                                    validation::field(
                                        "external_reference_type",
                                        "Unknown external reference type"
                                    )
                                )]
                            )
                        ),
                        validation::r#struct(
                            "properties",
                            validation::list(
                                "inner",
                                [(
                                    0,
                                    validation::field(
                                        "value",
                                        "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                    )
                                )]
                            )
                        ),
                        validation::r#struct(
                            "components",
                            validation::list(
                                "inner",
                                [(
                                    0,
                                    validation::field("component_type", "Unknown classification")
                                )]
                            )
                        ),
                        validation::r#struct(
                            "evidence",
                            validation::r#struct(
                                "licenses",
                                validation::list(
                                    "inner",
                                    [(
                                        0,
                                        validation::r#enum("expression", "SPDX expression is not valid")
                                    )]
                                )
                            )
                        )
                    ]
                )]
            )
        );
    }

    fn invalid_component() -> Component {
        Component {
            component_type: Classification::UnknownClassification("unknown".to_string()),
            mime_type: None,
            bom_ref: None,
            supplier: None,
            author: None,
            publisher: None,
            group: None,
            name: NormalizedString::new("name"),
            version: Some(NormalizedString::new("version")),
            description: None,
            scope: None,
            hashes: None,
            licenses: None,
            copyright: None,
            cpe: None,
            purl: None,
            swid: None,
            modified: None,
            pedigree: None,
            external_references: None,
            properties: None,
            components: None,
            evidence: None,
            signature: None,
            model_card: None,
            data: None,
        }
    }

    #[test]
    fn test_validate_classification() {
        assert!(validate_classification(&Classification::Library, SpecVersion::V1_4).is_ok());
        assert!(validate_classification(&Classification::Library, SpecVersion::V1_5).is_ok());
        assert!(validate_classification(&Classification::Platform, SpecVersion::V1_5).is_ok());

        assert!(validate_classification(&Classification::Platform, SpecVersion::V1_4).is_err());
        assert!(validate_classification(
            &Classification::UnknownClassification("test".to_string()),
            SpecVersion::V1_4
        )
        .is_err());
        assert!(validate_classification(
            &Classification::UnknownClassification("foo".to_string()),
            SpecVersion::V1_5
        )
        .is_err());
    }
}
