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

use std::str::FromStr;

use packageurl::PackageUrl;

use crate::validation::{
    FailureReason, Validate, ValidationContext, ValidationError, ValidationResult,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Purl(pub(crate) Uri);

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

#[derive(Debug, PartialEq, Eq)]
pub struct Uri(pub(crate) String);

impl Validate for Uri {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match self.0.parse::<http::Uri>() {
            Ok(_) => Ok(ValidationResult::Passed),
            Err(_) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Uri does not conform to ISO 8601".to_string(),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::validation::FailureReason;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_purls_should_pass_validation() {
        let validation_result = Purl(Uri("pkg:cargo/cyclonedx-bom@0.3.1".to_string()))
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_purls_should_fail_validation() {
        let validation_result = Purl(Uri("invalid purl".to_string()))
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
                    message: "Uri does not conform to ISO 8601".to_string(),
                    context: ValidationContext::default()
                }]
            }
        );
    }
}
