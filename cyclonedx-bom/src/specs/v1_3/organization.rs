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
    models,
    utilities::convert_optional_vec,
    xml::{to_xml_write_error, write_simple_tag, ToInnerXml},
};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrganizationalContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
}

impl From<models::OrganizationalContact> for OrganizationalContact {
    fn from(other: models::OrganizationalContact) -> Self {
        Self {
            name: other.name.map(|n| n.to_string()),
            email: other.email.map(|e| e.to_string()),
            phone: other.phone.map(|p| p.to_string()),
        }
    }
}

impl From<OrganizationalContact> for models::OrganizationalContact {
    fn from(other: OrganizationalContact) -> Self {
        Self {
            name: other.name.map(NormalizedString::new_unchecked),
            email: other.email.map(NormalizedString::new_unchecked),
            phone: other.phone.map(NormalizedString::new_unchecked),
        }
    }
}

const NAME_TAG: &str = "name";
const EMAIL_TAG: &str = "email";
const PHONE_TAG: &str = "phone";

impl ToInnerXml for OrganizationalContact {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), XmlWriteError> {
        writer
            .write(XmlEvent::start_element(tag))
            .map_err(to_xml_write_error(tag))?;

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(email) = &self.email {
            write_simple_tag(writer, EMAIL_TAG, email)?;
        }

        if let Some(phone) = &self.phone {
            write_simple_tag(writer, PHONE_TAG, phone)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(tag))?;

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.name.is_some() || self.email.is_some() || self.phone.is_some()
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrganizationalEntity {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact: Option<Vec<OrganizationalContact>>,
}

impl From<models::OrganizationalEntity> for OrganizationalEntity {
    fn from(other: models::OrganizationalEntity) -> Self {
        Self {
            name: other.name.map(|n| n.to_string()),
            url: other
                .url
                .map(|urls| urls.into_iter().map(|url| url.0).collect()),
            contact: convert_optional_vec(other.contact),
        }
    }
}

impl From<OrganizationalEntity> for models::OrganizationalEntity {
    fn from(other: OrganizationalEntity) -> Self {
        Self {
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
        writer
            .write(XmlEvent::start_element(tag))
            .map_err(to_xml_write_error(tag))?;

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

#[cfg(test)]
pub(crate) mod test {
    use crate::xml::test::write_named_element_to_string;

    use super::*;

    pub(crate) fn example_contact() -> OrganizationalContact {
        OrganizationalContact {
            name: Some("name".to_string()),
            email: Some("email".to_string()),
            phone: Some("phone".to_string()),
        }
    }

    pub(crate) fn corresponding_contact() -> models::OrganizationalContact {
        models::OrganizationalContact {
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            email: Some(NormalizedString::new_unchecked("email".to_string())),
            phone: Some(NormalizedString::new_unchecked("phone".to_string())),
        }
    }

    pub(crate) fn example_entity() -> OrganizationalEntity {
        OrganizationalEntity {
            name: Some("name".to_string()),
            url: Some(vec!["url".to_string()]),
            contact: Some(vec![example_contact()]),
        }
    }

    pub(crate) fn corresponding_entity() -> models::OrganizationalEntity {
        models::OrganizationalEntity {
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            url: Some(vec![Uri("url".to_string())]),
            contact: Some(vec![corresponding_contact()]),
        }
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
                name: Some("name".to_string()),
                url: Some(vec!["url".to_string()]),
                contact: Some(vec![OrganizationalContact {
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
                name: Some("name".to_string()),
                url: Some(vec!["url".to_string(), "url".to_string()]),
                contact: Some(vec![example_contact(), example_contact()]),
            },
            "supplier",
        );
        insta::assert_snapshot!(xml_output);
    }
}
