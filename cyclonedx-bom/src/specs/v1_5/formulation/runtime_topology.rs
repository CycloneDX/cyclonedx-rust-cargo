use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    specs::common::dependency::Dependency,
    xml::{
        read_lax_validation_tag, to_xml_read_error, unexpected_element_error, write_close_tag,
        write_start_tag, FromXml, ToXml,
    },
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct RuntimeTopology(Vec<Dependency>);

impl RuntimeTopology {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

const RUNTIME_TOPOLOGY_TAG: &str = "runtimeTopology";
const DEPENDENCY_TAG: &str = "dependency";

impl FromXml for RuntimeTopology {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut dependencies = vec![];

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(DEPENDENCY_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == DEPENDENCY_TAG => {
                    dependencies.push(Dependency::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
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

        Ok(Self(dependencies))
    }
}

impl ToXml for RuntimeTopology {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, RUNTIME_TOPOLOGY_TAG)?;

        for dependency in &self.0 {
            dependency.write_xml_element(writer)?;
        }

        write_close_tag(writer, RUNTIME_TOPOLOGY_TAG)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;

    fn example_runtime_topology() -> RuntimeTopology {
        RuntimeTopology(vec![Dependency {
            dependency_ref: "component-1".into(),
            depends_on: vec!["component-2".into()],
        }])
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_runtime_topology());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<runtimeTopology>
    <dependency ref="component-1">
        <dependency ref="component-2" />
    </dependency>
</runtimeTopology>
"#;
        let actual: RuntimeTopology = read_element_from_string(input);
        let expected = example_runtime_topology();
        assert_eq!(actual, expected);
    }
}
