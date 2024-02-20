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
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use xml::{EmitterConfig, EventReader, EventWriter, ParserConfig};

use crate::errors::BomError;
use crate::models::component::{Component, Components};
use crate::models::composition::Compositions;
use crate::models::dependency::Dependencies;
use crate::models::external_reference::ExternalReferences;
use crate::models::metadata::Metadata;
use crate::models::property::Properties;
use crate::models::service::{Service, Services};
use crate::models::signature::Signature;
use crate::models::vulnerability::Vulnerabilities;
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};
use crate::xml::{FromXmlDocument, ToXml};

use super::composition::BomReference;

/// Represents the spec version of a BOM.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum SpecVersion {
    #[serde(rename = "1.3")]
    V1_3,
    #[serde(rename = "1.4")]
    V1_4,
}

impl Default for SpecVersion {
    fn default() -> Self {
        Self::V1_3
    }
}

impl FromStr for SpecVersion {
    type Err = BomError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "1.3" => Ok(SpecVersion::V1_3),
            "1.4" => Ok(SpecVersion::V1_4),
            s => Err(BomError::UnsupportedSpecVersion(s.to_string())),
        }
    }
}

impl ToString for SpecVersion {
    fn to_string(&self) -> String {
        let s = match self {
            SpecVersion::V1_3 => "1.3",
            SpecVersion::V1_4 => "1.4",
        };
        s.to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Added in version 1.4
    pub vulnerabilities: Option<Vulnerabilities>,
    /// Added in version 1.4
    pub signature: Option<Signature>,
}

impl Bom {
    /// General function to parse a JSON file, fetches the `specVersion` field first then applies the right conversion.
    pub fn parse_from_json<R: std::io::Read>(
        mut reader: R,
    ) -> Result<Self, crate::errors::JsonReadError> {
        let json: serde_json::Value = serde_json::from_reader(&mut reader)?;

        if let Some(version) = json.get("specVersion") {
            let version = version
                .as_str()
                .ok_or_else(|| BomError::UnsupportedSpecVersion(version.to_string()))?;

            match SpecVersion::from_str(version)? {
                SpecVersion::V1_3 => Ok(crate::specs::v1_3::bom::Bom::deserialize(json)?.into()),
                SpecVersion::V1_4 => Ok(crate::specs::v1_4::bom::Bom::deserialize(json)?.into()),
            }
        } else {
            Err(BomError::UnsupportedSpecVersion("No field 'specVersion' found".to_string()).into())
        }
    }

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
        let bom: crate::specs::v1_3::bom::Bom = self.try_into()?;
        serde_json::to_writer_pretty(writer, &bom)?;
        Ok(())
    }

    /// Output as an XML document conforming to [version 1.3 of the specification](https://cyclonedx.org/docs/1.3/xml/)
    pub fn output_as_xml_v1_3<W: std::io::Write>(
        self,
        writer: &mut W,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let config = EmitterConfig::default().perform_indent(true);
        let mut event_writer = EventWriter::new_with_config(writer, config);

        let bom: crate::specs::v1_3::bom::Bom = self.try_into()?;
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
            signature: None,
        }
    }
}

impl Validate for Bom {
    fn validate(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new()
            .add_field_option(
                "serial_number",
                self.serial_number.as_ref(),
                validate_urn_uuid,
            )
            .add_struct_option("metadata", self.metadata.as_ref(), version)
            .add_struct_option(
                "external_references",
                self.external_references.as_ref(),
                version,
            )
            .add_struct_option("properties", self.properties.as_ref(), version)
            .add_struct_option("vulnerabilities", self.vulnerabilities.as_ref(), version);

        // To keep track of all Bom references inside.
        let mut bom_refs = BomReferencesContext::default();

        if let Some(metadata) = &self.metadata {
            if let Some(component) = &metadata.component {
                context = validate_component_bom_refs(context, &mut bom_refs, component);
            }
        }

        if let Some(components) = &self.components {
            context = validate_components(context, &mut bom_refs, components);
        }

        if let Some(services) = &self.services {
            context = validate_services(context, &mut bom_refs, services);
        }

        // Check dependencies & sub dependencies
        if let Some(dependencies) = &self.dependencies {
            for dependency in &dependencies.0 {
                if !bom_refs.contains(&dependency.dependency_ref) {
                    context = context.add_custom(
                        "dependency_ref",
                        format!(
                            "Dependency ref '{}' does not exist in the BOM",
                            dependency.dependency_ref
                        ),
                    );
                }

                for sub_dependency in &dependency.dependencies {
                    if !bom_refs.contains(sub_dependency) {
                        context = context.add_custom(
                            "sub dependency_ref",
                            format!(
                                "Dependency ref '{}' does not exist in the BOM",
                                sub_dependency
                            ),
                        );
                    }
                }
            }
        }

        // Check compositions, its dependencies & assemblies
        if let Some(compositions) = &self.compositions {
            for composition in &compositions.0 {
                if let Some(assemblies) = &composition.assemblies {
                    for BomReference(assembly) in assemblies {
                        if !bom_refs.contains(assembly) {
                            context = context.add_custom(
                                "composition ref",
                                format!(
                                    "Composition reference '{assembly}' does not exist in the BOM"
                                ),
                            );
                        }
                    }
                }

                if let Some(dependencies) = &composition.dependencies {
                    for BomReference(dependency) in dependencies {
                        if !bom_refs.contains(&dependency) {
                            context = context.add_custom(
                                "composition ref",
                                format!(
                                    "Composition reference '{dependency}' does not exist in the BOM"
                                ),
                            );
                        }
                    }
                }
            }
        }

        context.into()
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

/// Validates the Bom references.
fn validate_component_bom_refs(
    mut context: ValidationContext,
    bom_refs: &mut BomReferencesContext,
    component: &Component,
) -> ValidationContext {
    if let Some(bom_ref) = &component.bom_ref {
        if bom_refs.contains(&bom_ref) {
            context =
                context.add_custom("bom_ref", format!(r#"Bom ref "{bom_ref}" is not unique"#));
        }
        bom_refs.add_component_bom_ref(bom_ref);
    }

    if let Some(components) = &component.components {
        context = validate_components(context, bom_refs, components);
    }

    context
}

fn validate_components(
    mut context: ValidationContext,
    bom_refs: &mut BomReferencesContext,
    components: &Components,
) -> ValidationContext {
    for component in &components.0 {
        context = validate_component_bom_refs(context, bom_refs, component);
    }
    context
}

fn validate_services(
    mut context: ValidationContext,
    bom_refs: &mut BomReferencesContext,
    services: &Services,
) -> ValidationContext {
    for service in &services.0 {
        context = validate_service_bom_refs(context, bom_refs, service);
    }
    context
}

fn validate_service_bom_refs(
    mut context: ValidationContext,
    bom_refs: &mut BomReferencesContext,
    service: &Service,
) -> ValidationContext {
    if let Some(bom_ref) = &service.bom_ref {
        if bom_refs.contains(bom_ref) {
            context =
                context.add_custom("bom_ref", format!(r#"Bom ref "{bom_ref}" is not unique"#));
        }
        bom_refs.add_service_bom_ref(bom_ref);
    }

    if let Some(services) = &service.services {
        context = validate_services(context, bom_refs, services);
    }

    context
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UrnUuid(pub(crate) String);

impl UrnUuid {
    pub fn new(value: String) -> Result<Self, UrnUuidError> {
        match matches_urn_uuid_regex(&value) {
            true => Ok(Self(value)),
            false => Err(UrnUuidError::InvalidUrnUuid(
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

/// Validates a given [`UrnUuid`].
pub fn validate_urn_uuid(urn_uuid: &UrnUuid) -> Result<(), ValidationError> {
    if !matches_urn_uuid_regex(&urn_uuid.0) {
        return Err("UrnUuid does not match regular expression".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UrnUuidError {
    InvalidUrnUuid(String),
}

fn matches_urn_uuid_regex(value: &str) -> bool {
    static UUID_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
            .expect("Failed to compile regex.")
    });
    UUID_REGEX.is_match(value)
}

#[cfg(test)]
mod test {
    use crate::{
        external_models::{date_time::DateTime, normalized_string::NormalizedString, uri::Uri},
        models::{
            component::{Classification, Component},
            composition::{AggregateType, BomReference, Composition},
            dependency::Dependency,
            external_reference::{ExternalReference, ExternalReferenceType},
            property::Property,
            service::Service,
            vulnerability::Vulnerability,
        },
        validation,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_parse_json_using_function_without_suffix() {
        let input = r#"{
            "bomFormat": "CycloneDX",
            "specVersion": "1.3",
            "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
            "version": 1,
            "components": []
        }"#;
        let result = Bom::parse_from_json(input.as_bytes());
        assert!(result.is_ok());
    }

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
            signature: None,
        };

        let actual = bom.validate_default();

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
            signature: None,
        };

        let actual = bom.validate_default();

        assert_eq!(
            actual.errors(),
            Some(
                vec![
                    validation::custom(
                        "dependency ref",
                        "Dependency reference does not exist in the BOM"
                    ),
                    validation::custom(
                        "dependency ref",
                        "Dependency reference does not exist in the BOM"
                    ),
                ]
                .into()
            )
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
                signature: None,
            }])),
            properties: None,
            vulnerabilities: None,
            signature: None,
        };

        let actual = bom.validate(SpecVersion::V1_3);

        assert_eq!(
            actual.errors(),
            Some(validation::list(
                "compositions",
                &[(
                    0,
                    vec![
                        validation::custom(
                            "composition ref",
                            "Composition reference 'abc' does not exist in the BOM"
                        ),
                        validation::custom(
                            "composition ref",
                            "Composition reference 'abc' does not exist in the BOM"
                        )
                    ]
                )]
            ))
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
                signature: None,
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
                signature: None,
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
                signature: None,
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
            signature: None,
        };

        let actual = bom.validate_default();

        /*
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
        */
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
            signature: None,
        }
        .validate_default();

        /*
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
        */
    }

    #[test]
    fn valid_uuids_should_pass_validation() {
        let validation_result = validate_urn_uuid(&UrnUuid::from(uuid::Uuid::new_v4()));

        assert!(validation_result.is_ok());
    }

    #[test]
    fn invalid_uuids_should_fail_validation() {
        let validation_result = validate_urn_uuid(&UrnUuid("invalid uuid".to_string()));

        assert_eq!(
            validation_result,
            Err("UrnUuid does not match regular expression".into()),
        );
    }
}
