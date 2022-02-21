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
    models,
    utilities::{convert_optional_vec, convert_vec},
    xml::{to_xml_write_error, write_simple_tag, ToInnerXml, ToXml},
};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Compositions(Vec<Composition>);

impl From<models::Compositions> for Compositions {
    fn from(other: models::Compositions) -> Self {
        Compositions(convert_vec(other.0))
    }
}

impl From<Compositions> for models::Compositions {
    fn from(other: Compositions) -> Self {
        models::Compositions(convert_vec(other.0))
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Composition {
    aggregate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    assemblies: Option<Vec<BomReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Vec<BomReference>>,
}

impl From<models::Composition> for Composition {
    fn from(other: models::Composition) -> Self {
        Self {
            aggregate: other.aggregate.to_string(),
            assemblies: convert_optional_vec(other.assemblies),
            dependencies: convert_optional_vec(other.dependencies),
        }
    }
}

impl From<Composition> for models::Composition {
    fn from(other: Composition) -> Self {
        Self {
            aggregate: models::AggregateType::new_unchecked(other.aggregate),
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct BomReference(String);

impl From<models::BomReference> for BomReference {
    fn from(other: models::BomReference) -> Self {
        Self(other.0)
    }
}

impl From<BomReference> for models::BomReference {
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

#[cfg(test)]
pub(crate) mod test {
    use crate::xml::test::write_element_to_string;

    use super::*;

    pub(crate) fn example_compositions() -> Compositions {
        Compositions(vec![example_composition()])
    }

    pub(crate) fn corresponding_compositions() -> models::Compositions {
        models::Compositions(vec![corresponding_composition()])
    }

    pub(crate) fn example_composition() -> Composition {
        Composition {
            aggregate: "aggregate".to_string(),
            assemblies: Some(vec![BomReference("assembly".to_string())]),
            dependencies: Some(vec![BomReference("dependency".to_string())]),
        }
    }

    pub(crate) fn corresponding_composition() -> models::Composition {
        models::Composition {
            aggregate: models::AggregateType::UnknownAggregateType("aggregate".to_string()),
            assemblies: Some(vec![models::BomReference("assembly".to_string())]),
            dependencies: Some(vec![models::BomReference("dependency".to_string())]),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_compositions());
        insta::assert_snapshot!(xml_output);
    }
}
