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

use crate::external_models::{
    date_time::DateTime,
    normalized_string::NormalizedString,
    spdx::SpdxIdentifier,
    uri::{Purl, Uri},
};

#[derive(Debug, PartialEq)]
pub struct Bom {
    pub version: u32,
    pub serial_number: Option<UrnUuid>,
    pub metadata: Option<Metadata>,
    pub components: Option<Vec<Component>>,
    pub services: Option<Services>,
    pub external_references: Option<ExternalReferences>,
    pub dependencies: Option<Dependencies>,
    pub compositions: Option<Vec<Composition>>,
    pub properties: Option<Properties>,
}

impl Default for Bom {
    fn default() -> Self {
        Self {
            version: 1,
            serial_number: Some(UrnUuid(format!("urn:uuid:{}", uuid::Uuid::new_v4()))),
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: None,
            properties: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AttachedText {
    pub(crate) content_type: Option<NormalizedString>,
    pub(crate) encoding: Option<Encoding>,
    pub(crate) content: String,
}

impl AttachedText {
    /// Construct a new `AttachedText`
    ///
    /// - `content_type` - Content type of the attached text (default: `"text/plain"`)
    /// - `content` - Raw content, which will be base64 encoded when added to the BOM
    pub fn new<T: AsRef<[u8]>>(content_type: Option<NormalizedString>, content: T) -> Self {
        Self {
            content_type,
            encoding: Some(Encoding::Base64),
            content: base64::encode(content),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BomReference(pub(crate) String);

#[derive(Debug, PartialEq)]
pub struct Commit {
    pub uid: Option<NormalizedString>,
    pub url: Option<Uri>,
    pub author: Option<IdentifiableAction>,
    pub committer: Option<IdentifiableAction>,
    pub message: Option<NormalizedString>,
}

#[derive(Debug, PartialEq)]
pub struct Component {
    pub component_type: Classification,
    pub mime_type: Option<MimeType>,
    pub bom_ref: Option<String>,
    pub supplier: Option<OrganizationalEntity>,
    pub author: Option<NormalizedString>,
    pub publisher: Option<NormalizedString>,
    pub group: Option<NormalizedString>,
    pub name: NormalizedString,
    pub version: NormalizedString,
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
    pub components: Option<Vec<Component>>,
    pub evidence: Option<ComponentEvidence>,
}

#[derive(Debug, PartialEq)]
pub struct ComponentEvidence {
    pub licenses: Option<Licenses>,
    pub copyright: Option<Vec<Copyright>>,
}

#[derive(Debug, PartialEq)]
pub struct Composition {
    pub aggregate: AggregateType,
    pub assemblies: Option<Vec<BomReference>>,
    pub dependencies: Option<Vec<BomReference>>,
}

#[derive(Debug, PartialEq)]
pub struct Copyright(pub(crate) String);

#[derive(Debug, PartialEq)]
pub struct DataClassification {
    pub flow: DataFlowType,
    pub classification: NormalizedString,
}

#[derive(Debug, PartialEq)]
pub struct Dependencies(pub(crate) Vec<Dependency>);

#[derive(Debug, PartialEq)]
pub struct Dependency {
    pub dependency_ref: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, PartialEq)]
pub struct Diff {
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
}

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
pub struct Hash {
    pub alg: HashAlgorithm,
    pub content: HashValue,
}

#[derive(Debug, PartialEq)]
pub struct Hashes(pub Vec<Hash>);

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
pub enum LicenseChoice {
    License(License),
    Expression(NormalizedString),
}

#[derive(Debug, PartialEq)]
pub struct License {
    pub license_identifier: LicenseIdentifier,
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
}

#[derive(Debug, PartialEq)]
pub struct Licenses(pub Vec<LicenseChoice>);

#[derive(Debug, PartialEq)]
pub enum LicenseIdentifier {
    SpdxId(SpdxIdentifier),
    Name(NormalizedString),
}

#[derive(Debug, PartialEq)]
pub struct Metadata {
    pub timestamp: Option<DateTime>,
    pub tools: Option<Tools>,
    pub authors: Option<Vec<OrganizationalContact>>,
    pub component: Option<Component>,
    pub manufacture: Option<OrganizationalEntity>,
    pub supplier: Option<OrganizationalEntity>,
    pub licenses: Option<Licenses>,
    pub properties: Option<Properties>,
}

#[derive(Debug, PartialEq)]
pub struct OrganizationalContact {
    pub name: Option<NormalizedString>,
    pub email: Option<NormalizedString>,
    pub phone: Option<NormalizedString>,
}

#[derive(Debug, PartialEq)]
pub struct OrganizationalEntity {
    pub name: Option<NormalizedString>,
    pub url: Option<Vec<Uri>>,
    pub contact: Option<Vec<OrganizationalContact>>,
}

#[derive(Debug, PartialEq)]
pub struct Patch {
    pub patch_type: PatchClassification,
    pub diff: Diff,
    pub resolves: Option<Vec<Issue>>,
}

#[derive(Debug, PartialEq)]
pub struct Pedigree {
    pub ancestors: Option<Vec<Component>>,
    pub descendants: Option<Vec<Component>>,
    pub variants: Option<Vec<Component>>,
    pub commits: Option<Vec<Commit>>,
    pub patches: Option<Vec<Patch>>,
    pub notes: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Properties(pub(crate) Vec<Property>);

#[derive(Debug, PartialEq)]
pub struct Property {
    pub name: String,
    pub value: NormalizedString,
}

#[derive(Debug, PartialEq)]
pub struct Source {
    pub name: Option<NormalizedString>,
    pub url: Option<Uri>,
}

#[derive(Debug, PartialEq)]
pub struct Swid {
    pub tag_id: String,
    pub name: String,
    pub version: Option<String>,
    pub tag_version: Option<u32>,
    pub patch: Option<bool>,
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
}

#[derive(Debug, PartialEq)]
pub struct Service {
    pub bom_ref: Option<String>,
    pub provider: Option<OrganizationalEntity>,
    pub group: Option<NormalizedString>,
    pub name: NormalizedString,
    pub version: Option<NormalizedString>,
    pub description: Option<NormalizedString>,
    pub endpoints: Option<Vec<Uri>>,
    pub authenticated: Option<bool>,
    pub x_trust_boundary: Option<bool>,
    pub data: Option<Vec<DataClassification>>,
    pub licenses: Option<Licenses>,
    pub external_references: Option<ExternalReferences>,
    pub properties: Option<Properties>,
    pub services: Option<Services>,
}

#[derive(Debug, PartialEq)]
pub struct Services(pub Vec<Service>);

#[derive(Debug, PartialEq)]
pub struct Tool {
    pub vendor: Option<NormalizedString>,
    pub name: Option<NormalizedString>,
    pub version: Option<NormalizedString>,
    pub hashes: Option<Hashes>,
}

#[derive(Debug, PartialEq)]
pub struct Tools(pub Vec<Tool>);

#[derive(Debug, PartialEq)]
pub enum AggregateType {
    Complete,
    Incomplete,
    IncompleteFirstPartyOnly,
    IncompleteThirdPartyOnly,
    Unknown,
    NotSpecified,
    #[doc(hidden)]
    UnknownAggregateType(String),
}

impl ToString for AggregateType {
    fn to_string(&self) -> String {
        match self {
            AggregateType::Complete => "complete",
            AggregateType::Incomplete => "incomplete",
            AggregateType::IncompleteFirstPartyOnly => "incomplete_first_party_only",
            AggregateType::IncompleteThirdPartyOnly => "incomplete_third_party_only",
            AggregateType::Unknown => "unknown",
            AggregateType::NotSpecified => "not_specified",
            AggregateType::UnknownAggregateType(uat) => uat,
        }
        .to_string()
    }
}

impl AggregateType {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "complete" => Self::Complete,
            "incomplete" => Self::Incomplete,
            "incomplete_first_party_only" => Self::IncompleteFirstPartyOnly,
            "incomplete_third_party_only" => Self::IncompleteThirdPartyOnly,
            "unknown" => Self::Unknown,
            "not_specified" => Self::NotSpecified,
            unknown => Self::UnknownAggregateType(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Classification {
    Application,
    Framework,
    Library,
    Container,
    OperatingSystem,
    Device,
    Firmware,
    File,
    #[doc(hidden)]
    UnknownClassification(String),
}

impl ToString for Classification {
    fn to_string(&self) -> String {
        match self {
            Classification::Application => "application",
            Classification::Framework => "framework",
            Classification::Library => "library",
            Classification::Container => "container",
            Classification::OperatingSystem => "operating-system",
            Classification::Device => "device",
            Classification::Firmware => "firmware",
            Classification::File => "file",
            Classification::UnknownClassification(uc) => uc,
        }
        .to_string()
    }
}

impl Classification {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "application" => Self::Application,
            "framework" => Self::Framework,
            "library" => Self::Library,
            "container" => Self::Container,
            "operating-system" => Self::OperatingSystem,
            "device" => Self::Device,
            "firmware" => Self::Firmware,
            "file" => Self::File,
            unknown => Self::UnknownClassification(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Cpe(pub(crate) String); // TODO: validate

#[derive(Debug, PartialEq)]
pub enum DataFlowType {
    Inbound,
    Outbound,
    BiDirectional,
    Unknown,
    #[doc(hidden)]
    UnknownDataFlow(String),
}

impl ToString for DataFlowType {
    fn to_string(&self) -> String {
        match self {
            DataFlowType::Inbound => "inbound",
            DataFlowType::Outbound => "outbound",
            DataFlowType::BiDirectional => "bi-directional",
            DataFlowType::Unknown => "unknown",
            DataFlowType::UnknownDataFlow(df) => df,
        }
        .to_string()
    }
}

impl DataFlowType {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "inbound" => Self::Inbound,
            "outbound" => Self::Outbound,
            "bi-directional" => Self::BiDirectional,
            "unknown" => Self::Unknown,
            unknown => Self::UnknownDataFlow(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Encoding {
    Base64,
    #[doc(hidden)]
    UnknownEncoding(String),
}

impl ToString for Encoding {
    fn to_string(&self) -> String {
        match self {
            Encoding::Base64 => "base64".to_string(),
            Encoding::UnknownEncoding(ue) => ue.clone(),
        }
    }
}

impl Encoding {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "base64" => Self::Base64,
            unknown => Self::UnknownEncoding(unknown.to_string()),
        }
    }
}

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

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum HashAlgorithm {
    MD5,
    SHA1,
    SHA256,
    SHA384,
    SHA512,
    SHA3_256,
    SHA3_384,
    SHA3_512,
    BLAKE2b_256,
    BLAKE2b_384,
    BLAKE2b_512,
    BLAKE3,
    #[doc(hidden)]
    UnknownHashAlgorithm(String),
}

impl ToString for HashAlgorithm {
    fn to_string(&self) -> String {
        match self {
            HashAlgorithm::MD5 => "MD5",
            HashAlgorithm::SHA1 => "SHA-1",
            HashAlgorithm::SHA256 => "SHA-256",
            HashAlgorithm::SHA384 => "SHA-384",
            HashAlgorithm::SHA512 => "SHA-512",
            HashAlgorithm::SHA3_256 => "SHA3-256",
            HashAlgorithm::SHA3_384 => "SHA3-384",
            HashAlgorithm::SHA3_512 => "SHA3-512",
            HashAlgorithm::BLAKE2b_256 => "BLAKE2b-256",
            HashAlgorithm::BLAKE2b_384 => "BLAKE2b-384",
            HashAlgorithm::BLAKE2b_512 => "BLAKE2b-512",
            HashAlgorithm::BLAKE3 => "BLAKE3",
            HashAlgorithm::UnknownHashAlgorithm(un) => un,
        }
        .to_string()
    }
}

impl HashAlgorithm {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "MD5" => Self::MD5,
            "SHA-1" => Self::SHA1,
            "SHA-256" => Self::SHA256,
            "SHA-384" => Self::SHA384,
            "SHA-512" => Self::SHA512,
            "SHA3-256" => Self::SHA3_256,
            "SHA3-384" => Self::SHA3_384,
            "SHA3-512" => Self::SHA3_512,
            "BLAKE2b-256" => Self::BLAKE2b_256,
            "BLAKE2b-384" => Self::BLAKE2b_384,
            "BLAKE2b-512" => Self::BLAKE2b_512,
            "BLAKE3" => Self::BLAKE3,
            unknown => Self::UnknownHashAlgorithm(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HashValue(pub(crate) String); // TODO: validate

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
pub struct MimeType(pub(crate) String); // TODO: validate regex

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
pub enum Scope {
    Required,
    Optional,
    Excluded,
    #[doc(hidden)]
    UnknownScope(String),
}

impl ToString for Scope {
    fn to_string(&self) -> String {
        match self {
            Scope::Required => "required",
            Scope::Optional => "optional",
            Scope::Excluded => "excluded",
            Scope::UnknownScope(us) => us,
        }
        .to_string()
    }
}

impl Scope {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "required" => Self::Required,
            "optional" => Self::Optional,
            "excluded" => Self::Excluded,
            unknown => Self::UnknownScope(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct UrnUuid(pub(crate) String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_construct_attached_text() {
        let actual = AttachedText::new(
            Some(NormalizedString::new("text/plain")),
            "this text is plain",
        );
        assert_eq!(
            actual,
            AttachedText {
                content_type: Some(NormalizedString::new("text/plain")),
                encoding: Some(Encoding::Base64),
                content: "dGhpcyB0ZXh0IGlzIHBsYWlu".to_string(),
            }
        )
    }
}
