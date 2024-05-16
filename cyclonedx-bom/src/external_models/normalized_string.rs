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

use crate::validation::ValidationError;
use std::fmt::Display;
use std::ops::Deref;

/// A string that does not contain carriage return, line feed, or tab characters
///
/// Defined via the [XML schema](https://www.w3.org/TR/xmlschema-2/#normalizedString)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct NormalizedString(pub(crate) String);

impl NormalizedString {
    /// Construct a `NormalizedString` by replacing all of the invalid characters with spaces
    /// ```
    /// use cyclonedx_bom::prelude::*;
    ///
    /// let normalized_string = NormalizedString::new("A\r\nstring\rwith\ninvalid\tcharacters");
    /// assert_eq!(normalized_string.to_string(), "A string with invalid characters".to_string());
    /// ```
    pub fn new(value: &str) -> Self {
        let value = value.replace("\r\n", " ").replace(['\r', '\n', '\t'], " ");
        NormalizedString(value)
    }

    /// Allow for the existence of invalid inputs from other data sources
    pub(crate) fn new_unchecked(value: String) -> Self {
        NormalizedString(value)
    }
}

impl From<&str> for NormalizedString {
    fn from(input: &str) -> Self {
        NormalizedString::new(input)
    }
}

impl From<NormalizedString> for String {
    fn from(value: NormalizedString) -> Self {
        value.0
    }
}

impl Deref for NormalizedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<NormalizedString> for NormalizedString {
    fn as_ref(&self) -> &NormalizedString {
        self
    }
}

impl AsRef<str> for NormalizedString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for NormalizedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validates a [`NormalizedString`].
pub fn validate_normalized_string(
    normalized_string: &NormalizedString,
) -> Result<(), ValidationError> {
    if normalized_string.contains("\r\n")
        || normalized_string.contains('\r')
        || normalized_string.contains('\n')
        || normalized_string.contains('\t')
    {
        return Err(ValidationError::new(
            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_normalize_strings() {
        assert_eq!(
            NormalizedString("no_whitespace".to_string()),
            NormalizedString::new("no_whitespace")
        );
        assert_eq!(
            NormalizedString("spaces and tabs".to_string()),
            NormalizedString::new("spaces and\ttabs")
        );
        assert_eq!(
            NormalizedString("carriage returns and linefeeds".to_string()),
            NormalizedString::new("carriage\r\nreturns\rand\nlinefeeds")
        );
    }

    #[test]
    fn it_should_pass_validation() {
        assert!(validate_normalized_string(&NormalizedString("no_whitespace".to_string())).is_ok());
    }

    #[test]
    fn it_should_fail_validation() {
        let result = validate_normalized_string(&NormalizedString("spaces and\ttabs".to_string()));

        assert_eq!(
            result,
            Err(ValidationError::new(
                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n",
            ))
        );
    }
}
