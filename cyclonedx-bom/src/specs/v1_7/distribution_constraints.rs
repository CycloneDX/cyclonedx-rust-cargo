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

use crate::{
    errors::XmlReadError,
    get_elements_lax,
    models::metadata as models,
    xml::{write_close_tag, write_simple_option_tag, write_start_tag, FromXml, ToXml},
};

/// bom-1.7.schema.json #definitions/metadata/properties/distributionConstraints
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DistributionConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    tlp: Option<String>,
}

impl From<models::DistributionConstraints> for DistributionConstraints {
    fn from(other: models::DistributionConstraints) -> Self {
        Self {
            tlp: other.tlp.map(|tlp| tlp.to_string()),
        }
    }
}

impl From<DistributionConstraints> for models::DistributionConstraints {
    fn from(other: DistributionConstraints) -> Self {
        Self {
            tlp: other.tlp.map(models::TlpClassification::new_unchecked),
        }
    }
}

const DISTRIBUTION_CONSTRAINTS_TAG: &str = "distributionConstraints";
const TLP_TAG: &str = "tlp";

impl ToXml for DistributionConstraints {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, DISTRIBUTION_CONSTRAINTS_TAG)?;
        write_simple_option_tag(writer, TLP_TAG, &self.tlp)?;
        write_close_tag(writer, DISTRIBUTION_CONSTRAINTS_TAG)
    }
}

impl FromXml for DistributionConstraints {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        get_elements_lax! {
            event_reader, element_name,
            TLP_TAG => tlp: String,
        };

        Ok(Self { tlp })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::xml::test::{read_element_from_string, write_element_to_string};
    use pretty_assertions::assert_eq;

    pub(crate) fn example_distribution_constraints() -> DistributionConstraints {
        DistributionConstraints {
            tlp: Some("AMBER_AND_STRICT".to_string()),
        }
    }

    pub(crate) fn corresponding_distribution_constraints() -> models::DistributionConstraints {
        models::DistributionConstraints {
            tlp: Some(models::TlpClassification::AmberAndStrict),
        }
    }

    #[test]
    fn it_should_convert_to_model() {
        let actual: models::DistributionConstraints = example_distribution_constraints().into();
        assert_eq!(actual, corresponding_distribution_constraints());
    }

    #[test]
    fn it_should_convert_from_model() {
        let actual: DistributionConstraints = corresponding_distribution_constraints().into();
        assert_eq!(actual, example_distribution_constraints());
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_distribution_constraints());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<distributionConstraints>
  <tlp>AMBER_AND_STRICT</tlp>
</distributionConstraints>
"#;
        let actual: DistributionConstraints = read_element_from_string(input);
        assert_eq!(actual, example_distribution_constraints());
    }
}
