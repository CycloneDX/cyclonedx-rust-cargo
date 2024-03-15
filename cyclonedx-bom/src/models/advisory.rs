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
use crate::validation::{Validate, ValidationContext, ValidationResult};

use super::bom::SpecVersion;

/// Represents an advisory, a notification of a threat to a component, service, or system.
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.4/xml/#type_advisoryType)
#[derive(Clone, Debug, PartialEq, Eq)]
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
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("title", self.title.as_ref(), validate_normalized_string)
            .add_field("url", &self.url, validate_uri)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Advisories(pub Vec<Advisory>);

impl Validate for Advisories {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |advisory| {
                advisory.validate_version(version)
            })
            .into()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        external_models::{normalized_string::NormalizedString, uri::Uri},
        validation,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Advisories(vec![Advisory {
            title: Some(NormalizedString::new("title")),
            url: Uri("https://example.com".to_string()),
        }])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Advisories(vec![Advisory {
            title: Some(NormalizedString("invalid\ttitle".to_string())),
            url: Uri("invalid url".to_string()),
        }])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    vec![
                        validation::field(
                            "title",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        ),
                        validation::field("url", "Uri does not conform to RFC 3986")
                    ]
                )]
            )
        );
    }
}
