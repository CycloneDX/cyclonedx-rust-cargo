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

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Hash {
    alg: String,
    content: HashValue,
}

impl From<models::Hash> for Hash {
    fn from(other: models::Hash) -> Self {
        Self {
            alg: other.alg.to_string(),
            content: other.content.into(),
        }
    }
}

impl From<Hash> for models::Hash {
    fn from(other: Hash) -> Self {
        Self {
            alg: models::HashAlgorithm::new_unchecked(other.alg),
            content: other.content.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct HashValue(String);

impl From<models::HashValue> for HashValue {
    fn from(other: models::HashValue) -> Self {
        Self(other.0)
    }
}

impl From<HashValue> for models::HashValue {
    fn from(other: HashValue) -> Self {
        Self(other.0)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub(crate) fn example_hash() -> Hash {
        Hash {
            alg: "algorithm".to_string(),
            content: HashValue("hash value".to_string()),
        }
    }

    pub(crate) fn corresponding_hash() -> models::Hash {
        models::Hash {
            alg: models::HashAlgorithm::UnknownHashAlgorithm("algorithm".to_string()),
            content: models::HashValue("hash value".to_string()),
        }
    }
}
