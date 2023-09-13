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
    external_models::{normalized_string::NormalizedString, uri::Uri},
    validation::{
        Validate, ValidationContext, ValidationError, ValidationPathComponent, ValidationResult,
    },
};

/// Represents the contact information for an organization
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_organizationalContact)
#[derive(Debug, Default, PartialEq, Eq)]
pub struct OrganizationalContact {
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
            name: Some(NormalizedString::new(name)),
            email: email.map(NormalizedString::new),
            phone: None,
        }
    }
}

impl Validate for OrganizationalContact {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut name_result = ValidationResult::default();
        if let Some(name) = &self.name {
            let name_context =
                context.extend_context_with_struct_field("OrganizationalContact", "name");

            name_result = name.validate_with_context(name_context)?;
        }

        let mut email_result = ValidationResult::default();
        if let Some(email) = &self.email {
            let email_context =
                context.extend_context_with_struct_field("OrganizationalContact", "email");

            email_result = email.validate_with_context(email_context)?;
        }

        let mut phone_result = ValidationResult::default();
        if let Some(phone) = &self.phone {
            let phone_context =
                context.extend_context_with_struct_field("OrganizationalContact", "phone");

            phone_result = phone.validate_with_context(phone_context)?;
        }

        Ok(name_result.merge(email_result).merge(phone_result))
    }
}

/// Represents an organization with name, url, and contact information
///
/// Defined via the [CycloneDX XML schema](https://cyclonedx.org/docs/1.3/xml/#type_organizationalEntity)
#[derive(Debug, PartialEq, Eq)]
pub struct OrganizationalEntity {
    pub name: Option<NormalizedString>,
    pub url: Option<Vec<Uri>>,
    pub contact: Option<Vec<OrganizationalContact>>,
}

impl Validate for OrganizationalEntity {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut results: Vec<ValidationResult> = vec![];

        if let Some(name) = &self.name {
            let name_context =
                context.extend_context_with_struct_field("OrganizationalEntity", "name");

            results.push(name.validate_with_context(name_context)?);
        }

        if let Some(urls) = &self.url {
            for (index, url) in urls.iter().enumerate() {
                let uri_context = context.extend_context(vec![
                    ValidationPathComponent::Struct {
                        struct_name: "OrganizationalEntity".to_string(),
                        field_name: "url".to_string(),
                    },
                    ValidationPathComponent::Array { index },
                ]);

                results.push(url.validate_with_context(uri_context)?);
            }
        }

        if let Some(contacts) = &self.contact {
            for (index, contact) in contacts.iter().enumerate() {
                let uri_context = context.extend_context(vec![
                    ValidationPathComponent::Struct {
                        struct_name: "OrganizationalEntity".to_string(),
                        field_name: "contact".to_string(),
                    },
                    ValidationPathComponent::Array { index },
                ]);
                results.push(contact.validate_with_context(uri_context)?);
            }
        }

        Ok(results
            .into_iter()
            .fold(ValidationResult::default(), |acc, result| acc.merge(result)))
    }
}

#[cfg(test)]
mod test {
    use crate::validation::{FailureReason, ValidationPathComponent};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_should_validate_an_empty_contact_as_passed() {
        let contact = OrganizationalContact {
            name: None,
            email: None,
            phone: None,
        };
        let actual = contact
            .validate_with_context(ValidationContext::default())
            .expect("Failed to validate contact");
        assert_eq!(actual, ValidationResult::Passed);
    }

    #[test]
    fn it_should_validate_an_invalid_contact_as_failed() {
        let contact = OrganizationalContact {
            name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
            email: None,
            phone: None,
        };
        let actual = contact
            .validate_with_context(ValidationContext::default())
            .expect("Failed to validate contact");
        assert_eq!(
            actual,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        .to_string(),
                    context: ValidationContext(vec![ValidationPathComponent::Struct {
                        struct_name: "OrganizationalContact".to_string(),
                        field_name: "name".to_string()
                    }])
                }]
            }
        )
    }

    #[test]
    fn it_should_validate_a_contact_with_multiple_validation_issues_as_failed() {
        let contact = OrganizationalContact {
            name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
            email: Some(NormalizedString::new_unchecked(
                "invalid\temail".to_string(),
            )),
            phone: Some(NormalizedString::new_unchecked(
                "invalid\tphone".to_string(),
            )),
        };
        let actual = contact
            .validate_with_context(ValidationContext::default())
            .expect("Failed to validate contact");
        assert_eq!(
            actual,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![ValidationPathComponent::Struct {
                            struct_name: "OrganizationalContact".to_string(),
                            field_name: "name".to_string()
                        }])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![ValidationPathComponent::Struct {
                            struct_name: "OrganizationalContact".to_string(),
                            field_name: "email".to_string()
                        }])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![ValidationPathComponent::Struct {
                            struct_name: "OrganizationalContact".to_string(),
                            field_name: "phone".to_string()
                        }])
                    }
                ]
            }
        )
    }

    #[test]
    fn it_should_validate_an_invalid_entity_as_failed() {
        let entity = OrganizationalEntity {
            name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
            url: None,
            contact: None,
        };
        let actual = entity
            .validate_with_context(ValidationContext::default())
            .expect("Failed to validate entity");
        assert_eq!(
            actual,
            ValidationResult::Failed {
                reasons: vec![FailureReason {
                    message: "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                        .to_string(),
                    context: ValidationContext(vec![ValidationPathComponent::Struct {
                        struct_name: "OrganizationalEntity".to_string(),
                        field_name: "name".to_string()
                    }])
                }]
            }
        )
    }

    #[test]
    fn it_should_validate_an_entity_with_multiple_validation_issues_as_failed() {
        let entity = OrganizationalEntity {
            name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
            url: Some(vec![Uri("invalid uri".to_string())]),
            contact: Some(vec![OrganizationalContact {
                name: Some(NormalizedString::new_unchecked("invalid\tname".to_string())),
                email: None,
                phone: None,
            }]),
        };
        let actual = entity
            .validate_with_context(ValidationContext::default())
            .expect("Failed to validate entity");
        assert_eq!(
            actual,
            ValidationResult::Failed {
                reasons: vec![
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![ValidationPathComponent::Struct {
                            struct_name: "OrganizationalEntity".to_string(),
                            field_name: "name".to_string()
                        }])
                    },
                    FailureReason {
                        message: "Uri does not conform to RFC 3986".to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "OrganizationalEntity".to_string(),
                                field_name: "url".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::Struct {
                                struct_name: "OrganizationalEntity".to_string(),
                                field_name: "contact".to_string()
                            },
                            ValidationPathComponent::Array { index: 0 },
                            ValidationPathComponent::Struct {
                                struct_name: "OrganizationalContact".to_string(),
                                field_name: "name".to_string()
                            }
                        ])
                    }
                ]
            }
        )
    }
}
