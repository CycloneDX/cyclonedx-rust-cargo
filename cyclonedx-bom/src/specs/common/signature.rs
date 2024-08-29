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
    models,
    xml::{
        read_list_tag, read_simple_tag, to_xml_read_error, unexpected_element_error,
        write_close_tag, write_simple_tag, write_start_tag, FromXml, ToXml,
    },
};

/// For now the [`Signer`] struct only holds algorithm and value
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Signer {
    /// Signature algorithm.
    pub algorithm: Algorithm,
    /// The signature data.
    pub value: String,
}

impl Signer {
    pub fn new(algorithm: &str, value: &str) -> Self {
        Self {
            algorithm: Algorithm::new_unchecked(algorithm),
            value: value.to_string(),
        }
    }
}

impl ToXml for Signer {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_simple_tag(writer, ALGORITHM_TAG, &self.algorithm.to_string())?;
        write_simple_tag(writer, VALUE_TAG, &self.value)?;

        Ok(())
    }
}

impl FromXml for Signer {
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
                unexpected => {
                    return Err(unexpected_element_error(element_name, unexpected));
                }
            }
        }

        let algorithm = algorithm.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: ALGORITHM_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;
        let value = value.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: VALUE_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(Self {
            algorithm: Algorithm::new_unchecked(algorithm.as_str()),
            value,
        })
    }
}

/// Enveloped signature in [JSON Signature Format (JSF)](https://cyberphone.github.io/doc/security/jsf.html)
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Signature {
    /// Multiple signatures
    Signers(Vec<Signer>),
    /// A single signature chain
    Chain(Vec<Signer>),
    /// A single signature
    Single(Signer),
}

impl Signature {
    /// Creates a single [`Signature::Single`].
    pub fn single(algorithm: &str, value: &str) -> Self {
        Self::Single(Signer::new(algorithm, value))
    }

    /// Creates a [`Signature::Chain`].
    pub fn chain(chain: &[(&str, &str)]) -> Self {
        Self::Chain(
            chain
                .iter()
                .map(|(algorithm, value)| Signer::new(algorithm, value))
                .collect(),
        )
    }

    /// Creates a [`Signature::Signers`].
    pub fn signers(signers: &[(&str, &str)]) -> Self {
        Self::Signers(
            signers
                .iter()
                .map(|(algorithm, value)| Signer::new(algorithm, value))
                .collect(),
        )
    }
}

impl From<models::signature::Signature> for Signature {
    fn from(other: models::signature::Signature) -> Self {
        match other {
            models::signature::Signature::Signers(signers) => {
                Signature::Signers(signers.into_iter().map(From::from).collect())
            }
            models::signature::Signature::Chain(chain) => {
                Signature::Chain(chain.into_iter().map(From::from).collect())
            }
            models::signature::Signature::Single(signer) => Signature::Single(signer.into()),
        }
    }
}

impl From<models::signature::Signer> for Signer {
    fn from(signer: models::signature::Signer) -> Self {
        Self {
            algorithm: signer.algorithm.into(),
            value: signer.value,
        }
    }
}

impl From<Signer> for models::signature::Signer {
    fn from(signer: Signer) -> Self {
        Self {
            algorithm: signer.algorithm.into(),
            value: signer.value,
        }
    }
}

impl From<Signature> for models::signature::Signature {
    fn from(signature: Signature) -> Self {
        match signature {
            Signature::Signers(signers) => {
                models::signature::Signature::Signers(signers.into_iter().map(From::from).collect())
            }
            Signature::Chain(chain) => {
                models::signature::Signature::Chain(chain.into_iter().map(From::from).collect())
            }
            Signature::Single(signer) => models::signature::Signature::Single(signer.into()),
        }
    }
}

/// Supported signature algorithms.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, strum::Display)]
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
    Unknown(String),
}

impl Algorithm {
    pub fn new_unchecked(algorithm: &str) -> Self {
        match algorithm {
            "RS256" => Algorithm::RS256,
            "RS384" => Algorithm::RS384,
            "RS512" => Algorithm::RS512,
            "PS256" => Algorithm::PS256,
            "PS384" => Algorithm::PS384,
            "PS512" => Algorithm::PS512,
            "ES256" => Algorithm::ES256,
            "ES384" => Algorithm::ES384,
            "ES512" => Algorithm::ES512,
            "Ed25519" => Algorithm::Ed25519,
            "Ed448" => Algorithm::Ed448,
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            unknown => Algorithm::Unknown(unknown.to_string()),
        }
    }
}

impl From<models::signature::Algorithm> for Algorithm {
    fn from(other: models::signature::Algorithm) -> Self {
        Algorithm::new_unchecked(other.to_string().as_str())
    }
}

impl From<Algorithm> for models::signature::Algorithm {
    fn from(other: Algorithm) -> Self {
        Self::new_unchecked(other.to_string().as_str())
    }
}

impl ToXml for Signature {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, SIGNATURE_TAG)?;

        match self {
            Signature::Signers(signers) => {
                write_start_tag(writer, SIGNERS_TAG)?;
                for signer in signers {
                    write_start_tag(writer, SIGNER_TAG)?;
                    signer.write_xml_element(writer)?;
                    write_close_tag(writer, SIGNER_TAG)?;
                }
                write_close_tag(writer, SIGNERS_TAG)?;
            }
            Signature::Chain(chain) => {
                write_start_tag(writer, CHAIN_TAG)?;
                for signer in chain {
                    write_start_tag(writer, CHAIN_INNER_TAG)?;
                    signer.write_xml_element(writer)?;
                    write_close_tag(writer, CHAIN_INNER_TAG)?;
                }
                write_close_tag(writer, CHAIN_TAG)?;
            }
            Signature::Single(signer) => signer.write_xml_element(writer)?,
        }

        write_close_tag(writer, SIGNATURE_TAG)?;

        Ok(())
    }
}

const SIGNERS_TAG: &str = "signers";
const SIGNER_TAG: &str = "signer";
const CHAIN_TAG: &str = "chain";
const CHAIN_INNER_TAG: &str = "chain";
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
        let mut signature: Option<Signature> = None;
        let mut algorithm: Option<String> = None;
        let mut value: Option<String> = None;
        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(SIGNATURE_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == SIGNERS_TAG => {
                    let signers = read_list_tag(event_reader, &name, SIGNER_TAG)?;
                    signature = Some(Signature::Signers(signers));
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == CHAIN_TAG => {
                    let chain = read_list_tag(event_reader, &name, CHAIN_INNER_TAG)?;
                    signature = Some(Signature::Chain(chain));
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == ALGORITHM_TAG => {
                    algorithm = Some(read_simple_tag(event_reader, &name)?);
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == VALUE_TAG => {
                    value = Some(read_simple_tag(event_reader, &name)?);
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                _ => {
                    let signer = Signer::read_xml_element(event_reader, element_name, &[])?;
                    signature = Some(Signature::Single(signer));
                }
            }
        }

        // When multiple signers for a signature return them, otherwise we expect the flat variant.
        // Unfortunately this duplicates some code.
        let signature = if let Some(signature) = signature {
            signature
        } else {
            let algorithm = algorithm.ok_or_else(|| XmlReadError::RequiredDataMissing {
                required_field: ALGORITHM_TAG.to_string(),
                element: element_name.local_name.to_string(),
            })?;
            let value = value.ok_or_else(|| XmlReadError::RequiredDataMissing {
                required_field: VALUE_TAG.to_string(),
                element: element_name.local_name.to_string(),
            })?;

            Signature::single(&algorithm, &value)
        };

        Ok(signature)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use xml::{name::OwnedName, EmitterConfig, EventReader, EventWriter, ParserConfig};

    use crate::{
        models,
        xml::{test::read_element_from_string, FromXml, ToXml},
    };

    use super::Signature;

    pub(crate) fn example_signature() -> Signature {
        Signature::single("HS512", "1234567890")
    }

    pub(crate) fn corresponding_signature() -> models::signature::Signature {
        models::signature::Signature::single(models::signature::Algorithm::HS512, "1234567890")
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

    #[track_caller]
    fn assert_write_xml(signature: Signature, expected_output: &str) {
        let mut writer = Vec::new();
        let config = EmitterConfig::default()
            .perform_indent(true)
            .write_document_declaration(false);
        let mut event_writer = EventWriter::new_with_config(&mut writer, config);

        signature
            .write_xml_element(&mut event_writer)
            .expect("Failed to write signature");
        let actual_output = String::from_utf8_lossy(&writer);

        let expected_output = expected_output.trim();

        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn it_should_read_valid_signature() {
        let input = r#"
<signature>
    <algorithm>RS512</algorithm>
    <value>abcdefghijklmnopqrstuvwxyz</value>
</signature>
"#;
        let expected = Signature::single("RS512", "abcdefghijklmnopqrstuvwxyz");
        assert_valid_signature(input, expected);
    }

    #[test]
    fn it_should_fail_with_missing_value() {
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

    #[test]
    fn it_should_write_xml_successfully() {
        let expected = r#"
<signature>
  <algorithm>ES256</algorithm>
  <value>abcdefgh</value>
</signature>"#;
        let signature = Signature::single("ES256", "abcdefgh");
        assert_write_xml(signature, expected);
    }

    #[test]
    fn it_should_write_signature_signers_successfully() {
        let expected = r#"
<signature>
  <signers>
    <signer>
      <algorithm>ES256</algorithm>
      <value>abcdefgh</value>
    </signer>
    <signer>
      <algorithm>HS512</algorithm>
      <value>1234567890</value>
    </signer>
  </signers>
</signature>
"#;
        let signature = Signature::signers(&[("ES256", "abcdefgh"), ("HS512", "1234567890")]);
        assert_write_xml(signature, expected);
    }

    #[test]
    fn it_should_write_signature_chain_successfully() {
        let expected = r#"
<signature>
  <chain>
    <chain>
      <algorithm>ES256</algorithm>
      <value>abcdefgh</value>
    </chain>
    <chain>
      <algorithm>HS512</algorithm>
      <value>1234567890</value>
    </chain>
  </chain>
</signature>
"#;
        let signature = Signature::chain(&[("ES256", "abcdefgh"), ("HS512", "1234567890")]);
        assert_write_xml(signature, expected);
    }

    #[test]
    fn it_should_read_single_signature() {
        let input = r#"
<signature>
  <algorithm>HS512</algorithm>
  <value>abcdefgh</value>
</signature>
        "#;
        let expected = Signature::single("HS512", "abcdefgh");
        let actual: Signature = read_element_from_string(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_fail_to_read_with_empty_signature() {
        let input = r#"<signature></signature>"#;
        assert_invalid_signature(input);
    }

    #[test]
    fn it_should_read_multiple_signers_signature() {
        let input = r#"
<signature>
  <signers>
    <signer bom-ref="bom-ref">
      <algorithm>ES256</algorithm>
      <value>abcdefgh</value>
    </signer>
    <signer bom-ref="bom-ref">
      <algorithm>HS512</algorithm>
      <value>1234567890</value>
    </signer>
  </signers>
</signature>
        "#;
        let expected = Signature::signers(&[("ES256", "abcdefgh"), ("HS512", "1234567890")]);
        let actual: Signature = read_element_from_string(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_chain_signature() {
        let input = r#"
<signature>
  <chain>
    <chain bom-ref="bom-ref">
      <algorithm>ES256</algorithm>
      <value>abcdefgh</value>
    </chain>
    <chain bom-ref="bom-ref">
      <algorithm>HS512</algorithm>
      <value>1234567890</value>
    </chain>
  </chain>
</signature>
        "#;
        let expected = Signature::chain(&[("ES256", "abcdefgh"), ("HS512", "1234567890")]);
        let actual: Signature = read_element_from_string(input);
        assert_eq!(actual, expected);
    }
}
