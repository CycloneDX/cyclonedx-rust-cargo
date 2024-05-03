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
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Display,
    hash::Hash,
};

use indexmap::{
    map::{Entry::Vacant, IntoIter},
    IndexMap,
};

use crate::models::bom::SpecVersion;

/// Contains all collected validation errors.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Maps names to validation errors.
    pub(crate) inner: IndexMap<String, ValidationErrorsKind>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        ValidationResult::new()
    }
}

impl From<Vec<ValidationResult>> for ValidationResult {
    fn from(errors: Vec<ValidationResult>) -> Self {
        // merge all errors into one struct.
        let mut result = ValidationResult::new();
        for error in errors.into_iter() {
            for (key, value) in error.inner.into_iter() {
                result.inner.insert(key, value);
            }
        }
        result
    }
}

impl From<Result<(), ValidationError>> for ValidationResult {
    fn from(value: Result<(), ValidationError>) -> Self {
        match value {
            Ok(()) => ValidationResult::default(),
            Err(error) => {
                let mut result = ValidationResult::default();
                result.add_custom("", error);
                result
            }
        }
    }
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    /// Returns `true` if there are no errors.
    pub fn passed(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns `true` if there are errors.
    pub fn has_errors(&self) -> bool {
        !self.inner.is_empty()
    }

    /// Returns the error with given name, if available
    pub fn error(&self, field: &str) -> Option<&ValidationErrorsKind> {
        self.inner.get(&field.to_string())
    }

    pub fn has_error(&self, field: &str) -> bool {
        self.inner.contains_key(field)
    }

    /// Returns an Iterator over all errors, consumes the [`ValidationResult`].
    pub fn errors(self) -> IntoIter<String, ValidationErrorsKind> {
        self.inner.into_iter()
    }

    /// Adds a nested object kind
    fn add_nested(&mut self, nested_name: &str, errors_kind: ValidationErrorsKind) {
        if let Vacant(entry) = self.inner.entry(nested_name.to_string()) {
            entry.insert(errors_kind);
        } else {
            panic!("Attempt to replace non-empty nested entry")
        }
    }

    /// Adds a single [`ValidationError`] for an enum variant.
    fn add_enum(&mut self, enum_name: &str, validation_error: ValidationError) {
        if let Vacant(entry) = self.inner.entry(enum_name.to_string()) {
            entry.insert(ValidationErrorsKind::Enum(validation_error));
        } else {
            panic!("Attempt to replace non-empty enum entry")
        }
    }

    /// Adds a single field [`ValidationError`].
    fn add_field(&mut self, field_name: &str, validation_error: ValidationError) {
        if let ValidationErrorsKind::Field(ref mut vec) = self
            .inner
            .entry(field_name.to_string())
            .or_insert_with(|| ValidationErrorsKind::Field(vec![]))
        {
            vec.push(validation_error);
        } else {
            panic!("Found a non-field ValidationErrorsKind");
        }
    }

    /// Adds a list of validation errors for a custom entry.
    fn add_custom(&mut self, custom_name: &str, validation_error: ValidationError) {
        if let ValidationErrorsKind::Custom(ref mut vec) = self
            .inner
            .entry(custom_name.to_string())
            .or_insert_with(|| ValidationErrorsKind::Custom(vec![]))
        {
            vec.push(validation_error);
        } else {
            panic!("Found a non-custom ValidationErrorsKind");
        }
    }
}

/// Collects validation results in a hierarchy, recommended to use in `Validate` implementations.
#[derive(Debug)]
pub struct ValidationContext {
    state: ValidationResult,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationContext {
    pub fn new() -> Self {
        Self {
            state: ValidationResult::default(),
        }
    }

    pub fn add_field<T>(
        &mut self,
        field_name: &str,
        field: T,
        validation: impl FnOnce(T) -> Result<(), ValidationError>,
    ) -> &mut Self {
        if let Err(validation_error) = validation(field) {
            self.state.add_field(field_name, validation_error);
        }
        self
    }

    pub fn add_field_option<T>(
        &mut self,
        field_name: &str,
        field: Option<T>,
        validation: impl FnOnce(T) -> Result<(), ValidationError>,
    ) -> &mut Self {
        if let Some(field) = field {
            self.add_field(field_name, field, validation);
        }
        self
    }

    pub fn add_enum<T>(
        &mut self,
        enum_name: &str,
        enum_type: &T,
        validation: impl FnOnce(&T) -> Result<(), ValidationError>,
    ) -> &mut Self {
        if let Err(error) = validation(enum_type) {
            self.state.add_enum(enum_name, error);
        }
        self
    }

    pub fn add_enum_option<T>(
        &mut self,
        enum_name: &str,
        enum_type: Option<&T>,
        validation: impl FnOnce(&T) -> Result<(), ValidationError>,
    ) -> &mut Self {
        if let Some(enum_type) = enum_type {
            self.add_enum(enum_name, enum_type, validation);
        }
        self
    }

    pub fn add_list<'a, T, I, Output>(
        &mut self,
        field_name: &str,
        list: T,
        validation: impl Fn(&'a I) -> Output,
    ) -> &mut Self
    where
        I: 'a,
        T: IntoIterator<Item = &'a I>,
        Output: Into<ValidationResult>,
    {
        let child_errors = list
            .into_iter()
            .map(|item| validation(item).into())
            .enumerate()
            .filter_map(|(index, result)| {
                if result.has_errors() {
                    Some((index, result))
                } else {
                    None
                }
            })
            .collect::<BTreeMap<usize, ValidationResult>>();

        if !child_errors.is_empty() {
            self.state
                .add_nested(field_name, ValidationErrorsKind::List(child_errors));
        }
        self
    }
    pub fn add_unique_list<'a, T, I, Output>(
        &mut self,
        field_name: &str,
        list: T,
        validation: impl Fn(&'a I) -> Output,
    ) -> &mut Self
    where
        I: 'a + Eq + Hash,
        T: IntoIterator<Item = &'a I>,
        Output: Into<ValidationResult>,
    {
        let mut set = HashSet::new();
        let mut child_errors = BTreeMap::new();

        for (index, item) in list.into_iter().enumerate() {
            if !set.insert(item) {
                child_errors.insert(index, Err(ValidationError::new("repeated element")).into());
            } else {
                let result = validation(item).into();
                if result.has_errors() {
                    child_errors.insert(index, result);
                }
            }
        }

        if !child_errors.is_empty() {
            self.state
                .add_nested(field_name, ValidationErrorsKind::List(child_errors));
        }
        self
    }

    pub fn add_list_option<'a, T, I, Output>(
        &mut self,
        list_name: &str,
        list: Option<T>,
        validation: impl Fn(&'a I) -> Output,
    ) -> &mut Self
    where
        I: 'a,
        T: IntoIterator<Item = &'a I>,
        Output: Into<ValidationResult>,
    {
        if let Some(list) = list {
            self.add_list(list_name, list, validation);
        }
        self
    }

    pub fn add_unique_list_option<'a, T, I, Output>(
        &mut self,
        list_name: &str,
        list: Option<T>,
        validation: impl Fn(&'a I) -> Output,
    ) -> &mut Self
    where
        I: 'a + Eq + Hash,
        T: IntoIterator<Item = &'a I>,
        Output: Into<ValidationResult>,
    {
        if let Some(list) = list {
            self.add_unique_list(list_name, list, validation);
        }
        self
    }

    pub fn add_struct<T>(
        &mut self,
        struct_name: &str,
        r#struct: &T,
        version: SpecVersion,
    ) -> &mut Self
    where
        T: Validate,
    {
        let result = r#struct.validate_version(version);
        if result.has_errors() {
            self.state
                .add_nested(struct_name, ValidationErrorsKind::Struct(result));
        }
        self
    }

    pub fn add_struct_option<T: Validate>(
        &mut self,
        struct_name: &str,
        r#struct: Option<&T>,
        version: SpecVersion,
    ) -> &mut Self {
        if let Some(r#struct) = r#struct {
            self.add_struct(struct_name, r#struct, version);
        }
        self
    }

    /// Adds a custom validation error.
    ///
    /// A custom field is useful for properties that are not directly part of the Bom hierarchy, but
    /// should be validated too, for example: dependencies between fields, e.g. bom-ref.
    pub fn add_custom(
        &mut self,
        custom_name: &str,
        error: impl Into<ValidationError>,
    ) -> &mut Self {
        self.state.add_custom(custom_name, error.into());
        self
    }
}

impl From<ValidationContext> for ValidationResult {
    fn from(context: ValidationContext) -> Self {
        context.state
    }
}

impl From<&mut ValidationContext> for ValidationResult {
    fn from(context: &mut ValidationContext) -> Self {
        context.state.clone()
    }
}

/// The trait that SBOM structs need to implement to validate their content.
pub trait Validate {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult;

    fn validate(&self) -> ValidationResult {
        self.validate_version(SpecVersion::default())
    }
}

/// A single validation error with a message, useful to log / display for user.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub message: String,
}

impl From<String> for ValidationError {
    fn from(message: String) -> Self {
        ValidationError { message }
    }
}

impl From<&str> for ValidationError {
    fn from(message: &str) -> Self {
        ValidationError::new(message)
    }
}

impl ValidationError {
    pub fn new<D: Display>(message: D) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

/// Implements possible hierarchy of a structured SBOM to collect all [`ValidationError`] in.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorsKind {
    /// Collects all field validation errors in context of a struct
    Struct(ValidationResult),
    /// Collects all child elements in context of a list, the key is the index into the list, e.g. `Vec`
    List(BTreeMap<usize, ValidationResult>),
    /// Contains the list of validation errors for a single field, e.g. struct field.
    Field(Vec<ValidationError>),
    /// Represents a single error for an Enum variant.
    Enum(ValidationError),
    /// A list validation errors for a custom field.
    Custom(Vec<ValidationError>),
}

// --------------------------- Helper functions for tests -------------------------

/// Function to create an enum based error.
#[cfg(test)]
pub(crate) fn r#enum(enum_name: &str, error: impl Into<ValidationError>) -> ValidationResult {
    let mut result = ValidationResult::default();
    result.add_enum(enum_name, error.into());
    result
}

#[cfg(test)]
pub(crate) fn r#struct(struct_name: &str, errors: impl Into<ValidationResult>) -> ValidationResult {
    let mut result = ValidationResult::default();
    result.add_nested(struct_name, ValidationErrorsKind::Struct(errors.into()));
    result
}

#[cfg(test)]
pub(crate) fn list<T>(
    field_name: &str,
    validation_errors: impl IntoIterator<Item = (usize, T)>,
) -> ValidationResult
where
    T: Into<ValidationResult>,
{
    let list = validation_errors
        .into_iter()
        .map(|(index, errors)| (index, errors.into()))
        .collect::<BTreeMap<usize, ValidationResult>>();

    let mut result = ValidationResult::default();
    result.add_nested(field_name, ValidationErrorsKind::List(list));
    result
}

#[cfg(test)]
pub(crate) fn field(field_name: &str, error: impl Into<ValidationError>) -> ValidationResult {
    let mut result = ValidationResult::default();
    result.add_field(field_name, error.into());
    result
}

#[cfg(test)]
pub(crate) fn custom<I, T>(custom_name: &str, validation_errors: I) -> ValidationResult
where
    I: IntoIterator<Item = T>,
    T: Into<ValidationError>,
{
    let validation_errors = validation_errors
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<ValidationError>>();
    let mut result = ValidationResult::default();
    for error in validation_errors {
        result.add_custom(custom_name, error);
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::{
        models::bom::SpecVersion,
        validation::{field, r#enum, r#struct, Validate, ValidationErrorsKind, ValidationResult},
    };

    use super::{ValidationContext, ValidationError};

    #[test]
    fn has_error() {
        let mut result = ValidationResult::new();
        result.add_field("test", ValidationError::new("missing"));

        assert!(result.has_error("test"));
        assert!(!result.has_error("haha"));
    }

    #[test]
    fn has_errors() {
        let mut result = ValidationResult::new();
        assert!(!result.has_errors());

        result.add_field("hello", ValidationError::new("again"));
        assert!(result.has_errors());
    }

    #[test]
    fn build_validation_errors_enum() {
        let result = r#enum("hello", "world");
        assert_eq!(
            result.error("hello"),
            Some(&ValidationErrorsKind::Enum("world".into()))
        );
    }

    #[test]
    fn build_validation_errors_hierarchy() {
        struct Nested {
            name: String,
        }

        impl Validate for Nested {
            fn validate_version(&self, _version: SpecVersion) -> ValidationResult {
                ValidationContext::new()
                    .add_field("name", &self.name, |_name| {
                        Err(ValidationError::new("Failed"))
                    })
                    .into()
            }
        }

        let validation_result: ValidationResult = ValidationContext::new()
            .add_enum("test", &2, |_| Err("not a variant".into()))
            .add_struct(
                "nested",
                &Nested {
                    name: "hello".to_string(),
                },
                SpecVersion::V1_3,
            )
            .into();

        assert_eq!(
            validation_result,
            vec![
                r#enum("test", "not a variant"),
                r#struct("nested", field("name", "Failed")),
            ]
            .into()
        );
    }
}
