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

/// Enveloped signature in [JSON Signature Format (JSF)](https://cyberphone.github.io/doc/security/jsf.html)
#[derive(Debug, PartialEq, Eq)]
pub struct Signature {
    /// Signature algorithm.
    pub algorithm: Algorithm,
    /// The signature data.
    pub value: String,
}

/*
/// Enveloped signature in [JSON Signature Format (JSF)](https://cyberphone.github.io/doc/security/jsf.html)
#[derive(Debug, PartialEq, Eq)]
pub enum Signature {
    /// Multiple signatures
    Signers(Vec<Signer>),
    /// A single signature chain
    Chain(Signer),
    /// A single signature
    Signature(Signer),
}

/// For now the [`Signer`] struct only holds algorithm and value
#[derive(Debug, PartialEq, Eq)]
pub struct Signer {
    /// Signature algorithm.
    pub algorithm: Algorithm,
    /// The signature data.
    pub value: String,
}
*/

/// Supported signature algorithms.
#[derive(Debug, PartialEq, Eq)]
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

impl ToString for Algorithm {
    fn to_string(&self) -> String {
        let s = match self {
            Algorithm::RS256 => "RS256",
            Algorithm::RS384 => "RS384",
            Algorithm::RS512 => "RS512",
            Algorithm::PS256 => "PS256",
            Algorithm::PS384 => "PS384",
            Algorithm::PS512 => "PS512",
            Algorithm::ES256 => "ES256",
            Algorithm::ES384 => "ES384",
            Algorithm::ES512 => "ES512",
            Algorithm::Ed25519 => "Ed25519",
            Algorithm::Ed448 => "Ed448",
            Algorithm::HS256 => "HS256",
            Algorithm::HS384 => "HS384",
            Algorithm::HS512 => "HS512",
        };
        s.to_string()
    }
}
