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

use crate::external_models::normalized_string::validate_normalized_string;
use crate::external_models::uri::{validate_purl, validate_uri};
use crate::models::attached_text::AttachedText;
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
        uri::{Purl, Uri},
    },
    validation::{Validate, ValidationContext, ValidationResult},
};

use super::bom::SpecVersion;
use super::signature::Signature;

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
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
            Classification::Platform => "platform",
            Classification::DeviceDriver => "device-driver",
            Classification::MachineLearningModel => "machine-learning-model",
            Classification::Data => "data",
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MimeType(pub(crate) String);

#[derive(Clone, Debug, PartialEq, Eq)]
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
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("text", self.text.as_ref(), version)
            .add_field_option("url", self.url.as_ref(), validate_uri)
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cpe(pub(crate) String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentEvidence {
    pub licenses: Option<Licenses>,
    pub copyright: Option<CopyrightTexts>,
}

impl Validate for ComponentEvidence {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("licenses", self.licenses.as_ref(), version)
            .add_struct_option("copyright", self.copyright.as_ref(), version)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Copyright(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyrightTexts(pub(crate) Vec<Copyright>);

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
            code::{Commit, Patch, PatchClassification},
            external_reference::{ExternalReference, ExternalReferenceType},
            hash::{Hash, HashAlgorithm, HashValue},
            license::LicenseChoice,
            property::Property,
            signature::Algorithm,
        },
        validation,
    };

    use super::*;

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
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
        }])
        .validate();

        assert!(validation_result.passed());
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
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
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
                            "Purl does not conform to Package URL spec"
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
