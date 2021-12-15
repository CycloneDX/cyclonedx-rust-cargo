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
    external_models::{normalized_string::NormalizedString, spdx::SpdxIdentifier, uri::Uri},
    models,
};
use crate::{specs::v1_3::attached_text::AttachedText, utilities::convert_optional};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum LicenseChoice {
    License(Option<License>),
    Expression(Option<String>),
}

impl From<models::LicenseChoice> for LicenseChoice {
    fn from(other: models::LicenseChoice) -> Self {
        match other {
            models::LicenseChoice::License(l) => Self::License(convert_optional(l)),
            models::LicenseChoice::Expression(e) => Self::Expression(e.map(|e| e.to_string())),
        }
    }
}

impl From<LicenseChoice> for models::LicenseChoice {
    fn from(other: LicenseChoice) -> Self {
        match other {
            LicenseChoice::License(l) => Self::License(convert_optional(l)),
            LicenseChoice::Expression(e) => {
                Self::Expression(e.map(NormalizedString::new_unchecked))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct License {
    #[serde(flatten)]
    license_identifier: LicenseIdentifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<AttachedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl From<models::License> for License {
    fn from(other: models::License) -> Self {
        Self {
            license_identifier: other.license_identifier.into(),
            text: convert_optional(other.text),
            url: other.url.map(|u| u.to_string()),
        }
    }
}

impl From<License> for models::License {
    fn from(other: License) -> Self {
        Self {
            license_identifier: other.license_identifier.into(),
            text: convert_optional(other.text),
            url: other.url.map(Uri),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum LicenseIdentifier {
    #[serde(rename = "id")]
    SpdxId(String),
    Name(String),
}

impl From<models::LicenseIdentifier> for LicenseIdentifier {
    fn from(other: models::LicenseIdentifier) -> Self {
        match other {
            models::LicenseIdentifier::SpdxId(spdx) => Self::SpdxId(spdx.0),
            models::LicenseIdentifier::Name(name) => Self::Name(name.to_string()),
        }
    }
}

impl From<LicenseIdentifier> for models::LicenseIdentifier {
    fn from(other: LicenseIdentifier) -> Self {
        match other {
            LicenseIdentifier::SpdxId(spdx) => Self::SpdxId(SpdxIdentifier(spdx)),
            LicenseIdentifier::Name(name) => Self::Name(NormalizedString::new_unchecked(name)),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::specs::v1_3::attached_text::test::{
        corresponding_attached_text, example_attached_text,
    };

    use super::*;

    pub(crate) fn example_spdx_license() -> LicenseChoice {
        LicenseChoice::License(Some(License {
            license_identifier: LicenseIdentifier::SpdxId("spdx id".to_string()),
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        }))
    }

    pub(crate) fn corresponding_spdx_license() -> models::LicenseChoice {
        models::LicenseChoice::License(Some(models::License {
            license_identifier: models::LicenseIdentifier::SpdxId(SpdxIdentifier(
                "spdx id".to_string(),
            )),
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        }))
    }

    pub(crate) fn example_named_license() -> LicenseChoice {
        LicenseChoice::License(Some(License {
            license_identifier: LicenseIdentifier::Name("name".to_string()),
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        }))
    }

    pub(crate) fn corresponding_named_license() -> models::LicenseChoice {
        models::LicenseChoice::License(Some(models::License {
            license_identifier: models::LicenseIdentifier::Name(NormalizedString::new_unchecked(
                "name".to_string(),
            )),
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        }))
    }

    pub(crate) fn example_license_expression() -> LicenseChoice {
        LicenseChoice::Expression(Some("expression".to_string()))
    }

    pub(crate) fn corresponding_license_expression() -> models::LicenseChoice {
        models::LicenseChoice::Expression(Some(NormalizedString::new_unchecked(
            "expression".to_string(),
        )))
    }

    #[test]
    fn it_should_handle_licenses_correctly() {
        let actual = vec![
            example_spdx_license(),
            example_named_license(),
            example_license_expression(),
        ];

        insta::assert_json_snapshot!(actual);
    }
}
