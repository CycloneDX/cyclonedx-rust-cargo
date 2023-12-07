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
use crate::validation::{
    Validate, ValidationContext, ValidationError, ValidationPathComponent, ValidationResult,
};

/// Represents an advisory, a notification of a threat to a component, service, or system.
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.4/xml/#type_advisoryType)
#[derive(Debug, PartialEq, Eq)]
pub struct Advisory {
    pub title: Option<NormalizedString>,
    pub url: Uri,
}

impl Advisory {
    /// Constructs a new `Advisory` with an url
    /// ```
    /// use cyclonedx_bom::models::advisory::Advisory;
    /// use cyclonedx_bom::external_models::uri::{Uri, UriError};
    /// use std::convert::TryFrom;
    ///
    /// let url = Uri::try_from("https://github.com/FasterXML/jackson-databind/issues/1931".to_string())?;
    /// let advisory = Advisory::new(url);
    /// # Ok::<(), UriError>(())
    /// ```
    pub fn new(url: Uri) -> Self {
        Self { title: None, url }
    }
}

impl Validate for Advisory {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(title) = &self.title {
            let context = context.extend_context_with_struct_field("Advisory", "title");

            results.push(title.validate_with_context(context)?);
        }

        let url_context = context.extend_context_with_struct_field("Advisory", "url");
        results.push(self.url.validate_with_context(url_context)?);

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Advisories(pub Vec<Advisory>);

impl Validate for Advisories {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        for (index, advisory) in self.0.iter().enumerate() {
            let context = context.extend_context(vec![ValidationPathComponent::Array { index }]);
            results.push(advisory.validate_with_context(context)?);
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        external_models::{normalized_string::NormalizedString, uri::Uri},
        validation::FailureReason,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Advisories(vec![Advisory {
            title: Some(NormalizedString::new("title")),
            url: Uri("https://example.com".to_string()),
        }])
        .validate()
        .expect("Error while validating");

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Advisories(vec![Advisory {
            title: Some(NormalizedString("invalid\ttitle".to_string())),
            url: Uri("invalid url".to_string()),
        }])
        .validate()
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
                                struct_name: "Advisory".to_string(),
                                field_name: "title".to_string()
                            },
                        ])
                    },
                    FailureReason {
                        message: "Uri does not conform to RFC 3986".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "Advisory".to_string(),
                                field_name: "url".to_string()
                            }
                        ])
                    },
                ]
            }
        );
    }
}
