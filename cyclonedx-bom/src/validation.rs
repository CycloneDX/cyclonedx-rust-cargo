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

pub trait Validate {
    fn validate(&self) -> ValidationResult {
        self.validate_with_context(ValidationContext::default())
    }

    fn validate_with_context(&self, context: ValidationContext) -> ValidationResult;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationContext(pub(crate) Vec<ValidationPathComponent>);

#[allow(dead_code)]
impl ValidationContext {
    pub(crate) fn new() -> Self {
        ValidationContext::default()
    }

    pub(crate) fn extend_context(&self, components: Vec<ValidationPathComponent>) -> Self {
        let mut extended_context = self.0.clone();
        extended_context.extend(components);
        Self(extended_context)
    }

    /// Extends the [`ValidationContext`] with a struct field.
    pub(crate) fn with_struct(
        &self,
        struct_name: impl ToString,
        field_name: impl ToString,
    ) -> Self {
        let component = vec![ValidationPathComponent::Struct {
            struct_name: struct_name.to_string(),
            field_name: field_name.to_string(),
        }];
        self.extend_context(component)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationPathComponent {
    Struct {
        struct_name: String,
        field_name: String,
    },
    Array {
        index: usize,
    },
    EnumVariant {
        variant_name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationResult {
    Passed,
    Failed { reasons: Vec<FailureReason> },
}

impl ValidationResult {
    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Passed, Self::Passed) => Self::Passed,
            (Self::Passed, Self::Failed { reasons }) => Self::Failed { reasons },
            (Self::Failed { reasons }, Self::Passed) => Self::Failed { reasons },
            (
                Self::Failed {
                    reasons: mut left_reasons,
                },
                Self::Failed {
                    reasons: mut right_reasons,
                },
            ) => {
                left_reasons.append(&mut right_reasons);
                Self::Failed {
                    reasons: left_reasons,
                }
            }
        }
    }

    /// Returns a [`ValidationResult::Failed`] with a single failure.
    pub fn failure(reason: &str, context: ValidationContext) -> Self {
        Self::Failed {
            reasons: vec![FailureReason::new(reason, context)],
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::Passed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureReason {
    pub message: String,
    pub context: ValidationContext,
}

impl FailureReason {
    pub fn new(message: &str, context: ValidationContext) -> Self {
        Self {
            message: message.to_string(),
            context,
        }
    }
}
