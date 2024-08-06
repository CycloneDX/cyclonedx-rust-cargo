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

use once_cell::sync::Lazy;
use regex::Regex;

use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

use super::bom::SpecVersion;

/// Represents the hash of the component
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_hashType)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hash {
    pub alg: HashAlgorithm,
    pub content: HashValue,
}

impl Validate for Hash {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("alg", &self.alg, validate_hash_algorithm)
            .add_field("content", &self.content, validate_hash_value)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hashes(pub Vec<Hash>);

impl Validate for Hashes {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |hash| hash.validate_version(version))
            .into()
    }
}

pub fn validate_hash_algorithm(algorithm: &HashAlgorithm) -> Result<(), ValidationError> {
    if matches!(algorithm, HashAlgorithm::UnknownHashAlgorithm(_)) {
        return Err(ValidationError::new("Unknown HashAlgorithm"));
    }
    Ok(())
}

/// Represents the algorithm used to create the hash
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_hashAlg)
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, strum::Display)]
#[strum(serialize_all = "SCREAMING-KEBAB-CASE")]
pub enum HashAlgorithm {
    MD5,
    #[strum(serialize = "SHA-1")]
    SHA1,
    SHA_256,
    SHA_384,
    SHA_512,
    SHA3_256,
    SHA3_384,
    SHA3_512,
    #[strum(serialize = "BLAKE2b-256")]
    BLAKE2b_256,
    #[strum(serialize = "BLAKE2b-384")]
    BLAKE2b_384,
    #[strum(serialize = "BLAKE2b-512")]
    BLAKE2b_512,
    BLAKE3,
    #[doc(hidden)]
    #[strum(default)]
    UnknownHashAlgorithm(String),
}
impl HashAlgorithm {
    pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "MD5" => Self::MD5,
            "SHA-1" => Self::SHA1,
            "SHA-256" => Self::SHA_256,
            "SHA-384" => Self::SHA_384,
            "SHA-512" => Self::SHA_512,
            "SHA3-256" => Self::SHA3_256,
            "SHA3-384" => Self::SHA3_384,
            "SHA3-512" => Self::SHA3_512,
            "BLAKE2b-256" => Self::BLAKE2b_256,
            "BLAKE2b-384" => Self::BLAKE2b_384,
            "BLAKE2b-512" => Self::BLAKE2b_512,
            "BLAKE3" => Self::BLAKE3,
            unknown => Self::UnknownHashAlgorithm(unknown.to_string()),
        }
    }
}

pub fn validate_hash_value(value: &HashValue) -> Result<(), ValidationError> {
    static HASH_VALUE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"^([a-fA-F0-9]{32})|([a-fA-F0-9]{40})|([a-fA-F0-9]{64})|([a-fA-F0-9]{96})|([a-fA-F0-9]{128})$",
        ).expect("Failed to compile regex.")
    });

    if !HASH_VALUE_REGEX.is_match(&value.0) {
        return Err(ValidationError::new(
            "HashValue does not match regular expression",
        ));
    }

    Ok(())
}

/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_hashValue)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HashValue(pub String);

#[cfg(test)]
mod test {
    use crate::validation::{self};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Hashes(vec![Hash {
            alg: HashAlgorithm::MD5,
            content: HashValue("a3bf1f3d584747e2569483783ddee45b".to_string()),
        }])
        .validate_version(SpecVersion::V1_3);

        assert!(validation_result.passed());
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Hashes(vec![Hash {
            alg: HashAlgorithm::UnknownHashAlgorithm("unknown algorithm".to_string()),
            content: HashValue("not a hash".to_string()),
        }])
        .validate_version(SpecVersion::V1_3);

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    vec![
                        validation::field("alg", "Unknown HashAlgorithm"),
                        validation::field("content", "HashValue does not match regular expression")
                    ]
                )]
            )
        );
    }
}
