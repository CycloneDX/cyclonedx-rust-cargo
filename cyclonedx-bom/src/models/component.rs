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
use std::str::FromStr;

use crate::models::attached_text::AttachedText;
use crate::models::code::{Commits, Patches};
use crate::models::external_reference::ExternalReferences;
use crate::models::hash::Hashes;
use crate::models::license::Licenses;
use crate::models::organization::OrganizationalEntity;
use crate::models::property::Properties;
use crate::validation::{FailureReason, ValidationPathComponent};
use crate::{
    external_models::{
        normalized_string::NormalizedString,
        uri::{Purl, Uri},
    },
    validation::{Validate, ValidationContext, ValidationError, ValidationResult},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Component {
    pub component_type: Classification,
    pub mime_type: Option<MimeType>,
    pub bom_ref: Option<String>,
    pub supplier: Option<OrganizationalEntity>,
    pub author: Option<NormalizedString>,
    pub publisher: Option<NormalizedString>,
    pub group: Option<NormalizedString>,
    pub name: NormalizedString,
    // todo: 1.3 vs. 1.4 - this field is an Option in 1.4
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
        }
    }
}

impl Validate for Component {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        let component_type_context =
            context.extend_context_with_struct_field("Component", "component_type");

        results.push(
            self.component_type
                .validate_with_context(component_type_context)?,
        );

        if let Some(mime_type) = &self.mime_type {
            let context = context.extend_context_with_struct_field("Component", "mime_type");

            results.push(mime_type.validate_with_context(context)?);
        }

        if let Some(supplier) = &self.supplier {
            let context = context.extend_context_with_struct_field("Component", "supplier");

            results.push(supplier.validate_with_context(context)?);
        }

        if let Some(author) = &self.author {
            let context = context.extend_context_with_struct_field("Component", "author");

            results.push(author.validate_with_context(context)?);
        }

        if let Some(publisher) = &self.publisher {
            let context = context.extend_context_with_struct_field("Component", "publisher");

            results.push(publisher.validate_with_context(context)?);
        }

        if let Some(group) = &self.group {
            let context = context.extend_context_with_struct_field("Component", "group");

            results.push(group.validate_with_context(context)?);
        }

        let name_context = context.extend_context_with_struct_field("Component", "name");

        results.push(self.name.validate_with_context(name_context)?);

        if let Some(version) = &self.version {
            let context = context.extend_context_with_struct_field("Component", "version");

            results.push(version.validate_with_context(context)?);
        }

        if let Some(description) = &self.description {
            let context = context.extend_context_with_struct_field("Component", "description");

            results.push(description.validate_with_context(context)?);
        }

        if let Some(scope) = &self.scope {
            let context = context.extend_context_with_struct_field("Component", "scope");

            results.push(scope.validate_with_context(context)?);
        }

        if let Some(hashes) = &self.hashes {
            let context = context.extend_context_with_struct_field("Component", "hashes");

            results.push(hashes.validate_with_context(context)?);
        }

        if let Some(licenses) = &self.licenses {
            let context = context.extend_context_with_struct_field("Component", "licenses");

            results.push(licenses.validate_with_context(context)?);
        }

        if let Some(copyright) = &self.copyright {
            let context = context.extend_context_with_struct_field("Component", "copyright");

            results.push(copyright.validate_with_context(context)?);
        }

        if let Some(cpe) = &self.cpe {
            let context = context.extend_context_with_struct_field("Component", "cpe");

            results.push(cpe.validate_with_context(context)?);
        }

        if let Some(purl) = &self.purl {
            let context = context.extend_context_with_struct_field("Component", "purl");

            results.push(purl.validate_with_context(context)?);
        }

        if let Some(swid) = &self.swid {
            let context = context.extend_context_with_struct_field("Component", "swid");

            results.push(swid.validate_with_context(context)?);
        }

        if let Some(pedigree) = &self.pedigree {
            let context = context.extend_context_with_struct_field("Component", "pedigree");

            results.push(pedigree.validate_with_context(context)?);
        }

        if let Some(external_references) = &self.external_references {
            let context =
                context.extend_context_with_struct_field("Component", "external_references");

            results.push(external_references.validate_with_context(context)?);
        }

        if let Some(properties) = &self.properties {
            let context = context.extend_context_with_struct_field("Component", "properties");

            results.push(properties.validate_with_context(context)?);
        }

        if let Some(components) = &self.components {
            let context = context.extend_context_with_struct_field("Component", "components");

            results.push(components.validate_with_context(context)?);
        }

        if let Some(evidence) = &self.evidence {
            let context = context.extend_context_with_struct_field("Component", "evidence");

            results.push(evidence.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Components(pub Vec<Component>);

impl Validate for Components {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, component) in self.0.iter().enumerate() {
            let context = context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(component.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
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

impl Validate for Classification {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match self {
            Classification::UnknownClassification(_) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Unknown classification".to_string(),
                    context,
                }],
            }),
            _ => Ok(ValidationResult::Passed),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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

impl Validate for Scope {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match self {
            Scope::UnknownScope(_) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Unknown scope".to_string(),
                    context,
                }],
            }),
            _ => Ok(ValidationResult::Passed),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MimeType(pub(crate) String);

impl Validate for MimeType {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        static UUID_REGEX: Lazy<Result<Regex, regex::Error>> =
            Lazy::new(|| Regex::new(r"^[-+a-z0-9.]+/[-+a-z0-9.]+$"));

        match UUID_REGEX.as_ref() {
            Ok(regex) => {
                if regex.is_match(&self.0) {
                    Ok(ValidationResult::Passed)
                } else {
                    Ok(ValidationResult::Failed {
                        reasons: vec![FailureReason {
                            message: "MimeType does not match regular expression".to_string(),
                            context,
                        }],
                    })
                }
            }
            Err(e) => Err(e.clone().into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Swid {
    pub tag_id: String,
    pub name: String,
    pub version: Option<String>,
    pub tag_version: Option<u32>,
    pub patch: Option<bool>,
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
}

impl Validate for Swid {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(text) = &self.text {
            let context = context.extend_context_with_struct_field("Swid", "text");

            results.push(text.validate_with_context(context)?);
        }

        if let Some(url) = &self.url {
            let context = context.extend_context_with_struct_field("Swid", "url");

            results.push(url.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cpe(pub(crate) String);

impl FromStr for Cpe {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = Cpe(s.to_string());
        result.validate()?;
        Ok(result)
    }
}

impl Validate for Cpe {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        static UUID_REGEX: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| {
            Regex::new(
                r##"([c][pP][eE]:/[AHOaho]?(:[A-Za-z0-9\._\-~%]*){0,6})|(cpe:2\.3:[aho\*\-](:(((\?*|\*?)([a-zA-Z0-9\-\._]|(\\[\\\*\?!"#$$%&'\(\)\+,/:;<=>@\[\]\^`\{\|}~]))+(\?*|\*?))|[\*\-])){5}(:(([a-zA-Z]{2,3}(-([a-zA-Z]{2}|[0-9]{3}))?)|[\*\-]))(:(((\?*|\*?)([a-zA-Z0-9\-\._]|(\\[\\\*\?!"#$$%&'\(\)\+,/:;<=>@\[\]\^`\{\|}~]))+(\?*|\*?))|[\*\-])){4})"##,
            )
        });

        match UUID_REGEX.as_ref() {
            Ok(regex) => {
                if regex.is_match(&self.0) {
                    Ok(ValidationResult::Passed)
                } else {
                    Ok(ValidationResult::Failed {
                        reasons: vec![FailureReason {
                            message: "Cpe does not match regular expression".to_string(),
                            context,
                        }],
                    })
                }
            }
            Err(e) => Err(e.clone().into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ComponentEvidence {
    pub licenses: Option<Licenses>,
    pub copyright: Option<CopyrightTexts>,
}

impl Validate for ComponentEvidence {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(licenses) = &self.licenses {
            let context = context.extend_context_with_struct_field("ComponentEvidence", "licenses");

            results.push(licenses.validate_with_context(context)?);
        }

        if let Some(copyright) = &self.copyright {
            let context =
                context.extend_context_with_struct_field("ComponentEvidence", "copyright");

            results.push(copyright.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pedigree {
    pub ancestors: Option<Components>,
    pub descendants: Option<Components>,
    pub variants: Option<Components>,
    pub commits: Option<Commits>,
    pub patches: Option<Patches>,
    pub notes: Option<String>,
}

impl Validate for Pedigree {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(ancestors) = &self.ancestors {
            let context = context.extend_context_with_struct_field("Pedigree", "ancestors");

            results.push(ancestors.validate_with_context(context)?);
        }

        if let Some(descendants) = &self.descendants {
            let context = context.extend_context_with_struct_field("Pedigree", "descendants");

            results.push(descendants.validate_with_context(context)?);
        }

        if let Some(variants) = &self.variants {
            let context = context.extend_context_with_struct_field("Pedigree", "variants");

            results.push(variants.validate_with_context(context)?);
        }

        if let Some(commits) = &self.commits {
            let context = context.extend_context_with_struct_field("Pedigree", "commits");

            results.push(commits.validate_with_context(context)?);
        }

        if let Some(patches) = &self.patches {
            let context = context.extend_context_with_struct_field("Pedigree", "patches");

            results.push(patches.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Copyright(pub String);

impl Validate for Copyright {
    fn validate_with_context(
        &self,
        _context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        Ok(ValidationResult::default())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CopyrightTexts(pub(crate) Vec<Copyright>);

impl Validate for CopyrightTexts {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, copyright) in self.0.iter().enumerate() {
            let context = context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(copyright.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[cfg(test)]
mod test {

    use crate::{
        external_models::spdx::SpdxExpression,
        models::{
            code::{Commit, Patch, PatchClassification},
            external_reference::{ExternalReference, ExternalReferenceType},
            hash::{Hash, HashAlgorithm, HashValue},
            license::LicenseChoice,
            property::Property,
        },
        validation::ValidationPathComponent,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_components_should_pass_validation() {
        let validation_result = Components(vec![Component {
            component_type: Classification::Application,
            mime_type: Some(MimeType("text/text".to_string())),
            bom_ref: Some("bom ref".to_string()),
            supplier: Some(OrganizationalEntity {
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
            licenses: Some(Licenses(vec![LicenseChoice::Expression(SpdxExpression(
                "MIT".to_string(),
            ))])),
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
                url: Some(Uri("https://example.com".to_string())),
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
                url: Uri("https://www.example.com".to_string()),
                comment: None,
                hashes: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString::new("value"),
            }])),
            components: Some(Components(vec![])),
            evidence: Some(ComponentEvidence {
                licenses: Some(Licenses(vec![LicenseChoice::Expression(SpdxExpression(
                    "MIT".to_string(),
                ))])),
                copyright: Some(CopyrightTexts(vec![Copyright("copyright".to_string())])),
            }),
        }])
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_components_should_fail_validation() {
        let validation_result = Components(vec![Component {
            component_type: Classification::UnknownClassification("unknown".to_string()),
            mime_type: Some(MimeType("invalid mime type".to_string())),
            bom_ref: Some("bom ref".to_string()),
            supplier: Some(OrganizationalEntity {
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
            licenses: Some(Licenses(vec![LicenseChoice::Expression(SpdxExpression(
                "invalid license".to_string(),
            ))])),
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
                url: Some(Uri("invalid url".to_string())),
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
                url: Uri("https://www.example.com".to_string()),
                comment: None,
                hashes: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString("invalid\tvalue".to_string()),
            }])),
            components: Some(Components(vec![invalid_component()])),
            evidence: Some(ComponentEvidence {
                licenses: Some(Licenses(vec![LicenseChoice::Expression(SpdxExpression(
                    "invalid license".to_string(),
                ))])),
                copyright: Some(CopyrightTexts(vec![Copyright("copyright".to_string())])),
            }),
        }])
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message: "Unknown classification".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "component_type".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "MimeType does not match regular expression".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "mime_type".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "supplier".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "OrganizationalEntity".to_string(),
                                field_name: "name".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "author".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "publisher".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "group".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "name".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "version".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "description".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "Unknown scope".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "scope".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "HashValue does not match regular expression".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "hashes".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Hash".to_string(),
                                field_name: "content".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "SPDX expression is not valid".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "licenses".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::EnumVariant {
                                variant_name: "Expression".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "copyright".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "Cpe does not match regular expression".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "cpe".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "Purl does not conform to Package URL spec: missing scheme"
                            .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "purl".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "swid".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Swid".to_string(),
                                field_name: "text".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "AttachedText".to_string(),
                                field_name: "content_type".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "Uri does not conform to RFC 3986".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "swid".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Swid".to_string(),
                                field_name: "url".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "Unknown classification".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "pedigree".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Pedigree".to_string(),
                                field_name: "ancestors".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "component_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown classification".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "pedigree".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Pedigree".to_string(),
                                field_name: "descendants".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "component_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown classification".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "pedigree".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Pedigree".to_string(),
                                field_name: "variants".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "component_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "pedigree".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Pedigree".to_string(),
                                field_name: "commits".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Commit".to_string(),
                                field_name: "uid".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown patch classification".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "pedigree".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Pedigree".to_string(),
                                field_name: "patches".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Patch".to_string(),
                                field_name: "patch_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown external reference type".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "external_references".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "ExternalReference".to_string(),
                                field_name: "external_reference_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "properties".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Property".to_string(),
                                field_name: "value".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown classification".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "components".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "component_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "SPDX expression is not valid".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "evidence".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "ComponentEvidence".to_string(),
                                field_name: "licenses".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::EnumVariant {
                                variant_name: "Expression".to_string()
                            },
                        ])
                    },
                ]
            }
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
        }
    }
}
