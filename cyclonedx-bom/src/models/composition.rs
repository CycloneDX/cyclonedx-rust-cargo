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

use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

use super::{
    bom::{BomReference, SpecVersion},
    signature::Signature,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    pub bom_ref: Option<BomReference>,
    pub aggregate: AggregateType,
    pub assemblies: Option<Vec<BomReference>>,
    pub dependencies: Option<Vec<BomReference>>,
    pub vulnerabilities: Option<Vec<BomReference>>,
    pub signature: Option<Signature>,
}

impl Validate for Composition {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("aggregate", &self.aggregate, |at| {
                validate_aggregate_type(at, version)
            })
            .add_struct_option("signature", self.signature.as_ref(), version)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Compositions(pub Vec<Composition>);

impl Validate for Compositions {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("composition", &self.0, |composition| {
                composition.validate_version(version)
            })
            .into()
    }
}

/// Validates the given [`AggregateType`].
pub fn validate_aggregate_type(
    aggregate_type: &AggregateType,
    version: SpecVersion,
) -> Result<(), ValidationError> {
    if version <= SpecVersion::V1_4 {
        if AggregateType::IncompleteFirstPartyProprietaryOnly < *aggregate_type {
            return Err("Unknown aggregate type".into());
        }
    } else if version <= SpecVersion::V1_5
        && matches!(aggregate_type, AggregateType::UnknownAggregateType(_))
    {
        return Err(ValidationError::new("Unknown aggregate type"));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[repr(u16)]
pub enum AggregateType {
    Complete = 1,
    Incomplete = 2,
    IncompleteFirstPartyOnly = 3,
    IncompleteThirdPartyOnly = 4,
    Unknown = 5,
    NotSpecified = 6,
    /// Added in 1.5
    IncompleteFirstPartyProprietaryOnly = 7,
    /// Added in 1.5
    IncompleteFirstPartyOpensourceOnly = 8,
    /// Added in 1.5
    IncompleteThirdPartyProprietaryOnly = 9,
    /// Added in 1.5
    IncompleteThirdPartyOpensourceOnly = 10,
    #[doc(hidden)]
    #[strum(default)]
    UnknownAggregateType(String),
}

impl AggregateType {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "complete" => Self::Complete,
            "incomplete" => Self::Incomplete,
            "incomplete_first_party_only" => Self::IncompleteFirstPartyOnly,
            "incomplete_first_party_propprietary_only" => Self::IncompleteFirstPartyProprietaryOnly,
            "incomplete_first_party_opensource_only" => Self::IncompleteFirstPartyOpensourceOnly,
            "incomplete_third_party_only" => Self::IncompleteThirdPartyOnly,
            "incomplete_third_party_proprietary_only" => Self::IncompleteThirdPartyProprietaryOnly,
            "incomplete_third_party_opensource_only" => Self::IncompleteThirdPartyOpensourceOnly,
            "unknown" => Self::Unknown,
            "not_specified" => Self::NotSpecified,
            unknown => Self::UnknownAggregateType(unknown.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{models::signature::Algorithm, validation};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Compositions(vec![Composition {
            bom_ref: Some(BomReference::new("composition-1")),
            aggregate: AggregateType::Complete,
            assemblies: Some(vec![BomReference::new("assembly-ref")]),
            dependencies: Some(vec![BomReference::new("dependency-ref")]),
            vulnerabilities: Some(vec![BomReference::new("vulnerability-ref")]),
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
        }])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Compositions(vec![Composition {
            bom_ref: Some(BomReference::new("composition-1")),
            aggregate: AggregateType::UnknownAggregateType("unknown aggregate type".to_string()),
            assemblies: Some(vec![BomReference::new("assembly-ref")]),
            dependencies: Some(vec![BomReference::new("dependency-ref")]),
            vulnerabilities: Some(vec![BomReference::new("vulnerability-ref")]),
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
        }])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "composition",
                [(
                    0,
                    validation::r#field("aggregate", "Unknown aggregate type")
                )]
            )
        );
    }
}
