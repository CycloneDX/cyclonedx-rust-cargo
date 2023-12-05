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

use crate::xml::read_list_tag;
use crate::{
    errors::XmlReadError,
    external_models::{
        normalized_string::NormalizedString,
        spdx::{SpdxExpression, SpdxIdentifier},
        uri::Uri,
    },
    models,
    xml::{
        closing_tag_or_error, inner_text_or_error, read_lax_validation_tag, read_simple_tag,
        to_xml_read_error, to_xml_write_error, unexpected_element_error, FromXml, ToInnerXml,
        ToXml,
    },
};
use crate::{specs::v1_3::attached_text::AttachedText, utilities::convert_optional};
use crate::{utilities::convert, xml::write_simple_tag};
use serde::{Deserialize, Serialize};
use xml::{name::OwnedName, reader, writer};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Licenses(pub LicenseChoice);

impl From<models::license::Licenses> for Licenses {
    fn from(other: models::license::Licenses) -> Self {
        Licenses(convert(other.0))
    }
}

impl From<Licenses> for models::license::Licenses {
    fn from(other: Licenses) -> Self {
        models::license::Licenses(convert(other.0))
    }
}

const LICENSES_TAG: &str = "licenses";

impl ToXml for Licenses {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(LICENSES_TAG))
            .map_err(to_xml_write_error(LICENSES_TAG))?;

        self.0.write_xml_element(writer)?;

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(LICENSES_TAG))?;
        Ok(())
    }
}

impl FromXml for Licenses {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut licenses: Vec<Lic> = Vec::new();
        let mut expressions: Vec<Expr> = Vec::new();

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(LICENSES_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == LICENSE_TAG => {
                    let license = License::read_xml_element(event_reader, &name, &attributes)?;
                    licenses.push(Lic::Lic(license));
                }
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == EXPRESSION_TAG =>
                {
                    let expression = read_simple_tag(event_reader, &name)?;
                    expressions.push(Expr::Expr(expression));
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        if !licenses.is_empty() && !expressions.is_empty() {
            Err(XmlReadError::UnexpectedElementReadError {
                error: "Got unexpected element. Either license or expression is allowed."
                    .to_string(),
                element: "Licenses".to_string(),
            })
        } else if !licenses.is_empty() {
            Ok(Licenses(LicenseChoice::Licenses(licenses)))
        } else if !expressions.is_empty() {
            Ok(Licenses(LicenseChoice::Expressions(expressions)))
        } else {
            Ok(Licenses(LicenseChoice::Licenses(Vec::new())))
        }

        /*
        let license_choice = license_choice.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: format!("Either {} or {}", LICENSE_TAG.to_string(), EXPRESSION_TAG.to_string()),
            element: element_name.local_name.to_string(),
        })?;
        */
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum LicenseChoice {
    Licenses(Vec<Lic>),
    Expressions(Vec<Expr>),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum Lic {
    #[serde(rename = "license")]
    Lic(License),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum Expr {
    #[serde(rename = "expression")]
    Expr(String),
}

impl From<models::license::LicenseChoice> for LicenseChoice {
    fn from(other: models::license::LicenseChoice) -> Self {
        match other {
            models::license::LicenseChoice::Licenses(l) => {
                let mut licenses = Vec::new();
                for license in l {
                    licenses.push(Lic::Lic(convert(license)));
                }
                Self::Licenses(licenses)
            }
            models::license::LicenseChoice::Expressions(e) => {
                let mut expressions = Vec::new();
                for expression in e {
                    expressions.push(Expr::Expr(expression.0));
                }
                Self::Expressions(expressions)
            }
        }
    }
}

impl From<LicenseChoice> for models::license::LicenseChoice {
    fn from(other: LicenseChoice) -> Self {
        match other {
            LicenseChoice::Licenses(l) => {
                let mut licenses = Vec::new();
                for license in l {
                    match license {
                        Lic::Lic(license) => {
                            licenses.push(convert(license));
                        }
                    }
                }
                Self::Licenses(licenses)
            }
            LicenseChoice::Expressions(e) => {
                let mut expressions = Vec::new();
                for expression in e {
                    match expression {
                        Expr::Expr(expression) => {
                            expressions.push(SpdxExpression(expression));
                        }
                    }
                }
                Self::Expressions(expressions)
            }
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
            LicenseChoice::Licenses(licenses) => {
                for license in licenses {
                    match license {
                        Lic::Lic(license) => {
                            license.write_xml_element(writer)?;
                        }
                    }
                }
            }
            LicenseChoice::Expressions(expressions) => {
                for expression in expressions {
                    match expression {
                        Expr::Expr(expression) => {
                            write_simple_tag(writer, EXPRESSION_TAG, expression)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl FromXml for LicenseChoice {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        match element_name.local_name.as_ref() {
            LICENSE_TAG => Ok(Self::Licenses(read_list_tag(
                event_reader,
                element_name,
                LICENSE_TAG,
            )?)),
            EXPRESSION_TAG => Ok(Self::Expressions(read_list_tag(
                event_reader,
                element_name,
                EXPRESSION_TAG,
            )?)),
            unexpected => Err(XmlReadError::UnexpectedElementReadError {
                error: format!("Got unexpected HUGO2 element {:?}", unexpected),
                element: "Licenses".to_string(),
            }),
        }
    }
}

impl FromXml for Lic {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut lic: Option<Lic> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == LICENSE_TAG => {
                    lic = Some(Lic::Lic(License::read_xml_element(
                        event_reader,
                        element_name,
                        _attributes,
                    )?))
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let lic = lic.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: LICENSE_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(lic)
    }
}

impl FromXml for Expr {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut expr: Option<Expr> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == EXPRESSION_TAG =>
                {
                    expr = Some(Expr::Expr(read_simple_tag(event_reader, &name)?))
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let expr = expr.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: EXPRESSION_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(expr)
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

impl From<models::license::License> for License {
    fn from(other: models::license::License) -> Self {
        Self {
            license_identifier: other.license_identifier.into(),
            text: convert_optional(other.text),
            url: other.url.map(|u| u.to_string()),
        }
    }
}

impl From<License> for models::license::License {
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
            .write(writer::XmlEvent::start_element(LICENSE_TAG))
            .map_err(to_xml_write_error(LICENSE_TAG))?;

        self.license_identifier.write_xml_element(writer)?;

        if let Some(attached_text) = &self.text {
            attached_text.write_xml_named_element(writer, TEXT_TAG)?;
        }

        if let Some(url) = &self.url {
            write_simple_tag(writer, URL_TAG, url)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(LICENSE_TAG))?;

        Ok(())
    }
}

impl FromXml for License {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut license_identifier: Option<LicenseIdentifier> = None;
        let mut text: Option<AttachedText> = None;
        let mut url: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(LICENSE_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ID_TAG || name.local_name == NAME_TAG => {
                    // ID_TAG and NAME_TAG are only allowed once within a LICENSE_TAG
                    if license_identifier.is_none() {
                        license_identifier = Some(LicenseIdentifier::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    } else {
                        return Err(XmlReadError::UnexpectedElementReadError {
                            error: format!(
                                "Got a second {} not allowed within {}",
                                name.local_name, LICENSE_TAG
                            ),
                            element: LICENSE_TAG.to_string(),
                        });
                    }
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == TEXT_TAG => {
                    text = Some(AttachedText::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == URL_TAG => {
                    url = Some(read_simple_tag(event_reader, &name)?)
                }
                // lax validation of any elements from a different schema
                reader::XmlEvent::StartElement { name, .. } => {
                    read_lax_validation_tag(event_reader, &name)?
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }
        let license_identifier =
            license_identifier.ok_or_else(|| XmlReadError::RequiredDataMissing {
                required_field: format!("{} or {}", ID_TAG, NAME_TAG),
                element: LICENSE_TAG.to_string(),
            })?;
        Ok(Self {
            license_identifier,
            text,
            url,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum LicenseIdentifier {
    #[serde(rename = "id")]
    SpdxId(String),
    Name(String),
}

impl From<models::license::LicenseIdentifier> for LicenseIdentifier {
    fn from(other: models::license::LicenseIdentifier) -> Self {
        match other {
            models::license::LicenseIdentifier::SpdxId(spdx) => Self::SpdxId(spdx.0),
            models::license::LicenseIdentifier::Name(name) => Self::Name(name.to_string()),
        }
    }
}

impl From<LicenseIdentifier> for models::license::LicenseIdentifier {
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

impl FromXml for LicenseIdentifier {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        match name.local_name.as_str() {
            ID_TAG => {
                let id = event_reader
                    .next()
                    .map_err(to_xml_read_error(ID_TAG))
                    .and_then(inner_text_or_error(ID_TAG))?;

                event_reader
                    .next()
                    .map_err(to_xml_read_error(ID_TAG))
                    .and_then(closing_tag_or_error(name))?;

                Ok(Self::SpdxId(id))
            }
            NAME_TAG => {
                let license_name = event_reader
                    .next()
                    .map_err(to_xml_read_error(NAME_TAG))
                    .and_then(inner_text_or_error(NAME_TAG))?;

                event_reader
                    .next()
                    .map_err(to_xml_read_error(NAME_TAG))
                    .and_then(closing_tag_or_error(name))?;

                Ok(Self::Name(license_name))
            }
            other => Err(XmlReadError::UnexpectedElementReadError {
                error: format!("Got {} instead of \"name\" or \"id\"", other),
                element: "license identifier".to_string(),
            }),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::{
        external_models::spdx::SpdxExpression,
        specs::v1_3::attached_text::test::{corresponding_attached_text, example_attached_text},
        xml::test::{read_element_from_string, write_element_to_string},
    };

    pub(crate) fn example_licenses() -> Licenses {
        Licenses(LicenseChoice::Expressions(vec![
            example_license_expression(),
        ]))
    }

    pub(crate) fn corresponding_licenses() -> models::license::Licenses {
        models::license::Licenses(corresponding_license_expression())
    }

    pub(crate) fn example_spdx_license() -> License {
        License {
            license_identifier: LicenseIdentifier::SpdxId("spdx id".to_string()),
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        }
    }

    #[allow(unused)]
    pub(crate) fn corresponding_spdx_license() -> models::license::LicenseChoice {
        models::license::LicenseChoice::Licenses(vec![models::license::License {
            license_identifier: models::license::LicenseIdentifier::SpdxId(SpdxIdentifier(
                "spdx id".to_string(),
            )),
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        }])
    }

    pub(crate) fn example_named_license() -> License {
        License {
            license_identifier: LicenseIdentifier::Name("name".to_string()),
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        }
    }

    #[allow(unused)]
    pub(crate) fn corresponding_named_license() -> models::license::LicenseChoice {
        models::license::LicenseChoice::Licenses(vec![models::license::License {
            license_identifier: models::license::LicenseIdentifier::Name(
                NormalizedString::new_unchecked("name".to_string()),
            ),
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        }])
    }

    pub(crate) fn example_license_expression() -> Expr {
        Expr::Expr("expression".to_string())
    }

    pub(crate) fn corresponding_license_expression() -> models::license::LicenseChoice {
        models::license::LicenseChoice::Expressions(vec![SpdxExpression("expression".to_string())])
    }

    #[test]
    fn it_should_read_licenses_without_license_choices_correctly() {
        let input = r#"
<licenses>
</licenses>
"#;
        let actual: Licenses = read_element_from_string(input);
        let expected = Licenses(LicenseChoice::Licenses(Vec::new()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_write_licenses_without_license_choices_correctly() {
        let xml_output = write_element_to_string(Licenses(LicenseChoice::Licenses(Vec::new())));

        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_handle_licenses_correctly_license_choice_licenses() {
        let actual = Licenses(LicenseChoice::Licenses(vec![
            Lic::Lic(example_spdx_license()),
            Lic::Lic(example_named_license()),
        ]));

        insta::assert_json_snapshot!(actual);
    }

    #[test]
    fn it_should_handle_licenses_correctly_license_choice_expressions() {
        let actual = Licenses(LicenseChoice::Expressions(vec![
            example_license_expression(),
            example_license_expression(),
        ]));

        insta::assert_json_snapshot!(actual);
    }

    #[test]
    fn it_should_write_xml_full_license_choice_licenses() {
        let xml_output = write_element_to_string(Licenses(LicenseChoice::Licenses(vec![
            Lic::Lic(example_spdx_license()),
            Lic::Lic(example_named_license()),
        ])));
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_write_xml_full_license_choice_expressions() {
        let xml_output = write_element_to_string(Licenses(LicenseChoice::Expressions(vec![
            example_license_expression(),
            example_license_expression(),
        ])));
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full_license_choice_licenses() {
        let input = r#"
<licenses>
  <license>
    <id>spdx id</id>
    <text content-type="content type" encoding="encoding">content</text>
    <url>url</url>
  </license>
  <license>
    <name>name</name>
    <text content-type="content type" encoding="encoding">content</text>
    <url>url</url>
  </license>
</licenses>
"#;
        let actual: Licenses = read_element_from_string(input);
        let expected = Licenses(LicenseChoice::Licenses(vec![
            Lic::Lic(example_spdx_license()),
            Lic::Lic(example_named_license()),
        ]));
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_xml_full_license_choice_expressions() {
        let input = r#"
<licenses>
  <expression>expression</expression>
  <expression>expression</expression>
</licenses>
"#;
        let actual: Licenses = read_element_from_string(input);
        let expected = Licenses(LicenseChoice::Expressions(vec![
            example_license_expression(),
            example_license_expression(),
        ]));
        assert_eq!(actual, expected);
    }

    /*
        todo: check this test
        #[test]
        fn it_should_fail_xml_full_license_choice_mixed() {
            let input = r#"
    <licenses>
      <license>
        <id>spdx id</id>
        <text content-type="content type" encoding="encoding">content</text>
        <url>url</url>
      </license>
      <license>
        <name>name</name>
        <text content-type="content type" encoding="encoding">content</text>
        <url>url</url>
      </license>
      <expression>MIT</expression>
    </licenses>
    "#;
            let actual: Licenses = read_element_from_string(input);
            let expected = Licenses(
                LicenseChoice::Licenses(vec![
                    Lic::Lic(example_spdx_license()),
                    Lic::Lic(example_named_license()),
                ]));
            assert_eq!(actual, expected);
        }
        */
}
