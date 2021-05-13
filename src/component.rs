use std::io;

use cargo::core::Package;
use packageurl::PackageUrl;
use regex::Regex;
use xml_writer::XmlWriter;

use crate::traits::ToXml;

pub struct Component<'a>(&'a Package);

impl<'a> From<&'a Package> for Component<'a> {
    fn from(pkg: &'a Package) -> Self {
        Self(pkg)
    }
}

impl ToXml for Component<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        let package = self.0;
        let name = package.name().to_owned().as_str().trim();
        let version = package.version().to_string();
        xml.begin_elem("component")?;
        xml.attr("type", "library")?;

        xml.begin_elem("name")?;
        xml.text(name)?;
        xml.end_elem()?;

        xml.begin_elem("version")?;
        xml.text(version.trim())?;
        xml.end_elem()?;

        if let Some(x) = &package.manifest().metadata().description {
            xml.begin_elem("description")?;
            xml.cdata(x.trim())?;
            xml.end_elem()?;
        }

        xml.begin_elem("scope")?;
        xml.text("required")?;
        xml.end_elem()?;

        //TODO: Add hashes. May require file components and manual calculation of all files

        if let Some(x) = &package.manifest().metadata().license {
            xml.begin_elem("licenses")?;
            xml.begin_elem("license")?;
            xml.begin_elem("expression")?;
            xml.text(x.trim())?;
            xml.end_elem()?;
            xml.end_elem()?;
            xml.end_elem()?;
        }

        let purl = PackageUrl::new("cargo", name)
            .with_version(version.trim())
            .to_string();
        xml.begin_elem("purl")?;
        xml.text(&purl)?;
        xml.end_elem()?;

        ExternalReferences::from(self.0).to_xml(xml)?;

        xml.end_elem()
    }
}

struct ExternalReferences<'a>(Vec<ExternalReference<'a>>);

impl<'a> From<&'a Package> for ExternalReferences<'a> {
    fn from(v: &'a Package) -> Self {
        fn ext_ref<'a>(
            ref_type: &'a str,
            uri: &'a Option<String>,
        ) -> Option<ExternalReference<'a>> {
            ExternalReference::new(ref_type, uri.as_ref()?).ok()
        }

        let metadata = v.manifest().metadata();
        Self(
            ext_ref("documentation", &metadata.documentation)
                .into_iter()
                .chain(ext_ref("website", &metadata.homepage))
                .chain(ext_ref("other", &metadata.links))
                .chain(ext_ref("vcs", &metadata.repository))
                .collect(),
        )
    }
}

impl ToXml for ExternalReferences<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        if !self.0.is_empty() {
            xml.begin_elem("externalReferences")?;
            for reference in &self.0 {
                reference.to_xml(xml)?;
            }
            xml.end_elem()?;
        }

        Ok(())
    }
}

struct ExternalReferenceError;

/// A reference to external materials, such as documentation.
struct ExternalReference<'a> {
    ref_type: &'a str,
    uri: &'a str,
}

impl<'a> ExternalReference<'a> {
    fn new(ref_type: &'a str, uri: &'a str) -> Result<Self, ExternalReferenceError> {
        let re = Regex::new(r"^([a-z0-9+.-]+):(?://(?:((?:[a-z0-9-._~!$&'()*+,;=:]|%[0-9A-F]{2})*)@)?((?:[a-z0-9-._~!$&'()*+,;=]|%[0-9A-F]{2})*)(?::(\d*))?(/(?:[a-z0-9-._~!$&'()*+,;=:@/]|%[0-9A-F]{2})*)?|(/?(?:[a-z0-9-._~!$&'()*+,;=:@]|%[0-9A-F]{2})+(?:[a-z0-9-._~!$&'()*+,;=:@/]|%[0-9A-F]{2})*)?)(?:\?((?:[a-z0-9-._~!$&'()*+,;=:/?@]|%[0-9A-F]{2})*))?(?:#((?:[a-z0-9-._~!$&'()*+,;=:/?@]|%[0-9A-F]{2})*))?$").unwrap();
        if re.is_match(uri) {
            Ok(Self { ref_type, uri })
        } else {
            Err(ExternalReferenceError)
        }
    }
}

impl ToXml for ExternalReference<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("reference")?;
        xml.attr("type", self.ref_type)?;
        // XXX is this trim() needed? The regex doesn't permit leading or trailing whitespace.
        xml.text(self.uri.trim())?;
        xml.end_elem()
    }
}
