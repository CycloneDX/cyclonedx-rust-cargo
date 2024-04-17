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
    errors::XmlWriteError,
    external_models::{normalized_string::NormalizedString, uri::Uri},
    models::{self, bom::BomReference},
    utilities::convert_optional_vec,
    xml::{
        optional_attribute, read_lax_validation_tag, read_simple_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_close_tag, write_simple_tag, FromXml,
        ToInnerXml,
    },
};
use serde::{Deserialize, Serialize};
use xml::{reader, writer::XmlEvent};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrganizationalContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bom_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) phone: Option<String>,
}

impl From<models::organization::OrganizationalContact> for OrganizationalContact {
    fn from(other: models::organization::OrganizationalContact) -> Self {
        Self {
            bom_ref: other.bom_ref.map(|r| r.0),
            name: other.name.map(|n| n.to_string()),
            email: other.email.map(|e| e.to_string()),
            phone: other.phone.map(|p| p.to_string()),
        }
    }
}

impl From<OrganizationalContact> for models::organization::OrganizationalContact {
    fn from(other: OrganizationalContact) -> Self {
        Self {
            bom_ref: other.bom_ref.map(BomReference::new),
            name: other.name.map(NormalizedString::new_unchecked),
            email: other.email.map(NormalizedString::new_unchecked),
            phone: other.phone.map(NormalizedString::new_unchecked),
        }
    }
}

const BOM_REF_ATTR: &str = "bom-ref";
const NAME_TAG: &str = "name";
const EMAIL_TAG: &str = "email";
const PHONE_TAG: &str = "phone";

impl ToInnerXml for OrganizationalContact {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), XmlWriteError> {
        let mut start_tag = XmlEvent::start_element(tag);

        if let Some(bom_ref) = &self.bom_ref {
            start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
        }

        writer.write(start_tag).map_err(to_xml_write_error(tag))?;

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(email) = &self.email {
            write_simple_tag(writer, EMAIL_TAG, email)?;
        }

        if let Some(phone) = &self.phone {
            write_simple_tag(writer, PHONE_TAG, phone)?;
        }

        write_close_tag(writer, tag)?;

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.name.is_some() || self.email.is_some() || self.phone.is_some()
    }
}

impl FromXml for OrganizationalContact {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);

        let mut contact_name: Option<String> = None;
        let mut email: Option<String> = None;
        let mut phone: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    contact_name = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == EMAIL_TAG => {
                    email = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == PHONE_TAG => {
                    phone = Some(read_simple_tag(event_reader, &name)?)
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

        Ok(Self {
            bom_ref,
            name: contact_name,
            email,
            phone,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrganizationalEntity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bom_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) url: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) contact: Option<Vec<OrganizationalContact>>,
}

impl OrganizationalEntity {
    #[allow(unused)]
    pub fn new(name: &str) -> Self {
        Self {
            bom_ref: None,
            name: Some(name.to_string()),
            url: None,
            contact: None,
        }
    }
}

impl From<models::organization::OrganizationalEntity> for OrganizationalEntity {
    fn from(other: models::organization::OrganizationalEntity) -> Self {
        Self {
            bom_ref: other.bom_ref.map(|r| r.0),
            name: other.name.map(|n| n.to_string()),
            url: other
                .url
                .map(|urls| urls.into_iter().map(|url| url.0).collect()),
            contact: convert_optional_vec(other.contact),
        }
    }
}

impl From<OrganizationalEntity> for models::organization::OrganizationalEntity {
    fn from(other: OrganizationalEntity) -> Self {
        Self {
            bom_ref: other.bom_ref.map(BomReference::new),
            name: other.name.map(NormalizedString::new_unchecked),
            url: other.url.map(|urls| urls.into_iter().map(Uri).collect()),
            contact: convert_optional_vec(other.contact),
        }
    }
}

const URL_TAG: &str = "url";
const CONTACT_TAG: &str = "contact";

impl ToInnerXml for OrganizationalEntity {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), XmlWriteError> {
        let mut start_tag = XmlEvent::start_element(tag);

        if let Some(bom_ref) = &self.bom_ref {
            start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
        }

        writer.write(start_tag).map_err(to_xml_write_error(tag))?;

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(urls) = &self.url {
            for url in urls {
                write_simple_tag(writer, URL_TAG, url)?;
            }
        }

        if let Some(contacts) = &self.contact {
            for contact in contacts {
                if contact.will_write() {
                    contact.write_xml_named_element(writer, CONTACT_TAG)?;
                }
            }
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(tag))?;

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.name.is_some() || self.url.is_some() || self.contact.is_some()
    }
}

impl FromXml for OrganizationalEntity {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);

        let mut contact_name: Option<String> = None;
        let mut url: Option<Vec<String>> = None;
        let mut contact: Option<Vec<OrganizationalContact>> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    contact_name = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == URL_TAG => {
                    url.get_or_insert(Vec::new())
                        .push(read_simple_tag(event_reader, &name)?);
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == CONTACT_TAG => {
                    contact
                        .get_or_insert(Vec::new())
                        .push(OrganizationalContact::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
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

        Ok(Self {
            bom_ref,
            name: contact_name,
            url,
            contact,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::xml::test::{read_element_from_string, write_named_element_to_string};

    use super::*;
    use pretty_assertions::assert_eq;

    pub(crate) fn example_contact() -> OrganizationalContact {
        OrganizationalContact {
            bom_ref: None,
            name: Some("name".to_string()),
            email: Some("email".to_string()),
            phone: Some("phone".to_string()),
        }
    }

    pub(crate) fn corresponding_contact() -> models::organization::OrganizationalContact {
        models::organization::OrganizationalContact {
            bom_ref: None,
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            email: Some(NormalizedString::new_unchecked("email".to_string())),
            phone: Some(NormalizedString::new_unchecked("phone".to_string())),
        }
    }

    pub(crate) fn example_entity() -> OrganizationalEntity {
        OrganizationalEntity {
            bom_ref: None,
            name: Some("name".to_string()),
            url: Some(vec!["url".to_string()]),
            contact: Some(vec![example_contact()]),
        }
    }

    pub(crate) fn corresponding_entity() -> models::organization::OrganizationalEntity {
        models::organization::OrganizationalEntity {
            bom_ref: None,
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            url: Some(vec![Uri("url".to_string())]),
            contact: Some(vec![corresponding_contact()]),
        }
    }

    #[test]
    fn it_should_read_xml_organizational_contact() {
        let input = r#"
<contact bom-ref="contact-1">
  <name>name</name>
  <email>email</email>
  <phone>phone</phone>
</contact>
"#;
        let actual: OrganizationalContact = read_element_from_string(input);
        let expected = OrganizationalContact {
            bom_ref: Some("contact-1".to_string()),
            name: Some("name".to_string()),
            email: Some("email".to_string()),
            phone: Some("phone".to_string()),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_named_element_to_string(example_entity(), "supplier");
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_not_write_xml_empty_contacts() {
        let xml_output = write_named_element_to_string(
            OrganizationalEntity {
                bom_ref: Some("Supplier".to_string()),
                name: Some("name".to_string()),
                url: Some(vec!["url".to_string()]),
                contact: Some(vec![OrganizationalContact {
                    bom_ref: None,
                    name: None,
                    email: None,
                    phone: None,
                }]),
            },
            "supplier",
        );
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_write_xml_multiple_urls_contacts() {
        let xml_output = write_named_element_to_string(
            OrganizationalEntity {
                bom_ref: Some("Supplier".to_string()),
                name: Some("name".to_string()),
                url: Some(vec!["url".to_string(), "url".to_string()]),
                contact: Some(vec![example_contact(), example_contact()]),
            },
            "supplier",
        );
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<supplier>
  <name>name</name>
  <url>url</url>
  <contact>
    <name>name</name>
    <email>email</email>
    <phone>phone</phone>
  </contact>
</supplier>
"#;
        let actual: OrganizationalEntity = read_element_from_string(input);
        let expected = example_entity();
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_xml_multiple_urls_contacts() {
        let input = r#"
<supplier bom-ref="Supplier">
  <name>name</name>
  <url>url</url>
  <url>url</url>
  <contact>
    <name>name</name>
    <email>email</email>
    <phone>phone</phone>
  </contact>
  <contact>
    <name>name</name>
    <email>email</email>
    <phone>phone</phone>
  </contact>
</supplier>
"#;
        let actual: OrganizationalEntity = read_element_from_string(input);
        let expected = OrganizationalEntity {
            bom_ref: Some("Supplier".to_string()),
            name: Some("name".to_string()),
            url: Some(vec!["url".to_string(), "url".to_string()]),
            contact: Some(vec![example_contact(), example_contact()]),
        };
        assert_eq!(actual, expected);
    }
}
