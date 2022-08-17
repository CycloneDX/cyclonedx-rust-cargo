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

use crate::{
    external_models::normalized_string::NormalizedString,
    validation::{
        Validate, ValidationContext, ValidationError, ValidationPathComponent, ValidationResult,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct Properties(pub(crate) Vec<Property>);

impl Validate for Properties {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, property) in self.0.iter().enumerate() {
            let property_context =
                context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(property.validate_with_context(property_context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub value: NormalizedString,
}

impl Validate for Property {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        let value_context = context.extend_context_with_struct_field("Property", "value");

        results.push(self.value.validate_with_context(value_context)?);

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::validation::FailureReason;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Properties(vec![Property {
            name: "property name".to_string(),
            value: NormalizedString("property value".to_string()),
        }])
        .validate()
        .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Properties(vec![Property {
            name: "property name".to_string(),
            value: NormalizedString("spaces and \ttabs".to_string()),
        }])
        .validate()
        .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        .to_string(),
                    context: ValidationContext(vec![
                        ValidationPathComponent::Array { index: 0 },
                        ValidationPathComponent::Struct {
                            struct_name: "Property".to_string(),
                            field_name: "value".to_string(),
                        },
                    ]),
                }],
            }
        );
    }
}
