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

use crate::validation::{
    FailureReason, Validate, ValidationContext, ValidationPathComponent, ValidationResult,
};

use super::signature::Signature;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    pub aggregate: AggregateType,
    pub assemblies: Option<Vec<BomReference>>,
    pub dependencies: Option<Vec<BomReference>>,
    pub signature: Option<Signature>,
}

impl Validate for Composition {
    fn validate_with_context(&self, context: ValidationContext) -> ValidationResult {
        let mut results: Vec<ValidationResult> = vec![];

        let aggregate_context =
            context.extend_context_with_struct_field("Composition", "aggregate");

        results.push(self.aggregate.validate_with_context(aggregate_context));

        results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Compositions(pub Vec<Composition>);

impl Validate for Compositions {
    fn validate_with_context(&self, context: ValidationContext) -> ValidationResult {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, composition) in self.0.iter().enumerate() {
            let composition_context =
                context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(composition.validate_with_context(composition_context));
        }

        results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AggregateType {
    Complete,
    Incomplete,
    IncompleteFirstPartyOnly,
    IncompleteThirdPartyOnly,
    Unknown,
    NotSpecified,
    #[doc(hidden)]
    UnknownAggregateType(String),
}

impl ToString for AggregateType {
    fn to_string(&self) -> String {
        match self {
            AggregateType::Complete => "complete",
            AggregateType::Incomplete => "incomplete",
            AggregateType::IncompleteFirstPartyOnly => "incomplete_first_party_only",
            AggregateType::IncompleteThirdPartyOnly => "incomplete_third_party_only",
            AggregateType::Unknown => "unknown",
            AggregateType::NotSpecified => "not_specified",
            AggregateType::UnknownAggregateType(uat) => uat,
        }
        .to_string()
    }
}

impl AggregateType {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "complete" => Self::Complete,
            "incomplete" => Self::Incomplete,
            "incomplete_first_party_only" => Self::IncompleteFirstPartyOnly,
            "incomplete_third_party_only" => Self::IncompleteThirdPartyOnly,
            "unknown" => Self::Unknown,
            "not_specified" => Self::NotSpecified,
            unknown => Self::UnknownAggregateType(unknown.to_string()),
        }
    }
}

impl Validate for AggregateType {
    fn validate_with_context(&self, context: ValidationContext) -> ValidationResult {
        match self {
            AggregateType::UnknownAggregateType(_) => ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Unknown aggregate type".to_string(),
                    context,
                }],
            },
            _ => ValidationResult::Passed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BomReference(pub(crate) String);

#[cfg(test)]
mod test {
    use crate::models::signature::Algorithm;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Compositions(vec![Composition {
            aggregate: AggregateType::Complete,
            assemblies: Some(vec![BomReference("reference".to_string())]),
            dependencies: Some(vec![BomReference("reference".to_string())]),
            signature: Some(Signature {
                algorithm: Algorithm::HS512,
                value: "abcdefgh".to_string(),
            }),
        }])
        .validate();

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Compositions(vec![Composition {
            aggregate: AggregateType::UnknownAggregateType("unknown aggregate type".to_string()),
            assemblies: Some(vec![BomReference("reference".to_string())]),
            dependencies: Some(vec![BomReference("reference".to_string())]),
            signature: Some(Signature {
                algorithm: Algorithm::HS512,
                value: "abcdefgh".to_string(),
            }),
        }])
        .validate();

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Unknown aggregate type".to_string(),
                    context: ValidationContext(vec![
                        ValidationPathComponent::Array { index: 0 },
                        ValidationPathComponent::Struct {
                            struct_name: "Composition".to_string(),
                            field_name: "aggregate".to_string()
                        }
                    ])
                }]
            }
        );
    }
}
