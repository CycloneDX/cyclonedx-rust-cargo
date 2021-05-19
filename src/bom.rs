use std::{io, iter::FromIterator};

use cargo::core::Package;
use serde::{Serialize, Serializer};
use uuid::Uuid;
use xml_writer::XmlWriter;

use crate::{Component, ToXml};

#[derive(Clone, Copy, Serialize)]
enum BomFormat {
    CycloneDX,
}

fn uuid_to_urn<S: Serializer>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&uuid.to_urn())
}

/// A software bill of materials for a Rust crate.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bom<'a> {
    bom_format: BomFormat,
    spec_version: &'static str,
    #[serde(serialize_with = "uuid_to_urn")]
    serial_number: Uuid,
    version: u32,
    components: Vec<Component<'a>>,
}

/// Create a new BOM from a sequence of cargo package references.
impl<'a> FromIterator<&'a Package> for Bom<'a> {
    fn from_iter<T: IntoIterator<Item = &'a Package>>(iter: T) -> Self {
        Self {
            bom_format: BomFormat::CycloneDX,
            spec_version: "1.3",
            version: 1,
            serial_number: Uuid::new_v4(),
            components: iter.into_iter().map(Component::library).collect(),
        }
    }
}

impl ToXml for Bom<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.dtd("UTF-8")?;
        xml.begin_elem("bom")?;
        xml.attr("serialNumber", &self.serial_number.to_urn().to_string())?;
        xml.attr("version", "1")?;
        xml.attr("xmlns", "http://cyclonedx.org/schema/bom/1.1")?;

        xml.begin_elem("components")?;
        for component in &self.components {
            component.to_xml(xml)?;
        }
        xml.end_elem()?;

        xml.end_elem()
    }
}
