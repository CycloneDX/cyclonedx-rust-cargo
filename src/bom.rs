use std::{io, iter::FromIterator};

use cargo::core::Package;
use serde::{Serialize, Serializer};
use uuid::Uuid;
use xml_writer::XmlWriter;

use crate::{Component, ToXml};

mod metadata;

pub use self::metadata::Metadata;

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
    metadata: Metadata<'a>,
    components: Vec<Component<'a>>,
}

impl<'a> Bom<'a> {
    /// Create a BOM for a specific package.
    pub fn new(pkg: &'a Package) -> Self {
        Self {
            metadata: Metadata::from(pkg),
            ..Default::default()
        }
    }

    /// Amend the BOM with the specified crates as `components` entries.
    pub fn with_dependencies(mut self, packages: impl IntoIterator<Item = &'a Package>) -> Self {
        self.components
            .extend(packages.into_iter().map(Component::from));
        self
    }
}

impl<'a> Default for Bom<'a> {
    fn default() -> Self {
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

        self.metadata.to_xml(xml)?;

        xml.begin_elem("components")?;
        for component in &self.components {
            component.to_xml(xml)?;
        }
        xml.end_elem()?;

        xml.end_elem()
    }
}
