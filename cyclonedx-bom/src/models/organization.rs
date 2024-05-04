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
    external_models::{
        normalized_string::{validate_normalized_string, NormalizedString},
        uri::{validate_uri, Uri},
    },
    validation::{Validate, ValidationContext, ValidationResult},
};

use super::bom::{validate_bom_ref, BomReference, SpecVersion};

/// Represents the contact information for an organization
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_organizationalContact)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct OrganizationalContact {
    pub bom_ref: Option<BomReference>,
    pub name: Option<NormalizedString>,
    pub email: Option<NormalizedString>,
    pub phone: Option<NormalizedString>,
}

impl OrganizationalContact {
    /// Construct an `OrganizationalContact` with name and email
    /// ```
    /// use cyclonedx_bom::models::organization::OrganizationalContact;
    ///
    /// let organizational_contact = OrganizationalContact::new("Example Support AMER Distribution", Some("support@example.com"));
    /// ```
    pub fn new(name: &str, email: Option<&str>) -> Self {
        Self {
            bom_ref: None,
            name: Some(NormalizedString::new(name)),
            email: email.map(NormalizedString::new),
            phone: None,
        }
    }
}

impl Validate for OrganizationalContact {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("bom-ref", self.bom_ref.as_ref(), |bom_ref| {
                validate_bom_ref(bom_ref, version)
            })
            .add_field_option("name", self.name.as_ref(), validate_normalized_string)
            .add_field_option("email", self.email.as_ref(), validate_normalized_string)
            .add_field_option("phone", self.phone.as_ref(), validate_normalized_string)
            .into()
    }
}

/// Represents an organization with name, url, and contact information
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_organizationalEntity)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrganizationalEntity {
    pub bom_ref: Option<BomReference>,
    pub name: Option<NormalizedString>,
    pub url: Option<Vec<Uri>>,
    pub contact: Option<Vec<OrganizationalContact>>,
}

impl OrganizationalEntity {
    pub fn new(name: &str) -> Self {
        Self {
            bom_ref: None,
            name: Some(NormalizedString::new_unchecked(name.to_string())),
            url: None,
            contact: None,
        }
    }
}

impl Validate for OrganizationalEntity {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option("name", self.name.as_ref(), validate_normalized_string)
            .add_list_option("url", self.url.as_ref(), validate_uri)
            .add_list_option("contact", self.contact.as_ref(), |contact| {
                contact.validate_version(version)
            })
            .into()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        models::{
            bom::BomReference,
            organization::{OrganizationalContact, OrganizationalEntity},
        },
        prelude::{NormalizedString, Uri, Validate},
        validation,
    };

    #[test]
    fn it_should_validate_an_empty_contact_as_passed() {
        let contact = OrganizationalContact {
            bom_ref: None,
            name: None,
            email: None,
            phone: None,
        };
        let actual = contact.validate();
        assert!(actual.passed());
    }

    #[test]
    fn it_should_validate_an_invalid_contact_as_failed() {
        let contact = OrganizationalContact {
            bom_ref: None,
            name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
            email: None,
            phone: None,
        };
        let actual = contact.validate();

        assert_eq!(
            actual,
            validation::field(
                "name",
                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
            )
        );
    }

    #[test]
    fn it_should_validate_bom_ref_correctly() {
        let contact = OrganizationalContact {
            bom_ref: Some(BomReference::new("contact-a")),
            name: Some(NormalizedString::new("Contact")),
            email: Some("test@example.com".into()),
            phone: Some("0123456789".into()),
        };

        assert!(contact
            .validate_version(crate::prelude::SpecVersion::V1_3)
            .has_errors());
        assert!(contact
            .validate_version(crate::prelude::SpecVersion::V1_4)
            .has_errors());
        assert!(contact
            .validate_version(crate::prelude::SpecVersion::V1_5)
            .passed());
    }

    #[test]
    fn it_should_validate_a_contact_with_multiple_validation_issues_as_failed() {
        let contact = OrganizationalContact {
            bom_ref: None,
            name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
            email: Some(NormalizedString::new_unchecked(
                "invalid\temail".to_string(),
            )),
            phone: Some(NormalizedString::new_unchecked(
                "invalid\tphone".to_string(),
            )),
        };
        let actual = contact.validate();

        assert_eq!(
            actual,
            vec![
                validation::field(
                    "name",
                    "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                ),
                validation::field(
                    "email",
                    "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                ),
                validation::field(
                    "phone",
                    "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                )
            ]
            .into()
        );
    }

    #[test]
    fn it_should_validate_an_invalid_entity_as_failed() {
        let entity = OrganizationalEntity::new("invalid\tname");
        let actual = entity.validate();

        assert_eq!(
            actual,
            validation::field(
                "name",
                "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
            )
        );
    }

    #[test]
    fn it_should_validate_an_entity_with_multiple_validation_issues_as_failed() {
        let entity = OrganizationalEntity {
            bom_ref: None,
            name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
            url: Some(vec![Uri("invalid uri".to_string())]),
            contact: Some(vec![OrganizationalContact {
                bom_ref: None,
                name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
                email: None,
                phone: None,
            }]),
        };
        let actual = entity.validate();

        assert_eq!(
            actual,
            vec![
                validation::field(
                    "name",
                    "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                ),
                validation::list(
                    "url",
                    [(
                        0,
                        validation::custom("", ["Uri does not conform to RFC 3986"])
                    )]
                ),
                validation::list(
                    "contact",
                    [(
                        0,
                        validation::field(
                            "name",
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        )
                    )]
                )
            ]
            .into()
        );
    }
}
