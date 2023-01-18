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

use std::collections::HashSet;
use std::fmt;

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use xml::{EmitterConfig, EventReader, EventWriter, ParserConfig};

use crate::models::component::{Component, Components};
use crate::models::composition::{BomReference, Compositions};
use crate::models::dependency::Dependencies;
use crate::models::external_reference::ExternalReferences;
use crate::models::metadata::Metadata;
use crate::models::property::Properties;
use crate::models::service::{Service, Services};
use crate::models::vulnerability::Vulnerabilities;
use crate::validation::{
    FailureReason, Validate, ValidationContext, ValidationError, ValidationPathComponent,
    ValidationResult,
};
use crate::xml::{FromXmlDocument, ToXml};

/// todo: derive(Eq)
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
    pub vulnerabilities: Option<Vulnerabilities>,
}

impl Bom {
    /// Parse the input as a JSON document conforming to [version 1.3 of the specification](https://cyclonedx.org/docs/1.3/json/)
    pub fn parse_from_json_v1_3<R: std::io::Read>(
        mut reader: R,
    ) -> Result<Self, crate::errors::JsonReadError> {
        let bom: crate::specs::v1_3::bom::Bom = serde_json::from_reader(&mut reader)?;
        Ok(bom.into())
    }

    /// Parse the input as a JSON document conforming to [version 1.3 of the specification](https://cyclonedx.org/docs/1.3/json/)
    /// from an existing [`Value`].
    pub fn parse_from_json_value_v1_3(value: Value) -> Result<Self, crate::errors::JsonReadError> {
        let bom: crate::specs::v1_3::bom::Bom = serde_json::from_value(value)?;
        Ok(bom.into())
    }

    /// Parse the input as an XML document conforming to [version 1.3 of the specification](https://cyclonedx.org/docs/1.3/xml/)
    pub fn parse_from_xml_v1_3<R: std::io::Read>(
        reader: R,
    ) -> Result<Self, crate::errors::XmlReadError> {
        let config = ParserConfig::default().trim_whitespace(true);
        let mut event_reader = EventReader::new_with_config(reader, config);
        let bom = crate::specs::v1_3::bom::Bom::read_xml_document(&mut event_reader)?;
        Ok(bom.into())
    }

    /// Output as a JSON document conforming to [version 1.3 of the specification](https://cyclonedx.org/docs/1.3/json/)
    pub fn output_as_json_v1_3<W: std::io::Write>(
        self,
        writer: &mut W,
    ) -> Result<(), crate::errors::JsonWriteError> {
        let bom: crate::specs::v1_3::bom::Bom = self.into();
        serde_json::to_writer_pretty(writer, &bom)?;
        Ok(())
    }

    /// Output as a JSON document conforming to [version 1.3 of the specification](https://cyclonedx.org/docs/1.3/json/)
    pub fn output_as_json_value_v1_3(self) -> Result<Value, crate::errors::JsonWriteError> {
        let bom: crate::specs::v1_3::bom::Bom = self.into();
        Ok(serde_json::to_value(bom)?)
    }

    /// Output as an XML document conforming to [version 1.3 of the specification](https://cyclonedx.org/docs/1.3/xml/)
    pub fn output_as_xml_v1_3<W: std::io::Write>(
        self,
        writer: &mut W,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let config = EmitterConfig::default().perform_indent(true);
        let mut event_writer = EventWriter::new_with_config(writer, config);

        let bom: crate::specs::v1_3::bom::Bom = self.into();
        bom.write_xml_element(&mut event_writer)
    }

    /// Parse the input as a JSON document conforming to [version 1.4 of the specification](https://cyclonedx.org/docs/1.4/json/)
    pub fn parse_from_json_v1_4<R: std::io::Read>(
        mut reader: R,
    ) -> Result<Self, crate::errors::JsonReadError> {
        let bom: crate::specs::v1_4::bom::Bom = serde_json::from_reader(&mut reader)?;
        Ok(bom.into())
    }

    /// Parse the input as an XML document conforming to [version 1.4 of the specification](https://cyclonedx.org/docs/1.4/xml/)
    pub fn parse_from_xml_v1_4<R: std::io::Read>(
        reader: R,
    ) -> Result<Self, crate::errors::XmlReadError> {
        let config = ParserConfig::default().trim_whitespace(true);
        let mut event_reader = EventReader::new_with_config(reader, config);
        let bom = crate::specs::v1_4::bom::Bom::read_xml_document(&mut event_reader)?;
        Ok(bom.into())
    }

    /// Output as a JSON document conforming to [version 1.4 of the specification](https://cyclonedx.org/docs/1.4/json/)
    pub fn output_as_json_v1_4<W: std::io::Write>(
        self,
        writer: &mut W,
    ) -> Result<(), crate::errors::JsonWriteError> {
        let bom: crate::specs::v1_4::bom::Bom = self.into();
        serde_json::to_writer_pretty(writer, &bom)?;
        Ok(())
    }

    /// Output as an XML document conforming to [version 1.4 of the specification](https://cyclonedx.org/docs/1.4/xml/)
    pub fn output_as_xml_v1_4<W: std::io::Write>(
        self,
        writer: &mut W,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let config = EmitterConfig::default().perform_indent(true);
        let mut event_writer = EventWriter::new_with_config(writer, config);

        let bom: crate::specs::v1_4::bom::Bom = self.into();
        bom.write_xml_element(&mut event_writer)
    }
}

impl Default for Bom {
    /// Construct a BOM with a default `version` of `1` and `serial_number` with a random UUID
    fn default() -> Self {
        Self {
            version: 1,
            serial_number: Some(UrnUuid::generate()),
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: None,
            properties: None,
            vulnerabilities: None,
        }
    }
}

impl Validate for Bom {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        let mut bom_refs_context = BomReferencesContext::default();

        if let Some(serial_number) = &self.serial_number {
            let context = context.extend_context_with_struct_field("Bom", "serial_number");

            results.push(serial_number.validate_with_context(context)?);
        }

        if let Some(metadata) = &self.metadata {
            let context = context.extend_context_with_struct_field("Bom", "metadata");
            let component_bom_ref_context =
                context.extend_context_with_struct_field("Metadata", "component");

            results.push(metadata.validate_with_context(context)?);

            if let Some(component) = &metadata.component {
                validate_component_bom_refs(
                    component,
                    &mut bom_refs_context,
                    &component_bom_ref_context,
                    &mut results,
                );
            }
        }

        if let Some(components) = &self.components {
            let context = context.extend_context_with_struct_field("Bom", "components");
            let component_bom_ref_context = context.clone();

            results.push(components.validate_with_context(context)?);

            // record the component references
            validate_components(
                components,
                &mut bom_refs_context,
                &component_bom_ref_context,
                &mut results,
            );
        }

        if let Some(services) = &self.services {
            let context = context.extend_context_with_struct_field("Bom", "services");
            let service_bom_ref_context = context.clone();

            results.push(services.validate_with_context(context)?);

            // record the service references
            validate_services(
                services,
                &mut bom_refs_context,
                &service_bom_ref_context,
                &mut results,
            );
        }

        if let Some(external_references) = &self.external_references {
            let context = context.extend_context_with_struct_field("Bom", "external_references");

            results.push(external_references.validate_with_context(context)?);
        }

        if let Some(dependencies) = &self.dependencies {
            let context = context.extend_context_with_struct_field("Bom", "dependencies");

            for (dependency_index, dependency) in dependencies.0.iter().enumerate() {
                let context = context.extend_context(vec![ValidationPathComponent::Array {
                    index: dependency_index,
                }]);
                if !bom_refs_context.contains(&dependency.dependency_ref) {
                    let dependency_context =
                        context.extend_context_with_struct_field("Dependency", "dependency_ref");

                    results.push(ValidationResult::Failed {
                        reasons: vec![FailureReason {
                            message: "Dependency reference does not exist in the BOM".to_string(),
                            context: dependency_context,
                        }],
                    })
                }

                for (sub_dependency_index, sub_dependency) in
                    dependency.dependencies.iter().enumerate()
                {
                    if !bom_refs_context.contains(sub_dependency) {
                        let context = context.extend_context(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Dependency".to_string(),
                                field_name: "dependencies".to_string(),
                            },
                            ValidationPathComponent::Array {
                                index: sub_dependency_index,
                            },
                        ]);

                        results.push(ValidationResult::Failed {
                            reasons: vec![FailureReason {
                                message: "Dependency reference does not exist in the BOM"
                                    .to_string(),
                                context,
                            }],
                        })
                    }
                }
            }
        }

        if let Some(compositions) = &self.compositions {
            let context = context.extend_context_with_struct_field("Bom", "compositions");
            let compositions_context = context.clone();

            results.push(compositions.validate_with_context(context)?);

            for (composition_index, composition) in compositions.0.iter().enumerate() {
                let compositions_context =
                    compositions_context.extend_context(vec![ValidationPathComponent::Array {
                        index: composition_index,
                    }]);

                if let Some(assemblies) = &composition.assemblies {
                    let compositions_context = compositions_context
                        .extend_context_with_struct_field("Composition", "assemblies");
                    for (assembly_index, BomReference(assembly)) in assemblies.iter().enumerate() {
                        if !bom_refs_context.contains(assembly) {
                            let compositions_context = compositions_context.extend_context(vec![
                                ValidationPathComponent::Array {
                                    index: assembly_index,
                                },
                            ]);
                            results.push(ValidationResult::Failed {
                                reasons: vec![FailureReason {
                                    message: "Composition reference does not exist in the BOM"
                                        .to_string(),
                                    context: compositions_context,
                                }],
                            });
                        }
                    }
                }

                if let Some(dependencies) = &composition.dependencies {
                    let compositions_context = compositions_context
                        .extend_context_with_struct_field("Composition", "dependencies");
                    for (dependency_index, BomReference(dependency)) in
                        dependencies.iter().enumerate()
                    {
                        if !bom_refs_context.contains(dependency) {
                            let compositions_context = compositions_context.extend_context(vec![
                                ValidationPathComponent::Array {
                                    index: dependency_index,
                                },
                            ]);
                            results.push(ValidationResult::Failed {
                                reasons: vec![FailureReason {
                                    message: "Composition reference does not exist in the BOM"
                                        .to_string(),
                                    context: compositions_context,
                                }],
                            });
                        }
                    }
                }
            }
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

#[derive(Default)]
struct BomReferencesContext {
    component_bom_refs: HashSet<String>,
    service_bom_refs: HashSet<String>,
}

impl BomReferencesContext {
    fn contains(&self, bom_ref: &String) -> bool {
        self.component_bom_refs.contains(bom_ref) || self.service_bom_refs.contains(bom_ref)
    }

    fn add_component_bom_ref(&mut self, bom_ref: impl ToString) {
        self.component_bom_refs.insert(bom_ref.to_string());
    }

    fn add_service_bom_ref(&mut self, bom_ref: impl ToString) {
        self.service_bom_refs.insert(bom_ref.to_string());
    }
}

fn validate_component_bom_refs(
    component: &Component,
    bom_refs: &mut BomReferencesContext,
    context: &ValidationContext,
    results: &mut Vec<ValidationResult>,
) {
    if let Some(bom_ref) = &component.bom_ref {
        if bom_refs.contains(bom_ref) {
            let context = context.extend_context_with_struct_field("Component", "bom_ref");
            results.push(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: format!(r#"Bom ref "{bom_ref}" is not unique"#),
                    context,
                }],
            });
        }
        bom_refs.add_component_bom_ref(bom_ref);
    }

    if let Some(components) = &component.components {
        let context = context.extend_context_with_struct_field("Component", "components");
        validate_components(components, bom_refs, &context, results);
    }
}

fn validate_components(
    components: &Components,
    bom_refs: &mut BomReferencesContext,
    context: &ValidationContext,
    results: &mut Vec<ValidationResult>,
) {
    // record the component references
    for (component_index, component) in components.0.iter().enumerate() {
        let context = context.extend_context(vec![ValidationPathComponent::Array {
            index: component_index,
        }]);

        validate_component_bom_refs(component, bom_refs, &context, results);
    }
}

fn validate_service_bom_refs(
    service: &Service,
    bom_refs: &mut BomReferencesContext,
    context: &ValidationContext,
    results: &mut Vec<ValidationResult>,
) {
    if let Some(bom_ref) = &service.bom_ref {
        if bom_refs.contains(bom_ref) {
            let context = context.extend_context_with_struct_field("Service", "bom_ref");
            results.push(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: format!(r#"Bom ref "{bom_ref}" is not unique"#),
                    context,
                }],
            });
        }
        bom_refs.add_service_bom_ref(bom_ref);
    }

    if let Some(services) = &service.services {
        let context = context.extend_context_with_struct_field("Service", "services");
        validate_services(services, bom_refs, &context, results);
    }
}

fn validate_services(
    services: &Services,
    bom_refs: &mut BomReferencesContext,
    context: &ValidationContext,
    results: &mut Vec<ValidationResult>,
) {
    // record the service references
    for (service_index, service) in services.0.iter().enumerate() {
        let context = context.extend_context(vec![ValidationPathComponent::Array {
            index: service_index,
        }]);

        validate_service_bom_refs(service, bom_refs, &context, results);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UrnUuid(pub(crate) String);

impl UrnUuid {
    pub fn new(value: String) -> Result<Self, UrnUuidError> {
        match matches_urn_uuid_regex(&value) {
            Ok(true) => Ok(Self(value)),
            Ok(false) | Err(_) => Err(UrnUuidError::InvalidUrnUuid(
                "UrnUuid does not match regular expression".to_string(),
            )),
        }
    }

    pub fn generate() -> Self {
        Self::from(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for UrnUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<uuid::Uuid> for UrnUuid {
    fn from(uuid: uuid::Uuid) -> Self {
        Self(format!("urn:uuid:{}", uuid))
    }
}

impl Validate for UrnUuid {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match matches_urn_uuid_regex(&self.0) {
            Ok(true) => Ok(ValidationResult::Passed),
            Ok(false) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "UrnUuid does not match regular expression".to_string(),
                    context,
                }],
            }),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UrnUuidError {
    InvalidUrnUuid(String),
}

fn matches_urn_uuid_regex(value: &str) -> Result<bool, regex::Error> {
    static UUID_REGEX: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| {
        Regex::new(r"^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
    });

    UUID_REGEX
        .as_ref()
        .map(|regex| regex.is_match(value))
        .map_err(Clone::clone)
}

#[cfg(test)]
mod test {
    use crate::{
        external_models::{
            date_time::DateTime,
            normalized_string::NormalizedString,
            uri::Uri},
        models::{
            component::{Classification, Component},
            composition::{AggregateType, BomReference, Composition},
            dependency::Dependency,
            external_reference::{ExternalReference, ExternalReferenceType},
            property::Property,
            service::Service,
            vulnerability::Vulnerability,
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
            vulnerabilities: None,
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
            vulnerabilities: None,
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
    fn it_should_validate_broken_composition_refs_as_failed() {
        let bom = Bom {
            version: 1,
            serial_number: None,
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: Some(Compositions(vec![Composition {
                aggregate: AggregateType::Complete,
                assemblies: Some(vec![BomReference("assembly".to_string())]),
                dependencies: Some(vec![BomReference("dependencies".to_string())]),
            }])),
            properties: None,
            vulnerabilities: None,
        };

        let actual = bom.validate().expect("Failed to validate bom");

        assert_eq!(
            actual,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message: "Composition reference does not exist in the BOM".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "compositions".to_string(),
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Composition".to_string(),
                                field_name: "assemblies".to_string(),
                            },
                            ValidationPathComponent::Array { index: 0 },
                        ])
                    },
                    FailureReason {
                        message: "Composition reference does not exist in the BOM".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "compositions".to_string(),
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Composition".to_string(),
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
                bom_ref: Some("dependency".to_string()),
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
            vulnerabilities: Some(Vulnerabilities(vec![Vulnerability {
                bom_ref: None,
                id: None,
                vulnerability_source: None,
                vulnerability_references: None,
                vulnerability_ratings: None,
                cwes: None,
                description: None,
                detail: None,
                recommendation: None,
                advisories: None,
                created: None,
                published: None,
                updated: None,
                vulnerability_credits: None,
                tools: None,
                vulnerability_analysis: None,
                vulnerability_targets: None,
                properties: None,
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
                        message: "DateTime does not conform to ISO 8601".to_string(),
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
        let component_builder = |bom_ref: &str| {
            Component::new(
                Classification::Library,
                "lib-x",
                "v0.1.0",
                Some(bom_ref.to_string()),
            )
        };
        let mut component_with_sub_components = component_builder("subcomponent-component");
        component_with_sub_components.components = Some(Components(vec![component_builder(
            "subcomponent-component",
        )]));

        let service_builder = |bom_ref: &str| Service::new("service-x", Some(bom_ref.to_string()));
        let mut service_with_sub_services = service_builder("subservice-service");
        service_with_sub_services.services =
            Some(Services(vec![service_builder("subservice-service")]));

        let validation_result = Bom {
            version: 1,
            serial_number: None,
            metadata: Some(Metadata {
                timestamp: None,
                tools: None,
                authors: None,
                component: Some(component_builder("metadata-component")),
                manufacture: None,
                supplier: None,
                licenses: None,
                properties: None,
            }),
            components: Some(Components(vec![
                component_builder("metadata-component"),
                component_builder("component-component"),
                component_builder("component-component"),
                component_with_sub_components,
                component_builder("component-service"),
            ])),
            services: Some(Services(vec![
                service_builder("service-service"),
                service_builder("service-service"),
                service_with_sub_services,
                service_builder("component-service"),
            ])),
            external_references: None,
            dependencies: None,
            compositions: None,
            properties: None,
            vulnerabilities: None,
        }
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message: r#"Bom ref "metadata-component" is not unique"#.to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "components".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "bom_ref".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: r#"Bom ref "component-component" is not unique"#.to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "components".to_string()
                            },
                            ValidationPathComponent::Array { index: 2 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "bom_ref".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: r#"Bom ref "subcomponent-component" is not unique"#.to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "components".to_string()
                            },
                            ValidationPathComponent::Array { index: 3 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "components".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Component".to_string(),
                                field_name: "bom_ref".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: r#"Bom ref "service-service" is not unique"#.to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "services".to_string()
                            },
                            ValidationPathComponent::Array { index: 1 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "bom_ref".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: r#"Bom ref "subservice-service" is not unique"#.to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "services".to_string()
                            },
                            ValidationPathComponent::Array { index: 2 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "services".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "bom_ref".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: r#"Bom ref "component-service" is not unique"#.to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "Bom".to_string(),
                                field_name: "services".to_string()
                            },
                            ValidationPathComponent::Array { index: 3 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "bom_ref".to_string()
                            },
                        ])
                    },
                ]
            },
        );
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
