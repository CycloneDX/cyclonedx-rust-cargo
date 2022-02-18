use crate::errors::XmlWriteError;
use std::io::Write;
use xml::writer::EventWriter;

pub(crate) trait ToXmlDocument {
    fn write_xml_document<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), XmlWriteError>;
}

pub(crate) trait ToXml {
    fn write_xml_element<W: Write>(&self, writer: &mut EventWriter<W>)
        -> Result<(), XmlWriteError>;

    fn will_write(&self) -> bool {
        true
    }
}

// TODO: is there a name for this pattern
pub(crate) trait ToInnerXml {
    fn write_xml_named_element<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        tag: &str,
    ) -> Result<(), XmlWriteError>;

    fn will_write(&self) -> bool {
        true
    }
}

pub(crate) fn to_xml_write_error(
    element: impl AsRef<str>,
) -> impl FnOnce(xml::writer::Error) -> XmlWriteError {
    let element = element.as_ref().to_owned();
    |error| XmlWriteError::XmlElementWriteError { error, element }
}

#[cfg(test)]
pub(crate) mod test {
    use xml::EmitterConfig;

    use super::*;

    fn emitter_config() -> EmitterConfig {
        EmitterConfig::default().perform_indent(true)
    }

    pub(crate) fn write_element_to_string<X: ToXml>(element: X) -> String {
        let mut output = Vec::new();
        let mut event_writer = EventWriter::new_with_config(&mut output, emitter_config());
        element
            .write_xml_element(&mut event_writer)
            .expect("Should have written the element");
        String::from_utf8_lossy(&output).to_string()
    }

    pub(crate) fn write_named_element_to_string<X: ToInnerXml>(element: X, tag: &str) -> String {
        let mut output = Vec::new();
        let mut event_writer = EventWriter::new_with_config(&mut output, emitter_config());
        element
            .write_xml_named_element(&mut event_writer, tag)
            .expect("Should have written the element");
        String::from_utf8_lossy(&output).to_string()
    }
}
