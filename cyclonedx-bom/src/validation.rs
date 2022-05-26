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
    fn validate(&self) -> Result<ValidationResult, ValidationError> {
        self.validate_with_context(ValidationContext::default())
    }

    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError>;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValidationContext(pub(crate) Vec<ValidationPathComponent>);

impl ValidationContext {
    pub(crate) fn extend_context(&self, components: Vec<ValidationPathComponent>) -> Self {
        let mut extended_context = self.0.clone();
        extended_context.extend(components);
        Self(extended_context)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(unused)] // TODO: remove
pub enum ValidationPathComponent {
    StructComponent {
        struct_name: String,
        field_name: String,
    },
    ArrayComponent {
        index: u32,
    },
    TupleComponent {
        index: u32,
    },
}

#[derive(Debug, PartialEq)]
pub enum ValidationResult {
    Passed,
    Failed { reasons: Vec<FailureReason> },
}



#[derive(Clone, Debug, PartialEq)]
pub struct FailureReason {
    pub message: String,
    pub context: ValidationContext,
}
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ValidationError {
    #[error("Failed to compile regular expression: {0}")]
    InvalidRegularExpressionError(#[from] regex::Error),
}
