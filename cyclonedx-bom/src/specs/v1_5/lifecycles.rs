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
use xml::{name::OwnedName, reader};

use crate::{
    errors::XmlReadError,
    models,
    prelude::NormalizedString,
    utilities::convert_vec,
    xml::{
        read_list_tag, read_simple_tag, to_xml_read_error, unexpected_element_error,
        write_close_tag, write_simple_tag, write_start_tag, FromXml, ToXml,
    },
};

/// Represents a list of `Lifecycle`.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Lifecycles(Vec<Lifecycle>);

impl From<models::lifecycle::Lifecycles> for Lifecycles {
    fn from(other: models::lifecycle::Lifecycles) -> Self {
        Lifecycles(convert_vec(other.0))
    }
}

impl From<Lifecycles> for models::lifecycle::Lifecycles {
    fn from(other: Lifecycles) -> Self {
        models::lifecycle::Lifecycles(convert_vec(other.0))
    }
}

const LIFECYCLES_TAG: &str = "lifecycles";

impl ToXml for Lifecycles {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, LIFECYCLES_TAG)?;

        for lifecycle in &self.0 {
            lifecycle.write_xml_element(writer)?;
        }

        write_close_tag(writer, LIFECYCLES_TAG)?;

        Ok(())
    }
}

const LIFECYCLE_TAG: &str = "lifecycle";
const PHASE_TAG: &str = "phase";
const DESCRIPTION_TAG: &str = "description";
const NAME_TAG: &str = "name";

impl FromXml for Lifecycles {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, LIFECYCLE_TAG).map(Lifecycles)
    }
}

/// Represents a `Lifecycle`, see https://cyclonedx.org/docs/1.5/json/#metadata_lifecycles
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Lifecycle {
    Phase(Phase),
    #[serde(untagged)]
    Description(Description),
}

impl From<models::lifecycle::Lifecycle> for Lifecycle {
    fn from(other: models::lifecycle::Lifecycle) -> Self {
        match other {
            models::lifecycle::Lifecycle::Phase(phase) => Self::Phase(phase.into()),
            models::lifecycle::Lifecycle::Description(desc) => Self::Description(desc.into()),
        }
    }
}

impl From<Lifecycle> for models::lifecycle::Lifecycle {
    fn from(other: Lifecycle) -> Self {
        match other {
            Lifecycle::Phase(phase) => models::lifecycle::Lifecycle::Phase(phase.into()),
            Lifecycle::Description(desc) => models::lifecycle::Lifecycle::Description(desc.into()),
        }
    }
}

impl ToXml for Lifecycle {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, LIFECYCLE_TAG)?;

        match self {
            Lifecycle::Phase(phase) => {
                write_simple_tag(writer, PHASE_TAG, &phase.0)?;
            }
            Lifecycle::Description(desc) => {
                write_simple_tag(writer, NAME_TAG, &desc.name)?;
                if let Some(description) = &desc.description {
                    write_simple_tag(writer, DESCRIPTION_TAG, description)?;
                }
            }
        }

        write_close_tag(writer, LIFECYCLE_TAG)?;

        Ok(())
    }
}

impl FromXml for Lifecycle {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut phase: Option<String> = None;
        let mut desc_name: Option<String> = None;
        let mut description: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(LIFECYCLE_TAG))?;

            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == PHASE_TAG => {
                    phase = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    desc_name = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == DESCRIPTION_TAG =>
                {
                    description = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        // Either there is a "phase" or a tuple of ("name", "description") available.
        match (phase, desc_name, description) {
            (Some(phase), None, None) => Ok(Self::Phase(Phase(phase))),
            (None, Some(name), description) => {
                Ok(Self::Description(Description { name, description }))
            }
            _ => Err(XmlReadError::RequiredDataMissing {
                required_field: "phase or name".to_string(),
                element: element_name.local_name.to_string(),
            }),
        }
    }
}

/// Allowed values for a Phase Lifecycle
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub(crate) struct Phase(String);

impl Phase {
    #[allow(unused)]
    pub(crate) fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<models::lifecycle::Phase> for Phase {
    fn from(other: models::lifecycle::Phase) -> Self {
        Self(other.to_string())
    }
}

impl From<Phase> for models::lifecycle::Phase {
    fn from(other: Phase) -> Self {
        Self::new_unchecked(other.0)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Description {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl From<models::lifecycle::Description> for Description {
    fn from(other: models::lifecycle::Description) -> Self {
        Self {
            name: other.name.to_string(),
            description: other.description.map(|s| s.to_string()),
        }
    }
}

impl From<Description> for models::lifecycle::Description {
    fn from(other: Description) -> Self {
        Self {
            name: NormalizedString::new_unchecked(other.name),
            description: other.description.map(NormalizedString::new_unchecked),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        models,
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::{Description, Lifecycle, Lifecycles, Phase};

    pub(crate) fn example_lifecycles() -> Lifecycles {
        Lifecycles(vec![Lifecycle::Phase(Phase::new("design"))])
    }

    pub(crate) fn corresponding_lifecycles() -> models::lifecycle::Lifecycles {
        models::lifecycle::Lifecycles(vec![models::lifecycle::Lifecycle::Phase(
            models::lifecycle::Phase::Design,
        )])
    }

    #[test]
    fn it_should_read_json_with_multiple_lifecycles() {
        let input = r#"[
              {
                "phase": "build"
              },
              {
                "phase": "post-build"
              },
              {
                "name": "platform-integration-testing",
                "description": "Integration testing specific to the runtime platform"
              }
            ]"#;
        let actual: Lifecycles = serde_json::from_str(input).expect("Failed to parse JSON");
        let expected = Lifecycles(vec![
            Lifecycle::Phase(Phase::new("build")),
            Lifecycle::Phase(Phase::new("post-build")),
            Lifecycle::Description(Description {
                name: "platform-integration-testing".to_string(),
                description: Some(
                    "Integration testing specific to the runtime platform".to_string(),
                ),
            }),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_xml_empty_lifecycles() {
        let input = r#"
<lifecycles>
</lifecycles>
"#;
        let actual: Lifecycles = read_element_from_string(input);
        let expected = Lifecycles(vec![]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_full_xml_with_multiple_entries() {
        let input = r#"
<lifecycles>
  <lifecycle>
    <phase>build</phase>
  </lifecycle>
  <lifecycle>
    <phase>post-build</phase>
  </lifecycle>
  <lifecycle>
    <name>platform-integration-testing</name>
    <description>Integration testing specific to the runtime platform</description>
  </lifecycle>
</lifecycles>
"#;
        let actual: Lifecycles = read_element_from_string(input);
        let expected = Lifecycles(vec![
            Lifecycle::Phase(Phase::new("build")),
            Lifecycle::Phase(Phase::new("post-build")),
            Lifecycle::Description(Description {
                name: "platform-integration-testing".to_string(),
                description: Some(
                    "Integration testing specific to the runtime platform".to_string(),
                ),
            }),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_write_xml_multiple_lifecycles() {
        let lifecycles = vec![
            Lifecycle::Phase(Phase("build".to_string())),
            Lifecycle::Description(Description {
                name: "pre-production".to_string(),
                description: Some("Getting things together".to_string()),
            }),
            Lifecycle::Phase(Phase("decommission".to_string())),
        ];

        let xml_output = write_element_to_string(Lifecycles(lifecycles));
        insta::assert_snapshot!(xml_output);
    }
}
