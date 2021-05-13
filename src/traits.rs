use std::io::{self, Write};

use xml_writer::XmlWriter;

/// Write a CycloneDX XML representation of `self`.
pub trait ToXml {
    /// Write a CycloneDX XML representation of `self`.
    ///
    /// # Requirements
    /// * If `to_xml` returns `Ok`, then `xml` must have the same tag depth as it did when the
    ///   function was invoked.
    fn to_xml<W: Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()>;
}
