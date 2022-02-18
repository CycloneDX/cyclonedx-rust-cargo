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
    xml::{to_xml_write_error, ToInnerXml, ToXml},
};
use crate::{specs::v1_3::attached_text::AttachedText, utilities::convert_optional};
use crate::{utilities::convert_vec, xml::write_simple_tag};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Licenses(Vec<LicenseChoice>);

impl From<models::Licenses> for Licenses {
    fn from(other: models::Licenses) -> Self {
        Licenses(convert_vec(other.0))
    }
}

impl From<Licenses> for models::Licenses {
    fn from(other: Licenses) -> Self {
        models::Licenses(convert_vec(other.0))
    }
}

const LICENSES_TAG: &str = "licenses";

impl ToXml for Licenses {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(LICENSES_TAG))
            .map_err(to_xml_write_error(LICENSES_TAG))?;

        for license in &self.0 {
            license.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(LICENSES_TAG))?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum LicenseChoice {
    License(License),
    Expression(String),
}

impl From<models::LicenseChoice> for LicenseChoice {
    fn from(other: models::LicenseChoice) -> Self {
        match other {
            models::LicenseChoice::License(l) => Self::License(l.into()),
            models::LicenseChoice::Expression(e) => Self::Expression(e.to_string()),
        }
    }
}

impl From<LicenseChoice> for models::LicenseChoice {
    fn from(other: LicenseChoice) -> Self {
        match other {
            LicenseChoice::License(l) => Self::License(l.into()),
            LicenseChoice::Expression(e) => Self::Expression(NormalizedString::new_unchecked(e)),
        }
    }
}

const EXPRESSION_TAG: &str = "expression";

impl ToXml for LicenseChoice {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        match self {
            LicenseChoice::License(l) => {
                l.write_xml_element(writer)?;
            }
            LicenseChoice::Expression(e) => {
                write_simple_tag(writer, EXPRESSION_TAG, e)?;
            }
        }

        Ok(())
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

const LICENSE_TAG: &str = "license";
const TEXT_TAG: &str = "text";
const URL_TAG: &str = "url";

impl ToXml for License {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(LICENSE_TAG))
            .map_err(to_xml_write_error(LICENSE_TAG))?;

        self.license_identifier.write_xml_element(writer)?;

        if let Some(attached_text) = &self.text {
            attached_text.write_xml_named_element(writer, TEXT_TAG)?;
        }

        if let Some(url) = &self.url {
            write_simple_tag(writer, URL_TAG, url)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(LICENSE_TAG))?;

        Ok(())
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

const ID_TAG: &str = "id";
const NAME_TAG: &str = "name";

impl ToXml for LicenseIdentifier {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        match self {
            LicenseIdentifier::SpdxId(spdx_id) => {
                write_simple_tag(writer, ID_TAG, spdx_id)?;
            }
            LicenseIdentifier::Name(name) => {
                write_simple_tag(writer, NAME_TAG, name)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        specs::v1_3::attached_text::test::{corresponding_attached_text, example_attached_text},
        xml::test::write_element_to_string,
    };

    use super::*;

    pub(crate) fn example_licenses() -> Licenses {
        Licenses(vec![example_license_expression()])
    }

    pub(crate) fn corresponding_licenses() -> models::Licenses {
        models::Licenses(vec![corresponding_license_expression()])
    }

    pub(crate) fn example_spdx_license() -> LicenseChoice {
        LicenseChoice::License(License {
            license_identifier: LicenseIdentifier::SpdxId("spdx id".to_string()),
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        })
    }

    #[allow(unused)]
    pub(crate) fn corresponding_spdx_license() -> models::LicenseChoice {
        models::LicenseChoice::License(models::License {
            license_identifier: models::LicenseIdentifier::SpdxId(SpdxIdentifier(
                "spdx id".to_string(),
            )),
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        })
    }

    pub(crate) fn example_named_license() -> LicenseChoice {
        LicenseChoice::License(License {
            license_identifier: LicenseIdentifier::Name("name".to_string()),
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        })
    }

    #[allow(unused)]
    pub(crate) fn corresponding_named_license() -> models::LicenseChoice {
        models::LicenseChoice::License(models::License {
            license_identifier: models::LicenseIdentifier::Name(NormalizedString::new_unchecked(
                "name".to_string(),
            )),
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        })
    }

    pub(crate) fn example_license_expression() -> LicenseChoice {
        LicenseChoice::Expression("expression".to_string())
    }

    pub(crate) fn corresponding_license_expression() -> models::LicenseChoice {
        models::LicenseChoice::Expression(NormalizedString::new_unchecked("expression".to_string()))
    }

    #[test]
    fn it_should_handle_licenses_correctly() {
        let actual = Licenses(vec![
            example_spdx_license(),
            example_named_license(),
            example_license_expression(),
        ]);

        insta::assert_json_snapshot!(actual);
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(Licenses(vec![
            example_spdx_license(),
            example_named_license(),
            example_license_expression(),
        ]));
        insta::assert_snapshot!(xml_output);
    }
}
