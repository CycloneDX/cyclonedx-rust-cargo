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

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    models::{
        bom::{validate_bom_ref, BomReference, SpecVersion},
        external_reference::ExternalReferences,
        patent::PatentDefinition,
        property::Properties,
        signature::Signature,
    },
    validation::{Validate, ValidationContext, ValidationError, ValidationResult},
};

/// A collection of reusable objects that are defined and may be used elsewhere in the BOM.
///
/// Defined via the [CycloneDX JSON schema](https://cyclonedx.org/docs/1.6/json/#definitions)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Definitions {
    pub standards: Option<Vec<Standard>>,
    /// Added in version 1.7
    pub patents: Option<Vec<PatentDefinition>>,
}

impl Validate for Definitions {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("standards", self.standards.as_ref(), |standard| {
                standard.validate_version(version)
            })
            .add_list_option("patents", self.patents.as_ref(), |patent| {
                patent.validate_version(version)
            })
            .into()
    }
}

/// A standard may consist of regulations, industry or organizational-specific standards,
/// maturity models, best practices, or any other requirements which can be evaluated against or
/// attested to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Standard {
    pub bom_ref: Option<BomReference>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub requirements: Option<Vec<Requirement>>,
    pub levels: Option<Vec<Level>>,
    pub external_references: Option<ExternalReferences>,
    pub signature: Option<Signature>,
}

impl Validate for Standard {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("bom-ref", self.bom_ref.as_ref(), |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .add_list_option("requirements", self.requirements.as_ref(), |requirement| {
                requirement.validate_version(version)
            })
            .add_list_option("levels", self.levels.as_ref(), |level| {
                level.validate_version(version)
            })
            .add_struct_option(
                "external_references",
                self.external_references.as_ref(),
                version,
            )
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Requirement {
    pub bom_ref: Option<BomReference>,
    pub identifier: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub descriptions: Option<Vec<String>>,
    /// OWASP OpenCRE identifiers, e.g. `CRE:764-507`
    pub open_cre: Option<Vec<String>>,
    pub parent: Option<BomReference>,
    pub properties: Option<Properties>,
    pub external_references: Option<ExternalReferences>,
}

impl Validate for Requirement {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("bom-ref", self.bom_ref.as_ref(), |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .add_list_option("open_cre", self.open_cre.as_ref(), |cre| {
                validate_open_cre(cre)
            })
            .add_struct_option("properties", self.properties.as_ref(), version)
            .add_struct_option(
                "external_references",
                self.external_references.as_ref(),
                version,
            )
            .into()
    }
}

fn validate_open_cre(cre: &str) -> Result<(), ValidationError> {
    static OPEN_CRE_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^CRE:[0-9]+-[0-9]+$").expect("Failed to compile regex."));

    if !OPEN_CRE_REGEX.is_match(cre) {
        return Err(ValidationError::new(
            "OpenCRE identifier does not match regular expression",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Level {
    pub bom_ref: Option<BomReference>,
    pub identifier: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    /// The `bom-ref`s of the requirements that comprise the level.
    pub requirements: Option<Vec<BomReference>>,
}

impl Validate for Level {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("bom-ref", self.bom_ref.as_ref(), |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .into()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::validation;

    #[test]
    fn invalid_open_cre_should_fail_validation() {
        let definitions = Definitions {
            standards: Some(vec![Standard {
                bom_ref: Some(BomReference::new("standard-1")),
                name: Some("Sample Standard".to_string()),
                version: None,
                description: None,
                owner: None,
                requirements: Some(vec![Requirement {
                    bom_ref: Some(BomReference::new("requirement-1")),
                    identifier: Some("v1".to_string()),
                    title: None,
                    text: None,
                    descriptions: None,
                    open_cre: Some(vec!["CRE:616-305".to_string(), "616-305".to_string()]),
                    parent: None,
                    properties: None,
                    external_references: None,
                }]),
                levels: None,
                external_references: None,
                signature: None,
            }]),
            patents: None,
        };

        let result = definitions.validate_version(SpecVersion::V1_6);

        assert_eq!(
            result,
            validation::list(
                "standards",
                [(
                    0,
                    validation::list(
                        "requirements",
                        [(
                            0,
                            validation::list(
                                "open_cre",
                                [(
                                    1,
                                    validation::custom(
                                        "",
                                        ["OpenCRE identifier does not match regular expression"]
                                    )
                                )]
                            )
                        )]
                    )
                )]
            )
        );
    }
}
