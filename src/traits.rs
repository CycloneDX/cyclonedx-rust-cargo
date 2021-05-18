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

/// Check if `self` contains meaningful data.
pub trait IsEmpty {
    /// Check if `self` contains meaningful data.
    fn is_empty(&self) -> bool;
}

impl<'a> IsEmpty for &'a str {
    fn is_empty(&self) -> bool {
        !self.trim().is_empty()
    }
}

impl IsEmpty for String {
    fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }
}

impl<T: IsEmpty> IsEmpty for Option<T> {
    fn is_empty(&self) -> bool {
        if let Some(v) = self {
            v.is_empty()
        } else {
            true
        }
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::<T>::is_empty(self)
    }
}
