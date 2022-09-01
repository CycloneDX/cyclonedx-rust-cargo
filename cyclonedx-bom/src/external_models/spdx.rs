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

use spdx::{Expression, ParseMode};
use thiserror::Error;

use crate::validation::{FailureReason, Validate, ValidationResult};

#[derive(Debug, PartialEq, Eq)]
pub struct SpdxIdentifier(pub(crate) String);

impl Validate for SpdxIdentifier {
    fn validate_with_context(
        &self,
        context: crate::validation::ValidationContext,
    ) -> Result<ValidationResult, crate::validation::ValidationError> {
        match spdx::license_id(&self.0) {
            Some(_) => Ok(ValidationResult::Passed),
            None => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "SPDX identifier is not valid".to_string(),
                    context,
                }],
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SpdxExpression(pub(crate) String);

impl SpdxExpression {
    pub fn parse_lax(value: String) -> Result<Self, SpdxExpressionError> {
        match Expression::parse_mode(&value, ParseMode::LAX) {
            Ok(_) => Self(value).convert_lax(),
            Err(e) => Err(SpdxExpressionError::InvalidLaxSpdxExpression(format!(
                "{}",
                e.reason
            ))),
        }
    }

    fn convert_lax(self) -> Result<Self, SpdxExpressionError> {
        let converted = self.0.replace('/', " OR ");

        match Self::try_from(converted) {
            Ok(converted) => Ok(converted),
            Err(e) => Err(SpdxExpressionError::InvalidLaxSpdxExpression(format!(
                "{}",
                e
            ))),
        }
    }
}

impl TryFrom<String> for SpdxExpression {
    type Error = SpdxExpressionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match Expression::parse(&value) {
            Ok(_) => Ok(Self(value)),
            Err(e) => Err(SpdxExpressionError::InvalidSpdxExpression(format!(
                "{}",
                e.reason
            ))),
        }
    }
}

impl Validate for SpdxExpression {
    fn validate_with_context(
        &self,
        context: crate::validation::ValidationContext,
    ) -> Result<crate::validation::ValidationResult, crate::validation::ValidationError> {
        match SpdxExpression::try_from(self.0.clone()) {
            Ok(_) => Ok(ValidationResult::Passed),
            Err(_) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "SPDX expression is not valid".to_string(),
                    context,
                }],
            }),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpdxExpressionError {
    #[error("Invalid SPDX expression: {}", .0)]
    InvalidSpdxExpression(String),

    #[error("Invalid Lax SPDX expression: {}", .0)]
    InvalidLaxSpdxExpression(String),
}

#[cfg(test)]
mod test {
    use crate::validation::{FailureReason, ValidationContext, ValidationResult};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_spdx_identifiers_should_pass_validation() {
        let validation_result = SpdxIdentifier("MIT".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_spdx_identifiers_should_fail_validation() {
        let validation_result = SpdxIdentifier("MIT OR Apache-2.0".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "SPDX identifier is not valid".to_string(),
                    context: ValidationContext::default()
                }]
            }
        );
    }

    #[test]
    fn it_should_succeed_in_converting_an_spdx_expression() {
        let actual = SpdxExpression::try_from("MIT OR Apache-2.0".to_string())
            .expect("Failed to parse as a license");
        assert_eq!(actual, SpdxExpression("MIT OR Apache-2.0".to_string()));
    }

    #[test]
    fn it_should_succeed_in_converting_a_partially_valid_spdx_expression() {
        let actual = SpdxExpression::parse_lax("MIT/Apache-2.0".to_string())
            .expect("Failed to parse as a license");
        assert_eq!(actual, SpdxExpression("MIT OR Apache-2.0".to_string()));
    }

    #[test]
    fn it_should_fail_to_convert_an_invalid_spdx_expression() {
        let actual = SpdxExpression::try_from("not a real license".to_string())
            .expect_err("Should have failed to parse as a license");
        assert_eq!(
            actual,
            SpdxExpressionError::InvalidSpdxExpression("unknown term".to_string())
        );
    }

    #[test]
    fn valid_spdx_expressions_should_pass_validation() {
        let validation_result = SpdxExpression("MIT OR Apache-2.0".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_spdx_expressions_should_fail_validation() {
        let validation_result = SpdxExpression("not a real license".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "SPDX expression is not valid".to_string(),
                    context: ValidationContext::default()
                }]
            }
        );
    }
}
