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

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::{
    external_models::normalized_string::NormalizedString,
    validation::{Validate, ValidationContext, ValidationResult},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachedText {
    pub(crate) content_type: Option<NormalizedString>,
    pub(crate) encoding: Option<Encoding>,
    pub(crate) content: String,
}

impl AttachedText {
    /// Construct a new `AttachedText`
    ///
    /// - `content_type` - Content type of the attached text (default: `"text/plain"`)
    /// - `content` - Raw content, which will be base64 encoded when added to the BOM
    pub fn new<T: AsRef<[u8]>>(content_type: Option<NormalizedString>, content: T) -> Self {
        Self {
            content_type,
            encoding: Some(Encoding::Base64),
            content: STANDARD.encode(content),
        }
    }
}

impl Validate for AttachedText {
    fn validate(&self, version: SpecVersion) -> ValidationResult {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(content_type) = &self.content_type {
            let context = context.with_struct("AttachedText", "content_type");

            results.push(content_type.validate_with_context(context));
        }

        if let Some(encoding) = &self.encoding {
            match (encoding, STANDARD.decode(self.content.clone())) {
                (Encoding::Base64, Ok(_)) => results.push(ValidationResult::Passed),
                (Encoding::Base64, Err(_)) => {
                    let context = context.with_struct("AttachedText", "content");

                    results.push(ValidationResult::failure(
                        "Content is not Base64 encoded",
                        context,
                    ))
                }
                (Encoding::UnknownEncoding(_), _) => {
                    let context = context.with_struct("AttachedText", "encoding");

                    results.push(encoding.validate_with_context(context));
                }
            }
        }

        results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Encoding {
    Base64,
    #[doc(hidden)]
    UnknownEncoding(String),
}

impl ToString for Encoding {
    fn to_string(&self) -> String {
        match self {
            Encoding::Base64 => "base64".to_string(),
            Encoding::UnknownEncoding(ue) => ue.clone(),
        }
    }
}

impl Encoding {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "base64" => Self::Base64,
            unknown => Self::UnknownEncoding(unknown.to_string()),
        }
    }
}

impl Validate for Encoding {
    fn validate(&self, version: SpecVersion) -> ValidationResult {
        match self {
            Encoding::UnknownEncoding(_) => ValidationResult::failure("Unknown encoding", context),
            _ => ValidationResult::Passed,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{models::attached_text::Encoding, validation::FailureReason};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_construct_attached_text() {
        let actual = AttachedText::new(
            Some(NormalizedString::new("text/plain")),
            "this text is plain",
        );
        assert_eq!(
            actual,
            AttachedText {
                content_type: Some(NormalizedString::new("text/plain")),
                encoding: Some(Encoding::Base64),
                content: "dGhpcyB0ZXh0IGlzIHBsYWlu".to_string(),
            }
        )
    }

    #[test]
    fn valid_attached_text_should_pass_validation() {
        let validation_result = AttachedText {
            content_type: Some(NormalizedString("text/plain".to_string())),
            encoding: Some(Encoding::Base64),
            content: "dGhpcyB0ZXh0IGlzIHBsYWlu".to_string(),
        }
        .validate();

        assert_eq!(validation_result, ValidationResult::Passed);
    }

    #[test]
    fn invalid_attached_text_should_fail_validation() {
        let validation_result = AttachedText {
            content_type: Some(NormalizedString("spaces and \ttabs".to_string())),
            encoding: Some(Encoding::Base64),
            content: "not base64 encoded".to_string(),
        }
        .validate();

        assert_eq!(
            validation_result,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason::new(
                        "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n",
                        ValidationContext::new().with_struct("AttachedText", "content_type")
                    ),
                    FailureReason::new(
                        "Content is not Base64 encoded",
                        ValidationContext::new().with_struct("AttachedText", "content")
                    )
                ]
            }
        );
    }

    #[test]
    fn an_unknown_encoding_should_fail_validation() {
        let validation_result = AttachedText {
            content_type: Some(NormalizedString("text/plain".to_string())),
            encoding: Some(Encoding::UnknownEncoding("unknown".to_string())),
            content: "not base64 encoded".to_string(),
        }
        .validate();

        assert_eq!(
            validation_result,
            ValidationResult::failure(
                "Unknown encoding",
                ValidationContext::new().with_struct("AttachedText", "encoding")
            )
        )
    }

    #[test]
    fn no_supplied_encoding_should_pass_validation() {
        let validation_result = AttachedText {
            content_type: Some(NormalizedString("text/plain".to_string())),
            encoding: None,
            content: "not base64 encoded".to_string(),
        }
        .validate();

        assert_eq!(validation_result, ValidationResult::Passed);
    }
}
