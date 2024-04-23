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
    external_models::{normalized_string::NormalizedString, uri::Uri},
    models,
    utilities::convert_vec,
    xml::{
        read_lax_validation_list_tag, read_lax_validation_tag, read_simple_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_close_tag, write_simple_tag,
        write_start_tag, FromXml, ToXml,
    },
};
use serde::{Deserialize, Serialize};
use xml::{reader, writer::XmlEvent};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Advisories(Vec<Advisory>);

impl From<models::advisory::Advisories> for Advisories {
    fn from(other: models::advisory::Advisories) -> Self {
        Advisories(convert_vec(other.0))
    }
}

impl From<Advisories> for models::advisory::Advisories {
    fn from(other: Advisories) -> Self {
        models::advisory::Advisories(convert_vec(other.0))
    }
}

const ADVISORIES_TAG: &str = "advisories";

impl ToXml for Advisories {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, ADVISORIES_TAG)?;

        for advisory in &self.0 {
            advisory.write_xml_element(writer)?;
        }

        write_close_tag(writer, ADVISORIES_TAG)?;

        Ok(())
    }
}

impl FromXml for Advisories {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_lax_validation_list_tag(event_reader, element_name, ADVISORY_TAG).map(Advisories)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Advisory {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    url: String,
}

impl From<models::advisory::Advisory> for Advisory {
    fn from(other: models::advisory::Advisory) -> Self {
        Self {
            title: other.title.map(|t| t.to_string()),
            url: other.url.to_string(),
        }
    }
}

impl From<Advisory> for models::advisory::Advisory {
    fn from(other: Advisory) -> Self {
        Self {
            title: other.title.map(NormalizedString::new_unchecked),
            url: Uri(other.url),
        }
    }
}

const ADVISORY_TAG: &str = "advisory";
const TITLE_TAG: &str = "title";
const URL_TAG: &str = "url";

impl ToXml for Advisory {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let advisory_start_tag = XmlEvent::start_element(ADVISORY_TAG);

        writer
            .write(advisory_start_tag)
            .map_err(to_xml_write_error(ADVISORY_TAG))?;

        if let Some(title) = &self.title {
            write_simple_tag(writer, TITLE_TAG, title)?;
        }

        write_simple_tag(writer, URL_TAG, &self.url)?;

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(ADVISORY_TAG))?;

        Ok(())
    }
}

impl FromXml for Advisory {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut title: Option<String> = None;
        let mut url: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(ADVISORY_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == TITLE_TAG => {
                    title = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == URL_TAG => {
                    url = Some(read_simple_tag(event_reader, &name)?)
                }

                // lax validation of any elements from a different schema
                reader::XmlEvent::StartElement { name, .. } => {
                    read_lax_validation_tag(event_reader, &name)?
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let url = url.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: URL_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(Self { title, url })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    pub(crate) fn example_advisories() -> Advisories {
        Advisories(vec![example_advisory()])
    }

    pub(crate) fn corresponding_advisories() -> models::advisory::Advisories {
        models::advisory::Advisories(vec![corresponding_advisory()])
    }

    fn example_advisory() -> Advisory {
        Advisory {
            title: Some("title".to_string()),
            url: "url".to_string(),
        }
    }

    fn corresponding_advisory() -> models::advisory::Advisory {
        models::advisory::Advisory {
            title: Some(NormalizedString::new_unchecked("title".to_string())),
            url: Uri("url".to_string()),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_advisories());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<advisories>
  <advisory>
    <title>title</title>
    <url>url</url>
  </advisory>
</advisories>
"#;
        let actual: Advisories = read_element_from_string(input);
        let expected = example_advisories();
        assert_eq!(actual, expected);
    }
}
