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

use std::{convert::TryFrom, str::FromStr};

use packageurl::PackageUrl;
//use url::Url;
use fluent_uri::Uri as Url;
use thiserror::Error;

use crate::validation::{
    FailureReason, Validate, ValidationContext, ValidationError, ValidationResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purl(pub(crate) String);

impl Purl {
    pub fn new(package_type: &str, name: &str, version: &str) -> Result<Purl, UriError> {
        match packageurl::PackageUrl::new(package_type, name) {
            Ok(mut purl) => Ok(Self(purl.with_version(version.trim()).to_string())),
            Err(e) => Err(UriError::InvalidPurl(e.to_string())),
        }
    }
}

impl ToString for Purl {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Validate for Purl {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match PackageUrl::from_str(&self.0.to_string()) {
            Ok(_) => Ok(ValidationResult::Passed),
            Err(e) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: format!("Purl does not conform to Package URL spec: {}", e),
                    context,
                }],
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri(pub(crate) String);

impl TryFrom<String> for Uri {
    type Error = UriError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match Url::parse(value.as_str()) {
            Ok(_) => Ok(Uri(value)),
            Err(_) => Err(UriError::InvalidUri(
                "Uri does not conform to RFC 3986".to_string(),
            )),
        }
    }
}

impl Validate for Uri {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match Url::parse(&self.0.to_string()) {
            Ok(_) => Ok(ValidationResult::Passed),
            Err(_) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Uri does not conform to RFC 3986".to_string(),
                    context,
                }],
            }),
        }
    }
}

impl ToString for Uri {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UriError {
    #[error("Invalid URI: {}", .0)]
    InvalidUri(String),

    #[error("Invalid Purl: {}", .0)]
    InvalidPurl(String),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::validation::FailureReason;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_purls_should_pass_validation() {
        let validation_result = Purl("pkg:cargo/cyclonedx-bom@0.3.1".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_purls_should_fail_validation() {
        let validation_result = Purl("invalid purl".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Purl does not conform to Package URL spec: missing scheme"
                        .to_string(),
                    context: ValidationContext::default()
                }]
            }
        );
    }

    #[test]
    fn valid_uris_should_pass_validation() {
        let validation_result = Uri("https://example.com".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_uris_should_fail_validation() {
        let validation_result = Uri("invalid uri".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Uri does not conform to RFC 3986".to_string(),
                    context: ValidationContext::default()
                }]
            }
        );
    }
}
