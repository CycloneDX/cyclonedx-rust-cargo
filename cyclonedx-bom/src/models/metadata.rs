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

use thiserror::Error;

use crate::external_models::date_time::{DateTime, DateTimeError};
use crate::models::component::Component;
use crate::models::license::Licenses;
use crate::models::organization::{OrganizationalContact, OrganizationalEntity};
use crate::models::property::Properties;
use crate::models::tool::Tools;
use crate::validation::{
    Validate, ValidationContext, ValidationError, ValidationPathComponent, ValidationResult,
};

/// Represents additional information about a BOM
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_metadata)
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    pub timestamp: Option<DateTime>,
    pub tools: Option<Tools>,
    pub authors: Option<Vec<OrganizationalContact>>,
    pub component: Option<Component>,
    pub manufacture: Option<OrganizationalEntity>,
    pub supplier: Option<OrganizationalEntity>,
    pub licenses: Option<Licenses>,
    pub properties: Option<Properties>,
}

impl Metadata {
    /// Constructs a new `Metadata` with a timestamp based on the current time
    /// ```
    /// use cyclonedx_bom::models::metadata::{Metadata, MetadataError};
    ///
    /// let metadata = Metadata::new()?;
    /// # Ok::<(), MetadataError>(())
    /// ```
    /// # Errors
    ///
    /// Returns an error variant if unable to generate a valid timestamp
    pub fn new() -> Result<Self, MetadataError> {
        match DateTime::now() {
            Ok(timestamp) => Ok(Self {
                timestamp: Some(timestamp),
                ..Default::default()
            }),
            Err(e) => Err(MetadataError::InvalidTimestamp(e)),
        }
    }
}

impl Validate for Metadata {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(timestamp) = &self.timestamp {
            let context = context.extend_context_with_struct_field("Metadata", "timestamp");

            results.push(timestamp.validate_with_context(context)?);
        }

        if let Some(tools) = &self.tools {
            let context = context.extend_context_with_struct_field("Metadata", "tools");

            results.push(tools.validate_with_context(context)?);
        }

        if let Some(authors) = &self.authors {
            for (index, contact) in authors.iter().enumerate() {
                let uri_context = context.extend_context(vec![
                    ValidationPathComponent::Struct {
                        struct_name: "Metadata".to_string(),
                        field_name: "authors".to_string(),
                    },
                    ValidationPathComponent::Array { index },
                ]);
                results.push(contact.validate_with_context(uri_context)?);
            }
        }

        if let Some(component) = &self.component {
            let context = context.extend_context_with_struct_field("Metadata", "component");

            results.push(component.validate_with_context(context)?);
        }

        if let Some(manufacture) = &self.manufacture {
            let context = context.extend_context_with_struct_field("Metadata", "manufacture");

            results.push(manufacture.validate_with_context(context)?);
        }

        if let Some(supplier) = &self.supplier {
            let context = context.extend_context_with_struct_field("Metadata", "supplier");

            results.push(supplier.validate_with_context(context)?);
        }

        if let Some(licenses) = &self.licenses {
            let context = context.extend_context_with_struct_field("Metadata", "licenses");

            results.push(licenses.validate_with_context(context)?);
        }

        if let Some(properties) = &self.properties {
            let context = context.extend_context_with_struct_field("Metadata", "properties");

            results.push(properties.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MetadataError {
    #[error("Invalid timestamp")]
    InvalidTimestamp(#[from] DateTimeError),
}

#[cfg(test)]
mod test {
    use crate::{
        external_models::{normalized_string::NormalizedString, spdx::SpdxExpression},
        models::{
            component::Classification, license::LicenseChoice, property::Property, tool::Tool,
        },
        validation::FailureReason,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_metadata_should_pass_validation() {
        let validation_result = Metadata {
            timestamp: Some(DateTime("1969-06-28T01:20:00.00-04:00".to_string())),
            tools: Some(Tools(vec![Tool {
                vendor: Some(NormalizedString::new("vendor")),
                name: None,
                version: None,
                hashes: None,
            }])),
            authors: Some(vec![OrganizationalContact {
                name: Some(NormalizedString::new("name")),
                email: None,
                phone: None,
            }]),
            component: Some(Component {
                component_type: Classification::Application,
                mime_type: None,
                bom_ref: None,
                supplier: None,
                author: None,
                publisher: None,
                group: None,
                name: NormalizedString::new("name"),
                version: Some(NormalizedString::new("version")),
                description: None,
                scope: None,
                hashes: None,
                licenses: None,
                copyright: None,
                cpe: None,
                purl: None,
                swid: None,
                modified: None,
                pedigree: None,
                external_references: None,
                properties: None,
                components: None,
                evidence: None,
            }),
            manufacture: Some(OrganizationalEntity {
                name: Some(NormalizedString::new("name")),
                url: None,
                contact: None,
            }),
            supplier: Some(OrganizationalEntity {
                name: Some(NormalizedString::new("name")),
                url: None,
                contact: None,
            }),
            licenses: Some(Licenses(LicenseChoice::Expressions(vec![SpdxExpression(
                "MIT".to_string(),
            )]))),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString::new("value"),
            }])),
        }
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_metadata_should_fail_validation() {
        let validation_result = Metadata {
            timestamp: Some(DateTime("invalid date".to_string())),
            tools: Some(Tools(vec![Tool {
                vendor: Some(NormalizedString("invalid\tvendor".to_string())),
                name: None,
                version: None,
                hashes: None,
            }])),
            authors: Some(vec![OrganizationalContact {
                name: Some(NormalizedString("invalid\tname".to_string())),
                email: None,
                phone: None,
            }]),
            component: Some(Component {
                component_type: Classification::UnknownClassification("unknown".to_string()),
                mime_type: None,
                bom_ref: None,
                supplier: None,
                author: None,
                publisher: None,
                group: None,
                name: NormalizedString::new("name"),
                version: Some(NormalizedString::new("version")),
                description: None,
                scope: None,
                hashes: None,
                licenses: None,
                copyright: None,
                cpe: None,
                purl: None,
                swid: None,
                modified: None,
                pedigree: None,
                external_references: None,
                properties: None,
                components: None,
                evidence: None,
            }),
            manufacture: Some(OrganizationalEntity {
                name: Some(NormalizedString("invalid\tname".to_string())),
                url: None,
                contact: None,
            }),
            supplier: Some(OrganizationalEntity {
                name: Some(NormalizedString("invalid\tname".to_string())),
                url: None,
                contact: None,
            }),
            licenses: Some(Licenses(LicenseChoice::Expressions(vec![SpdxExpression(
                "invalid license".to_string(),
            )]))),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString("invalid\tvalue".to_string()),
            }])),
        }
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message: "DateTime does not conform to ISO 8601".to_string(),
                        context: ValidationContext(vec![ValidationPathComponent::Struct {
                            struct_name: "Metadata".to_string(),
                            field_name: "timestamp".to_string()
                        }])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Metadata".to_string(),
                                field_name: "tools".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Tool".to_string(),
                                field_name: "vendor".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Metadata".to_string(),
                                field_name: "authors".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "OrganizationalContact".to_string(),
                                field_name: "name".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown classification".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Metadata".to_string(),
                                field_name: "component".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "component_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Metadata".to_string(),
                                field_name: "manufacture".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "OrganizationalEntity".to_string(),
                                field_name: "name".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Metadata".to_string(),
                                field_name: "supplier".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "OrganizationalEntity".to_string(),
                                field_name: "name".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "SPDX expression is not valid".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Metadata".to_string(),
                                field_name: "licenses".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::EnumVariant {
                                variant_name: "Expression".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Metadata".to_string(),
                                field_name: "properties".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Property".to_string(),
                                field_name: "value".to_string()
                            }
                        ])
                    },
                ]
            }
        );
    }
}
