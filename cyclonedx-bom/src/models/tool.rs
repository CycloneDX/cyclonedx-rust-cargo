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

use crate::external_models::normalized_string::NormalizedString;
use crate::models::hash::Hashes;
use crate::validation::{
    Validate, ValidationContext, ValidationError, ValidationPathComponent, ValidationResult,
};

#[derive(Debug, PartialEq)]
pub struct Tool {
    pub vendor: Option<NormalizedString>,
    pub name: Option<NormalizedString>,
    pub version: Option<NormalizedString>,
    pub hashes: Option<Hashes>,
}

impl Validate for Tool {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(vendor) = &self.vendor {
            let context = context.extend_context_with_struct_field("Tool", "vendor");

            results.push(vendor.validate_with_context(context)?);
        }

        if let Some(name) = &self.name {
            let context = context.extend_context_with_struct_field("Tool", "name");

            results.push(name.validate_with_context(context)?);
        }

        if let Some(version) = &self.version {
            let context = context.extend_context_with_struct_field("Tool", "version");

            results.push(version.validate_with_context(context)?);
        }

        if let Some(hashes) = &self.hashes {
            let context = context.extend_context_with_struct_field("Tool", "hashes");

            results.push(hashes.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq)]
pub struct Tools(pub Vec<Tool>);

impl Validate for Tools {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, tool) in self.0.iter().enumerate() {
            let tool_context =
                context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(tool.validate_with_context(tool_context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[cfg(test)]
mod test {
    use crate::validation::FailureReason;

    use super::*;
    use pretty_assertions::assert_eq;
    
    #[test]
    fn it_should_pass_validation() {
        let validation_result = Tools(vec![Tool {
            vendor: Some(NormalizedString("no_whitespace".to_string())),
            name: None,
            version: None,
            hashes: None,
        }])
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Tools(vec![Tool {
            vendor: Some(NormalizedString("spaces and\ttabs".to_string())),
            name: None,
            version: None,
            hashes: None,
        }])
        .validate_with_context(ValidationContext::default())
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
                            struct_name: "Tool".to_string(),
                            field_name: "vendor".to_string(),
                        }
                    ])
                }]
            }
        );
    }

    #[test]
    fn it_should_merge_validations_correctly() {
        let validation_result = Tools(vec![
            Tool {
                vendor: Some(NormalizedString("no_whitespace".to_string())),
                name: None,
                version: None,
                hashes: None,
            },
            Tool {
                vendor: Some(NormalizedString("spaces and\ttabs".to_string())),
                name: None,
                version: None,
                hashes: None,
            },
            Tool {
                vendor: None,
                name: Some(NormalizedString("spaces and\ttabs".to_string())),
                version: None,
                hashes: None,
            },
        ])
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 1 },
                            ValidationPathComponent::Struct {
                                struct_name: "Tool".to_string(),
                                field_name: "vendor".to_string(),
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 2 },
                            ValidationPathComponent::Struct {
                                struct_name: "Tool".to_string(),
                                field_name: "name".to_string(),
                            }
                        ])
                    }
                ]
            }
        );
    }
}
