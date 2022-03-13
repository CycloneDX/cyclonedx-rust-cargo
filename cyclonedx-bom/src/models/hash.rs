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

#[derive(Debug, PartialEq)]
pub struct Hash {
    pub alg: HashAlgorithm,
    pub content: HashValue,
}

#[derive(Debug, PartialEq)]
pub struct Hashes(pub Vec<Hash>);

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum HashAlgorithm {
    MD5,
    SHA1,
    SHA256,
    SHA384,
    SHA512,
    SHA3_256,
    SHA3_384,
    SHA3_512,
    BLAKE2b_256,
    BLAKE2b_384,
    BLAKE2b_512,
    BLAKE3,
    #[doc(hidden)]
    UnknownHashAlgorithm(String),
}

impl ToString for HashAlgorithm {
    fn to_string(&self) -> String {
        match self {
            HashAlgorithm::MD5 => "MD5",
            HashAlgorithm::SHA1 => "SHA-1",
            HashAlgorithm::SHA256 => "SHA-256",
            HashAlgorithm::SHA384 => "SHA-384",
            HashAlgorithm::SHA512 => "SHA-512",
            HashAlgorithm::SHA3_256 => "SHA3-256",
            HashAlgorithm::SHA3_384 => "SHA3-384",
            HashAlgorithm::SHA3_512 => "SHA3-512",
            HashAlgorithm::BLAKE2b_256 => "BLAKE2b-256",
            HashAlgorithm::BLAKE2b_384 => "BLAKE2b-384",
            HashAlgorithm::BLAKE2b_512 => "BLAKE2b-512",
            HashAlgorithm::BLAKE3 => "BLAKE3",
            HashAlgorithm::UnknownHashAlgorithm(un) => un,
        }
        .to_string()
    }
}

impl HashAlgorithm {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "MD5" => Self::MD5,
            "SHA-1" => Self::SHA1,
            "SHA-256" => Self::SHA256,
            "SHA-384" => Self::SHA384,
            "SHA-512" => Self::SHA512,
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

#[derive(Debug, PartialEq)]
pub struct HashValue(pub(crate) String); // TODO: validate
