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

/// A string that does not contain carriage return, line feed, or tab characters.
/// Defined via the [XML schema](https://www.w3.org/TR/xmlschema-2/#normalizedString)
#[derive(Debug, Default, PartialEq)]
pub struct NormalizedString(pub(crate) String);

impl NormalizedString {
    pub fn new(value: &str) -> Self {
        let value = value
            .replace("\r\n", " ")
            .replace('\r', " ")
            .replace('\n', " ")
            .replace('\t', " ");
        NormalizedString(value)
    }

    /// Allow for the existence of invalid inputs from other data sources
    pub(crate) fn new_unchecked(value: String) -> Self {
        NormalizedString(value)
    }
}

impl ToString for NormalizedString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
}
