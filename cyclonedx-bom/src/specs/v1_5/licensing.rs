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
    external_models::date_time::DateTime,
    models,
    specs::common::organization::{OrganizationalContact, OrganizationalEntity},
    utilities::{convert_optional, convert_vec},
    xml::{to_xml_read_error, unexpected_element_error, FromXml},
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
}

impl From<Licensing> for models::license::Licensing {
    fn from(other: Licensing) -> Self {
        Self {
            alt_ids: other.alt_ids.map(convert_vec),
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
        }
    }
}

impl From<models::license::Licensing> for Licensing {
    fn from(other: models::license::Licensing) -> Self {
        Self {
            alt_ids: other.alt_ids.map(convert_vec),
            licensor: other.licensor.map(From::from),
            licensee: other.licensee.map(From::from),
            purchaser: other.purchaser.map(From::from),
            purchase_order: convert_optional(other.purchase_order),
            license_types: other
                .license_types
                .map(|types| types.into_iter().map(|t| t.to_string()).collect()),
            last_renewal: other.last_renewal.map(|l| l.0),
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

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?;
            match next_element {
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
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::prelude::{DateTime, NormalizedString};

    use super::*;

    pub(crate) fn example_licensing() -> Licensing {
        Licensing {
            alt_ids: Some(vec!["alt-id".to_string()]),
            licensor: Some(example_licensor()),
            licensee: Some(example_licensee()),
            purchaser: Some(example_purchaser()),
            purchase_order: Some("Subscription".to_string()),
            license_types: Some(vec!["User".to_string()]),
            last_renewal: Some("2024-01-10T10:10:12".to_string()),
        }
    }

    pub(crate) fn corresponding_licensing() -> models::license::Licensing {
        models::license::Licensing {
            alt_ids: Some(vec!["alt-id".to_string()]),
            licensor: Some(corresponding_licensor()),
            licensee: Some(corresponding_licensee()),
            purchaser: Some(corresponding_purchaser()),
            purchase_order: Some("Subscription".to_string()),
            license_types: Some(vec![models::license::LicenseType::User]),
            last_renewal: Some(DateTime("2024-01-10T10:10:12".to_string())),
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
}
