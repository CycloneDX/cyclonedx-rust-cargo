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
use crate::models::annotation::Annotations;
use crate::models::citation::Citations;
use crate::models::component::{Component, Components};
use crate::models::composition::Compositions;
use crate::models::declarations::Declarations;
use crate::models::definitions::Definitions;
use crate::models::dependency::Dependencies;
use crate::models::external_reference::ExternalReferences;
use crate::models::formulation::Formula;
use crate::models::metadata::Metadata;
use crate::models::property::Properties;
use crate::models::service::{Service, Services};
use crate::models::signature::Signature;
use crate::models::vulnerability::Vulnerabilities;
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};
use crate::xml::{FromXmlDocument, ToXml};

use super::vulnerability::Vulnerability;

/// Represents the spec version of a BOM.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, PartialOrd, strum::Display)]
pub enum SpecVersion {
    #[strum(to_string = "1.3")]
    #[serde(rename = "1.3")]
    V1_3 = 1,
    #[strum(to_string = "1.4")]
    #[serde(rename = "1.4")]
    V1_4 = 2,
    #[strum(to_string = "1.5")]
    #[serde(rename = "1.5")]
    V1_5 = 3,
    #[strum(to_string = "1.6")]
    #[serde(rename = "1.6")]
    V1_6 = 4,
    #[strum(to_string = "1.7")]
    #[serde(rename = "1.7")]
    V1_7 = 5,
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
            "1.5" => Ok(SpecVersion::V1_5),
            "1.6" => Ok(SpecVersion::V1_6),
            "1.7" => Ok(SpecVersion::V1_7),
            s => Err(BomError::UnsupportedSpecVersion(s.to_string())),
        }
    }
}

pub fn validate_bom_ref(
    _bom_ref: &BomReference,
    version: SpecVersion,
) -> Result<(), ValidationError> {
    if version <= SpecVersion::V1_4 {
        return Err("Attribute 'bom-ref' not supported in this format version".into());
    }
    Ok(())
}

/// A reference to a Bom element
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BomReference(pub String);

impl BomReference {
    pub fn new<T>(input: T) -> Self
    where
        T: ToString,
    {
        Self(input.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// Added in version 1.5
    pub annotations: Option<Annotations>,
    /// Added in version 1.5
    pub formulation: Option<Vec<Formula>>,
    /// Added in version 1.6
    pub declarations: Option<Declarations>,
    /// Added in version 1.6
    pub definitions: Option<Definitions>,
    /// Added in version 1.7
    pub citations: Option<Citations>,
    pub spec_version: SpecVersion,
}

impl Bom {
    /// General function to parse a JSON file, fetches the `specVersion` field first then applies the right conversion.
    pub fn parse_from_json<R: std::io::Read>(
        mut reader: R,
    ) -> Result<Self, crate::errors::JsonReadError> {
        Self::parse_json_value(serde_json::from_reader(&mut reader)?)
    }

    /// General function to parse a pre-parsed JSON file, fetches the `specVersion` field first,
    /// then applies the right conversion.
    pub fn parse_json_value(json: Value) -> Result<Self, crate::errors::JsonReadError> {
        if let Some(version) = json.get("specVersion") {
            let version = version
                .as_str()
                .ok_or_else(|| BomError::UnsupportedSpecVersion(version.to_string()))?;

            let version = SpecVersion::from_str(version)?;
            Self::parse_json_value_with_version(json, version)
        } else {
            Err(BomError::UnsupportedSpecVersion("No field 'specVersion' found".to_string()).into())
        }
    }

    /// Parse an existing [`Value`] using the specification version that you provide.
    /// Use [`parse_json_value`](Self::parse_json_value) to detect the version from the document instead.
    pub fn parse_json_value_with_version(
        value: Value,
        version: SpecVersion,
    ) -> Result<Self, crate::errors::JsonReadError> {
        match version {
            SpecVersion::V1_3 => Ok(crate::specs::v1_3::bom::Bom::deserialize(value)?.into()),
            SpecVersion::V1_4 => Ok(crate::specs::v1_4::bom::Bom::deserialize(value)?.into()),
            SpecVersion::V1_5 => Ok(crate::specs::v1_5::bom::Bom::deserialize(value)?.into()),
            SpecVersion::V1_6 => Ok(crate::specs::v1_6::bom::Bom::deserialize(value)?.into()),
            SpecVersion::V1_7 => Ok(crate::specs::v1_7::bom::Bom::deserialize(value)?.into()),
        }
    }

    /// Parse the input as a JSON document conforming to the version of the specification that you provide.
    /// Use [`parse_from_json`](Self::parse_from_json) if you want to support multiple versions instead.
    pub fn parse_from_json_with_version<R: std::io::Read>(
        reader: R,
        version: SpecVersion,
    ) -> Result<Self, crate::errors::JsonReadError> {
        match version {
            SpecVersion::V1_3 => {
                Ok(serde_json::from_reader::<_, crate::specs::v1_3::bom::Bom>(reader)?.into())
            }
            SpecVersion::V1_4 => {
                Ok(serde_json::from_reader::<_, crate::specs::v1_4::bom::Bom>(reader)?.into())
            }
            SpecVersion::V1_5 => {
                Ok(serde_json::from_reader::<_, crate::specs::v1_5::bom::Bom>(reader)?.into())
            }
            SpecVersion::V1_6 => {
                Ok(serde_json::from_reader::<_, crate::specs::v1_6::bom::Bom>(reader)?.into())
            }
            SpecVersion::V1_7 => {
                Ok(serde_json::from_reader::<_, crate::specs::v1_7::bom::Bom>(reader)?.into())
            }
        }
    }

    /// Output as a JSON document conforming to the specification version that you provide.
    pub fn output_as_json<W: std::io::Write>(
        self,
        writer: &mut W,
        version: SpecVersion,
    ) -> Result<(), crate::errors::JsonWriteError> {
        match version {
            SpecVersion::V1_3 => {
                let bom: crate::specs::v1_3::bom::Bom = self.try_into()?;
                serde_json::to_writer_pretty(writer, &bom)?;
            }
            SpecVersion::V1_4 => {
                let bom: crate::specs::v1_4::bom::Bom = self.try_into()?;
                serde_json::to_writer_pretty(writer, &bom)?;
            }
            SpecVersion::V1_5 => {
                let bom: crate::specs::v1_5::bom::Bom = self.try_into()?;
                serde_json::to_writer_pretty(writer, &bom)?;
            }
            SpecVersion::V1_6 => {
                let bom: crate::specs::v1_6::bom::Bom = self.try_into()?;
                serde_json::to_writer_pretty(writer, &bom)?;
            }
            SpecVersion::V1_7 => {
                let bom: crate::specs::v1_7::bom::Bom = self.try_into()?;
                serde_json::to_writer_pretty(writer, &bom)?;
            }
        }
        Ok(())
    }

    /// Parse the input as an XML document conforming to the version of the specification that you provide.
    pub fn parse_from_xml_with_version<R: std::io::Read>(
        reader: R,
        version: SpecVersion,
    ) -> Result<Self, crate::errors::XmlReadError> {
        let config = ParserConfig::default().trim_whitespace(true);
        let mut event_reader = EventReader::new_with_config(reader, config);
        match version {
            SpecVersion::V1_3 => {
                Ok(crate::specs::v1_3::bom::Bom::read_xml_document(&mut event_reader)?.into())
            }
            SpecVersion::V1_4 => {
                Ok(crate::specs::v1_4::bom::Bom::read_xml_document(&mut event_reader)?.into())
            }
            SpecVersion::V1_5 => {
                Ok(crate::specs::v1_5::bom::Bom::read_xml_document(&mut event_reader)?.into())
            }
            SpecVersion::V1_6 => {
                Ok(crate::specs::v1_6::bom::Bom::read_xml_document(&mut event_reader)?.into())
            }
            SpecVersion::V1_7 => {
                Ok(crate::specs::v1_7::bom::Bom::read_xml_document(&mut event_reader)?.into())
            }
        }
    }

    /// Output as an XML document conforming to the specification version that you provide.
    pub fn output_as_xml<W: std::io::Write>(
        self,
        writer: &mut W,
        version: SpecVersion,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let config = EmitterConfig::default().perform_indent(true);
        let mut event_writer = EventWriter::new_with_config(writer, config);
        match version {
            SpecVersion::V1_3 => {
                let bom: crate::specs::v1_3::bom::Bom = self.try_into()?;
                bom.write_xml_element(&mut event_writer)
            }
            SpecVersion::V1_4 => {
                let bom: crate::specs::v1_4::bom::Bom = self.try_into()?;
                bom.write_xml_element(&mut event_writer)
            }
            SpecVersion::V1_5 => {
                let bom: crate::specs::v1_5::bom::Bom = self.try_into()?;
                bom.write_xml_element(&mut event_writer)
            }
            SpecVersion::V1_6 => {
                let bom: crate::specs::v1_6::bom::Bom = self.try_into()?;
                bom.write_xml_element(&mut event_writer)
            }
            SpecVersion::V1_7 => {
                let bom: crate::specs::v1_7::bom::Bom = self.try_into()?;
                bom.write_xml_element(&mut event_writer)
            }
        }
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
            annotations: None,
            formulation: None,
            declarations: None,
            definitions: None,
            citations: None,
            spec_version: SpecVersion::V1_3,
        }
    }
}

impl Validate for Bom {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();
        context.add_field_option(
            "serial_number",
            self.serial_number.as_ref(),
            validate_urn_uuid,
        );
        context.add_struct_option("metadata", self.metadata.as_ref(), version);
        context.add_struct_option("components", self.components.as_ref(), version);
        context.add_struct_option("services", self.services.as_ref(), version);
        context.add_struct_option(
            "external_references",
            self.external_references.as_ref(),
            version,
        );
        context.add_struct_option("compositions", self.compositions.as_ref(), version);
        context.add_struct_option("properties", self.properties.as_ref(), version);
        context.add_struct_option("vulnerabilities", self.vulnerabilities.as_ref(), version);
        context.add_struct_option("citations", self.citations.as_ref(), version);
        context.add_struct_option("declarations", self.declarations.as_ref(), version);
        context.add_struct_option("definitions", self.definitions.as_ref(), version);

        // To keep track of all Bom references inside.
        let mut bom_refs = BomReferencesContext::default();

        if let Some(metadata) = &self.metadata {
            if let Some(component) = &metadata.component {
                validate_component_bom_refs(&mut context, &mut bom_refs, component);
            }
        }

        if let Some(components) = &self.components {
            validate_components(&mut context, &mut bom_refs, components);
        }

        if let Some(services) = &self.services {
            validate_services(&mut context, &mut bom_refs, services);
        }

        if let Some(vulnerabilities) = &self.vulnerabilities {
            validate_vulnerabilities(&mut context, &mut bom_refs, vulnerabilities);
        }

        // Check dependencies & sub dependencies
        if let Some(dependencies) = &self.dependencies {
            for dependency in &dependencies.0 {
                if !bom_refs.contains(&dependency.dependency_ref) {
                    context.add_custom(
                        "dependency_ref",
                        format!(
                            "Dependency ref '{}' does not exist in the BOM",
                            dependency.dependency_ref
                        ),
                    );
                }

                for sub_dependency in &dependency.dependencies {
                    if !bom_refs.contains(sub_dependency) {
                        context.add_custom(
                            "sub dependency_ref",
                            format!(
                                "Dependency ref '{}' does not exist in the BOM",
                                sub_dependency
                            ),
                        );
                    }
                }

                for provided in &dependency.provides {
                    if !bom_refs.contains(provided) {
                        context.add_custom(
                            "provides_ref",
                            format!("Provides ref '{}' does not exist in the BOM", provided),
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
                            context.add_custom(
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
                        if !bom_refs.contains(dependency) {
                            context.add_custom(
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

    fn validate(&self) -> ValidationResult {
        self.validate_version(self.spec_version)
    }
}

#[derive(Default)]
struct BomReferencesContext {
    component_bom_refs: HashSet<String>,
    service_bom_refs: HashSet<String>,
    vulnerabilities_bom_refs: HashSet<String>,
}

impl BomReferencesContext {
    fn contains(&self, bom_ref: &String) -> bool {
        self.component_bom_refs.contains(bom_ref)
            || self.service_bom_refs.contains(bom_ref)
            || self.vulnerabilities_bom_refs.contains(bom_ref)
    }

    fn add_component_bom_ref(&mut self, bom_ref: impl ToString) {
        self.component_bom_refs.insert(bom_ref.to_string());
    }

    fn add_service_bom_ref(&mut self, bom_ref: impl ToString) {
        self.service_bom_refs.insert(bom_ref.to_string());
    }

    fn add_vulnerability_bom_ref(&mut self, bom_ref: impl ToString) {
        self.vulnerabilities_bom_refs.insert(bom_ref.to_string());
    }
}

/// Validates the Bom references.
fn validate_component_bom_refs(
    context: &mut ValidationContext,
    bom_refs: &mut BomReferencesContext,
    component: &Component,
) {
    if let Some(bom_ref) = &component.bom_ref {
        if bom_refs.contains(bom_ref) {
            context.add_custom("bom_ref", format!(r#"Bom ref "{bom_ref}" is not unique"#));
        }
        bom_refs.add_component_bom_ref(bom_ref);
    }

    if let Some(components) = &component.components {
        validate_components(context, bom_refs, components);
    }
}

fn validate_components(
    context: &mut ValidationContext,
    bom_refs: &mut BomReferencesContext,
    components: &Components,
) {
    for component in &components.0 {
        validate_component_bom_refs(context, bom_refs, component);
    }
}

fn validate_services(
    context: &mut ValidationContext,
    bom_refs: &mut BomReferencesContext,
    services: &Services,
) {
    for service in &services.0 {
        validate_service_bom_refs(context, bom_refs, service);
    }
}

fn validate_service_bom_refs(
    context: &mut ValidationContext,
    bom_refs: &mut BomReferencesContext,
    service: &Service,
) {
    if let Some(bom_ref) = &service.bom_ref {
        if bom_refs.contains(bom_ref) {
            context.add_custom("bom_ref", format!(r#"Bom ref "{bom_ref}" is not unique"#));
        }
        bom_refs.add_service_bom_ref(bom_ref);
    }

    if let Some(services) = &service.services {
        validate_services(context, bom_refs, services);
    }
}

fn validate_vulnerabilities(
    context: &mut ValidationContext,
    bom_refs: &mut BomReferencesContext,
    vulnerabilities: &Vulnerabilities,
) {
    for vulnerability in &vulnerabilities.0 {
        validate_vulnerabilities_bom_refs(context, bom_refs, vulnerability);
    }
}

fn validate_vulnerabilities_bom_refs(
    context: &mut ValidationContext,
    bom_refs: &mut BomReferencesContext,
    vulnerability: &Vulnerability,
) {
    if let Some(bom_ref) = &vulnerability.bom_ref {
        if bom_refs.contains(bom_ref) {
            context.add_custom("bom_ref", format!(r#"Bom ref "{bom_ref}" is not unique"#));
        }
        bom_refs.add_vulnerability_bom_ref(bom_ref);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UrnUuid(pub String);

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
        external_models::{
            date_time::DateTime, normalized_string::NormalizedString, uri::Uri as Url,
        },
        models::{
            component::{Classification, Component},
            composition::{AggregateType, Composition},
            dependency::Dependency,
            external_reference::{ExternalReference, ExternalReferenceType, Uri},
            property::Property,
            service::Service,
            vulnerability::Vulnerability,
        },
        validation,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_parse_json_with_explicit_and_detected_versions() {
        for version in [
            SpecVersion::V1_3,
            SpecVersion::V1_4,
            SpecVersion::V1_5,
            SpecVersion::V1_6,
            SpecVersion::V1_7,
        ] {
            let value = serde_json::json!({
                "bomFormat": "CycloneDX",
                "specVersion": version.to_string(),
                "version": 1,
                "components": []
            });
            let input = value.to_string();
            let expected = Bom {
                spec_version: version,
                serial_number: None,
                components: Some(Components(vec![])),
                ..Bom::default()
            };

            assert_eq!(
                Bom::parse_json_value_with_version(value.clone(), version).unwrap(),
                expected
            );
            assert_eq!(
                Bom::parse_from_json_with_version(input.as_bytes(), version).unwrap(),
                expected
            );
            assert_eq!(Bom::parse_json_value(value).unwrap(), expected);
            assert_eq!(Bom::parse_from_json(input.as_bytes()).unwrap(), expected);
        }
    }

    #[test]
    fn it_should_reject_invalid_versions_during_json_detection() {
        for (version, expected) in [
            (None, "No field 'specVersion' found"),
            (Some(serde_json::json!(13)), "13"),
            (Some(serde_json::json!("1.2")), "1.2"),
        ] {
            let mut value = serde_json::json!({
                "bomFormat": "CycloneDX",
                "version": 1
            });
            if let Some(version) = version {
                value["specVersion"] = version;
            }
            let input = value.to_string();
            for result in [
                Bom::parse_json_value(value),
                Bom::parse_from_json(input.as_bytes()),
            ] {
                assert!(matches!(
                    result,
                    Err(crate::errors::JsonReadError::BomError {
                        error: BomError::UnsupportedSpecVersion(actual),
                    }) if actual == expected
                ));
            }
        }
    }

    #[test]
    fn it_should_preserve_explicit_version_mismatch_behavior() {
        let value = serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.4",
            "version": 1
        });
        let input = value.to_string();
        let bom = Bom::parse_json_value_with_version(value, SpecVersion::V1_3).unwrap();
        assert_eq!(bom.spec_version, SpecVersion::V1_4);
        assert_eq!(
            Bom::parse_from_json_with_version(input.as_bytes(), SpecVersion::V1_3).unwrap(),
            bom
        );

        let xml = r#"<?xml version="1.0"?>
            <bom xmlns="http://cyclonedx.org/schema/bom/1.4" version="1"/>"#;
        assert!(matches!(
            Bom::parse_from_xml_with_version(xml.as_bytes(), SpecVersion::V1_3),
            Err(crate::errors::XmlReadError::InvalidNamespaceError { .. })
        ));
    }

    #[test]
    fn it_should_validate_an_empty_bom_as_passed() {
        let bom = Bom {
            version: 1,
            spec_version: SpecVersion::V1_3,
            serial_number: None,
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: None,
            vulnerabilities: None,
            signature: None,
            annotations: None,
            properties: None,
            formulation: None,
            declarations: None,
            definitions: None,
            citations: None,
        };

        let actual = bom.validate();

        assert!(actual.passed());
    }

    #[test]
    fn it_should_validate_broken_dependency_refs_as_failed() {
        let bom = Bom {
            version: 1,
            spec_version: SpecVersion::V1_3,
            serial_number: None,
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: Some(Dependencies(vec![Dependency {
                dependency_ref: "dependency".to_string(),
                dependencies: vec!["sub-dependency".to_string()],
                provides: vec!["provided".to_string()],
            }])),
            compositions: None,
            properties: None,
            vulnerabilities: None,
            signature: None,
            annotations: None,
            formulation: None,
            declarations: None,
            definitions: None,
            citations: None,
        };

        let actual = bom.validate();

        assert_eq!(
            actual,
            vec![
                validation::custom(
                    "dependency_ref",
                    ["Dependency ref 'dependency' does not exist in the BOM",],
                ),
                validation::custom(
                    "sub dependency_ref",
                    ["Dependency ref 'sub-dependency' does not exist in the BOM"]
                ),
                validation::custom(
                    "provides_ref",
                    ["Provides ref 'provided' does not exist in the BOM"]
                )
            ]
            .into()
        );
    }

    #[test]
    fn it_should_validate_broken_composition_refs_as_failed() {
        let bom = Bom {
            version: 1,
            spec_version: SpecVersion::V1_5,
            serial_number: None,
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: Some(Compositions(vec![Composition {
                bom_ref: None,
                aggregate: AggregateType::Complete,
                assemblies: Some(vec![BomReference("assembly".to_string())]),
                dependencies: Some(vec![BomReference("dependencies".to_string())]),
                vulnerabilities: None,
                signature: None,
            }])),
            properties: None,
            vulnerabilities: None,
            signature: None,
            annotations: None,
            formulation: None,
            declarations: None,
            definitions: None,
            citations: None,
        };

        let actual = bom.validate_version(SpecVersion::V1_3);

        assert_eq!(
            actual,
            validation::custom(
                "composition ref",
                [
                    "Composition reference 'assembly' does not exist in the BOM",
                    "Composition reference 'dependencies' does not exist in the BOM"
                ]
            )
        );
    }

    #[test]
    fn it_should_validate_a_bom_with_multiple_validation_issues_as_failed() {
        let bom = Bom {
            version: 1,
            spec_version: SpecVersion::V1_3,
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
                lifecycles: None,
                manufacturer: None,
                distribution_constraints: None,
            }),
            components: Some(Components(vec![Component {
                tags: None,
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
                model_card: None,
                data: None,
                manufacturer: None,
                authors: None,
                omnibor_id: None,
                swhid: None,
                crypto_properties: None,
                is_external: None,
                version_range: None,
                patent_assertions: None,
            }])),
            services: Some(Services(vec![Service::new("invalid\tname", None)])),
            external_references: Some(ExternalReferences(vec![ExternalReference {
                properties: None,
                external_reference_type: ExternalReferenceType::UnknownExternalReferenceType(
                    "unknown".to_string(),
                ),
                url: Uri::Url(Url("https://example.com".to_string())),
                comment: None,
                hashes: None,
            }])),
            dependencies: Some(Dependencies(vec![Dependency {
                dependency_ref: "dependency".to_string(),
                dependencies: vec![],
                provides: vec![],
            }])),
            compositions: Some(Compositions(vec![Composition {
                bom_ref: Some(BomReference::new("composition-1")),
                aggregate: AggregateType::UnknownAggregateType("unknown".to_string()),
                assemblies: None,
                dependencies: None,
                vulnerabilities: None,
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
                workaround: None,
                proof_of_concept: None,
                advisories: None,
                created: None,
                published: None,
                updated: None,
                rejected: None,
                vulnerability_credits: None,
                tools: None,
                vulnerability_analysis: None,
                vulnerability_targets: None,
                properties: None,
            }])),
            signature: None,
            annotations: None,
            formulation: None,
            declarations: None,
            definitions: None,
            citations: None,
        };

        let actual = bom.validate();

        assert_eq!(
            actual,
            vec![
                validation::field("serial_number", "UrnUuid does not match regular expression"),
                validation::r#struct(
                    "metadata",
                    validation::field(
                        "timestamp",
                        "DateTime does not conform to ISO 8601"
                    )
                ),
                validation::r#struct(
                    "components",
                    validation::list(
                        "inner",
                        [(
                            0,
                            validation::field("component_type", "Unknown classification")
                        )]
                    )
                ),
                validation::r#struct(
                    "services",
                    validation::list(
                        "inner",
                        [(
                            0,
                            validation::field(
                                "name",
                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                            )
                        )]
                    )
                ),
                validation::r#struct(
                    "external_references",
                    validation::list(
                        "inner",
                        [(
                            0,
                            validation::field("external_reference_type", "Unknown external reference type")
                        )]
                    )
                ),
                validation::r#struct(
                    "compositions",
                    validation::list(
                        "composition",
                        [(
                            0,
                            validation::field("aggregate", "Unknown aggregate type")
                        )]
                    )
                ),
                validation::r#struct(
                    "properties",
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
                )
            ]
            .into()
        );
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
            spec_version: SpecVersion::V1_4,
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
                lifecycles: None,
                manufacturer: None,
                distribution_constraints: None,
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
            annotations: None,
            formulation: None,
            declarations: None,
            definitions: None,
            citations: None,
        }
        .validate();

        assert_eq!(
            validation_result,
            validation::custom(
                "bom_ref",
                [
                    r#"Bom ref "metadata-component" is not unique"#,
                    r#"Bom ref "component-component" is not unique"#,
                    r#"Bom ref "subcomponent-component" is not unique"#,
                    r#"Bom ref "service-service" is not unique"#,
                    r#"Bom ref "subservice-service" is not unique"#,
                    r#"Bom ref "component-service" is not unique"#,
                ]
            ),
        );
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
