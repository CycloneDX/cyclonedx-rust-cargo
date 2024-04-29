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

use std::str::FromStr;

use crate::{
    prelude::{SpecVersion, Validate, ValidationResult},
    validation::{ValidationContext, ValidationError},
};

/// Enveloped signature in [JSON Signature Format (JSF)](https://cyberphone.github.io/doc/security/jsf.html)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signature {
    /// Multiple signatures
    Signers(Vec<Signer>),
    /// A signature chain consisting of multiple signatures
    Chain(Vec<Signer>),
    /// A single signature
    Single(Signer),
}

impl Validate for Signature {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut context = ValidationContext::new();
        match self {
            Signature::Signers(signers) => context
                .add_list("Signers", signers, |signer| {
                    signer.validate_version(version)
                })
                .into(),
            Signature::Chain(signers) => context
                .add_list("Chain", signers, |signer| signer.validate_version(version))
                .into(),
            Signature::Single(signer) => context.add_struct("Single", signer, version).into(),
        }
    }
}

/// For now the [`Signer`] struct only holds algorithm and value
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signer {
    /// Signature algorithm.
    pub algorithm: Algorithm,
    /// The signature data.
    pub value: String,
}

impl Signer {
    pub fn new(algorithm: &str, value: &str) -> Self {
        Self {
            algorithm: Algorithm::new_unchecked(algorithm),
            value: value.to_string(),
        }
    }
}

impl Validate for Signer {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("algorithm", &self.algorithm, validate_algorithm)
            .into()
    }
}

impl Signature {
    /// Creates a single signature.
    pub fn single(algorithm: &str, value: &str) -> Self {
        Self::Single(Signer::new(algorithm, value))
    }

    /// Creates a chain of multiple signatures
    pub fn chain(chain: &[(&str, &str)]) -> Self {
        Self::Chain(
            chain
                .iter()
                .map(|(algorithm, value)| Signer::new(algorithm, value))
                .collect(),
        )
    }

    /// Creates a list of multiple signatures.
    pub fn signers(signers: &[(&str, &str)]) -> Self {
        Self::Signers(
            signers
                .iter()
                .map(|(algorithm, value)| Signer::new(algorithm, value))
                .collect(),
        )
    }
}

/// Supported signature algorithms.
#[derive(Clone, Debug, PartialEq, Eq, strum::Display)]
pub enum Algorithm {
    RS256,
    RS384,
    RS512,
    PS256,
    PS384,
    PS512,
    ES256,
    ES384,
    ES512,
    Ed25519,
    Ed448,
    HS256,
    HS384,
    HS512,
    Unknown(String),
}

pub fn validate_algorithm(algorithm: &Algorithm) -> Result<(), ValidationError> {
    if let Algorithm::Unknown(unknown) = algorithm {
        return Err(format!("Unknown algorithm '{unknown}'").into());
    }
    Ok(())
}

impl Algorithm {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "RS256" => Algorithm::RS256,
            "RS384" => Algorithm::RS384,
            "RS512" => Algorithm::RS512,
            "PS256" => Algorithm::PS256,
            "PS384" => Algorithm::PS384,
            "PS512" => Algorithm::PS512,
            "ES256" => Algorithm::ES256,
            "ES384" => Algorithm::ES384,
            "ES512" => Algorithm::ES512,
            "Ed25519" => Algorithm::Ed25519,
            "Ed448" => Algorithm::Ed448,
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            unknown => Algorithm::Unknown(unknown.to_string()),
        }
    }
}

impl FromStr for Algorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RS256" => Ok(Algorithm::RS256),
            "RS384" => Ok(Algorithm::RS384),
            "RS512" => Ok(Algorithm::RS512),
            "PS256" => Ok(Algorithm::PS256),
            "PS384" => Ok(Algorithm::PS384),
            "PS512" => Ok(Algorithm::PS512),
            "ES256" => Ok(Algorithm::ES256),
            "ES384" => Ok(Algorithm::ES384),
            "ES512" => Ok(Algorithm::ES512),
            "Ed25519" => Ok(Algorithm::Ed25519),
            "Ed448" => Ok(Algorithm::Ed448),
            "HS256" => Ok(Algorithm::HS256),
            "HS384" => Ok(Algorithm::HS384),
            "HS512" => Ok(Algorithm::HS512),
            _ => Err(format!("Invalid signature algorithm '{}' found", s)),
        }
    }
}
