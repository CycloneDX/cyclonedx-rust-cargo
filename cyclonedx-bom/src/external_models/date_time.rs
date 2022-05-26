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

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::validation::{
    FailureReason, Validate, ValidationContext, ValidationError, ValidationResult,
};

#[derive(Debug, PartialEq)]
pub struct DateTime(pub(crate) String);

impl Validate for DateTime {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match OffsetDateTime::parse(&self.0.to_string(), &Rfc3339) {
            Ok(_) => Ok(ValidationResult::Passed),
            Err(_) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "DateTime does not conform to RFC 3339".to_string(),
                    context,
                }],
            }),
        }
    }
}

impl ToString for DateTime {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::validation::FailureReason;

    #[test]
    fn valid_datetimes_should_pass_validation() {
        let validation_result = DateTime("1969-06-28T01:20:00.00-04:00".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed)
    }

    #[test]
    fn invalid_datetimes_should_fail_validation() {
        let validation_result = DateTime("invalid date".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "DateTime does not conform to RFC 3339".to_string(),
                    context: ValidationContext::default()
                }]
            }
        )
    }
}
