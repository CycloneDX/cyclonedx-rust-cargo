use std::io;

use cargo::core::Package;
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::traits::ToXml;

#[derive(Serialize)]
pub struct Licenses<'a>(Vec<License<'a>>);

impl<'a> Licenses<'a> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> From<&'a Package> for Licenses<'a> {
    fn from(pkg: &'a Package) -> Self {
        Self(
            pkg.manifest()
                .metadata()
                .license
                .as_ref()
                .map(|s| License::Expression(s.as_str()))
                .into_iter()
                .collect(),
        )
    }
}

impl ToXml for Licenses<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        if !self.0.is_empty() {
            xml.begin_elem("licenses")?;

            for license in &self.0 {
                license.to_xml(xml)?;
            }

            xml.end_elem()?;
        }

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum License<'a> {
    Expression(&'a str),
}

impl ToXml for License<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("license")?;
        match self {
            Self::Expression(expr) => {
                xml.begin_elem("expression")?;
                xml.text(expr.trim())?;
                xml.end_elem()?;
            }
        }
        xml.end_elem()
    }
}
