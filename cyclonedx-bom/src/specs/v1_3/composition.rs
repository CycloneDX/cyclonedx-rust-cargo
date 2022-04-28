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
    utilities::{convert_optional_vec, convert_vec},
    xml::{
        attribute_or_error, closing_tag_or_error, read_lax_validation_list_tag, read_simple_tag,
        to_xml_read_error, to_xml_write_error, unexpected_element_error, write_simple_tag, FromXml,
        ToInnerXml, ToXml,
    },
};
use serde::{Deserialize, Serialize};
use xml::{reader, writer::XmlEvent};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Compositions(Vec<Composition>);

impl From<models::composition::Compositions> for Compositions {
    fn from(other: models::composition::Compositions) -> Self {
        Compositions(convert_vec(other.0))
    }
}

impl From<Compositions> for models::composition::Compositions {
    fn from(other: Compositions) -> Self {
        models::composition::Compositions(convert_vec(other.0))
    }
}

const COMPOSITIONS_TAG: &str = "compositions";

impl ToXml for Compositions {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(COMPOSITIONS_TAG))
            .map_err(to_xml_write_error(COMPOSITIONS_TAG))?;

        for composition in &self.0 {
            composition.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(COMPOSITIONS_TAG))?;
        Ok(())
    }
}

impl FromXml for Compositions {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        read_lax_validation_list_tag(event_reader, element_name, COMPOSITION_TAG).map(Compositions)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Composition {
    aggregate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    assemblies: Option<Vec<BomReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Vec<BomReference>>,
}

impl From<models::composition::Composition> for Composition {
    fn from(other: models::composition::Composition) -> Self {
        Self {
            aggregate: other.aggregate.to_string(),
            assemblies: convert_optional_vec(other.assemblies),
            dependencies: convert_optional_vec(other.dependencies),
        }
    }
}

impl From<Composition> for models::composition::Composition {
    fn from(other: Composition) -> Self {
        Self {
            aggregate: models::composition::AggregateType::new_unchecked(other.aggregate),
            assemblies: convert_optional_vec(other.assemblies),
            dependencies: convert_optional_vec(other.dependencies),
        }
    }
}

const COMPOSITION_TAG: &str = "composition";
const AGGREGATE_TAG: &str = "aggregate";
const ASSEMBLIES_TAG: &str = "assemblies";
const ASSEMBLY_TAG: &str = "assembly";
const DEPENDENCIES_TAG: &str = "dependencies";
const DEPENDENCY_TAG: &str = "dependency";

impl ToXml for Composition {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(COMPOSITION_TAG))
            .map_err(to_xml_write_error(COMPOSITION_TAG))?;

        write_simple_tag(writer, AGGREGATE_TAG, &self.aggregate)?;

        if let Some(assemblies) = &self.assemblies {
            writer
                .write(XmlEvent::start_element(ASSEMBLIES_TAG))
                .map_err(to_xml_write_error(ASSEMBLIES_TAG))?;

            for assembly in assemblies {
                assembly.write_xml_named_element(writer, ASSEMBLY_TAG)?;
            }

            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(ASSEMBLIES_TAG))?;
        }

        if let Some(dependencies) = &self.dependencies {
            writer
                .write(XmlEvent::start_element(DEPENDENCIES_TAG))
                .map_err(to_xml_write_error(DEPENDENCIES_TAG))?;

            for dependency in dependencies {
                dependency.write_xml_named_element(writer, DEPENDENCY_TAG)?;
            }

            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(DEPENDENCIES_TAG))?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(COMPOSITION_TAG))?;

        Ok(())
    }
}

impl FromXml for Composition {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut aggregate: Option<String> = None;
        let mut assemblies: Option<Vec<BomReference>> = None;
        let mut dependencies: Option<Vec<BomReference>> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(COMPOSITION_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == AGGREGATE_TAG => {
                    aggregate = Some(read_simple_tag(event_reader, &name)?);
                }
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == ASSEMBLIES_TAG =>
                {
                    assemblies = Some(read_lax_validation_list_tag(
                        event_reader,
                        &name,
                        ASSEMBLY_TAG,
                    )?)
                }
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == DEPENDENCIES_TAG =>
                {
                    dependencies = Some(read_lax_validation_list_tag(
                        event_reader,
                        &name,
                        DEPENDENCY_TAG,
                    )?)
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let aggregate = aggregate.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: AGGREGATE_TAG.to_string(),
            element: COMPOSITION_TAG.to_string(),
        })?;

        Ok(Self {
            aggregate,
            assemblies,
            dependencies,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct BomReference(String);

impl From<models::composition::BomReference> for BomReference {
    fn from(other: models::composition::BomReference) -> Self {
        Self(other.0)
    }
}

impl From<BomReference> for models::composition::BomReference {
    fn from(other: BomReference) -> Self {
        Self(other.0)
    }
}

const REF_ATTR: &str = "ref";

impl ToInnerXml for BomReference {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(tag).attr(REF_ATTR, &self.0))
            .map_err(to_xml_write_error(tag))?;

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(tag))?;

        Ok(())
    }
}

impl FromXml for BomReference {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let reference = attribute_or_error(element_name, attributes, REF_ATTR)?;
        event_reader
            .next()
            .map_err(to_xml_read_error(&element_name.local_name))
            .and_then(closing_tag_or_error(element_name))?;

        Ok(Self(reference))
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;

    pub(crate) fn example_compositions() -> Compositions {
        Compositions(vec![example_composition()])
    }

    pub(crate) fn corresponding_compositions() -> models::composition::Compositions {
        models::composition::Compositions(vec![corresponding_composition()])
    }

    pub(crate) fn example_composition() -> Composition {
        Composition {
            aggregate: "aggregate".to_string(),
            assemblies: Some(vec![BomReference("assembly".to_string())]),
            dependencies: Some(vec![BomReference("dependency".to_string())]),
        }
    }

    pub(crate) fn corresponding_composition() -> models::composition::Composition {
        models::composition::Composition {
            aggregate: models::composition::AggregateType::UnknownAggregateType(
                "aggregate".to_string(),
            ),
            assemblies: Some(vec![models::composition::BomReference(
                "assembly".to_string(),
            )]),
            dependencies: Some(vec![models::composition::BomReference(
                "dependency".to_string(),
            )]),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_compositions());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<compositions>
  <composition>
    <aggregate>aggregate</aggregate>
    <assemblies>
      <assembly ref="assembly" />
    </assemblies>
    <dependencies>
      <dependency ref="dependency" />
    </dependencies>
  </composition>
</compositions>
"#;
        let actual: Compositions = read_element_from_string(input);
        let expected = example_compositions();
        assert_eq!(actual, expected);
    }
}
