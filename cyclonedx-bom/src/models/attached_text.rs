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
    external_models::normalized_string::{validate_normalized_string, NormalizedString},
    validation::{Validate, ValidationContext, ValidationError, ValidationResult},
};

use super::bom::SpecVersion;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttachedText {
    pub content_type: Option<NormalizedString>,
    pub encoding: Option<Encoding>,
    pub content: String,
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
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();
        context.add_field_option(
            "content_type",
            self.content_type.as_ref(),
            validate_normalized_string,
        );

        if let Some(encoding) = &self.encoding {
            match (encoding, STANDARD.decode(self.content.clone())) {
                (Encoding::Base64, Ok(_)) => (),
                (Encoding::Base64, Err(_)) => {
                    context.add_field("content", &self.content, |_| {
                        Err("Content is not Base64 encoded".into())
                    });
                }
                (Encoding::UnknownEncoding(_), _) => {
                    context.add_field("encoding", encoding, validate_encoding);
                }
            }
        }

        context.into()
    }
}

/// Function to check [`Encoding`].
pub fn validate_encoding(encoding: &Encoding) -> Result<(), ValidationError> {
    if matches!(encoding, Encoding::UnknownEncoding(_)) {
        return Err(ValidationError::new("Unknown encoding"));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, strum::Display, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum Encoding {
    Base64,
    #[doc(hidden)]
    #[strum(default)]
    UnknownEncoding(String),
}

impl Encoding {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "base64" => Self::Base64,
            unknown => Self::UnknownEncoding(unknown.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        models::attached_text::{AttachedText, Encoding},
        prelude::{NormalizedString, Validate},
        validation,
    };

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

        assert!(validation_result.passed());
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
            vec![
                validation::field(
                    "content_type",
                    "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                ),
                validation::field("content", "Content is not Base64 encoded")
            ]
            .into()
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
            validation::field("encoding", "Unknown encoding"),
        );
    }

    #[test]
    fn no_supplied_encoding_should_pass_validation() {
        let validation_result = AttachedText {
            content_type: Some(NormalizedString("text/plain".to_string())),
            encoding: None,
            content: "not base64 encoded".to_string(),
        }
        .validate();

        assert!(validation_result.passed());
    }
}
