use std::io;

use cargo::core::Package;
use packageurl::PackageUrl;
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::traits::ToXml;

mod license;
mod reference;

use self::license::Licenses;
use self::reference::ExternalReferences;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Component<'a> {
    #[serde(flatten)]
    metadata: Metadata<'a>,
    #[serde(skip_serializing_if = "Licenses::is_empty")]
    licenses: Licenses<'a>,
    #[serde(skip_serializing_if = "ExternalReferences::is_empty")]
    external_references: ExternalReferences<'a>,
}

impl<'a> From<&'a Package> for Component<'a> {
    fn from(pkg: &'a Package) -> Self {
        Self {
            metadata: Metadata::from(pkg),
            licenses: Licenses::from(pkg),
            external_references: ExternalReferences::from(pkg),
        }
    }
}

impl ToXml for Component<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("component")?;
        xml.attr("type", "library")?;

        self.metadata.to_xml(xml)?;

        xml.begin_elem("scope")?;
        xml.text("required")?;
        xml.end_elem()?;

        //TODO: Add hashes. May require file components and manual calculation of all files

        self.licenses.to_xml(xml)?;
        self.external_references.to_xml(xml)?;

        xml.end_elem()
    }
}

#[derive(Serialize)]
struct Metadata<'a> {
    name: &'a str,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    purl: String,
}

impl<'a> From<&'a Package> for Metadata<'a> {
    fn from(package: &'a Package) -> Self {
        let name = package.name().to_owned().as_str().trim();
        let version = package.version().to_string();

        Self {
            name,
            purl: PackageUrl::new("cargo", name)
                .with_version(version.trim())
                .to_string(),
            version,
            description: package
                .manifest()
                .metadata()
                .description
                .as_ref()
                .map(|s| s.as_str()),
        }
    }
}

impl ToXml for Metadata<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("name")?;
        xml.text(self.name)?;
        xml.end_elem()?;

        xml.begin_elem("version")?;
        xml.text(self.version.trim())?;
        xml.end_elem()?;

        if let Some(x) = self.description {
            xml.begin_elem("description")?;
            xml.cdata(x.trim())?;
            xml.end_elem()?;
        }

        xml.begin_elem("purl")?;
        xml.text(&self.purl.to_string())?;
        xml.end_elem()?;

        Ok(())
    }
}
