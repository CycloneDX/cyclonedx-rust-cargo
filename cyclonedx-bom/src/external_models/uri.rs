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

use std::{convert::TryFrom, str::FromStr};

use fluent_uri::Uri as Url;
use purl::{GenericPurl, GenericPurlBuilder};
use thiserror::Error;

use crate::validation::ValidationError;

pub fn validate_purl(purl: &Purl) -> Result<(), ValidationError> {
    match GenericPurl::<String>::from_str(&purl.0) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Purl does not conform to Package URL spec: {e}").into()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Purl(pub(crate) String);

impl Purl {
    pub fn new(package_type: &str, name: &str, version: &str) -> Result<Purl, UriError> {
        let builder = GenericPurlBuilder::new(package_type.to_string(), name).with_version(version);

        match builder.build() {
            Ok(purl) => Ok(Self(purl.to_string())),
            Err(e) => Err(UriError::InvalidPurl(e.to_string())),
        }
    }
}

impl std::fmt::Display for Purl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Purl {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl AsRef<str> for Purl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub fn validate_uri(uri: &Uri) -> Result<(), ValidationError> {
    if Url::parse(uri.0.as_str()).is_err() {
        return Err(ValidationError::new("Uri does not conform to RFC 3986"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uri(pub(crate) String);

impl Uri {
    pub fn new(uri: &str) -> Self {
        Self(uri.to_string())
    }

    pub fn is_bomlink(&self) -> bool {
        self.0.starts_with("urn:cdx")
    }
}

impl TryFrom<String> for Uri {
    type Error = UriError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match Url::parse(value.as_str()) {
            Ok(_) => Ok(Uri(value)),
            Err(_) => Err(UriError::InvalidUri(
                "Uri does not conform to RFC 3986".to_string(),
            )),
        }
    }
}

impl AsRef<str> for Uri {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UriError {
    #[error("Invalid URI: {}", .0)]
    InvalidUri(String),

    #[error("Invalid Purl: {}", .0)]
    InvalidPurl(String),
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        external_models::uri::{validate_purl, validate_uri},
        prelude::{Purl, Uri},
    };

    #[test]
    fn valid_purls_should_pass_validation() {
        let validation_result = validate_purl(&Purl("pkg:cargo/cyclonedx-bom@0.3.1".to_string()));

        assert_eq!(Ok(()), validation_result);
    }

    #[test]
    fn invalid_purls_should_fail_validation() {
        let validation_result = validate_purl(&Purl("invalid purl".to_string()));
        assert_eq!(
            validation_result,
            Err("Purl does not conform to Package URL spec: URL scheme must be pkg".into()),
        );
    }

    #[test]
    fn valid_uris_should_pass_validation() {
        let validation_result = validate_uri(&Uri("https://example.com".to_string()));
        assert_eq!(Ok(()), validation_result);
    }

    #[test]
    fn invalid_uris_should_fail_validation() {
        let validation_result = validate_uri(&Uri("invalid uri".to_string()));

        assert_eq!(
            validation_result,
            Err("Uri does not conform to RFC 3986".into()),
        );
    }
}
