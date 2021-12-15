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

use crate::{external_models::normalized_string::NormalizedString, models};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Properties(Vec<Property>);

impl From<models::Properties> for Properties {
    fn from(other: models::Properties) -> Self {
        Self(other.0.into_iter().map(std::convert::Into::into).collect())
    }
}

impl From<Properties> for models::Properties {
    fn from(other: Properties) -> Self {
        Self(other.0.into_iter().map(std::convert::Into::into).collect())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Property {
    name: String,
    value: String,
}

impl From<models::Property> for Property {
    fn from(other: models::Property) -> Self {
        Self {
            name: other.name,
            value: other.value.0,
        }
    }
}

impl From<Property> for models::Property {
    fn from(other: Property) -> Self {
        Self {
            name: other.name,
            value: NormalizedString::new_unchecked(other.value),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub(crate) fn example_properties() -> Properties {
        Properties(vec![Property {
            name: "name".to_string(),
            value: "value".to_string(),
        }])
    }

    pub(crate) fn corresponding_properties() -> models::Properties {
        models::Properties(vec![models::Property {
            name: "name".to_string(),
            value: NormalizedString::new_unchecked("value".to_string()),
        }])
    }
}
