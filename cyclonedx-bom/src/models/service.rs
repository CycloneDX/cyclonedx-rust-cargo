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
use crate::external_models::uri::validate_uri;
use crate::external_models::{normalized_string::NormalizedString, uri::Uri};
use crate::models::external_reference::ExternalReferences;
use crate::models::license::Licenses;
use crate::models::organization::OrganizationalEntity;
use crate::models::property::Properties;
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

use super::bom::SpecVersion;
use super::signature::Signature;

/// Represents a service as described in the [CycloneDX use cases](https://cyclonedx.org/use-cases/#service-definition)
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_service)
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub data: Option<Vec<DataClassification>>,
    pub licenses: Option<Licenses>,
    pub external_references: Option<ExternalReferences>,
    pub properties: Option<Properties>,
    pub services: Option<Services>,
    /// Added in version 1.4
    pub signature: Option<Signature>,
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
            name: NormalizedString::new(name),
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
            .add_list_option("endpoints", self.endpoints.as_ref(), validate_uri)
            .add_list_option("data", self.data.as_ref(), |data| data.validate_version(version))
            .add_struct_option("licenses", self.licenses.as_ref(), version)
            .add_struct_option(
                "external_references",
                self.external_references.as_ref(),
                version,
            )
            .add_struct_option("properties", self.properties.as_ref(), version)
            .add_struct_option("services", self.services.as_ref(), version)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Services(pub Vec<Service>);

impl Validate for Services {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |service| service.validate_version(version))
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataClassification {
    pub flow: DataFlowType,
    pub classification: NormalizedString,
}

impl Validate for DataClassification {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_enum("flow", &self.flow, validate_data_flow_type)
            .into()

        /*
        let mut results: Vec<ValidationResult> = vec![];

        let flow_context = context.with_struct("DataClassification", "flow");

        results.push(self.flow.validate_with_context(flow_context));

        let classification_context = context.with_struct("DataClassification", "classification");

        results.push(
            self.classification
                .validate_with_context(classification_context),
        );

        results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result))
        */
    }
}

/// Represents the flow direction of the data
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_dataFlowType)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataFlowType {
    Inbound,
    Outbound,
    BiDirectional,
    Unknown,
    #[doc(hidden)]
    UnknownDataFlow(String),
}

impl ToString for DataFlowType {
    fn to_string(&self) -> String {
        match self {
            DataFlowType::Inbound => "inbound",
            DataFlowType::Outbound => "outbound",
            DataFlowType::BiDirectional => "bi-directional",
            DataFlowType::Unknown => "unknown",
            DataFlowType::UnknownDataFlow(df) => df,
        }
        .to_string()
    }
}

impl DataFlowType {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
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
            external_reference::{ExternalReference, ExternalReferenceType},
            license::LicenseChoice,
            property::Property,
            signature::Algorithm,
        },
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_services_should_pass_validation() {
        let validation_result = Services(vec![Service {
            bom_ref: Some("bom ref".to_string()),
            provider: Some(OrganizationalEntity {
                name: Some(NormalizedString::new("name")),
                url: None,
                contact: None,
            }),
            group: Some(NormalizedString::new("group")),
            name: NormalizedString::new("name"),
            version: Some(NormalizedString::new("version")),
            description: Some(NormalizedString::new("description")),
            endpoints: Some(vec![Uri("https://example.com".to_string())]),
            authenticated: Some(true),
            x_trust_boundary: Some(true),
            data: Some(vec![DataClassification {
                flow: DataFlowType::Inbound,
                classification: NormalizedString::new("classification"),
            }]),
            licenses: Some(Licenses(vec![LicenseChoice::Expression(SpdxExpression(
                "MIT".to_string(),
            ))])),
            external_references: Some(ExternalReferences(vec![ExternalReference {
                external_reference_type: ExternalReferenceType::Bom,
                url: Uri("https://www.example.com".to_string()),
                comment: None,
                hashes: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString::new("value"),
            }])),
            services: Some(Services(vec![])),
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
        }])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn invalid_services_should_fail_validation() {
        let validation_result = Services(vec![Service {
            bom_ref: Some("bom ref".to_string()),
            provider: Some(OrganizationalEntity {
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
            data: Some(vec![DataClassification {
                flow: DataFlowType::UnknownDataFlow("unknown".to_string()),
                classification: NormalizedString("invalid\tclassification".to_string()),
            }]),
            licenses: Some(Licenses(vec![LicenseChoice::Expression(SpdxExpression(
                "invalid license".to_string(),
            ))])),
            external_references: Some(ExternalReferences(vec![ExternalReference {
                external_reference_type: ExternalReferenceType::UnknownExternalReferenceType(
                    "unknown".to_string(),
                ),
                url: Uri("https://www.example.com".to_string()),
                comment: None,
                hashes: None,
            }])),
            properties: Some(Properties(vec![Property {
                name: "name".to_string(),
                value: NormalizedString("invalid\tvalue".to_string()),
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
            signature: Some(Signature::single(Algorithm::HS512, "abcdefgh")),
        }])
        .validate();

        /*
        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "provider".to_string()
                            },
                            ValidationPathComponent::Struct {
                                struct_name: "OrganizationalEntity".to_string(),
                                field_name: "name".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "group".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "name".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "version".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "description".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "Uri does not conform to RFC 3986".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "endpoints".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                        ])
                    },
                    FailureReason {
                        message: "Unknown data flow type".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "data".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "DataClassification".to_string(),
                                field_name: "flow".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "data".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "DataClassification".to_string(),
                                field_name: "classification".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message: "SPDX expression is not valid".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "licenses".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::EnumVariant {
                                variant_name: "Expression".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "Unknown external reference type".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
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
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "properties".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Property".to_string(),
                                field_name: "value".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "services".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Service".to_string(),
                                field_name: "name".to_string()
                            },
                        ])
                    },
                ]
            }
        );
        */
    }
}
