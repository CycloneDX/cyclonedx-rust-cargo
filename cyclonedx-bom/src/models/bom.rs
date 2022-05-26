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
use xml::{EmitterConfig, EventReader, EventWriter, ParserConfig};

use crate::models::component::Components;
use crate::models::composition::Compositions;
use crate::models::dependency::Dependencies;
use crate::models::external_reference::ExternalReferences;
use crate::models::metadata::Metadata;
use crate::models::property::Properties;
use crate::models::service::Services;
use crate::validation::{
    FailureReason, Validate, ValidationContext, ValidationError, ValidationResult,
};
use crate::xml::{FromXmlDocument, ToXml};

#[derive(Debug, PartialEq)]
pub struct Bom {
    pub version: u32,
    pub serial_number: Option<UrnUuid>,
    pub metadata: Option<Metadata>,
    pub components: Option<Components>,
    pub services: Option<Services>,
    pub external_references: Option<ExternalReferences>,
    pub dependencies: Option<Dependencies>,
    pub compositions: Option<Compositions>,
    pub properties: Option<Properties>,
}

impl Bom {
    pub fn parse_from_json_v1_3<R: std::io::Read>(
        mut reader: R,
    ) -> Result<Self, crate::errors::JsonReadError> {
        let bom: crate::specs::v1_3::bom::Bom = serde_json::from_reader(&mut reader)?;
        Ok(bom.into())
    }

    pub fn parse_from_xml_v1_3<R: std::io::Read>(
        reader: R,
    ) -> Result<Self, crate::errors::XmlReadError> {
        let config = ParserConfig::default().trim_whitespace(true);
        let mut event_reader = EventReader::new_with_config(reader, config);
        let bom = crate::specs::v1_3::bom::Bom::read_xml_document(&mut event_reader)?;
        Ok(bom.into())
    }

    pub fn output_as_json_v1_3<W: std::io::Write>(
        self,
        writer: &mut W,
    ) -> Result<(), crate::errors::JsonWriteError> {
        let bom: crate::specs::v1_3::bom::Bom = self.into();
        serde_json::to_writer_pretty(writer, &bom)?;
        Ok(())
    }

    pub fn output_as_xml_v1_3<W: std::io::Write>(
        self,
        writer: &mut W,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let config = EmitterConfig::default().perform_indent(true);
        let mut event_writer = EventWriter::new_with_config(writer, config);

        let bom: crate::specs::v1_3::bom::Bom = self.into();
        bom.write_xml_element(&mut event_writer)
    }
}

impl Default for Bom {
    fn default() -> Self {
        Self {
            version: 1,
            serial_number: Some(UrnUuid(format!("urn:uuid:{}", uuid::Uuid::new_v4()))),
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: None,
            properties: None,
        }
    }
}

impl Validate for Bom {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(serial_number) = &self.serial_number {
            let context = context.extend_context_with_struct_field("Bom", "serial_number");

            results.push(serial_number.validate_with_context(context)?);
        }

        if let Some(metadata) = &self.metadata {
            let context = context.extend_context_with_struct_field("Bom", "metadata");

            results.push(metadata.validate_with_context(context)?);
        }

        if let Some(components) = &self.components {
            let context = context.extend_context_with_struct_field("Bom", "components");

            results.push(components.validate_with_context(context)?);
        }

        if let Some(services) = &self.services {
            let context = context.extend_context_with_struct_field("Bom", "services");

            results.push(services.validate_with_context(context)?);
        }

        if let Some(external_references) = &self.external_references {
            let context = context.extend_context_with_struct_field("Bom", "external_references");

            results.push(external_references.validate_with_context(context)?);
        }

        /* TODO validate dependency references appear in the components or services
        if let Some(dependencies) = &self.dependencies {
            let context = context.extend_context_with_struct_field("Bom", "dependencies");

            results.push(dependencies.validate_with_context(context)?);
        }
        */

        if let Some(compositions) = &self.compositions {
            let context = context.extend_context_with_struct_field("Bom", "compositions");

            results.push(compositions.validate_with_context(context)?);
        }

        if let Some(properties) = &self.properties {
            let context = context.extend_context_with_struct_field("Bom", "properties");

            results.push(properties.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq)]
pub struct UrnUuid(pub(crate) String);

impl Validate for UrnUuid {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        static UUID_REGEX: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| {
            Regex::new(r"^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
        });

        match UUID_REGEX.as_ref() {
            Ok(regex) => {
                if regex.is_match(&self.0) {
                    Ok(ValidationResult::Passed)
                } else {
                    Ok(ValidationResult::Failed {
                        reasons: vec![FailureReason {
                            message: "UrnUuid does not match regular expression".to_string(),
                            context,
                        }],
                    })
                }
            }
            Err(e) => Err(e.clone().into()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        external_models::{date_time::DateTime, normalized_string::NormalizedString, uri::Uri},
        models::{
            component::{Classification, Component},
            composition::{AggregateType, Composition},
            dependency::Dependency,
            external_reference::{ExternalReference, ExternalReferenceType},
            property::Property,
            service::Service,
        },
        validation::ValidationPathComponent,
    };

    use super::*;
    use pretty_assertions::assert_eq;
    
    #[test]
    fn it_should_validate_an_empty_bom_as_passed() {
        let bom = Bom {
            version: 1,
            serial_number: None,
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: None,
            properties: None,
        };

        let actual = bom
            .validate_with_context(ValidationContext::default())
            .expect("Failed to validate bom");

        assert_eq!(actual, ValidationResult::Passed);
    }

    #[test]
    fn it_should_validate_broken_dependency_refs_as_failed() {
        let bom = Bom {
            version: 1,
            serial_number: None,
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: Some(Dependencies(vec![Dependency {
                dependency_ref: "dependency".to_string(),
                dependencies: vec!["sub-dependency".to_string()],
            }])),
            compositions: None,
            properties: None,
        };

        let actual = bom.validate().expect("Failed to validate bom");

        assert_eq!(
            actual,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message: "Dependency reference does not exist in the BOM".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "dependencies".to_string(),
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Dependency".to_string(),
                                field_name: "dependency_ref".to_string(),
                            },
                        ])
                    },
                    FailureReason {
                        message: "Dependency reference does not exist in the BOM".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "dependencies".to_string(),
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Dependency".to_string(),
                                field_name: "dependencies".to_string(),
                            },
                            ValidationPathComponent::Array { index: 0 },
                        ])
                    },
                ]
            }
        );
    }

    #[test]
    fn it_should_validate_a_bom_with_multiple_validation_issues_as_failed() {
        let bom = Bom {
            version: 1,
            serial_number: Some(UrnUuid("invalid uuid".to_string())),
            metadata: Some(Metadata {
                timestamp: Some(DateTime("invalid datetime".to_string())),
                tools: None,
                authors: None,
                component: None,
                manufacture: None,
                supplier: None,
                licenses: None,
                properties: None,
            }),
            components: Some(Components(vec![Component {
                component_type: Classification::UnknownClassification("unknown".to_string()),
                mime_type: None,
                bom_ref: None,
                supplier: None,
                author: None,
                publisher: None,
                group: None,
                name: NormalizedString::new("name"),
                version: NormalizedString::new("version"),
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
            }])),
            services: Some(Services(vec![Service {
                bom_ref: None,
                provider: None,
                group: None,
                name: NormalizedString("invalid\tname".to_string()),
                version: None,
                description: None,
                endpoints: None,
                authenticated: None,
                x_trust_boundary: None,
                data: None,
                licenses: None,
                external_references: None,
                properties: None,
                services: None,
            }])),
            external_references: Some(ExternalReferences(vec![ExternalReference {
                external_reference_type: ExternalReferenceType::UnknownExternalReferenceType(
                    "unknown".to_string(),
                ),
                url: Uri("https://example.com".to_string()),
                comment: None,
                hashes: None,
            }])),
            dependencies: Some(Dependencies(vec![Dependency {
                dependency_ref: "dependency".to_string(),
                dependencies: vec![],
            }])),
            compositions: Some(Compositions(vec![Composition {
                aggregate: AggregateType::UnknownAggregateType("unknown".to_string()),
                assemblies: None,
                dependencies: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString("invalid\tvalue".to_string()),
            }])),
        };

        let actual = bom
            .validate_with_context(ValidationContext::default())
            .expect("Failed to validate bom");

        assert_eq!(
            actual,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message: "UrnUuid does not match regular expression".to_string(),
                        context: ValidationContext(vec![ValidationPathComponent::Struct {
                            struct_name: "Bom".to_string(),
                            field_name: "serial_number".to_string()
                        }])
                    },
                    FailureReason {
                        message: "DateTime does not conform to RFC 3339".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "metadata".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "Metadata".to_string(),
                                field_name: "timestamp".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown classification".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "components".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
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
                                struct_name: "Bom".to_string(),
                                field_name: "services".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "name".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown external reference type".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "external_references".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "ExternalReference".to_string(),
                                field_name: "external_reference_type".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "Unknown aggregate type".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "compositions".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Composition".to_string(),
                                field_name: "aggregate".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
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
        )
    }

    #[test]
    fn it_should_validate_that_bom_references_are_unique() {
        todo!()
    }

    #[test]
    fn valid_uuids_should_pass_validation() {
        let validation_result = UrnUuid(format!("urn:uuid:{}", uuid::Uuid::new_v4()))
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_uuids_should_fail_validation() {
        let validation_result = UrnUuid("invalid uuid".to_string())
            .validate_with_context(ValidationContext::default())
            .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "UrnUuid does not match regular expression".to_string(),
                    context: ValidationContext::default()
                }]
            }
        );
    }
}
