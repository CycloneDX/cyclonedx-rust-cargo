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
    external_models::date_time::{validate_date_time, DateTime},
    models::{bom::BomReference, signature::Signature},
    prelude::{Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

use super::bom::SpecVersion;

/// A collection of attributions indicating which entity supplied information for specific fields
/// within the BOM.
///
/// Defined via the [CycloneDX JSON schema](https://cyclonedx.org/docs/1.7/json/#citations).
/// Added in 1.7
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Citations(pub Vec<Citation>);

impl Validate for Citations {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_unique_list("inner", &self.0, |citation| {
                citation.validate_version(version)
            })
            .into()
    }
}

/// Details a specific attribution of data within the BOM to a contributing entity or process.
///
/// Exactly one of `pointers` or `expressions` must be set, and at least one of `attributed_to`
/// or `process` must be set.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Citation {
    pub bom_ref: Option<BomReference>,
    /// JSON Pointers (RFC 6901) identifying the BOM fields to which the attribution applies.
    pub pointers: Option<Vec<String>>,
    /// Path expressions (JSONPath for JSON, XPath for XML) locating values within the BOM.
    pub expressions: Option<Vec<String>>,
    pub timestamp: DateTime,
    /// The `bom-ref` of the entity that supplied the cited information.
    pub attributed_to: Option<BomReference>,
    /// The `bom-ref` of a formulation process that executed or generated the attributed data.
    pub process: Option<BomReference>,
    pub note: Option<String>,
    pub signature: Option<Signature>,
}

impl Validate for Citation {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();
        context
            .add_field("timestamp", &self.timestamp, validate_date_time)
            .add_field_option("pointers", self.pointers.as_deref(), validate_non_empty)
            .add_field_option(
                "expressions",
                self.expressions.as_deref(),
                validate_non_empty,
            )
            .add_struct_option("signature", self.signature.as_ref(), version);

        if self.pointers.is_some() == self.expressions.is_some() {
            context.add_custom(
                "pointers",
                ValidationError::new("Exactly one of pointers or expressions must be present"),
            );
        }
        if self.attributed_to.is_none() && self.process.is_none() {
            context.add_custom(
                "attributed_to",
                ValidationError::new("At least one of attributedTo or process must be present"),
            );
        }

        context.into()
    }
}

fn validate_non_empty(list: &[String]) -> Result<(), ValidationError> {
    if list.is_empty() {
        return Err(ValidationError::new("Must contain at least one item"));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::validation;
    use pretty_assertions::assert_eq;

    fn valid_citation() -> Citation {
        Citation {
            bom_ref: Some(BomReference::new("citation-1")),
            pointers: Some(vec!["/components/0/name".to_string()]),
            expressions: None,
            timestamp: DateTime("2025-05-01T14:00:00Z".to_string()),
            attributed_to: Some(BomReference::new("person-1")),
            process: None,
            note: None,
            signature: None,
        }
    }

    #[test]
    fn valid_citations_should_pass_validation() {
        let result = Citations(vec![valid_citation()]).validate_version(SpecVersion::V1_7);
        assert!(result.passed());
    }

    #[test]
    fn citation_without_target_or_source_should_fail_validation() {
        let citation = Citation {
            pointers: None,
            attributed_to: None,
            ..valid_citation()
        };

        let result = citation.validate_version(SpecVersion::V1_7);

        assert_eq!(
            result,
            vec![
                validation::custom(
                    "pointers",
                    ["Exactly one of pointers or expressions must be present"]
                ),
                validation::custom(
                    "attributed_to",
                    ["At least one of attributedTo or process must be present"]
                ),
            ]
            .into()
        );
    }

    #[test]
    fn citation_with_both_pointers_and_expressions_should_fail_validation() {
        let citation = Citation {
            expressions: Some(vec!["$.components[0].name".to_string()]),
            ..valid_citation()
        };

        let result = citation.validate_version(SpecVersion::V1_7);

        assert_eq!(
            result,
            validation::custom(
                "pointers",
                ["Exactly one of pointers or expressions must be present"]
            )
        );
    }

    #[test]
    fn citation_with_empty_pointers_should_fail_validation() {
        let citation = Citation {
            pointers: Some(vec![]),
            ..valid_citation()
        };

        let result = citation.validate_version(SpecVersion::V1_7);

        assert_eq!(
            result,
            validation::field("pointers", "Must contain at least one item")
        );
    }
}
