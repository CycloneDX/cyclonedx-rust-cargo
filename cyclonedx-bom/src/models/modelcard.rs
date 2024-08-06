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

use crate::{
    prelude::{Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

use super::{
    bom::{BomReference, SpecVersion},
    component_data::{ComponentData, GraphicsCollection},
    property::Properties,
};

/// This model was added in spec version 1.5
///
/// For more details see: https://cyclonedx.org/docs/1.5/json/#metadata_component_modelCard
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelCard {
    pub bom_ref: Option<BomReference>,
    pub model_parameters: Option<ModelParameters>,
    pub quantitative_analysis: Option<QuantitativeAnalysis>,
    pub considerations: Option<Considerations>,
    pub properties: Option<Properties>,
}

impl Validate for ModelCard {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("model_parameters", self.model_parameters.as_ref(), version)
            .add_struct_option(
                "quantitative_analysis",
                self.quantitative_analysis.as_ref(),
                version,
            )
            .add_struct_option("considerations", self.considerations.as_ref(), version)
            .add_struct_option("properties", self.properties.as_ref(), version)
            .into()
    }
}

/// This model was added in spec version 1.5.
///
/// For more details see: https://cyclonedx.org/docs/1.5/json/#metadata_component_modelCard_modelParameters
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelParameters {
    pub approach: Option<ModelParametersApproach>,
    pub task: Option<String>,
    pub architecture_family: Option<String>,
    pub model_architecture: Option<String>,
    pub datasets: Option<Datasets>,
    pub inputs: Option<Inputs>,
    pub outputs: Option<Outputs>,
}

impl Validate for ModelParameters {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option("approach", self.approach.as_ref(), version)
            .add_struct_option("datasets", self.datasets.as_ref(), version)
            .add_struct_option("inputs", self.inputs.as_ref(), version)
            .add_struct_option("outputs", self.outputs.as_ref(), version)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelParametersApproach {
    pub approach_type: Option<ApproachType>,
}

impl Validate for ModelParametersApproach {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("type", self.approach_type.as_ref(), validate_approach_type)
            .into()
    }
}

impl ModelParametersApproach {
    pub fn new(approach_type: &str) -> Self {
        Self {
            approach_type: Some(ApproachType::new_unchecked(approach_type)),
        }
    }
}

/// Checks the given [`ApproachType`] is valid.
pub fn validate_approach_type(approach_type: &ApproachType) -> Result<(), ValidationError> {
    if let ApproachType::Unknown(unknown) = approach_type {
        return Err(format!("Unknown approach type '{unknown}'").into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ApproachType {
    Supervised,
    Unsupervised,
    ReinforcementLearning,
    SemiSupervised,
    SelfSupervised,
    #[doc(hidden)]
    Unknown(String),
}

impl ApproachType {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "supervised" => Self::Supervised,
            "unsupervised" => Self::Unsupervised,
            "reinforcement-learning" => Self::ReinforcementLearning,
            "semi-supervised" => Self::SemiSupervised,
            "self-supervised" => Self::SelfSupervised,
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

impl std::fmt::Display for ApproachType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ApproachType::Supervised => "supervised",
            ApproachType::Unsupervised => "unsupervised",
            ApproachType::ReinforcementLearning => "reinforcement-learning",
            ApproachType::SemiSupervised => "semi-supervised",
            ApproachType::SelfSupervised => "self-supervised",
            ApproachType::Unknown(unknown) => unknown,
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Datasets(pub Vec<Dataset>);

impl Validate for Datasets {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |dataset| {
                dataset.validate_version(version)
            })
            .into()
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Dataset {
    Component(ComponentData),
    Reference(String),
}

impl Validate for Dataset {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        match self {
            Dataset::Component(component) => component.validate_version(version),
            Dataset::Reference(_) => ValidationContext::new().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Inputs(pub Vec<MLParameter>);

impl Validate for Inputs {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, validate_mlparameter)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Outputs(pub Vec<MLParameter>);

impl Validate for Outputs {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, validate_mlparameter)
            .into()
    }
}

pub fn validate_mlparameter(_parameter: &MLParameter) -> Result<(), ValidationError> {
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MLParameter {
    pub format: Option<String>,
}

impl MLParameter {
    pub fn new(format: &str) -> Self {
        Self {
            format: Some(format.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QuantitativeAnalysis {
    pub performance_metrics: Option<PerformanceMetrics>,
    pub graphics: Option<GraphicsCollection>,
}

impl Validate for QuantitativeAnalysis {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_struct_option(
                "performance_metrics",
                self.performance_metrics.as_ref(),
                version,
            )
            .add_struct_option("graphics", self.graphics.as_ref(), version)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PerformanceMetrics(pub Vec<PerformanceMetric>);

impl Validate for PerformanceMetrics {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |metric| metric.validate_version(version))
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PerformanceMetric {
    pub metric_type: Option<String>,
    pub value: Option<String>,
    pub slice: Option<String>,
    pub confidence_interval: Option<ConfidenceInterval>,
}

impl Validate for PerformanceMetric {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new().into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConfidenceInterval {
    pub lower_bound: Option<String>,
    pub upper_bound: Option<String>,
}

/// TODO: implement struct
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Considerations {}

impl Validate for Considerations {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new().into()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        models::{
            attachment::Attachment,
            bom::BomReference,
            component_data::{
                ComponentData, ComponentDataType, DataContents, Graphic, GraphicsCollection,
            },
            data_governance::{DataGovernance, DataGovernanceResponsibleParty},
            modelcard::{
                ApproachType, ConfidenceInterval, Considerations, Dataset, Datasets, Inputs,
                MLParameter, ModelCard, ModelParameters, ModelParametersApproach, Outputs,
                PerformanceMetric, PerformanceMetrics, QuantitativeAnalysis,
            },
            organization::OrganizationalContact,
            property::{Properties, Property},
        },
        prelude::{NormalizedString, SpecVersion, Uri, Validate},
    };

    #[test]
    fn valid_modelcard_should_pass_validation() {
        let modelcard = ModelCard {
            bom_ref: None,
            model_parameters: Some(ModelParameters {
                approach: Some(ModelParametersApproach {
                    approach_type: Some(ApproachType::Supervised),
                }),
                task: Some("task".to_string()),
                architecture_family: Some("architecture family".to_string()),
                model_architecture: Some("model architecture".to_string()),
                datasets: Some(Datasets(vec![Dataset::Component(ComponentData {
                    bom_ref: None,
                    data_type: ComponentDataType::SourceCode,
                    name: Some("dataset".to_string()),
                    contents: Some(DataContents {
                        attachment: Some(Attachment {
                            content: "data content".to_string(),
                            content_type: Some("text/plain".to_string()),
                            encoding: Some("base64".to_string()),
                        }),
                        url: Some(Uri("https://example.com".to_string())),
                        properties: Some(Properties(vec![])),
                    }),
                    classification: Some("data classification".to_string()),
                    sensitive_data: Some("sensitive".to_string()),
                    graphics: Some(GraphicsCollection {
                        description: Some("All graphics".to_string()),
                        collection: Some(vec![Graphic {
                            name: Some("graphic-1".to_string()),
                            image: Some(Attachment {
                                content_type: Some("image/jpeg".to_string()),
                                encoding: Some("base64".to_string()),
                                content: "imagebytes".to_string(),
                            }),
                        }]),
                    }),
                    description: Some("Component data description".to_string()),
                    governance: Some(DataGovernance {
                        custodians: Some(vec![DataGovernanceResponsibleParty::Contact(
                            OrganizationalContact {
                                bom_ref: Some(BomReference::new("custodian-1")),
                                name: Some("custodian".into()),
                                email: None,
                                phone: None,
                            },
                        )]),
                        stewards: None,
                        owners: None,
                    }),
                })])),
                inputs: Some(Inputs(vec![MLParameter::new("string")])),
                outputs: Some(Outputs(vec![MLParameter::new("image")])),
            }),
            quantitative_analysis: Some(QuantitativeAnalysis {
                performance_metrics: Some(PerformanceMetrics(vec![PerformanceMetric {
                    metric_type: Some("performance".to_string()),
                    value: Some("value".to_string()),
                    slice: Some("slice".to_string()),
                    confidence_interval: Some(ConfidenceInterval {
                        lower_bound: Some("lower".to_string()),
                        upper_bound: Some("upper".to_string()),
                    }),
                }])),
                graphics: Some(GraphicsCollection {
                    description: Some("graphics".to_string()),
                    collection: None,
                }),
            }),
            considerations: Some(Considerations {}),
            properties: Some(Properties(vec![Property {
                name: "property-a".to_string(),
                value: NormalizedString::new("value"),
            }])),
        };

        let validation_result = modelcard.validate_version(SpecVersion::V1_5);
        assert!(validation_result.passed());
    }
}
