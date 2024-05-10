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

use cyclonedx_bom_macros::versioned;

#[versioned("1.3", "1.4", "1.5")]
pub(crate) mod base {
    use crate::models;
    use crate::models::bom::BomReference;
    #[versioned("1.5")]
    use crate::specs::{common::property::Properties, v1_5::licensing::Licensing};
    use crate::xml::{optional_attribute, write_close_tag, write_simple_tag};
    use crate::{
        errors::XmlReadError,
        external_models::{
            normalized_string::NormalizedString,
            spdx::{SpdxExpression, SpdxIdentifier},
            uri::Uri,
        },
        utilities::convert_vec,
        xml::{
            closing_tag_or_error, inner_text_or_error, read_lax_validation_tag, read_simple_tag,
            to_xml_read_error, to_xml_write_error, unexpected_element_error, FromXml, ToInnerXml,
            ToXml,
        },
    };
    use crate::{specs::common::attached_text::AttachedText, utilities::convert_optional};
    use serde::{Deserialize, Serialize};
    use xml::{name::OwnedName, reader, writer};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(transparent)]
    pub(crate) struct Licenses(Vec<LicenseChoice>);

    impl From<models::license::Licenses> for Licenses {
        fn from(other: models::license::Licenses) -> Self {
            Licenses(convert_vec(other.0))
        }
    }

    impl From<Licenses> for models::license::Licenses {
        fn from(other: Licenses) -> Self {
            models::license::Licenses(convert_vec(other.0))
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

            for license in &self.0 {
                license.write_xml_element(writer)?;
            }

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
            let mut licenses = Vec::new();

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(LICENSES_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == LICENSE_TAG || name.local_name == EXPRESSION_TAG => {
                        licenses.push(LicenseChoice::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            Ok(Licenses(licenses))
        }
    }

    #[allow(clippy::large_enum_variant)]
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) enum LicenseChoice {
        License(License),
        #[serde(untagged)]
        Expression(Expression),
    }

    impl From<models::license::LicenseChoice> for LicenseChoice {
        fn from(other: models::license::LicenseChoice) -> Self {
            match other {
                models::license::LicenseChoice::License(l) => Self::License(l.into()),
                models::license::LicenseChoice::Expression(e) => Self::Expression(e.into()),
            }
        }
    }

    impl From<LicenseChoice> for models::license::LicenseChoice {
        fn from(other: LicenseChoice) -> Self {
            match other {
                LicenseChoice::License(l) => Self::License(l.into()),
                LicenseChoice::Expression(e) => Self::Expression(e.into()),
            }
        }
    }

    const BOM_REF_ATTR: &str = "bom-ref";
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
                    e.write_xml_element(writer)?;
                }
            }

            Ok(())
        }
    }

    impl FromXml for LicenseChoice {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            match element_name.local_name.as_ref() {
                LICENSE_TAG => Ok(Self::License(License::read_xml_element(
                    event_reader,
                    element_name,
                    attributes,
                )?)),
                EXPRESSION_TAG => Ok(Self::Expression(Expression::read_xml_element(
                    event_reader,
                    element_name,
                    attributes,
                )?)),
                unexpected => Err(XmlReadError::UnexpectedElementReadError {
                    error: format!("Got unexpected element {:?}", unexpected),
                    element: "LicenseChoice".to_string(),
                }),
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct License {
        #[versioned("1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        bom_ref: Option<String>,
        #[serde(flatten)]
        license_identifier: LicenseIdentifier,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<AttachedText>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[versioned("1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        licensing: Option<Licensing>,
        #[versioned("1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        properties: Option<Properties>,
    }

    impl From<models::license::License> for License {
        fn from(other: models::license::License) -> Self {
            Self {
                #[versioned("1.5")]
                bom_ref: other.bom_ref.map(|b| b.0),
                license_identifier: other.license_identifier.into(),
                text: convert_optional(other.text),
                url: other.url.map(|u| u.to_string()),
                #[versioned("1.5")]
                licensing: convert_optional(other.licensing),
                #[versioned("1.5")]
                properties: convert_optional(other.properties),
            }
        }
    }

    impl From<License> for models::license::License {
        fn from(other: License) -> Self {
            Self {
                #[versioned("1.3", "1.4")]
                bom_ref: None,
                #[versioned("1.5")]
                bom_ref: other.bom_ref.map(models::bom::BomReference::new),
                license_identifier: other.license_identifier.into(),
                text: convert_optional(other.text),
                url: other.url.map(Uri),
                #[versioned("1.3", "1.4")]
                licensing: None,
                #[versioned("1.5")]
                licensing: convert_optional(other.licensing),
                #[versioned("1.3", "1.4")]
                properties: None,
                #[versioned("1.5")]
                properties: convert_optional(other.properties),
            }
        }
    }

    const LICENSE_TAG: &str = "license";
    const TEXT_TAG: &str = "text";
    const URL_TAG: &str = "url";
    #[versioned("1.5")]
    const LICENSING_TAG: &str = "licensing";
    #[versioned("1.5")]
    const PROPERTIES_TAG: &str = "properties";

    impl ToXml for License {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            #[versioned("1.3", "1.4")]
            let start_tag = xml::writer::XmlEvent::start_element(LICENSE_TAG);

            #[versioned("1.5")]
            let mut start_tag = xml::writer::XmlEvent::start_element(LICENSE_TAG);
            #[versioned("1.5")]
            if let Some(bom_ref) = &self.bom_ref {
                start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
            }

            writer
                .write(start_tag)
                .map_err(to_xml_write_error(LICENSE_TAG))?;

            self.license_identifier.write_xml_element(writer)?;

            if let Some(attached_text) = &self.text {
                attached_text.write_xml_named_element(writer, TEXT_TAG)?;
            }

            if let Some(url) = &self.url {
                write_simple_tag(writer, URL_TAG, url)?;
            }

            #[versioned("1.5")]
            if let Some(properties) = &self.properties {
                properties.write_xml_element(writer)?;
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
            #[allow(unused)] attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            #[versioned("1.5")]
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
            let mut license_identifier: Option<LicenseIdentifier> = None;
            let mut text: Option<AttachedText> = None;
            let mut url: Option<String> = None;
            #[versioned("1.5")]
            let mut licensing: Option<Licensing> = None;
            #[versioned("1.5")]
            let mut properties: Option<Properties> = None;

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
                    #[versioned("1.5")]
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == LICENSING_TAG => {
                        licensing = Some(Licensing::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }
                    #[versioned("1.5")]
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
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
                #[versioned("1.5")]
                bom_ref,
                license_identifier,
                text,
                url,
                #[versioned("1.5")]
                licensing,
                #[versioned("1.5")]
                properties,
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

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Expression {
        #[serde(skip_serializing_if = "Option::is_none")]
        bom_ref: Option<String>,
        expression: String,
    }

    impl Expression {
        #[allow(unused)]
        pub fn new(expression: &str) -> Self {
            Self {
                bom_ref: None,
                expression: expression.to_string(),
            }
        }
    }

    impl From<Expression> for SpdxExpression {
        fn from(other: Expression) -> Self {
            Self {
                bom_ref: other.bom_ref.map(BomReference::new),
                expression: other.expression,
            }
        }
    }

    impl From<SpdxExpression> for Expression {
        fn from(other: SpdxExpression) -> Self {
            Self {
                bom_ref: other.bom_ref.map(|b| b.0),
                expression: other.expression,
            }
        }
    }

    impl ToXml for Expression {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            let mut start_tag = xml::writer::XmlEvent::start_element(EXPRESSION_TAG);

            if let Some(bom_ref) = &self.bom_ref {
                start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
            }

            writer
                .write(start_tag)
                .map_err(to_xml_write_error(EXPRESSION_TAG))?;

            writer
                .write(writer::XmlEvent::characters(&self.expression))
                .map_err(to_xml_write_error(EXPRESSION_TAG))?;

            write_close_tag(writer, EXPRESSION_TAG)?;

            Ok(())
        }
    }

    impl FromXml for Expression {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &OwnedName,
            attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);
            let expression = read_simple_tag(event_reader, element_name)?;

            Ok(Expression {
                bom_ref,
                expression,
            })
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;
        use pretty_assertions::assert_eq;

        #[versioned("1.5")]
        use crate::specs::{
            common::property::test::{corresponding_properties, example_properties},
            v1_5::licensing::test::{corresponding_licensing, example_licensing},
        };

        use crate::{
            external_models::spdx::SpdxExpression,
            specs::common::attached_text::test::{
                corresponding_attached_text, example_attached_text,
            },
            xml::test::{read_element_from_string, write_element_to_string},
        };

        pub(crate) fn example_licenses() -> Licenses {
            Licenses(vec![example_license_expression()])
        }

        pub(crate) fn corresponding_licenses() -> models::license::Licenses {
            models::license::Licenses(vec![corresponding_license_expression()])
        }

        #[versioned("1.3", "1.4")]
        pub(crate) fn example_spdx_license() -> LicenseChoice {
            LicenseChoice::License(License {
                license_identifier: LicenseIdentifier::SpdxId("spdx id".to_string()),
                text: Some(example_attached_text()),
                url: Some("url".to_string()),
            })
        }

        #[versioned("1.5")]
        pub(crate) fn example_spdx_license() -> LicenseChoice {
            LicenseChoice::License(License {
                bom_ref: Some("license-id".to_string()),
                license_identifier: LicenseIdentifier::SpdxId("spdx id".to_string()),
                text: Some(example_attached_text()),
                url: Some("url".to_string()),
                licensing: Some(example_licensing()),
                properties: Some(example_properties()),
            })
        }

        #[allow(unused)]
        #[versioned("1.3", "1.4")]
        pub(crate) fn corresponding_spdx_license() -> models::license::LicenseChoice {
            models::license::LicenseChoice::License(models::license::License {
                bom_ref: None,
                license_identifier: models::license::LicenseIdentifier::SpdxId(SpdxIdentifier(
                    "spdx id".to_string(),
                )),
                text: Some(corresponding_attached_text()),
                url: Some(Uri("url".to_string())),
                licensing: None,
                properties: None,
            })
        }

        #[allow(unused)]
        #[versioned("1.5")]
        pub(crate) fn corresponding_spdx_license() -> models::license::LicenseChoice {
            models::license::LicenseChoice::License(models::license::License {
                bom_ref: Some(models::bom::BomReference::new("license-id")),
                license_identifier: models::license::LicenseIdentifier::SpdxId(SpdxIdentifier(
                    "spdx id".to_string(),
                )),
                text: Some(corresponding_attached_text()),
                url: Some(Uri("url".to_string())),
                licensing: Some(corresponding_licensing()),
                properties: Some(corresponding_properties()),
            })
        }

        #[versioned("1.3", "1.4")]
        pub(crate) fn example_named_license() -> LicenseChoice {
            LicenseChoice::License(License {
                license_identifier: LicenseIdentifier::Name("name".to_string()),
                text: Some(example_attached_text()),
                url: Some("url".to_string()),
            })
        }

        #[versioned("1.5")]
        pub(crate) fn example_named_license() -> LicenseChoice {
            LicenseChoice::License(License {
                bom_ref: Some("license-1".to_string()),
                license_identifier: LicenseIdentifier::Name("name".to_string()),
                text: Some(example_attached_text()),
                url: Some("url".to_string()),
                licensing: Some(example_licensing()),
                properties: Some(example_properties()),
            })
        }

        #[versioned("1.3", "1.4")]
        #[allow(unused)]
        pub(crate) fn corresponding_named_license() -> models::license::LicenseChoice {
            models::license::LicenseChoice::License(models::license::License {
                bom_ref: None,
                license_identifier: models::license::LicenseIdentifier::Name(
                    NormalizedString::new_unchecked("name".to_string()),
                ),
                text: Some(corresponding_attached_text()),
                url: Some(Uri("url".to_string())),
                licensing: None,
                properties: None,
            })
        }

        #[allow(unused)]
        #[versioned("1.5")]
        pub(crate) fn corresponding_named_license() -> models::license::LicenseChoice {
            models::license::LicenseChoice::License(models::license::License {
                bom_ref: Some(models::bom::BomReference::new("license-1".to_string())),
                license_identifier: models::license::LicenseIdentifier::Name(
                    NormalizedString::new_unchecked("name".to_string()),
                ),
                text: Some(corresponding_attached_text()),
                url: Some(Uri("url".to_string())),
                licensing: Some(corresponding_licensing()),
                properties: Some(corresponding_properties()),
            })
        }

        pub(crate) fn example_license_expression() -> LicenseChoice {
            LicenseChoice::Expression(Expression::new("expression"))
        }

        pub(crate) fn corresponding_license_expression() -> models::license::LicenseChoice {
            models::license::LicenseChoice::Expression(SpdxExpression::new("expression"))
        }

        #[test]
        fn it_should_read_licenses_without_license_choices_correctly() {
            let input = r#"
    <licenses>
    </licenses>
    "#;
            let actual: Licenses = read_element_from_string(input);
            let expected = Licenses(vec![]);

            assert_eq!(actual, expected);
        }

        #[test]
        fn it_should_write_licenses_without_license_choices_correctly() {
            let xml_output = write_element_to_string(Licenses(vec![]));

            insta::assert_snapshot!(xml_output);
        }

        #[test]
        fn it_should_handle_licenses_correctly_license_choice_licenses() {
            let actual = Licenses(vec![example_spdx_license(), example_named_license()]);

            insta::assert_json_snapshot!(actual);
        }

        #[test]
        fn it_should_handle_licenses_correctly_license_choice_expressions() {
            let actual = Licenses(vec![
                example_license_expression(),
                example_license_expression(),
            ]);

            insta::assert_json_snapshot!(actual);
        }

        #[test]
        fn it_should_write_xml_full_license_choice_licenses() {
            let xml_output = write_element_to_string(Licenses(vec![
                example_spdx_license(),
                example_named_license(),
            ]));
            insta::assert_snapshot!(xml_output);
        }

        #[test]
        fn it_should_write_xml_full_license_choice_expressions() {
            let xml_output = write_element_to_string(Licenses(vec![
                example_license_expression(),
                example_license_expression(),
            ]));
            insta::assert_snapshot!(xml_output);
        }

        #[versioned("1.3", "1.4")]
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
            let expected = Licenses(vec![example_spdx_license(), example_named_license()]);
            assert_eq!(actual, expected);
        }

        #[versioned("1.5")]
        #[test]
        fn it_should_read_xml_full_license_choice_licenses() {
            let input = r#"
    <licenses>
      <license bom-ref="license-id">
        <id>spdx id</id>
        <text content-type="content type" encoding="encoding">content</text>
        <url>url</url>
        <licensing>
          <altIds>
            <altId>alt-id</altId>
          </altIds>
          <licensor>
            <individual bom-ref="licensor-1">
              <name>licensor name</name>
            </individual>
          </licensor>
          <licensee>
            <organization bom-ref="licensee-1">
              <name>licensee name</name>
            </organization>
          </licensee>
          <purchaser>
            <organization bom-ref="purchaser-1">
              <name>purchaser name</name>
            </organization>
          </purchaser>
          <purchaseOrder>Subscription</purchaseOrder>
          <licenseTypes>
            <licenseType>User</licenseType>
          </licenseTypes>
          <lastRenewal>2024-01-10T10:10:12</lastRenewal>
          <expiration>2024-05-10T10:10:12</expiration>
        </licensing>
        <properties>
          <property name="name">value</property>
        </properties>
      </license>
      <license bom-ref="license-1">
        <name>name</name>
        <text content-type="content type" encoding="encoding">content</text>
        <url>url</url>
        <licensing>
          <altIds>
            <altId>alt-id</altId>
          </altIds>
          <licensor>
            <individual bom-ref="licensor-1">
              <name>licensor name</name>
            </individual>
          </licensor>
          <licensee>
            <organization bom-ref="licensee-1">
              <name>licensee name</name>
            </organization>
          </licensee>
          <purchaser>
            <organization bom-ref="purchaser-1">
              <name>purchaser name</name>
            </organization>
          </purchaser>
          <purchaseOrder>Subscription</purchaseOrder>
          <licenseTypes>
            <licenseType>User</licenseType>
          </licenseTypes>
          <lastRenewal>2024-01-10T10:10:12</lastRenewal>
          <expiration>2024-05-10T10:10:12</expiration>
        </licensing>
        <properties>
          <property name="name">value</property>
        </properties>
      </license>
    </licenses>
    "#;
            let actual: Licenses = read_element_from_string(input);
            let expected = Licenses(vec![example_spdx_license(), example_named_license()]);
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
            let expected = Licenses(vec![
                example_license_expression(),
                example_license_expression(),
            ]);
            assert_eq!(actual, expected);
        }
    }
}
