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
    external_models::normalized_string::NormalizedString,
    models,
    xml::{
        attribute_or_error, read_lax_validation_list_tag, read_simple_tag, to_xml_write_error,
        write_close_tag, write_start_tag, FromXml, ToXml,
    },
};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Properties(pub(crate) Vec<Property>);

impl From<models::property::Properties> for Properties {
    fn from(other: models::property::Properties) -> Self {
        Self(other.0.into_iter().map(std::convert::Into::into).collect())
    }
}

impl From<Properties> for models::property::Properties {
    fn from(other: Properties) -> Self {
        Self(other.0.into_iter().map(std::convert::Into::into).collect())
    }
}

const PROPERTIES_TAG: &str = "properties";

impl ToXml for Properties {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        write_start_tag(writer, PROPERTIES_TAG)?;

        for property in &self.0 {
            property.write_xml_element(writer)?;
        }

        write_close_tag(writer, PROPERTIES_TAG)?;

        Ok(())
    }
}

impl FromXml for Properties {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        read_lax_validation_list_tag(event_reader, element_name, PROPERTY_TAG).map(Properties)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Property {
    pub(crate) name: String,
    pub(crate) value: String,
}

impl From<models::property::Property> for Property {
    fn from(other: models::property::Property) -> Self {
        Self {
            name: other.name,
            value: other.value.0,
        }
    }
}

impl From<Property> for models::property::Property {
    fn from(other: Property) -> Self {
        Self {
            name: other.name,
            value: NormalizedString::new_unchecked(other.value),
        }
    }
}

const PROPERTY_TAG: &str = "property";
const NAME_ATTR: &str = "name";

impl ToXml for Property {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        writer
            .write(XmlEvent::start_element(PROPERTY_TAG).attr("name", &self.name))
            .map_err(to_xml_write_error(PROPERTY_TAG))?;

        writer
            .write(XmlEvent::characters(&self.value))
            .map_err(to_xml_write_error(PROPERTY_TAG))?;

        write_close_tag(writer, PROPERTY_TAG)?;

        Ok(())
    }
}

impl FromXml for Property {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let name = attribute_or_error(element_name, attributes, NAME_ATTR)?;
        let value = read_simple_tag(event_reader, element_name)?;
        Ok(Self { name, value })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    pub(crate) fn example_properties() -> Properties {
        Properties(vec![Property {
            name: "name".to_string(),
            value: "value".to_string(),
        }])
    }

    pub(crate) fn corresponding_properties() -> models::property::Properties {
        models::property::Properties(vec![models::property::Property {
            name: "name".to_string(),
            value: NormalizedString::new_unchecked("value".to_string()),
        }])
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_properties());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_write_xml_properties_with_no_children() {
        let xml_output = write_element_to_string(Properties(Vec::new()));
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<properties>
  <property name="name">value</property>
</properties>
"#;
        let actual: Properties = read_element_from_string(input);
        let expected = example_properties();
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_xml_properties_with_no_children() {
        let input = r#"
<properties />
"#;
        let actual: Properties = read_element_from_string(input);
        let expected = Properties(Vec::new());
        assert_eq!(actual, expected);
    }
}
