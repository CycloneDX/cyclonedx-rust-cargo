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

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    errors::XmlReadError,
    models,
    xml::{read_simple_tag, to_xml_read_error, unexpected_element_error, FromXml},
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Signature {
    pub algorithm: Algorithm,
    pub value: String,
}

impl From<models::signature::Signature> for Signature {
    fn from(other: models::signature::Signature) -> Self {
        Signature {
            algorithm: other.algorithm.into(),
            value: other.value,
        }
    }
}

impl From<Signature> for models::signature::Signature {
    fn from(other: Signature) -> Self {
        models::signature::Signature {
            algorithm: other.algorithm.into(),
            value: other.value,
        }
    }
}

/// Supported signature algorithms.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Algorithm {
    RS256,
    RS384,
    RS512,
    PS256,
    PS384,
    PS512,
    ES256,
    ES384,
    ES512,
    Ed25519,
    Ed448,
    HS256,
    HS384,
    HS512,
}

impl FromStr for Algorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RS256" => Ok(Algorithm::RS256),
            "RS384" => Ok(Algorithm::RS384),
            "RS512" => Ok(Algorithm::RS512),
            "PS256" => Ok(Algorithm::PS256),
            "PS384" => Ok(Algorithm::PS384),
            "PS512" => Ok(Algorithm::PS512),
            "ES256" => Ok(Algorithm::ES256),
            "ES384" => Ok(Algorithm::ES384),
            "ES512" => Ok(Algorithm::ES512),
            "Ed25519" => Ok(Algorithm::Ed25519),
            "Ed448" => Ok(Algorithm::Ed448),
            "HS256" => Ok(Algorithm::HS256),
            "HS384" => Ok(Algorithm::HS384),
            "HS512" => Ok(Algorithm::HS512),
            _ => Err(format!("Invalid algorithm '{}' found", s)),
        }
    }
}

impl ToString for Algorithm {
    fn to_string(&self) -> String {
        let s = match self {
            Algorithm::RS256 => "RS256",
            Algorithm::RS384 => "RS384",
            Algorithm::RS512 => "RS512",
            Algorithm::PS256 => "PS256",
            Algorithm::PS384 => "PS384",
            Algorithm::PS512 => "PS512",
            Algorithm::ES256 => "ES256",
            Algorithm::ES384 => "ES384",
            Algorithm::ES512 => "ES512",
            Algorithm::Ed25519 => "Ed25519",
            Algorithm::Ed448 => "Ed448",
            Algorithm::HS256 => "HS256",
            Algorithm::HS384 => "HS384",
            Algorithm::HS512 => "HS512",
        };
        s.to_string()
    }
}

impl From<models::signature::Algorithm> for Algorithm {
    fn from(other: models::signature::Algorithm) -> Self {
        other
            .to_string()
            .parse::<Algorithm>()
            .expect("Failed to convert algorithm")
    }
}

impl From<Algorithm> for models::signature::Algorithm {
    fn from(other: Algorithm) -> Self {
        other
            .to_string()
            .parse::<models::signature::Algorithm>()
            .expect("Failed to convert algorithm")
    }
}

const SIGNATURE_TAG: &str = "signature";
const ALGORITHM_TAG: &str = "algorithm";
const VALUE_TAG: &str = "value";

impl FromXml for Signature {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut algorithm: Option<String> = None;
        let mut value: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(SIGNATURE_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == ALGORITHM_TAG => {
                    algorithm = Some(read_simple_tag(event_reader, &name)?);
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == VALUE_TAG => {
                    value = Some(read_simple_tag(event_reader, &name)?);
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        // get required attributesInvalidEnumVariant
        let algorithm = algorithm.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: ALGORITHM_TAG.to_string(),
            element: SIGNATURE_TAG.to_string(),
        })?;
        let value = value.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: VALUE_TAG.to_string(),
            element: SIGNATURE_TAG.to_string(),
        })?;

        let algorithm =
            algorithm
                .parse::<Algorithm>()
                .map_err(|_| XmlReadError::InvalidEnumVariant {
                    value: algorithm.to_string(),
                    element: ALGORITHM_TAG.to_string(),
                })?;

        Ok(Self { algorithm, value })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use xml::{name::OwnedName, EventReader, ParserConfig};

    use crate::{
        models,
        xml::{test::read_element_from_string, FromXml},
    };

    use super::{Algorithm, Signature};

    pub(crate) fn example_signature() -> Signature {
        Signature {
            algorithm: Algorithm::HS512,
            value: "1234567890".to_string(),
        }
    }

    pub(crate) fn corresponding_signature() -> models::signature::Signature {
        models::signature::Signature {
            algorithm: models::signature::Algorithm::HS512,
            value: "1234567890".to_string(),
        }
    }

    #[track_caller]
    fn assert_valid_signature(input: &str, expected: Signature) {
        let actual: Signature = read_element_from_string(input);
        assert_eq!(actual, expected);
    }

    #[track_caller]
    fn assert_invalid_signature(input: &str) {
        let reader = input.to_string();
        let config = ParserConfig::default().trim_whitespace(true);
        let mut event_reader = EventReader::new_with_config(reader.as_bytes(), config);

        let element_name = OwnedName::local("signature");
        let actual = Signature::read_xml_element(&mut event_reader, &element_name, &[]);
        assert!(actual.is_err());
    }

    #[test]
    fn it_should_read_valid_signature() {
        let input = r#"
<signature>
    <algorithm>RS512</algorithm>
    <value>abcdefghijklmnopqrstuvwxyz</value>
</signature>
"#;
        let expected = Signature {
            algorithm: Algorithm::RS512,
            value: "abcdefghijklmnopqrstuvwxyz".to_string(),
        };
        assert_valid_signature(input, expected);
    }

    #[test]
    fn it_shoud_fail_with_missing_value() {
        let input = r#"
<signature>
    <algorithm><RS512/algorithm>
</signature>
"#;
        assert_invalid_signature(input);
    }

    #[test]
    fn it_should_fail_with_missing_algorithm() {
        let input = r#"
<signature>
    <value>abcdefghijklmnopqrstuvwxyz</value>
</signature>
"#;
        assert_invalid_signature(input);
    }

    #[test]
    fn it_should_fail_with_invalid_algorithm() {
        let input = r#"
<signature>
    <algorithm><ABCD/algorithm>
    <value>abcdefghijklmnopqrstuvwxyz</value>
</signature>
"#;
        assert_invalid_signature(input);
    }
}
