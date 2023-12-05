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

use crate::external_models::{normalized_string::NormalizedString, uri::Uri};
use crate::models::external_reference::ExternalReferences;
use crate::models::license::Licenses;
use crate::models::organization::OrganizationalEntity;
use crate::models::property::Properties;
use crate::validation::{
    FailureReason, Validate, ValidationContext, ValidationError, ValidationPathComponent,
    ValidationResult,
};

/// Represents a service as described in the [CycloneDX use cases](https://cyclonedx.org/use-cases/#service-definition)
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_service)
#[derive(Debug, PartialEq, Eq)]
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
        }
    }
}

impl Validate for Service {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(provider) = &self.provider {
            let context = context.extend_context_with_struct_field("Service", "provider");

            results.push(provider.validate_with_context(context)?);
        }

        if let Some(group) = &self.group {
            let context = context.extend_context_with_struct_field("Service", "group");

            results.push(group.validate_with_context(context)?);
        }

        let name_context = context.extend_context_with_struct_field("Service", "name");

        results.push(self.name.validate_with_context(name_context)?);

        if let Some(version) = &self.version {
            let context = context.extend_context_with_struct_field("Service", "version");

            results.push(version.validate_with_context(context)?);
        }

        if let Some(description) = &self.description {
            let context = context.extend_context_with_struct_field("Service", "description");

            results.push(description.validate_with_context(context)?);
        }

        if let Some(endpoints) = &self.endpoints {
            for (index, endpoint) in endpoints.iter().enumerate() {
                let context = context.extend_context(vec![
                    ValidationPathComponent::Struct {
                        struct_name: "Service".to_string(),
                        field_name: "endpoints".to_string(),
                    },
                    ValidationPathComponent::Array { index },
                ]);
                results.push(endpoint.validate_with_context(context)?);
            }
        }

        if let Some(data) = &self.data {
            for (index, classification) in data.iter().enumerate() {
                let context = context.extend_context(vec![
                    ValidationPathComponent::Struct {
                        struct_name: "Service".to_string(),
                        field_name: "data".to_string(),
                    },
                    ValidationPathComponent::Array { index },
                ]);
                results.push(classification.validate_with_context(context)?);
            }
        }

        if let Some(licenses) = &self.licenses {
            let context = context.extend_context_with_struct_field("Service", "licenses");

            results.push(licenses.validate_with_context(context)?);
        }

        if let Some(external_references) = &self.external_references {
            let context =
                context.extend_context_with_struct_field("Service", "external_references");

            results.push(external_references.validate_with_context(context)?);
        }

        if let Some(properties) = &self.properties {
            let context = context.extend_context_with_struct_field("Service", "properties");

            results.push(properties.validate_with_context(context)?);
        }

        if let Some(services) = &self.services {
            let context = context.extend_context_with_struct_field("Service", "services");

            results.push(services.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Services(pub Vec<Service>);

impl Validate for Services {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, service) in self.0.iter().enumerate() {
            let context = context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(service.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

/// Represents the data classification and data flow
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_dataClassificationType)
#[derive(Debug, PartialEq, Eq)]
pub struct DataClassification {
    pub flow: DataFlowType,
    pub classification: NormalizedString,
}

impl Validate for DataClassification {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        let flow_context = context.extend_context_with_struct_field("DataClassification", "flow");

        results.push(self.flow.validate_with_context(flow_context)?);

        let classification_context =
            context.extend_context_with_struct_field("DataClassification", "classification");

        results.push(
            self.classification
                .validate_with_context(classification_context)?,
        );

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

/// Represents the flow direction of the data
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_dataFlowType)
#[derive(Debug, PartialEq, Eq)]
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

impl Validate for DataFlowType {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        match self {
            DataFlowType::UnknownDataFlow(_) => Ok(ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "Unknown data flow type".to_string(),
                    context,
                }],
            }),
            _ => Ok(ValidationResult::Passed),
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
        }])
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
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
            }])),
        }])
        .validate_with_context(ValidationContext::default())
        .expect("Error while validating");

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
    }
}
