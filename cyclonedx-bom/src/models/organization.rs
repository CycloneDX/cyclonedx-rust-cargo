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
        Validate, ValidationContext, ValidationError, ValidationPathComponent,
        ValidationResult,
    },
};

#[derive(Debug, PartialEq)]
pub struct OrganizationalContact {
    pub name: Option<NormalizedString>,
    pub email: Option<NormalizedString>,
    pub phone: Option<NormalizedString>,
}

impl Validate for OrganizationalContact {
    fn validate_with_context(
        &self,
        context: ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut all_reasons = Vec::new();

        if let Some(name) = &self.name {
            let name_component = vec![ValidationPathComponent::StructComponent {
                struct_name: "OrganizationalContact".to_string(),
                field_name: "name".to_string(),
            }];
            let name_context = context.extend_context(name_component);

            if let ValidationResult::Failed { reasons } =
                name.validate_with_context(name_context)?
            {
                all_reasons.append(&mut reasons.clone());
            }
        }

        if let Some(email) = &self.email {
            let email_component = vec![ValidationPathComponent::StructComponent {
                struct_name: "OrganizationalContact".to_string(),
                field_name: "email".to_string(),
            }];
            let email_context = context.extend_context(email_component);
            if let ValidationResult::Failed { reasons } =
                email.validate_with_context(email_context)?
            {
                all_reasons.append(&mut reasons.clone());
            }
        }

        if let Some(phone) = &self.phone {
            let phone_component = vec![ValidationPathComponent::StructComponent {
                struct_name: "OrganizationalContact".to_string(),
                field_name: "phone".to_string(),
            }];
            let phone_context = context.extend_context(phone_component);
            if let ValidationResult::Failed { reasons } =
                phone.validate_with_context(phone_context)?
            {
                all_reasons.append(&mut reasons.clone());
            }
        }

        if all_reasons.is_empty() {
            Ok(ValidationResult::Passed)
        } else {
            Ok(ValidationResult::Failed {
                reasons: all_reasons,
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct OrganizationalEntity {
    pub name: Option<NormalizedString>,
    pub url: Option<Vec<Uri>>,
    pub contact: Option<Vec<OrganizationalContact>>,
}

#[cfg(test)]
mod test {
    use crate::validation::FailureReason;

    use super::*;

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
                    context: ValidationContext(vec![ValidationPathComponent::StructComponent {
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
                        context: ValidationContext(vec![
                            ValidationPathComponent::StructComponent {
                                struct_name: "OrganizationalContact".to_string(),
                                field_name: "name".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::StructComponent {
                                struct_name: "OrganizationalContact".to_string(),
                                field_name: "email".to_string()
                            }
                        ])
                    },
                    FailureReason {
                        message:
                            "NormalizedString contains invalid characters \\r \\n \\t or \\r\\n"
                                .to_string(),
                        context: ValidationContext(vec![
                            ValidationPathComponent::StructComponent {
                                struct_name: "OrganizationalContact".to_string(),
                                field_name: "phone".to_string()
                            }
                        ])
                    }
                ]
            }
        )
    }
}
