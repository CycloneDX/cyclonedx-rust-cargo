use serde::{Deserialize, Serialize};
use xml::{reader, writer};

use crate::{
    specs::common::bom_reference::BomReference,
    xml::{
        attribute_or_error, read_lax_validation_tag, to_xml_read_error, to_xml_write_error,
        unexpected_element_error, write_close_tag, FromXml, ToInnerXml, ToXml,
    },
};

#[derive(Serialize, Deserialize)]
struct Dependency {
    r#ref: String,
    depends_on: Vec<BomReference>,
}

const DEPENDENCY_TAG: &str = "dependency";
const REF_ATTR: &str = "ref";

impl FromXml for Dependency {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let r#ref = attribute_or_error(element_name, attributes, REF_ATTR)?;

        let mut depends_on = vec![];

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(DEPENDENCY_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == DEPENDENCY_TAG => {
                    depends_on.push(BomReference::read_xml_element(
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

        Ok(Self { r#ref, depends_on })
    }
}

impl ToXml for Dependency {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let dependency_start_tag =
            writer::XmlEvent::start_element(DEPENDENCY_TAG).attr(REF_ATTR, &self.r#ref);
        writer
            .write(dependency_start_tag)
            .map_err(to_xml_write_error(DEPENDENCY_TAG))?;

        for dep in &self.depends_on {
            dep.write_xml_named_element(writer, DEPENDENCY_TAG)?;
        }

        write_close_tag(writer, DEPENDENCY_TAG)?;

        Ok(())
    }
}
