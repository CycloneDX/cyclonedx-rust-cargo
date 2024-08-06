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

use crate::external_models::normalized_string::validate_normalized_string;
use crate::external_models::uri::validate_uri as validate_url;
use crate::external_models::{normalized_string::NormalizedString, uri::Uri};
use crate::models::external_reference::ExternalReferences;
use crate::models::license::Licenses;
use crate::models::organization::OrganizationalEntity;
use crate::models::property::Properties;
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

use super::bom::SpecVersion;
use super::data_governance::DataGovernance;
use super::signature::Signature;

/// Represents a service as described in the [CycloneDX use cases](https://cyclonedx.org/use-cases/#service-definition)
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_service)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Service {
    pub bom_ref: Option<String>,
    pub provider: Option<OrganizationalEntity>,
    pub group: Option<NormalizedString>,
    pub name: NormalizedString,
    pub version: Option<NormalizedString>,
    pub description: Option<NormalizedString>,
    pub endpoints: Option<Vec<Uri>>,
    pub authenticated: Option<bool>,
    pub x_trust_boundary: Option<bool>,
    pub data: Option<Data>,
    pub licenses: Option<Licenses>,
    pub external_references: Option<ExternalReferences>,
    pub properties: Option<Properties>,
    pub services: Option<Services>,
    /// Added in version 1.4
    pub signature: Option<Signature>,
    /// Added in version 1.5
    pub trust_zone: Option<NormalizedString>,
}

impl Service {
    /// Construct a `Service` with a name and BOM reference
    /// ```
    /// use cyclonedx_bom::models::service::Service;
    ///
    /// let service = Service::new("service-x", Some("12a34a5b-6780-1bae-2345-67890cfe12a3".to_string()));
    /// ```
    pub fn new(name: &str, bom_ref: Option<String>) -> Self {
        Self {
            name: NormalizedString(name.to_string()),
            bom_ref,
            provider: None,
            group: None,
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
            trust_zone: None,
        }
    }
}

impl Validate for Service {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("provider", self.provider.as_ref(), version)
            .add_field_option("group", self.group.as_ref(), validate_normalized_string)
            .add_field("name", &self.name, validate_normalized_string)
            .add_field_option("version", self.version.as_ref(), validate_normalized_string)
            .add_field_option(
                "description",
                self.description.as_ref(),
                validate_normalized_string,
            )
            .add_list_option("endpoints", self.endpoints.as_ref(), validate_url)
            .add_struct_option("data", self.data.as_ref(), version)
            .add_struct_option("licenses", self.licenses.as_ref(), version)
            .add_struct_option(
                "external_references",
                self.external_references.as_ref(),
                version,
            )
            .add_struct_option("properties", self.properties.as_ref(), version)
            .add_struct_option("services", self.services.as_ref(), version)
            .add_field_option(
                "trust_zone",
                self.trust_zone.as_ref(),
                validate_normalized_string,
            )
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Services(pub Vec<Service>);

impl Validate for Services {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |service| {
                service.validate_version(version)
            })
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Data {
    ServiceData(Vec<ServiceData>),
    Classification(Vec<DataClassification>),
}

impl Validate for Data {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match self {
            Data::ServiceData(data) => ValidationContext::new()
                .add_list("inner", data, |d| d.validate_version(version))
                .into(),
            Data::Classification(classification) => ValidationContext::new()
                .add_list("inner", classification, |c| c.validate_version(version))
                .into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ServiceData {
    pub name: Option<NormalizedString>,
    pub description: Option<NormalizedString>,
    pub classification: DataClassification,
    pub governance: Option<DataGovernance>,
    pub source: Option<Vec<Uri>>,
    pub destination: Option<Vec<Uri>>,
}

impl Validate for ServiceData {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("name", self.name.as_ref(), validate_normalized_string)
            .add_field_option(
                "description",
                self.description.as_ref(),
                validate_normalized_string,
            )
            .add_struct("classification", &self.classification, version)
            .add_struct_option("governance", self.governance.as_ref(), version)
            .add_list_option("source", self.source.as_ref(), validate_url)
            .add_list_option("destination", self.destination.as_ref(), validate_url)
            .into()
    }
}

pub fn validate_data_flow_type(data_flow_type: &DataFlowType) -> Result<(), ValidationError> {
    if matches!(data_flow_type, DataFlowType::UnknownDataFlow(_)) {
        return Err(ValidationError::new("Unknown data flow type"));
    }
    Ok(())
}

/// Represents the data classification and data flow
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_dataClassificationType)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataClassification {
    pub flow: DataFlowType,
    pub classification: NormalizedString,
}

impl Validate for DataClassification {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_enum("flow", &self.flow, validate_data_flow_type)
            .add_enum(
                "classification",
                &self.classification,
                validate_normalized_string,
            )
            .into()
    }
}

/// Represents the flow direction of the data
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_dataFlowType)
#[derive(Clone, Debug, PartialEq, Eq, strum::Display, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum DataFlowType {
    Inbound,
    Outbound,
    BiDirectional,
    Unknown,
    #[doc(hidden)]
    #[strum(default)]
    UnknownDataFlow(String),
}

impl DataFlowType {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "inbound" => Self::Inbound,
            "outbound" => Self::Outbound,
            "bi-directional" => Self::BiDirectional,
            "unknown" => Self::Unknown,
            unknown => Self::UnknownDataFlow(unknown.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        external_models::spdx::SpdxExpression,
        models::{
            external_reference::{ExternalReference, ExternalReferenceType, Uri},
            license::LicenseChoice,
            property::Property,
            signature::Algorithm,
        },
        validation,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_services_should_pass_validation() {
        let validation_result = Services(vec![Service {
            bom_ref: Some("bom ref".to_string()),
            provider: Some(OrganizationalEntity::new("name")),
            group: Some(NormalizedString::new("group")),
            name: NormalizedString::new("name"),
            version: Some(NormalizedString::new("version")),
            description: Some(NormalizedString::new("description")),
            endpoints: Some(vec![Uri("https://example.com".to_string())]),
            authenticated: Some(true),
            x_trust_boundary: Some(true),
            data: Some(Data::Classification(vec![DataClassification {
                flow: DataFlowType::Inbound,
                classification: NormalizedString::new("classification"),
            }])),
            licenses: Some(Licenses(vec![LicenseChoice::Expression(
                SpdxExpression::new("MIT"),
            )])),
            external_references: Some(ExternalReferences(vec![ExternalReference {
                external_reference_type: ExternalReferenceType::Bom,
                url: Uri::Url(Uri("https://www.example.com".to_string())),
                comment: None,
                hashes: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString::new("value"),
            }])),
            services: Some(Services(vec![])),
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
            trust_zone: Some("Trust Zone".into()),
        }])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn invalid_services_should_fail_validation() {
        let validation_result = Services(vec![Service {
            bom_ref: Some("bom ref".to_string()),
            provider: Some(OrganizationalEntity {
                bom_ref: None,
                name: Some(NormalizedString("invalid\tname".to_string())),
                url: None,
                contact: None,
            }),
            group: Some(NormalizedString("invalid\tgroup".to_string())),
            name: NormalizedString("invalid\tname".to_string()),
            version: Some(NormalizedString("invalid\tversion".to_string())),
            description: Some(NormalizedString("invalid\tdescription".to_string())),
            endpoints: Some(vec![Uri("invalid url".to_string())]),
            authenticated: Some(true),
            x_trust_boundary: Some(true),
            data: Some(Data::Classification(vec![DataClassification {
                flow: DataFlowType::UnknownDataFlow("unknown".to_string()),
                classification: NormalizedString("invalid\tclassification".to_string()),
            }])),
            licenses: Some(Licenses(vec![LicenseChoice::Expression(
                SpdxExpression::new("invalid license"),
            )])),
            external_references: Some(ExternalReferences(vec![ExternalReference {
                external_reference_type: ExternalReferenceType::UnknownExternalReferenceType(
                    "unknown".to_string(),
                ),
                url: Uri::Url(Uri("https://www.example.com".to_string())),
                comment: None,
                hashes: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString("invalid\tvalue".to_string()),
            }])),
            services: Some(Services(vec![Service::new("invalid\tname", None)])),
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
            trust_zone: Some("Trust Zone".into()),
        }])
        .validate();

        assert_eq!(
            validation_result,
            vec![
                validation::list(
                    "inner",
                    [(
                        0,
                        vec![
                            validation::r#struct(
                                "provider",
                                validation::field(
                                    "name",
                                    "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                )
                            ),
                            validation::field(
                                "group",
                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                            ),
                            validation::field(
                                "name",
                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                            ),
                            validation::field(
                                "version",
                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                            ),
                            validation::field(
                                "description",
                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                            ),
                            validation::list(
                                "endpoints",
                                [(
                                    0,
                                    validation::custom("", ["Uri does not conform to RFC 3986"])
                                )]
                            ),
                            validation::r#struct(
                                "data",
                                validation::list(
                                    "inner",
                                    [(
                                        0,
                                        vec![
                                            validation::r#enum(
                                                "flow",
                                                "Unknown data flow type"
                                            ),
                                            validation::r#enum(
                                                "classification",
                                                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                            )
                                        ]
                                    )]
                                )
                            ),
                            validation::r#struct(
                                "licenses",
                                validation::list(
                                    "inner",
                                    [(
                                        0,
                                        validation::r#enum(
                                            "expression",
                                            "SPDX expression is not valid"
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
                                        validation::field(
                                            "external_reference_type",
                                            "Unknown external reference type"
                                        )
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
                            )
                        ]
                    )]
                )
            ].into()
        );
    }
}
