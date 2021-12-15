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

use crate::{models, utilities::convert_optional_vec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Composition {
    aggregate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    assemblies: Option<Vec<BomReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Vec<BomReference>>,
}

impl From<models::Composition> for Composition {
    fn from(other: models::Composition) -> Self {
        Self {
            aggregate: other.aggregate.to_string(),
            assemblies: convert_optional_vec(other.assemblies),
            dependencies: convert_optional_vec(other.dependencies),
        }
    }
}

impl From<Composition> for models::Composition {
    fn from(other: Composition) -> Self {
        Self {
            aggregate: models::AggregateType::new_unchecked(other.aggregate),
            assemblies: convert_optional_vec(other.assemblies),
            dependencies: convert_optional_vec(other.dependencies),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct BomReference(String);

impl From<models::BomReference> for BomReference {
    fn from(other: models::BomReference) -> Self {
        Self(other.0)
    }
}

impl From<BomReference> for models::BomReference {
    fn from(other: BomReference) -> Self {
        Self(other.0)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub(crate) fn example_composition() -> Composition {
        Composition {
            aggregate: "aggregate".to_string(),
            assemblies: Some(vec![BomReference("assembly".to_string())]),
            dependencies: Some(vec![BomReference("dependency".to_string())]),
        }
    }

    pub(crate) fn corresponding_composition() -> models::Composition {
        models::Composition {
            aggregate: models::AggregateType::UnknownAggregateType("aggregate".to_string()),
            assemblies: Some(vec![models::BomReference("assembly".to_string())]),
            dependencies: Some(vec![models::BomReference("dependency".to_string())]),
        }
    }
}
