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

use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct SpdxIdentifier(pub(crate) String);

impl TryFrom<String> for SpdxIdentifier {
    type Error = SpdxIdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match license_exprs::validate_license_expr(&value) {
            Ok(()) => Ok(Self(value)),
            Err(e) => Err(SpdxIdentifierError::InvalidSpdxExpression(format!("{}", e))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SpdxIdentifierError {
    InvalidSpdxExpression(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_succeed_in_converting_an_spdx_expression() {
        let actual = SpdxIdentifier::try_from("MIT OR Apache-2.0".to_string())
            .expect("Failed to parse as a license");
        assert_eq!(actual, SpdxIdentifier("MIT OR Apache-2.0".to_string()));
    }

    #[test]
    fn it_should_fail_to_convert_an_invalid_spdx_expression() {
        let actual = SpdxIdentifier::try_from("not a real license".to_string())
            .expect_err("Should have failed to parse as a license");
        assert_eq!(
            actual,
            SpdxIdentifierError::InvalidSpdxExpression(
                "unknown license or other term: not".to_string()
            )
        );
    }
}
