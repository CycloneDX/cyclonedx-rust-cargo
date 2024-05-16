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

use thiserror::Error;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

use crate::validation::ValidationError;

/// For the purposes of CycloneDX SBOM documents, `DateTime` is a ISO8601 formatted timestamp
///
/// The corresponding CycloneDX XML schema definition is the [`xs` namespace](https://cyclonedx.org/docs/1.3/xml/#ns_xs), which defines the [`dateTime`](https://www.w3.org/TR/xmlschema11-2/#dateTime)) format.
///
/// A valid timestamp can be created from a [`String`](std::string::String) using the [`TryFrom`](std::convert::TryFrom) / [`TryInto`](std::convert::TryInto) traits.
///
/// ```
/// use cyclonedx_bom::external_models::date_time::DateTime;
/// use std::convert::TryInto;
///
/// let timestamp = String::from("1970-01-01T00:00:00Z");
/// let date_time: DateTime = timestamp.clone().try_into().expect("Failed to parse as DateTime");
///
/// assert_eq!(date_time.to_string(), timestamp);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DateTime(pub(crate) String);

pub fn validate_date_time(date_time: &DateTime) -> Result<(), ValidationError> {
    if OffsetDateTime::parse(&date_time.0, &Iso8601::DEFAULT).is_err() {
        return Err("DateTime does not conform to ISO 8601".into());
    }
    Ok(())
}

impl DateTime {
    pub fn now() -> Result<Self, DateTimeError> {
        let now = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .map_err(|_| DateTimeError::FailedCurrentTime)?;
        Ok(Self(now))
    }
}

impl TryFrom<String> for DateTime {
    type Error = DateTimeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match OffsetDateTime::parse(&value, &Iso8601::DEFAULT) {
            Ok(_) => Ok(Self(value)),
            Err(e) => Err(DateTimeError::InvalidDateTime(format!(
                "DateTime does not conform to ISO 8601: {}",
                e
            ))),
        }
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for DateTime {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DateTimeError {
    #[error("Invalid DateTime: {}", .0)]
    InvalidDateTime(String),

    #[error("Failed to get current time")]
    FailedCurrentTime,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{external_models::validate_date_time, prelude::DateTime};

    #[test]
    fn valid_datetimes_should_pass_validation() {
        let validation_result =
            validate_date_time(&DateTime("1969-06-28T01:20:00.00-04:00".to_string()));

        assert!(validation_result.is_ok());
    }

    #[test]
    fn invalid_datetimes_should_fail_validation() {
        let validation_result = validate_date_time(&DateTime("invalid date".to_string()));

        assert_eq!(
            validation_result,
            Err("DateTime does not conform to ISO 8601".into()),
        );
    }
}
