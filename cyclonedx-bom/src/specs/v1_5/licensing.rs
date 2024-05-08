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

use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    errors::XmlReadError,
    external_models::date_time::DateTime,
    models,
    prelude::NormalizedString,
    specs::common::organization::{OrganizationalContact, OrganizationalEntity},
    utilities::convert_optional,
    xml::{
        read_list_tag, read_simple_tag, to_xml_read_error, unexpected_element_error,
        write_close_tag, write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

/// Represents Licensing Information.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Licensing {
    alt_ids: Option<Vec<String>>,
    licensor: Option<LicenseContact>,
    licensee: Option<LicenseContact>,
    purchaser: Option<LicenseContact>,
    purchase_order: Option<String>,
    license_types: Option<Vec<String>>,
    last_renewal: Option<String>,
    expiration: Option<String>,
}

impl From<Licensing> for models::license::Licensing {
    fn from(other: Licensing) -> Self {
        Self {
            alt_ids: other.alt_ids.map(|ids| {
                ids.into_iter()
                    .map(NormalizedString::new_unchecked)
                    .collect()
            }),
            licensor: other.licensor.map(From::from),
            licensee: other.licensee.map(From::from),
            purchaser: other.purchaser.map(From::from),
            purchase_order: convert_optional(other.purchase_order),
            license_types: other.license_types.map(|licenses| {
                licenses
                    .iter()
                    .map(|l| models::license::LicenseType::new_unchecked(l))
                    .collect()
            }),
            last_renewal: other.last_renewal.map(DateTime),
            expiration: other.expiration.map(DateTime),
        }
    }
}

impl From<models::license::Licensing> for Licensing {
    fn from(other: models::license::Licensing) -> Self {
        Self {
            alt_ids: other
                .alt_ids
                .map(|ids| ids.into_iter().map(|id| id.0).collect()),
            licensor: other.licensor.map(From::from),
            licensee: other.licensee.map(From::from),
            purchaser: other.purchaser.map(From::from),
            purchase_order: convert_optional(other.purchase_order),
            license_types: other
                .license_types
                .map(|types| types.into_iter().map(|t| t.to_string()).collect()),
            last_renewal: other.last_renewal.map(|l| l.0),
            expiration: other.expiration.map(|l| l.0),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum LicenseContact {
    Organization(OrganizationalEntity),
    Individual(OrganizationalContact),
}

impl From<LicenseContact> for models::license::LicenseContact {
    fn from(other: LicenseContact) -> Self {
        match other {
            LicenseContact::Organization(org) => Self::Organization(org.into()),
            LicenseContact::Individual(contact) => Self::Contact(contact.into()),
        }
    }
}

impl From<models::license::LicenseContact> for LicenseContact {
    fn from(other: models::license::LicenseContact) -> Self {
        match other {
            models::license::LicenseContact::Organization(org) => Self::Organization(org.into()),
            models::license::LicenseContact::Contact(contact) => Self::Individual(contact.into()),
        }
    }
}

const ORGANIZATION_TAG: &str = "organization";
const INDIVIDUAL_TAG: &str = "individual";

impl ToInnerXml for LicenseContact {
    fn write_xml_named_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, tag)?;
        self.write_xml_element(writer)?;
        write_close_tag(writer, tag)?;

        Ok(())
    }
}

impl ToXml for LicenseContact {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        match self {
            LicenseContact::Organization(org) => {
                org.write_xml_named_element(writer, ORGANIZATION_TAG)
            }
            LicenseContact::Individual(contact) => {
                contact.write_xml_named_element(writer, INDIVIDUAL_TAG)
            }
        }
    }
}

impl FromXml for LicenseContact {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut contact: Option<LicenseContact> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ORGANIZATION_TAG => {
                    contact = Some(Self::Organization(OrganizationalEntity::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?));
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == INDIVIDUAL_TAG => {
                    contact = Some(Self::Individual(OrganizationalContact::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?));
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let contact = contact.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: "contact".to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(contact)
    }
}

const LICENSING_TAG: &str = "licensing";
const ALT_IDS_TAG: &str = "altIds";
const ALT_ID_TAG: &str = "altId";
const LICENSOR_TAG: &str = "licensor";
const LICENSEE_TAG: &str = "licensee";
const PURCHASER_TAG: &str = "purchaser";
const PURCHASE_ORDER_TAG: &str = "purchaseOrder";
const LICENSE_TYPES_TAG: &str = "licenseTypes";
const LICENSE_TYPE_TAG: &str = "licenseType";
const LAST_RENEWAL_TAG: &str = "lastRenewal";
const EXPIRATION_TAG: &str = "expiration";

impl ToXml for Licensing {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, LICENSING_TAG)?;

        if let Some(alt_ids) = &self.alt_ids {
            write_start_tag(writer, ALT_IDS_TAG)?;

            for alt_id in alt_ids {
                write_simple_tag(writer, ALT_ID_TAG, alt_id)?;
            }

            write_close_tag(writer, ALT_IDS_TAG)?;
        }

        if let Some(licensor) = &self.licensor {
            licensor.write_xml_named_element(writer, LICENSOR_TAG)?;
        }

        if let Some(licensee) = &self.licensee {
            licensee.write_xml_named_element(writer, LICENSEE_TAG)?;
        }

        if let Some(purchaser) = &self.purchaser {
            purchaser.write_xml_named_element(writer, PURCHASER_TAG)?;
        }

        if let Some(purchase_order) = &self.purchase_order {
            write_simple_tag(writer, PURCHASE_ORDER_TAG, purchase_order)?;
        }

        if let Some(license_types) = &self.license_types {
            write_start_tag(writer, LICENSE_TYPES_TAG)?;

            for license_type in license_types {
                write_simple_tag(writer, LICENSE_TYPE_TAG, license_type)?;
            }

            write_close_tag(writer, LICENSE_TYPES_TAG)?;
        }

        if let Some(last_renewal) = &self.last_renewal {
            write_simple_tag(writer, LAST_RENEWAL_TAG, last_renewal)?;
        }

        if let Some(expiration) = &self.expiration {
            write_simple_tag(writer, EXPIRATION_TAG, expiration)?;
        }

        write_close_tag(writer, LICENSING_TAG)?;

        Ok(())
    }
}

impl FromXml for Licensing {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut alt_ids: Option<Vec<String>> = None;
        let mut licensor: Option<LicenseContact> = None;
        let mut licensee: Option<LicenseContact> = None;
        let mut purchaser: Option<LicenseContact> = None;
        let mut purchase_order: Option<String> = None;
        let mut license_types: Option<Vec<String>> = None;
        let mut last_renewal: Option<String> = None;
        let mut expiration: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == ALT_IDS_TAG => {
                    alt_ids = Some(read_list_tag(event_reader, &name, ALT_ID_TAG)?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == LICENSOR_TAG => {
                    licensor = Some(LicenseContact::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == LICENSEE_TAG => {
                    licensee = Some(LicenseContact::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PURCHASER_TAG => {
                    purchaser = Some(LicenseContact::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == PURCHASE_ORDER_TAG =>
                {
                    purchase_order = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == LICENSE_TYPES_TAG =>
                {
                    license_types = Some(read_list_tag(event_reader, &name, LICENSE_TYPE_TAG)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == LAST_RENEWAL_TAG =>
                {
                    last_renewal = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == EXPIRATION_TAG =>
                {
                    expiration = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            alt_ids,
            licensor,
            licensee,
            purchaser,
            purchase_order,
            license_types,
            last_renewal,
            expiration,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        prelude::{DateTime, NormalizedString},
        xml::test::{
            read_element_from_string, write_element_to_string, write_named_element_to_string,
        },
    };

    use super::*;
    use pretty_assertions::assert_eq;

    pub(crate) fn example_licensing() -> Licensing {
        Licensing {
            alt_ids: Some(vec!["alt-id".to_string()]),
            licensor: Some(example_licensor()),
            licensee: Some(example_licensee()),
            purchaser: Some(example_purchaser()),
            purchase_order: Some("Subscription".to_string()),
            license_types: Some(vec!["User".to_string()]),
            last_renewal: Some("2024-01-10T10:10:12".to_string()),
            expiration: Some("2024-05-10T10:10:12".to_string()),
        }
    }

    pub(crate) fn corresponding_licensing() -> models::license::Licensing {
        models::license::Licensing {
            alt_ids: Some(vec![NormalizedString::new_unchecked("alt-id".to_string())]),
            licensor: Some(corresponding_licensor()),
            licensee: Some(corresponding_licensee()),
            purchaser: Some(corresponding_purchaser()),
            purchase_order: Some("Subscription".to_string()),
            license_types: Some(vec![models::license::LicenseType::User]),
            last_renewal: Some(DateTime("2024-01-10T10:10:12".to_string())),
            expiration: Some(DateTime("2024-05-10T10:10:12".to_string())),
        }
    }

    fn example_licensor() -> LicenseContact {
        LicenseContact::Individual(OrganizationalContact {
            bom_ref: Some("licensor-1".to_string()),
            name: Some("licensor name".to_string()),
            email: None,
            phone: None,
        })
    }

    fn corresponding_licensor() -> models::license::LicenseContact {
        models::license::LicenseContact::Contact(models::organization::OrganizationalContact {
            bom_ref: Some(models::bom::BomReference::new("licensor-1")),
            name: Some(NormalizedString::new_unchecked("licensor name".to_string())),
            email: None,
            phone: None,
        })
    }

    fn example_licensee() -> LicenseContact {
        LicenseContact::Organization(OrganizationalEntity {
            bom_ref: Some("licensee-1".to_string()),
            name: Some("licensee name".to_string()),
            url: None,
            contact: None,
        })
    }

    fn corresponding_licensee() -> models::license::LicenseContact {
        models::license::LicenseContact::Contact(models::organization::OrganizationalContact {
            bom_ref: Some(models::bom::BomReference::new("purchaser-1")),
            name: Some(NormalizedString::new_unchecked(
                "purchaser name".to_string(),
            )),
            email: None,
            phone: None,
        })
    }

    fn example_purchaser() -> LicenseContact {
        LicenseContact::Organization(OrganizationalEntity {
            bom_ref: Some("purchaser-1".to_string()),
            name: Some("purchaser name".to_string()),
            url: None,
            contact: None,
        })
    }

    fn corresponding_purchaser() -> models::license::LicenseContact {
        models::license::LicenseContact::Contact(models::organization::OrganizationalContact {
            bom_ref: Some(models::bom::BomReference::new("licensee-1")),
            name: Some(NormalizedString::new_unchecked("licensee name".to_string())),
            email: None,
            phone: None,
        })
    }

    #[test]
    fn it_should_read_xml_license_contact() {
        let input = r#"
<licensor>
  <organization>
    <name>Acme Inc</name>
    <contact>
      <name>Acme Licensing Fulfillment</name>
      <email>licensing@example.com</email>
    </contact>
  </organization>
</licensor>
        "#;
        let actual: LicenseContact = read_element_from_string(input);
        let expected = LicenseContact::Organization(OrganizationalEntity {
            bom_ref: None,
            name: Some("Acme Inc".to_string()),
            url: None,
            contact: Some(vec![OrganizationalContact {
                bom_ref: None,
                name: Some("Acme Licensing Fulfillment".to_string()),
                email: Some("licensing@example.com".to_string()),
                phone: None,
            }]),
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_write_xml_named_license_contact() {
        let expected = LicenseContact::Organization(OrganizationalEntity {
            bom_ref: None,
            name: Some("Acme Inc".to_string()),
            url: None,
            contact: Some(vec![OrganizationalContact {
                bom_ref: None,
                name: Some("Acme Licensing Fulfillment".to_string()),
                email: Some("licensing@example.com".to_string()),
                phone: None,
            }]),
        });
        let xml_output = write_named_element_to_string(expected, "licensor");
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_write_xml_licensing() {
        let xml_output = write_element_to_string(example_licensing());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_licensing() {
        let input = r#"
<licensing>
  <altIds>
    <altId>acme</altId>
    <altId>acme-license</altId>
  </altIds>
  <licensor>
    <organization>
      <name>Acme Inc</name>
      <contact>
        <name>Acme Licensing Fulfillment</name>
        <email>licensing@example.com</email>
      </contact>
    </organization>
  </licensor>
  <licensee>
    <organization>
      <name>Example Co.</name>
    </organization>
  </licensee>
  <purchaser>
    <individual>
      <name>Samantha Wright</name>
      <email>samantha.wright@gmail.com</email>
      <phone>800-555-1212</phone>
    </individual>
  </purchaser>
  <purchaseOrder>PO-12345</purchaseOrder>
  <licenseTypes>
    <licenseType>appliance</licenseType>
  </licenseTypes>
  <lastRenewal>2022-04-13T20:20:39+00:00</lastRenewal>
  <expiration>2023-04-13T20:20:39+00:00</expiration>
</licensing>
        "#;
        let actual: Licensing = read_element_from_string(input);
        let expected = Licensing {
            alt_ids: Some(vec!["acme".to_string(), "acme-license".to_string()]),
            licensor: Some(LicenseContact::Organization(OrganizationalEntity {
                bom_ref: None,
                name: Some("Acme Inc".to_string()),
                url: None,
                contact: Some(vec![OrganizationalContact {
                    bom_ref: None,
                    name: Some("Acme Licensing Fulfillment".to_string()),
                    email: Some("licensing@example.com".to_string()),
                    phone: None,
                }]),
            })),
            licensee: Some(LicenseContact::Organization(OrganizationalEntity {
                bom_ref: None,
                name: Some("Example Co.".to_string()),
                url: None,
                contact: None,
            })),
            purchaser: Some(LicenseContact::Individual(OrganizationalContact {
                bom_ref: None,
                name: Some("Samantha Wright".to_string()),
                email: Some("samantha.wright@gmail.com".to_string()),
                phone: Some("800-555-1212".to_string()),
            })),
            purchase_order: Some("PO-12345".to_string()),
            license_types: Some(vec!["appliance".to_string()]),
            last_renewal: Some("2022-04-13T20:20:39+00:00".to_string()),
            expiration: Some("2023-04-13T20:20:39+00:00".to_string()),
        };
        assert_eq!(actual, expected);
    }
}
