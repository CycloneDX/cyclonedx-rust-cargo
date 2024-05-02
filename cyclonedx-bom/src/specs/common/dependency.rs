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
    errors::{XmlReadError, XmlWriteError},
    models,
    xml::{
        attribute_or_error, closing_tag_or_error, read_list_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_close_tag, write_start_tag, FromXml,
        ToXml,
    },
};
use serde::{Deserialize, Serialize};
use xml::{reader, writer::XmlEvent};

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub(crate) struct Dependencies(Vec<Dependency>);

impl From<models::dependency::Dependencies> for Dependencies {
    fn from(other: models::dependency::Dependencies) -> Self {
        Self(other.0.into_iter().map(std::convert::Into::into).collect())
    }
}

impl From<Dependencies> for models::dependency::Dependencies {
    fn from(other: Dependencies) -> Self {
        Self(other.0.into_iter().map(std::convert::Into::into).collect())
    }
}

const DEPENDENCIES_TAG: &str = "dependencies";

impl ToXml for Dependencies {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        write_start_tag(writer, DEPENDENCIES_TAG)?;

        for dependency in &self.0 {
            dependency.write_xml_element(writer)?;
        }

        write_close_tag(writer, DEPENDENCIES_TAG)?;

        Ok(())
    }
}

impl FromXml for Dependencies {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, DEPENDENCY_TAG).map(Dependencies)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Dependency {
    #[serde(rename = "ref")]
    pub(crate) dependency_ref: String,
    #[serde(default)]
    pub(crate) depends_on: Vec<String>,
}

impl From<Dependency> for models::dependency::Dependency {
    fn from(other: Dependency) -> Self {
        Self {
            dependency_ref: other.dependency_ref,
            dependencies: other.depends_on,
        }
    }
}

impl From<models::dependency::Dependency> for Dependency {
    fn from(other: models::dependency::Dependency) -> Self {
        Self {
            dependency_ref: other.dependency_ref,
            depends_on: other.dependencies,
        }
    }
}

const DEPENDENCY_TAG: &str = "dependency";
const REF_ATTR: &str = "ref";

impl ToXml for Dependency {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        writer
            .write(XmlEvent::start_element(DEPENDENCY_TAG).attr(REF_ATTR, &self.dependency_ref))
            .map_err(to_xml_write_error(DEPENDENCY_TAG))?;

        for dependency in &self.depends_on {
            writer
                .write(XmlEvent::start_element(DEPENDENCY_TAG).attr(REF_ATTR, dependency))
                .map_err(to_xml_write_error(DEPENDENCY_TAG))?;

            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(DEPENDENCY_TAG))?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(DEPENDENCY_TAG))?;

        Ok(())
    }
}

impl FromXml for Dependency {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let dependency_ref = attribute_or_error(element_name, attributes, REF_ATTR)?;
        let mut depends_on: Vec<String> = Vec::new();

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(DEPENDENCY_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == DEPENDENCY_TAG => {
                    let dep_ref = attribute_or_error(&name, &attributes, REF_ATTR)?;
                    event_reader
                        .next()
                        .map_err(to_xml_read_error(DEPENDENCY_TAG))
                        .and_then(closing_tag_or_error(&name))?;
                    depends_on.push(dep_ref);
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            dependency_ref,
            depends_on,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    pub(crate) fn example_dependencies() -> Dependencies {
        Dependencies(vec![Dependency {
            dependency_ref: "ref".to_string(),
            depends_on: vec!["depends on".to_string()],
        }])
    }

    pub(crate) fn corresponding_dependencies() -> models::dependency::Dependencies {
        models::dependency::Dependencies(vec![models::dependency::Dependency {
            dependency_ref: "ref".to_string(),
            dependencies: vec!["depends on".to_string()],
        }])
    }

    #[test]
    fn it_flattens_dependencies() {
        let actual: Dependencies =
            models::dependency::Dependencies(vec![models::dependency::Dependency {
                dependency_ref: "a".to_string(),
                dependencies: vec!["b".to_string(), "c".to_string()],
            }])
            .into();
        let expected = Dependencies(vec![Dependency {
            dependency_ref: "a".to_string(),
            depends_on: vec!["b".to_string(), "c".to_string()],
        }]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_dependencies());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_write_xml_empty_dependencies() {
        let xml_output = write_element_to_string(Dependencies(Vec::new()));
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_write_xml_dependencies_with_no_children() {
        let xml_output = write_element_to_string(Dependencies(vec![Dependency {
            dependency_ref: "dependency".to_string(),
            depends_on: Vec::new(),
        }]));
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<dependencies>
  <dependency ref="ref">
    <dependency ref="depends on" />
  </dependency>
</dependencies>
"#;
        let actual: Dependencies = read_element_from_string(input);
        let expected = example_dependencies();
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_xml_empty_dependencies() {
        let input = r#"
<dependencies/>
"#;
        let actual: Dependencies = read_element_from_string(input);
        let expected = Dependencies(Vec::new());
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_xml_dependencies_with_no_children() {
        let input = r#"
<dependencies>
  <dependency ref="dependency" />
</dependencies>
"#;
        let actual: Dependencies = read_element_from_string(input);
        let expected = Dependencies(vec![Dependency {
            dependency_ref: "dependency".to_string(),
            depends_on: Vec::new(),
        }]);
        assert_eq!(actual, expected);
    }
}
