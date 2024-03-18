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
    errors::XmlReadError,
    models,
    utilities::convert_vec,
    xml::{attribute_or_error, read_list_tag, read_simple_tag, to_xml_write_error, FromXml, ToXml},
};
use serde::{Deserialize, Serialize};
use xml::writer;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Hashes(pub(crate) Vec<Hash>);

impl From<models::hash::Hashes> for Hashes {
    fn from(other: models::hash::Hashes) -> Self {
        Hashes(convert_vec(other.0))
    }
}

impl From<Hashes> for models::hash::Hashes {
    fn from(other: Hashes) -> Self {
        models::hash::Hashes(convert_vec(other.0))
    }
}

const HASHES_TAG: &str = "hashes";

impl ToXml for Hashes {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(HASHES_TAG))
            .map_err(to_xml_write_error(HASHES_TAG))?;

        for hash in &self.0 {
            hash.write_xml_element(writer)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(HASHES_TAG))?;
        Ok(())
    }
}

impl FromXml for Hashes {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, HASH_TAG).map(Hashes)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Hash {
    pub(crate) alg: String,
    pub(crate) content: HashValue,
}

impl From<models::hash::Hash> for Hash {
    fn from(other: models::hash::Hash) -> Self {
        Self {
            alg: other.alg.to_string(),
            content: other.content.into(),
        }
    }
}

impl From<Hash> for models::hash::Hash {
    fn from(other: Hash) -> Self {
        Self {
            alg: models::hash::HashAlgorithm::new_unchecked(other.alg),
            content: other.content.into(),
        }
    }
}

const HASH_TAG: &str = "hash";
const ALG_ATTR: &str = "alg";

impl ToXml for Hash {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(HASH_TAG).attr(ALG_ATTR, &self.alg))
            .map_err(to_xml_write_error(HASH_TAG))?;

        writer
            .write(writer::XmlEvent::characters(&self.content.0))
            .map_err(to_xml_write_error(HASH_TAG))?;

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(HASH_TAG))?;
        Ok(())
    }
}

impl FromXml for Hash {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let alg = attribute_or_error(element_name, attributes, ALG_ATTR)?;
        let value = read_simple_tag(event_reader, element_name)?;

        Ok(Self {
            alg,
            content: HashValue(value),
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct HashValue(pub(crate) String);

impl From<models::hash::HashValue> for HashValue {
    fn from(other: models::hash::HashValue) -> Self {
        Self(other.0)
    }
}

impl From<HashValue> for models::hash::HashValue {
    fn from(other: HashValue) -> Self {
        Self(other.0)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;

    pub(crate) fn example_hashes() -> Hashes {
        Hashes(vec![example_hash()])
    }

    pub(crate) fn corresponding_hashes() -> models::hash::Hashes {
        models::hash::Hashes(vec![corresponding_hash()])
    }

    pub(crate) fn example_hash() -> Hash {
        Hash {
            alg: "algorithm".to_string(),
            content: HashValue("hash value".to_string()),
        }
    }

    pub(crate) fn corresponding_hash() -> models::hash::Hash {
        models::hash::Hash {
            alg: models::hash::HashAlgorithm::UnknownHashAlgorithm("algorithm".to_string()),
            content: models::hash::HashValue("hash value".to_string()),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_hashes());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<hashes>
  <hash alg="algorithm">hash value</hash>
</hashes>
"#;
        let actual: Hashes = read_element_from_string(input);
        let expected = example_hashes();
        assert_eq!(actual, expected);
    }
}
