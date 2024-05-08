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

use cyclonedx_bom_macros::versioned;

#[versioned("1.3", "1.4", "1.5")]
pub(crate) mod base {
    use crate::{
        errors::XmlReadError,
        models,
        specs::common::bom_reference::BomReference,
        utilities::{convert_optional_vec, convert_vec},
        xml::{
            read_lax_validation_list_tag, read_simple_tag, to_xml_read_error, to_xml_write_error,
            unexpected_element_error, write_close_tag, write_simple_tag, write_start_tag, FromXml,
            ToInnerXml, ToXml,
        },
    };
    #[versioned("1.4", "1.5")]
    use crate::{specs::common::signature::Signature, utilities::convert_optional};
    use serde::{Deserialize, Serialize};
    use xml::reader;

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
            write_start_tag(writer, COMPOSITIONS_TAG)?;

            for composition in &self.0 {
                composition.write_xml_element(writer)?;
            }

            write_close_tag(writer, COMPOSITIONS_TAG)?;

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
            read_lax_validation_list_tag(event_reader, element_name, COMPOSITION_TAG)
                .map(Compositions)
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Composition {
        #[versioned("1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        bom_ref: Option<String>,
        aggregate: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        assemblies: Option<Vec<BomReference>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dependencies: Option<Vec<BomReference>>,
        #[versioned("1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        vulnerabilities: Option<Vec<BomReference>>,
        #[versioned("1.4", "1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<Signature>,
    }

    impl From<models::composition::Composition> for Composition {
        fn from(other: models::composition::Composition) -> Self {
            Self {
                #[versioned("1.5")]
                bom_ref: other.bom_ref.map(|b| b.0),
                aggregate: other.aggregate.to_string(),
                assemblies: convert_optional_vec(other.assemblies),
                dependencies: convert_optional_vec(other.dependencies),
                #[versioned("1.5")]
                vulnerabilities: convert_optional_vec(other.vulnerabilities),
                #[versioned("1.4", "1.5")]
                signature: convert_optional(other.signature),
            }
        }
    }

    impl From<Composition> for models::composition::Composition {
        fn from(other: Composition) -> Self {
            Self {
                #[versioned("1.3", "1.4")]
                bom_ref: None,
                #[versioned("1.5")]
                bom_ref: other.bom_ref.map(models::bom::BomReference),
                aggregate: models::composition::AggregateType::new_unchecked(other.aggregate),
                assemblies: convert_optional_vec(other.assemblies),
                dependencies: convert_optional_vec(other.dependencies),
                #[versioned("1.3", "1.4")]
                vulnerabilities: None,
                #[versioned("1.5")]
                vulnerabilities: convert_optional_vec(other.vulnerabilities),
                #[versioned("1.3")]
                signature: None,
                #[versioned("1.4", "1.5")]
                signature: convert_optional(other.signature),
            }
        }
    }

    #[versioned("1.5")]
    const BOM_REF_ATTR: &str = "bom-ref";
    const COMPOSITION_TAG: &str = "composition";
    const AGGREGATE_TAG: &str = "aggregate";
    const ASSEMBLIES_TAG: &str = "assemblies";
    const ASSEMBLY_TAG: &str = "assembly";
    const DEPENDENCIES_TAG: &str = "dependencies";
    const DEPENDENCY_TAG: &str = "dependency";
    #[versioned("1.5")]
    const VULNERABILITIES_TAG: &str = "vulnerabilities";
    #[versioned("1.5")]
    const VULNERABILITY_TAG: &str = "vulnerability";
    #[versioned("1.4", "1.5")]
    const SIGNATURE_TAG: &str = "signature";

    impl ToXml for Composition {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            #[versioned("1.3", "1.4")]
            let start_tag = xml::writer::XmlEvent::start_element(COMPOSITION_TAG);
            #[versioned("1.5")]
            let mut start_tag = xml::writer::XmlEvent::start_element(COMPOSITION_TAG);
            #[versioned("1.5")]
            if let Some(bom_ref) = &self.bom_ref {
                start_tag = start_tag.attr(BOM_REF_ATTR, bom_ref);
            }

            writer
                .write(start_tag)
                .map_err(to_xml_write_error(COMPOSITION_TAG))?;

            write_simple_tag(writer, AGGREGATE_TAG, &self.aggregate)?;

            if let Some(assemblies) = &self.assemblies {
                write_start_tag(writer, ASSEMBLIES_TAG)?;

                for assembly in assemblies {
                    assembly.write_xml_named_element(writer, ASSEMBLY_TAG)?;
                }

                write_close_tag(writer, ASSEMBLIES_TAG)?;
            }

            if let Some(dependencies) = &self.dependencies {
                write_start_tag(writer, DEPENDENCIES_TAG)?;

                for dependency in dependencies {
                    dependency.write_xml_named_element(writer, DEPENDENCY_TAG)?;
                }

                write_close_tag(writer, DEPENDENCIES_TAG)?;
            }

            #[versioned("1.5")]
            if let Some(vulnerabilities) = &self.vulnerabilities {
                write_start_tag(writer, VULNERABILITIES_TAG)?;

                for vulnerability in vulnerabilities {
                    vulnerability.write_xml_named_element(writer, VULNERABILITY_TAG)?;
                }

                write_close_tag(writer, VULNERABILITIES_TAG)?;
            }

            #[versioned("1.4", "1.5")]
            if let Some(signature) = &self.signature {
                signature.write_xml_element(writer)?;
            }

            write_close_tag(writer, COMPOSITION_TAG)?;

            Ok(())
        }
    }

    impl FromXml for Composition {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            #[allow(unused)] attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, crate::errors::XmlReadError>
        where
            Self: Sized,
        {
            #[versioned("1.5")]
            let bom_ref: Option<String> = crate::xml::optional_attribute(attributes, BOM_REF_ATTR);
            let mut aggregate: Option<String> = None;
            let mut assemblies: Option<Vec<BomReference>> = None;
            let mut dependencies: Option<Vec<BomReference>> = None;
            #[versioned("1.5")]
            let mut vulnerabilities: Option<Vec<BomReference>> = None;
            #[versioned("1.4", "1.5")]
            let mut signature: Option<Signature> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(COMPOSITION_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == AGGREGATE_TAG =>
                    {
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
                    #[versioned("1.5")]
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == VULNERABILITIES_TAG =>
                    {
                        vulnerabilities = Some(read_lax_validation_list_tag(
                            event_reader,
                            &name,
                            VULNERABILITY_TAG,
                        )?);
                    }
                    #[versioned("1.4", "1.5")]
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == SIGNATURE_TAG => {
                        signature = Some(Signature::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
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
                #[versioned("1.5")]
                bom_ref,
                aggregate,
                assemblies,
                dependencies,
                #[versioned("1.5")]
                vulnerabilities,
                #[versioned("1.4", "1.5")]
                signature,
            })
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;
        use pretty_assertions::assert_eq;

        #[versioned("1.4", "1.5")]
        use crate::specs::common::signature::test::{corresponding_signature, example_signature};
        use crate::xml::test::{read_element_from_string, write_element_to_string};

        pub(crate) fn example_compositions() -> Compositions {
            Compositions(vec![example_composition()])
        }

        pub(crate) fn corresponding_compositions() -> models::composition::Compositions {
            models::composition::Compositions(vec![corresponding_composition()])
        }

        pub(crate) fn example_composition() -> Composition {
            Composition {
                #[versioned("1.5")]
                bom_ref: Some("composition-ref".to_string()),
                aggregate: "aggregate".to_string(),
                assemblies: Some(vec![BomReference::new("assembly-ref")]),
                dependencies: Some(vec![BomReference::new("dependency-ref")]),
                #[versioned("1.5")]
                vulnerabilities: Some(vec![BomReference::new("vulnerability-ref")]),
                #[versioned("1.4", "1.5")]
                signature: Some(example_signature()),
            }
        }

        pub(crate) fn corresponding_composition() -> models::composition::Composition {
            models::composition::Composition {
                #[versioned("1.3", "1.4")]
                bom_ref: None,
                #[versioned("1.5")]
                bom_ref: Some(models::bom::BomReference::new("composition-ref")),
                aggregate: models::composition::AggregateType::UnknownAggregateType(
                    "aggregate".to_string(),
                ),
                assemblies: Some(vec![models::bom::BomReference::new("assembly-ref")]),
                dependencies: Some(vec![models::bom::BomReference::new("dependency-ref")]),
                #[versioned("1.3", "1.4")]
                vulnerabilities: None,
                #[versioned("1.5")]
                vulnerabilities: Some(vec![models::bom::BomReference::new("vulnerability-ref")]),
                #[versioned("1.3")]
                signature: None,
                #[versioned("1.4", "1.5")]
                signature: Some(corresponding_signature()),
            }
        }

        #[test]
        fn it_should_write_xml_full() {
            let xml_output = write_element_to_string(example_compositions());
            insta::assert_snapshot!(xml_output);
        }

        #[test]
        fn it_should_read_xml_full() {
            #[versioned("1.3")]
            let input = r#"
<compositions>
  <composition>
    <aggregate>aggregate</aggregate>
    <assemblies>
      <assembly ref="assembly-ref" />
    </assemblies>
    <dependencies>
      <dependency ref="dependency-ref" />
    </dependencies>
  </composition>
</compositions>
"#;
            #[versioned("1.4")]
            let input = r#"
<compositions>
  <composition>
    <aggregate>aggregate</aggregate>
    <assemblies>
      <assembly ref="assembly-ref" />
    </assemblies>
    <dependencies>
      <dependency ref="dependency-ref" />
    </dependencies>
    <signature>
      <algorithm>HS512</algorithm>
      <value>1234567890</value>
    </signature>
  </composition>
</compositions>
"#;
            #[versioned("1.5")]
            let input = r#"
<compositions>
  <composition bom-ref="composition-ref">
    <aggregate>aggregate</aggregate>
    <assemblies>
      <assembly ref="assembly-ref" />
    </assemblies>
    <dependencies>
      <dependency ref="dependency-ref" />
    </dependencies>
    <vulnerabilities>
      <vulnerability ref="vulnerability-ref" />
    </vulnerabilities>
    <signature>
      <algorithm>HS512</algorithm>
      <value>1234567890</value>
    </signature>
  </composition>
</compositions>
"#;
            let actual: Compositions = read_element_from_string(input);
            let expected = example_compositions();
            assert_eq!(actual, expected);
        }
    }
}
