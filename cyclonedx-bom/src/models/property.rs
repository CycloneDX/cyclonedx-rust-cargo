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

use crate::{
    external_models::normalized_string::{validate_normalized_string, NormalizedString},
    validation::{Validate, ValidationContext, ValidationResult},
};

use super::bom::SpecVersion;

/// Represents a name-value store that can be used to describe additional data about the components, services, or the BOM that
/// isn’t native to the core specification.
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_propertyType). Please see the
/// [CycloneDX use case](https://cyclonedx.org/use-cases/#properties--name-value-store) for more information and examples.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Properties(pub Vec<Property>);

impl Validate for Properties {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list("inner", &self.0, |property| {
                property.validate_version(version)
            })
            .into()
    }
}

/// Represents an individual property with a name and value
///
/// Defined via the [XML schema](https://cyclonedx.org/docs/1.3/xml/#type_propertyType)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Property {
    pub name: String,
    pub value: NormalizedString,
}

impl Property {
    /// Constructs a `Property` with a name and value
    /// ```
    /// use cyclonedx_bom::models::property::Property;
    ///
    /// let property = Property::new("Foo", "Bar");
    /// ```
    pub fn new(name: impl ToString, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: NormalizedString::new(value),
        }
    }
}

impl Validate for Property {
    fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field("value", &self.value, validate_normalized_string)
            .into()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        models::property::{Properties, Property},
        prelude::NormalizedString,
        validation,
    };
    use pretty_assertions::assert_eq;
    use validation::Validate;

    #[test]
    fn it_should_pass_validation() {
        let validation_result = Properties(vec![Property {
            name: "property name".to_string(),
            value: NormalizedString("property value".to_string()),
        }])
        .validate();

        assert!(validation_result.passed());
    }

    #[test]
    fn it_should_fail_validation() {
        let validation_result = Properties(vec![Property {
            name: "property name".to_string(),
            value: NormalizedString("spaces and \ttabs".to_string()),
        }])
        .validate();

        assert_eq!(
            validation_result,
            validation::list(
                "inner",
                [(
                    0,
                    validation::field(
                        "value",
                        "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                    )
                )]
            ),
        );
    }
}
