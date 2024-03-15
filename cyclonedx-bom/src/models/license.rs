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

use std::convert::TryFrom;

use crate::external_models::normalized_string::validate_normalized_string;
use crate::external_models::spdx::{
    validate_spdx_expression, validate_spdx_identifier, SpdxIdentifierError,
};
use crate::external_models::uri::validate_uri;
use crate::external_models::{
    normalized_string::NormalizedString,
    spdx::{SpdxExpression, SpdxIdentifier},
    uri::Uri,
};
use crate::models::attached_text::AttachedText;
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

use super::bom::SpecVersion;

/// Represents whether a license is a named license or an SPDX license expression
///
/// As defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_licenseChoiceType)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LicenseChoice {
    License(License),
    Expression(SpdxExpression),
}

impl LicenseChoice {
    pub fn is_license(&self) -> bool {
        matches!(self, Self::License(_))
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct License {
    pub license_identifier: LicenseIdentifier,
    pub text: Option<AttachedText>,
    pub url: Option<Uri>,
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
            license_identifier: LicenseIdentifier::Name(NormalizedString::new(license)),
            text: None,
            url: None,
        }
    }

    /// Constructs a `License` with an SPDX license identifier
    /// ```
    /// use cyclonedx_bom::models::license::License;
    ///
    /// let license = License::license_id("LGPL-3.0-or-later");
    /// ```
    pub fn license_id(license: &str) -> Result<Self, SpdxIdentifierError> {
        Ok(Self {
            license_identifier: LicenseIdentifier::SpdxId(SpdxIdentifier::try_from(
                license.to_owned(),
            )?),
            text: None,
            url: None,
        })
    }
}

impl Validate for License {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct("license_identifier", &self.license_identifier, version)
            .add_struct_option("text", self.text.as_ref(), version)
            .add_field_option("url", self.url.as_ref(), validate_uri)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Licenses(pub Vec<LicenseChoice>);

impl Validate for Licenses {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |choice| choice.validate_version(version))
            .into()
    }
}

pub fn validate_license_identifier(identifier: &LicenseIdentifier) -> Result<(), ValidationError> {
    match identifier {
        LicenseIdentifier::Name(name) => validate_normalized_string(name),
        LicenseIdentifier::SpdxId(id) => validate_spdx_identifier(id),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod test {
    use crate::validation;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Licenses(vec![LicenseChoice::Expression(SpdxExpression(
            "MIT OR Apache-2.0".to_string(),
        ))])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn it_should_fail_validation_for_license_name() {
        let validation_result = Licenses(vec![LicenseChoice::License(License {
            license_identifier: LicenseIdentifier::Name(NormalizedString(
                "spaces and \ttabs".to_string(),
            )),
            text: None,
            url: None,
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
        let validation_result = Licenses(vec![LicenseChoice::License(License {
            license_identifier: LicenseIdentifier::SpdxId(SpdxIdentifier("Apache=2.0".to_string())),
            text: None,
            url: None,
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
                            validation::r#enum("SpdxId", "SPDX identifier is not valid")
                        )
                    )
                )]
            )
        );
    }

    #[test]
    fn it_should_fail_validation_for_license_expression() {
        let validation_result = Licenses(vec![LicenseChoice::Expression(SpdxExpression(
            "MIT OR".to_string(),
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
                license_identifier: LicenseIdentifier::Name(NormalizedString("MIT".to_string())),
                text: None,
                url: None,
            }),
            LicenseChoice::License(License {
                license_identifier: LicenseIdentifier::Name(NormalizedString(
                    "spaces and \ttabs".to_string(),
                )),
                text: None,
                url: None,
            }),
            LicenseChoice::License(License {
                license_identifier: LicenseIdentifier::SpdxId(SpdxIdentifier(
                    "Apache=2.0".to_string(),
                )),
                text: None,
                url: None,
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
            LicenseChoice::Expression(SpdxExpression("MIT OR Apache-2.0".to_string())),
            LicenseChoice::Expression(SpdxExpression("MIT OR".to_string())),
            LicenseChoice::Expression(SpdxExpression("MIT OR".to_string())),
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
}
