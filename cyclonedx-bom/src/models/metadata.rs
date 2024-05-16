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
use crate::external_models::validate_date_time;
use crate::models::component::Component;
use crate::models::license::Licenses;
use crate::models::lifecycle::Lifecycles;
use crate::models::organization::{OrganizationalContact, OrganizationalEntity};
use crate::models::property::Properties;
use crate::models::tool::Tools;
use crate::validation::{Validate, ValidationContext, ValidationResult};

use super::bom::SpecVersion;

/// Represents additional information about a BOM
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_metadata)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    pub timestamp: Option<DateTime>,
    pub tools: Option<Tools>,
    pub authors: Option<Vec<OrganizationalContact>>,
    pub component: Option<Component>,
    pub manufacture: Option<OrganizationalEntity>,
    pub supplier: Option<OrganizationalEntity>,
    pub licenses: Option<Licenses>,
    pub properties: Option<Properties>,
    /// Added in 1.5
    pub lifecycles: Option<Lifecycles>,
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
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("timestamp", self.timestamp.as_ref(), validate_date_time)
            .add_list("tools", self.tools.as_ref(), |tools| {
                tools.validate_version(version)
            })
            .add_list_option("authors", self.authors.as_ref(), |author| {
                author.validate_version(version)
            })
            .add_struct_option("component", self.component.as_ref(), version)
            .add_struct_option("manufacture", self.manufacture.as_ref(), version)
            .add_struct_option("supplier", self.supplier.as_ref(), version)
            .add_list("licenses", self.licenses.as_ref(), |license| {
                license.validate_version(version)
            })
            .add_list("properties", self.properties.as_ref(), |property| {
                property.validate_version(version)
            })
            .into()
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
            bom::BomReference,
            component::Classification,
            license::LicenseChoice,
            lifecycle::{Description, Lifecycle, Phase},
            property::Property,
            tool::Tool,
        },
        validation,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_metadata_should_pass_validation() {
        let validation_result = Metadata {
            timestamp: Some(DateTime("1969-06-28T01:20:00.00-04:00".to_string())),
            tools: Some(Tools::List(vec![Tool {
                vendor: Some(NormalizedString::new("vendor")),
                name: None,
                version: None,
                hashes: None,
                external_references: None,
            }])),
            authors: Some(vec![OrganizationalContact {
                bom_ref: None,
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
                signature: None,
                model_card: None,
                data: None,
            }),
            manufacture: Some(OrganizationalEntity {
                bom_ref: Some(BomReference::new("Manufacturer")),
                name: Some(NormalizedString::new("name")),
                url: None,
                contact: None,
            }),
            supplier: Some(OrganizationalEntity {
                bom_ref: Some(BomReference::new("Supplier")),
                name: Some(NormalizedString::new("name")),
                url: None,
                contact: None,
            }),
            licenses: Some(Licenses(vec![LicenseChoice::Expression(
                SpdxExpression::new("MIT"),
            )])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString::new("value"),
            }])),
            lifecycles: Some(Lifecycles(vec![Lifecycle::Phase(Phase::Build)])),
        }
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn invalid_metadata_should_fail_validation() {
        let validation_result = Metadata {
            timestamp: Some(DateTime("invalid date".to_string())),
            tools: Some(Tools::List(vec![Tool {
                vendor: Some(NormalizedString("invalid\tvendor".to_string())),
                name: None,
                version: None,
                hashes: None,
                external_references: None,
            }])),
            authors: Some(vec![OrganizationalContact {
                bom_ref: None,
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
                signature: None,
                model_card: None,
                data: None,
            }),
            manufacture: Some(OrganizationalEntity {
                bom_ref: Some(BomReference::new("Manufacturer")),
                name: Some(NormalizedString("invalid\tname".to_string())),
                url: None,
                contact: None,
            }),
            supplier: Some(OrganizationalEntity {
                bom_ref: Some(BomReference::new("Supplier")),
                name: Some(NormalizedString("invalid\tname".to_string())),
                url: None,
                contact: None,
            }),
            licenses: Some(Licenses(vec![LicenseChoice::Expression(
                SpdxExpression::new("invalid license"),
            )])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString("invalid\tvalue".to_string()),
            }])),
            lifecycles: Some(Lifecycles(vec![Lifecycle::Description(Description {
                name: "lifecycle".into(),
                description: Some(NormalizedString("invalid\tvalue".to_string())),
            })])),
        }
        .validate();

        assert_eq!(
            validation_result,
            vec![
                validation::field("timestamp", "DateTime does not conform to ISO 8601"),
                validation::list(
                    "tools",
                    [(
                        0,
                        validation::list(
                            "inner",
                            [(
                                0,
                                validation::field(
                                    "vendor",
                                    "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                )
                            )]
                        )
                    )]
                ),
                validation::list(
                    "authors",
                    [(
                        0,
                        validation::field(
                            "name",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        )
                    )]
                ),
                validation::r#struct(
                    "component",
                    validation::field("component_type", "Unknown classification")
                ),
                validation::r#struct(
                    "manufacture",
                    validation::field(
                        "name",
                        "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                    )
                ),
                validation::r#struct(
                    "supplier",
                    validation::field(
                        "name",
                        "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                    )
                ),
                validation::list(
                    "licenses",
                    [(
                        0,
                        validation::list(
                            "inner",
                            [(
                                0,
                                validation::r#enum("expression", "SPDX expression is not valid")
                            )]
                        )
                    )]
                ),
                validation::list(
                    "properties",
                    [(
                        0,
                        validation::list(
                            "inner",
                            [(
                                0,
                                validation::field(
                                    "value",
                                    "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                )
                            )]
                        )
                    )]
                )
            ]
            .into()
        );
    }
}
