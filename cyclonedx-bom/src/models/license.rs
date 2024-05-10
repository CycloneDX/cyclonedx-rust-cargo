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

use crate::external_models::normalized_string::validate_normalized_string;
use crate::external_models::spdx::{validate_spdx_expression, validate_spdx_identifier};
use crate::external_models::uri::validate_uri;
use crate::external_models::validate_date_time;
use crate::external_models::{
    date_time::DateTime,
    normalized_string::NormalizedString,
    spdx::{SpdxExpression, SpdxIdentifier},
    uri::Uri,
};
use crate::models::{
    attached_text::AttachedText,
    bom::{BomReference, SpecVersion},
    organization::{OrganizationalContact, OrganizationalEntity},
};
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

use super::property::Properties;

/// Represents whether a license is a named license or an SPDX license expression
///
/// As defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_licenseChoiceType)
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LicenseChoice {
    License(License),
    Expression(SpdxExpression),
}

impl LicenseChoice {
    pub fn is_license(&self) -> bool {
        matches!(self, LicenseChoice::License(_))
    }

    /// Creates a new license with given string.
    pub fn license(license: &str) -> Self {
        Self::License(License::named_license(license))
    }

    /// Creates a new expression.
    pub fn expression(expression: &str) -> Self {
        Self::Expression(SpdxExpression::new(expression))
    }
}

impl Validate for LicenseChoice {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();

        match self {
            LicenseChoice::License(license) => {
                context.add_struct("license", license, version);
            }
            LicenseChoice::Expression(expression) => {
                context.add_enum("expression", expression, validate_spdx_expression);
            }
        }

        context.into()
    }
}

/// Represents a license with identifier, text, and url
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_licenseType)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct License {
    pub bom_ref: Option<BomReference>,
    pub license_identifier: LicenseIdentifier,
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
    pub licensing: Option<Licensing>,
    pub properties: Option<Properties>,
}

impl License {
    /// Constructs a `License` with a named license identifier
    /// ```
    /// use cyclonedx_bom::models::license::License;
    ///
    /// let license = License::named_license("Example License 1.0");
    /// ```
    pub fn named_license(license: &str) -> Self {
        Self {
            bom_ref: None,
            license_identifier: LicenseIdentifier::Name(NormalizedString::new(license)),
            text: None,
            url: None,
            licensing: None,
            properties: None,
        }
    }

    /// Constructs a `License` with an SPDX license identifier
    /// ```
    /// use cyclonedx_bom::models::license::License;
    ///
    /// let license = License::license_id("LGPL-3.0-or-later");
    /// ```
    pub fn license_id(license: &str) -> Self {
        let identifier = SpdxIdentifier(license.to_string());
        Self {
            bom_ref: None,
            license_identifier: LicenseIdentifier::SpdxId(identifier),
            text: None,
            url: None,
            licensing: None,
            properties: None,
        }
    }
}

impl Validate for License {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct("license_identifier", &self.license_identifier, version)
            .add_struct_option("text", self.text.as_ref(), version)
            .add_field_option("url", self.url.as_ref(), validate_uri)
            .add_struct_option("licensing", self.licensing.as_ref(), version)
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Licenses(pub Vec<LicenseChoice>);

impl Validate for Licenses {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();
        context.add_list("inner", &self.0, |choice| choice.validate_version(version));

        // In version 1.5 the `licenses` field contains either an array of [`LicenseChoice::License`] or
        // a single entry of [`LicenseChoice::Expression`], but not both.
        // See https://cyclonedx.org/docs/1.5/json/#components_items_licenses for more details.
        if version >= SpecVersion::V1_5 {
            let (licenses, expressions): (Vec<_>, Vec<_>) =
                self.0.iter().partition(|l| l.is_license());
            match (licenses.len(), expressions.len()) {
                (0, e) if e > 1 => {
                    context.add_custom("licenses", "More than one 'expression' entry found.");
                }
                (l, e) if l > 0 && e > 0 => {
                    context.add_custom(
                        "licenses",
                        "Use either array of 'license' or a single 'expression', but not both.",
                    );
                }
                _ => {}
            }
        }

        context.into()
    }
}

pub fn validate_license_identifier(identifier: &LicenseIdentifier) -> Result<(), ValidationError> {
    match identifier {
        LicenseIdentifier::Name(name) => validate_normalized_string(name),
        LicenseIdentifier::SpdxId(id) => validate_spdx_identifier(id),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LicenseIdentifier {
    /// An SPDX license identifier from the list on the [SPDX website](https://spdx.org/licenses/).
    SpdxId(SpdxIdentifier),
    /// A license that is not in the SPDX license list (eg. a proprietary license or a license not yet recognized by SPDX).
    Name(NormalizedString),
}

impl Validate for LicenseIdentifier {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        match self {
            LicenseIdentifier::Name(name) => ValidationContext::new()
                .add_enum("Name", name, validate_normalized_string)
                .into(),
            LicenseIdentifier::SpdxId(id) => ValidationContext::new()
                .add_enum("SpdxId", id, validate_spdx_identifier)
                .into(),
        }
    }
}

/// Represents Licensing information, added in spec version 1.5.
///
/// For more details see: https://cyclonedx.org/docs/1.5/json/#metadata_licenses_oneOf_i0_items_license_licensing
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Licensing {
    pub alt_ids: Option<Vec<NormalizedString>>,
    pub licensor: Option<LicenseContact>,
    pub licensee: Option<LicenseContact>,
    pub purchaser: Option<LicenseContact>,
    pub purchase_order: Option<String>,
    pub license_types: Option<Vec<LicenseType>>,
    pub last_renewal: Option<DateTime>,
    pub expiration: Option<DateTime>,
}

impl Validate for Licensing {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("alt_ids", self.alt_ids.as_ref(), validate_normalized_string)
            .add_struct_option("licensor", self.licensor.as_ref(), version)
            .add_struct_option("licensee", self.licensee.as_ref(), version)
            .add_struct_option("purchaser", self.purchaser.as_ref(), version)
            .add_list_option(
                "license_types",
                self.license_types.as_ref(),
                validate_license_type,
            )
            .add_field_option(
                "last_renewal",
                self.last_renewal.as_ref(),
                validate_date_time,
            )
            .add_field_option("expiration", self.expiration.as_ref(), validate_date_time)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LicenseContact {
    Organization(OrganizationalEntity),
    Contact(OrganizationalContact),
}

impl Validate for LicenseContact {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match self {
            LicenseContact::Organization(org) => org.validate_version(version),
            LicenseContact::Contact(contact) => contact.validate_version(version),
        }
    }
}

fn validate_license_type(license_type: &LicenseType) -> Result<(), ValidationError> {
    if let LicenseType::Unknown(unknown) = license_type {
        return Err(format!("Unknown license type '{}'", unknown).into());
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, strum::Display, Hash)]
#[strum(serialize_all = "kebab-case")]
#[repr(u16)]
pub enum LicenseType {
    Academic = 1,
    Appliance,
    ClientAccess,
    ConcurrentUser,
    CorePoints,
    CustomMetric,
    Device,
    Evaluation,
    NamedUser,
    NodeLocked,
    Oem,
    Perpetual,
    ProcessorPoints,
    Subscription,
    User,
    Other,
    #[doc(hidden)]
    #[strum(default)]
    Unknown(String),
}

impl LicenseType {
    pub fn new_unchecked(value: &str) -> Self {
        match value {
            "academic" => Self::Academic,
            "appliance" => Self::Appliance,
            "client-access" => Self::ClientAccess,
            "concurrent-user" => Self::ConcurrentUser,
            "core-points" => Self::CorePoints,
            "custom-metric" => Self::CustomMetric,
            "device" => Self::Device,
            "evaluation" => Self::Evaluation,
            "named-user" => Self::NamedUser,
            "node-locked" => Self::NodeLocked,
            "oem" => Self::Oem,
            "perpetual" => Self::Perpetual,
            "processor-points" => Self::ProcessorPoints,
            "subscription" => Self::Subscription,
            "user" => Self::User,
            "other" => Self::Other,
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::validation;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Licenses(vec![LicenseChoice::Expression(SpdxExpression::new(
            "MIT OR Apache-2.0",
        ))])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn it_should_fail_validation_for_license_name() {
        let validation_result = Licenses(vec![LicenseChoice::License(License {
            bom_ref: None,
            license_identifier: LicenseIdentifier::Name(NormalizedString(
                "spaces and \ttabs".to_string(),
            )),
            text: None,
            url: None,
            licensing: None,
            properties: None,
        })])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    validation::r#struct(
                        "license",
                        validation::r#struct(
                            "license_identifier",
                            validation::r#enum(
                                "Name",
                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                            )
                        )
                    )
                )]
            )
        );
    }

    #[test]
    fn it_should_fail_validation_for_license_id() {
        let validation_result = Licenses(vec![LicenseChoice::License(License::license_id(
            "Apache=2.0",
        ))])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    validation::r#struct(
                        "license",
                        validation::r#struct(
                            "license_identifier",
                            validation::r#enum("SpdxId", "SPDX identifier is not valid")
                        )
                    )
                )]
            )
        );
    }

    #[test]
    fn it_should_fail_validation_for_license_expression() {
        let validation_result = Licenses(vec![LicenseChoice::Expression(SpdxExpression::new(
            "MIT OR",
        ))])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    validation::r#enum("expression", "SPDX expression is not valid")
                )],
            )
        );
    }

    #[test]
    fn it_should_merge_validations_correctly_license_choice_licenses() {
        let validation_result = Licenses(vec![
            LicenseChoice::License(License {
                bom_ref: None,
                license_identifier: LicenseIdentifier::Name(NormalizedString("MIT".to_string())),
                text: None,
                url: None,
                licensing: None,
                properties: None,
            }),
            LicenseChoice::License(License {
                bom_ref: None,
                license_identifier: LicenseIdentifier::Name(NormalizedString(
                    "spaces and \ttabs".to_string(),
                )),
                text: None,
                url: None,
                licensing: None,
                properties: None,
            }),
            LicenseChoice::License(License {
                bom_ref: None,
                license_identifier: LicenseIdentifier::SpdxId(SpdxIdentifier(
                    "Apache=2.0".to_string(),
                )),
                text: None,
                url: None,
                licensing: None,
                properties: None,
            }),
        ])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    1,
                    validation::r#struct(
                        "license",
                        validation::r#struct(
                            "license_identifier",
                            validation::r#enum(
                                "Name",
                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                            )
                        )
                    )
                ), (
                    2,
                    validation::r#struct(
                        "license",
                        validation::r#struct(
                            "license_identifier",
                            validation::r#enum("SpdxId", "SPDX identifier is not valid")
                        )
                    )
                )]
            )
        );
    }

    #[test]
    fn it_should_merge_validations_correctly_license_choice_expressions() {
        let validation_result = Licenses(vec![
            LicenseChoice::Expression(SpdxExpression::new("MIT OR Apache-2.0")),
            LicenseChoice::Expression(SpdxExpression::new("MIT OR")),
            LicenseChoice::Expression(SpdxExpression::new("MIT OR")),
        ])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [
                    (
                        1,
                        validation::r#enum("expression", "SPDX expression is not valid"),
                    ),
                    (
                        2,
                        validation::r#enum("expression", "SPDX expression is not valid"),
                    )
                ]
            )
        );
    }

    #[test]
    fn it_should_fail_with_mixed_license_nodes_in_version_15() {
        let licenses = Licenses(vec![
            LicenseChoice::License(License::named_license("MIT OR Apache-2.0")),
            LicenseChoice::Expression(SpdxExpression::new("MIT OR Apache-2.0")),
        ]);
        let validation_result = licenses.validate_version(SpecVersion::V1_5);

        assert_eq!(
            validation_result,
            validation::custom(
                "licenses",
                ["Use either array of 'license' or a single 'expression', but not both."]
            )
        );
    }

    #[test]
    fn it_should_fail_with_multiple_license_expressions_in_version_15() {
        let validation_result = Licenses(vec![
            LicenseChoice::Expression(SpdxExpression::new("MIT OR Apache-2.0")),
            LicenseChoice::Expression(SpdxExpression::new("MIT OR Apache-2.0")),
        ])
        .validate_version(SpecVersion::V1_5);

        assert_eq!(
            validation_result,
            validation::custom("licenses", ["More than one 'expression' entry found."])
        );
    }
}
