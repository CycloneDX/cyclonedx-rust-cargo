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

use spdx::{Expression, ParseMode};
use thiserror::Error;

use crate::{models::bom::BomReference, validation::ValidationError};

/// An identifier for a single, specific license
///
/// The list of valid SPDX license identifiers can be found on the [SPDX website](https://spdx.org/licenses/)
/// ```
/// use cyclonedx_bom::prelude::*;
/// # use cyclonedx_bom::external_models::spdx::SpdxIdentifierError;
/// use std::convert::TryFrom;
///
/// let identifier = String::from("MIT");
/// let spdx_identifier = SpdxIdentifier::try_from(identifier.clone())?;
/// assert_eq!(spdx_identifier.to_string(), identifier);
/// # Ok::<(), SpdxIdentifierError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpdxIdentifier(pub(crate) String);

impl SpdxIdentifier {
    /// Attempt to create an `SpdxIdentifier` using a best-effort translation of the license ID
    ///
    /// ```
    /// use cyclonedx_bom::prelude::*;
    /// # use cyclonedx_bom::external_models::spdx::SpdxIdentifierError;
    ///
    /// let spdx_identifier = SpdxIdentifier::imprecise("Apache 2.0".to_string())?;
    /// assert_eq!(spdx_identifier.to_string(), "Apache-2.0".to_string());
    /// # Ok::<(), SpdxIdentifierError>(())
    /// ```
    pub fn imprecise(value: String) -> Result<Self, SpdxIdentifierError> {
        match spdx::imprecise_license_id(&value) {
            Some(matched_license) => Ok(Self(matched_license.0.name.into())),
            None => Err(SpdxIdentifierError::InvalidImpreciseSpdxIdentifier(
                format!("Not a valid identifier: {value}"),
            )),
        }
    }
}

impl TryFrom<String> for SpdxIdentifier {
    type Error = SpdxIdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match spdx::license_id(&value) {
            Some(_) => Ok(Self(value)),
            None => Err(SpdxIdentifierError::InvalidSpdxIdentifier(format!(
                "Not a valid identifier: {}",
                value
            ))),
        }
    }
}

impl std::fmt::Display for SpdxIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub fn validate_spdx_identifier(identifier: &SpdxIdentifier) -> Result<(), ValidationError> {
    match SpdxIdentifier::try_from(identifier.0.to_string()) {
        Err(_error) => Err(ValidationError::new("SPDX identifier is not valid")),
        _ => Ok(()),
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpdxIdentifierError {
    #[error("Invalid SPDX identifier: {}", .0)]
    InvalidSpdxIdentifier(String),

    #[error("Invalid Imprecise SPDX identifier: {}", .0)]
    InvalidImpreciseSpdxIdentifier(String),
}

/// An expression that describes the set of licenses that cover the software
///
/// The specification for a valid SPDX license expression can be found on the
/// [SPDX website](https://spdx.github.io/spdx-spec/v3.0/annexes/SPDX-license-expressions//)
/// ```
/// use cyclonedx_bom::prelude::*;
/// # use cyclonedx_bom::external_models::spdx::SpdxExpressionError;
/// use std::convert::TryFrom;
///
/// let expression = String::from("MIT OR Apache-2.0");
/// let spdx_expression = SpdxExpression::try_from(expression.clone())?;
/// assert_eq!(spdx_expression.to_string(), expression);
/// # Ok::<(), SpdxExpressionError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpdxExpression {
    pub(crate) bom_ref: Option<BomReference>,
    pub(crate) expression: String,
}

impl SpdxExpression {
    /// Creates a new `SpdxExpression` without validation.
    pub fn new(expression: &str) -> Self {
        Self {
            bom_ref: None,
            expression: expression.to_string(),
        }
    }

    /// Parse a mostly-valid SPDX expression into a valid expression
    ///
    /// Some Rust repositories have a `license` field of `"MIT/Apache-2.0"`,
    /// which is interpreted as `"MIT OR Apache-2.0"`. In order to allow
    /// interoperability, `parse_lax` converts expression with the first form
    /// into the second.
    /// ```
    /// use cyclonedx_bom::prelude::*;
    /// # use cyclonedx_bom::external_models::spdx::SpdxExpressionError;
    ///
    /// let spdx_expression = SpdxExpression::parse_lax("MIT/Apache-2.0".to_string())?;
    /// assert_eq!(spdx_expression.to_string(), "MIT OR Apache-2.0".to_string());
    /// # Ok::<(), SpdxExpressionError>(())
    /// ```
    pub fn parse_lax(value: String) -> Result<Self, SpdxExpressionError> {
        match Expression::parse_mode(&value, ParseMode::LAX) {
            Ok(_) => Self::new(&value).convert_lax(),
            Err(e) => Err(SpdxExpressionError::InvalidLaxSpdxExpression(format!(
                "{}",
                e.reason
            ))),
        }
    }

    fn convert_lax(self) -> Result<Self, SpdxExpressionError> {
        let converted = self.expression.replace('/', " OR ");

        match Self::try_from(converted) {
            Ok(converted) => Ok(converted),
            Err(e) => Err(SpdxExpressionError::InvalidLaxSpdxExpression(format!(
                "{}",
                e
            ))),
        }
    }
}

impl TryFrom<String> for SpdxExpression {
    type Error = SpdxExpressionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match Expression::parse(&value) {
            Ok(_) => Ok(Self {
                bom_ref: None,
                expression: value,
            }),
            Err(e) => Err(SpdxExpressionError::InvalidSpdxExpression(format!(
                "{}",
                e.reason
            ))),
        }
    }
}

impl std::fmt::Display for SpdxExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expression)
    }
}

pub fn validate_spdx_expression(expression: &SpdxExpression) -> Result<(), ValidationError> {
    if Expression::parse(&expression.expression).is_err() {
        return Err(ValidationError::new("SPDX expression is not valid"));
    }
    Ok(())
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpdxExpressionError {
    #[error("Invalid SPDX expression: {}", .0)]
    InvalidSpdxExpression(String),

    #[error("Invalid Lax SPDX expression: {}", .0)]
    InvalidLaxSpdxExpression(String),
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_succeed_in_converting_an_spdx_identifier() {
        let actual =
            SpdxIdentifier::try_from("MIT".to_string()).expect("Failed to parse as an identifier");

        assert_eq!(actual, SpdxIdentifier("MIT".to_string()));
    }

    #[test]
    fn it_should_fail_to_convert_an_invalid_spdx_identifier() {
        let actual = SpdxIdentifier::try_from("MIT OR Apache-2.0".to_string())
            .expect_err("Should have failed to parse as an identifier");

        assert_eq!(
            actual,
            SpdxIdentifierError::InvalidSpdxIdentifier(
                "Not a valid identifier: MIT OR Apache-2.0".to_string()
            )
        );
    }

    #[test]
    fn it_should_succeed_in_converting_an_imprecise_spdx_identifier() {
        let actual =
            SpdxIdentifier::imprecise("mit".to_string()).expect("Failed to parse as an identifier");

        assert_eq!(actual, SpdxIdentifier("MIT".to_string()));
    }

    #[test]
    fn it_should_fail_to_convert_an_invalid_imprecise_spdx_identifier() {
        let actual = SpdxIdentifier::imprecise("GNU General Public License v3".to_string())
            .expect_err("Should have failed to parse as an identifier");

        assert_eq!(
            actual,
            SpdxIdentifierError::InvalidImpreciseSpdxIdentifier(
                "Not a valid identifier: GNU General Public License v3".to_string()
            )
        );
    }

    #[test]
    fn valid_spdx_identifiers_should_pass_validation() {
        let validation_result = validate_spdx_identifier(&SpdxIdentifier("MIT".to_string()));

        assert!(validation_result.is_ok());
    }

    #[test]
    fn invalid_spdx_identifiers_should_fail_validation() {
        let validation_result =
            validate_spdx_identifier(&SpdxIdentifier("MIT OR Apache-2.0".to_string()));

        assert_eq!(
            validation_result,
            Err("SPDX identifier is not valid".into()),
        );
    }

    #[test]
    fn it_should_succeed_in_converting_an_spdx_expression() {
        let actual = SpdxExpression::try_from("MIT OR Apache-2.0".to_string())
            .expect("Failed to parse as a license");
        assert_eq!(actual, SpdxExpression::new("MIT OR Apache-2.0"));
    }

    #[test]
    fn it_should_succeed_in_converting_a_partially_valid_spdx_expression() {
        let actual = SpdxExpression::parse_lax("MIT/Apache-2.0".to_string())
            .expect("Failed to parse as a license");
        assert_eq!(actual, SpdxExpression::new("MIT OR Apache-2.0"));
    }

    #[test]
    fn it_should_fail_to_convert_an_invalid_spdx_expression() {
        let actual = SpdxExpression::try_from("not a real license".to_string())
            .expect_err("Should have failed to parse as a license");
        assert_eq!(
            actual,
            SpdxExpressionError::InvalidSpdxExpression("unknown term".to_string())
        );
    }

    #[test]
    fn valid_spdx_expressions_should_pass_validation() {
        let validation_result = validate_spdx_expression(&SpdxExpression::new("MIT OR Apache-2.0"));

        assert!(validation_result.is_ok());
    }

    #[test]
    fn invalid_spdx_expressions_should_fail_validation() {
        let validation_result =
            validate_spdx_expression(&SpdxExpression::new("not a real license"));

        assert_eq!(
            validation_result,
            Err("SPDX expression is not valid".into()),
        );
    }
}
