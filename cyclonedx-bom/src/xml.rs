use crate::errors::{XmlReadError, XmlWriteError};
use std::io::{Read, Write};
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    namespace::{Namespace, NS_NO_PREFIX},
    reader::{self},
    writer::{self, EventWriter, XmlEvent},
    EventReader,
};

pub(crate) trait ToXml {
    fn write_xml_element<W: Write>(&self, writer: &mut EventWriter<W>)
        -> Result<(), XmlWriteError>;

    fn will_write(&self) -> bool {
        true
    }
}

impl<T: ToXml> ToXml for Option<T> {
    fn write_xml_element<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), XmlWriteError> {
        if let Some(item) = self {
            item.write_xml_element(writer)?;
        }

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.is_some()
    }
}

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

impl<T: ToInnerXml> ToInnerXml for Option<T> {
    fn write_xml_named_element<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
        tag: &str,
    ) -> Result<(), XmlWriteError> {
        if let Some(item) = self {
            item.write_xml_named_element(writer, tag)?;
        }

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.is_some()
    }
}

/// Write a tag that is of the form `<tag>content</tag>`
pub(crate) fn write_simple_tag<W: Write>(
    writer: &mut EventWriter<W>,
    tag: &str,
    content: &str,
) -> Result<(), XmlWriteError> {
    writer
        .write(writer::XmlEvent::start_element(tag))
        .map_err(to_xml_write_error(tag))?;

    writer
        .write(writer::XmlEvent::characters(content))
        .map_err(to_xml_write_error(tag))?;

    writer
        .write(writer::XmlEvent::end_element())
        .map_err(to_xml_write_error(tag))?;
    Ok(())
}

pub(crate) fn write_simple_option_tag<W: Write>(
    writer: &mut EventWriter<W>,
    tag: &str,
    content: &Option<impl AsRef<str>>,
) -> Result<(), XmlWriteError> {
    if let Some(content) = content.as_ref() {
        write_simple_tag(writer, tag, content.as_ref())?
    }

    Ok(())
}

/// Writes a simple start tag of the form `<tag>` without attributes.
pub(crate) fn write_start_tag<W: Write>(
    writer: &mut EventWriter<W>,
    tag: &str,
) -> Result<(), XmlWriteError> {
    writer
        .write(XmlEvent::start_element(tag))
        .map_err(to_xml_write_error(tag))
}

/// Writes the closing tag of the form `</tag>`
pub(crate) fn write_close_tag<W: Write>(
    writer: &mut EventWriter<W>,
    tag: &str,
) -> Result<(), XmlWriteError> {
    writer
        .write(XmlEvent::end_element())
        .map_err(to_xml_write_error(tag))
}

pub(crate) fn write_list_tag<W: Write>(
    writer: &mut EventWriter<W>,
    tag: &str,
    list: &[impl ToXml],
) -> Result<(), XmlWriteError> {
    write_start_tag(writer, tag)?;

    for item in list {
        item.write_xml_element(writer)?;
    }

    write_close_tag(writer, tag)
}

pub(crate) fn write_list_string_tag<W: Write>(
    writer: &mut EventWriter<W>,
    tag: &str,
    child_tag: &str,
    list: &[impl AsRef<str>],
) -> Result<(), XmlWriteError> {
    write_start_tag(writer, tag)?;

    for item in list {
        write_simple_tag(writer, child_tag, item.as_ref())?;
    }

    write_close_tag(writer, tag)
}

pub(crate) fn to_xml_write_error(
    element: impl AsRef<str>,
) -> impl FnOnce(xml::writer::Error) -> XmlWriteError {
    let element = element.as_ref().to_owned();
    |error| XmlWriteError::XmlElementWriteError { error, element }
}

pub(crate) trait FromXmlDocument {
    fn read_xml_document<R: Read>(event_reader: &mut EventReader<R>) -> Result<Self, XmlReadError>
    where
        Self: Sized;
}

pub(crate) trait FromXml {
    fn read_xml_element<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        attributes: &[OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized;
}

#[macro_export]
macro_rules! get_elements_lax {
    ($event_reader:ident, $element_name: ident, $($tag:pat => $name:ident: $type:ty,)+) => {
            $(let mut $name: Option<$type> = None;)*

            let mut got_end_tag = false;

            while !got_end_tag {
                let next_element = $event_reader.next().map_err($crate::xml::to_xml_read_error(&$element_name.local_name))?;
                match next_element {
                    xml::reader::XmlEvent::StartElement {
                        name: ref elem_name,
                        ref attributes,
                        ..
                    } => {
                        match elem_name.local_name.as_str() {
                            $($tag => {
                                $name = Some(<$type as $crate::xml::FromXml>::read_xml_element(
                                    $event_reader,
                                    &elem_name,
                                    &attributes,
                                )?);
                            },)*
                            _ => $crate::xml::read_lax_validation_tag($event_reader, &elem_name)?,
                        }
                    }
                    xml::reader::XmlEvent::EndElement { name } if &name == $element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err($crate::xml::unexpected_element_error($element_name, unexpected)),
                }
            }
    };
}

#[macro_export]
macro_rules! get_elements {
    ($event_reader:ident, $element_name: ident, $($tag:pat => $name:ident: $type:ty,)+) => {
        $(let mut $name: Option<$type> = None;)*

        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = $event_reader.next().map_err($crate::xml::to_xml_read_error(&$element_name.local_name))?;
            match next_element {
                xml::reader::XmlEvent::StartElement {
                    name: ref elem_name,
                    ref attributes,
                    ..
                } => {
                    match elem_name.local_name.as_str() {
                        $($tag => {
                            $name = Some(<$type as $crate::xml::FromXml>::read_xml_element(
                                $event_reader,
                                &elem_name,
                                &attributes,
                            )?);
                        },)*
                        unexpected => return Err($crate::xml::unexpected_element_error(unexpected.to_string(), next_element)),
                    }
                }
                xml::reader::XmlEvent::EndElement { name } if &name == $element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err($crate::xml::unexpected_element_error($element_name, unexpected)),
            }
        }
    };
}

/// Helper trait that represents the inner tag of a sequence of elements.
pub(crate) trait VecElemTag {
    const VALUE: &'static str;
}

#[macro_export]
macro_rules! elem_tag {
    ($name:ident = $value:literal) => {
        struct $name {}

        impl $crate::xml::VecElemTag for $name {
            const VALUE: &'static str = $value;
        }
    };
}

/// Helper type to deserialize sequences of elements
pub(crate) struct VecXmlReader<E: FromXml, T: VecElemTag> {
    inner: Vec<E>,
    _marker: std::marker::PhantomData<T>,
}

impl<E: FromXml, T: VecElemTag> From<VecXmlReader<E, T>> for Vec<E> {
    fn from(reader: VecXmlReader<E, T>) -> Self {
        reader.inner
    }
}

impl<E: FromXml, T: VecElemTag> FromXml for VecXmlReader<E, T> {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, T::VALUE).map(|inner| Self {
            inner,
            _marker: Default::default(),
        })
    }
}

pub(crate) fn to_xml_read_error(
    element_name: impl AsRef<str>,
) -> impl FnOnce(xml::reader::Error) -> XmlReadError {
    let element_name = element_name.as_ref().to_owned();
    |error| XmlReadError::ElementReadError {
        error,
        element: element_name,
    }
}

pub(crate) fn expected_namespace_or_error(
    expected_version_number: impl AsRef<str>,
    namespace: &Namespace,
) -> Result<(), XmlReadError> {
    let actual_namespace: Option<String> = namespace.get(NS_NO_PREFIX).map(String::from);
    let expected_namespace = format!(
        "http://cyclonedx.org/schema/bom/{}",
        expected_version_number.as_ref()
    );
    if actual_namespace.as_ref() == Some(&expected_namespace) {
        Ok(())
    } else {
        Err(XmlReadError::InvalidNamespaceError {
            expected_namespace,
            actual_namespace,
        })
    }
}

pub(crate) fn inner_text_or_error(
    element_name: impl AsRef<str>,
) -> impl FnOnce(xml::reader::XmlEvent) -> Result<String, XmlReadError> {
    let element_name = element_name.as_ref().to_owned();
    |event| match event {
        reader::XmlEvent::Characters(s) | reader::XmlEvent::CData(s) => Ok(s),
        unexpected => Err(unexpected_element_error(element_name, unexpected)),
    }
}

pub(crate) fn inner_text_or_none(
    element_name: impl AsRef<str>,
) -> impl FnOnce(xml::reader::XmlEvent) -> Result<Option<String>, XmlReadError> {
    let element_name = element_name.as_ref().to_owned();
    |event| match event {
        reader::XmlEvent::Characters(s) | reader::XmlEvent::CData(s) => Ok(Some(s)),
        reader::XmlEvent::EndElement { name } if name.to_string() == element_name => Ok(None),
        unexpected => Err(unexpected_element_error(element_name, unexpected)),
    }
}

pub(crate) fn closing_tag_or_error(
    element: &OwnedName,
) -> impl FnOnce(xml::reader::XmlEvent) -> Result<(), XmlReadError> {
    let element = element.clone();
    move |event| match event {
        reader::XmlEvent::EndElement { name } if name == element => Ok(()),
        unexpected => Err(unexpected_element_error(&element, unexpected)),
    }
}

pub(crate) fn attribute_or_error(
    element_name: &OwnedName,
    attributes: &[OwnedAttribute],
    expected_attribute: &str,
) -> Result<String, XmlReadError> {
    attributes
        .iter()
        .filter(|attr| attr.name.local_name == expected_attribute)
        .map(|attr| attr.value.to_owned())
        .next()
        .ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: expected_attribute.to_string(),
            element: element_name.local_name.to_string(),
        })
}

pub(crate) fn optional_attribute(
    attributes: &[OwnedAttribute],
    expected_attribute: &str,
) -> Option<String> {
    attributes
        .iter()
        .filter(|attr| attr.name.local_name == expected_attribute)
        .map(|attr| attr.value.to_owned())
        .next()
}

pub(crate) trait FromXmlType
where
    Self: Sized,
{
    fn xml_type_display() -> String;

    fn from_xml_value(element: impl ToString, value: impl AsRef<str>)
        -> Result<Self, XmlReadError>;
}

impl FromXmlType for bool {
    fn xml_type_display() -> String {
        "xs:boolean".to_string()
    }

    fn from_xml_value(
        element: impl ToString,
        value: impl AsRef<str>,
    ) -> Result<Self, XmlReadError> {
        let value = value.as_ref();
        match value {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(XmlReadError::InvalidParseError {
                value: value.to_string(),
                data_type: Self::xml_type_display(),
                element: element.to_string(),
            }),
        }
    }
}

impl FromXmlType for u32 {
    fn xml_type_display() -> String {
        "xs:integer".to_string()
    }

    fn from_xml_value(
        element: impl ToString,
        value: impl AsRef<str>,
    ) -> Result<Self, XmlReadError> {
        let value = value.as_ref();
        let value: u32 = value.parse().map_err(|_| XmlReadError::InvalidParseError {
            value: value.to_string(),
            data_type: Self::xml_type_display(),
            element: element.to_string(),
        })?;

        Ok(value)
    }
}

impl FromXmlType for f32 {
    fn xml_type_display() -> String {
        "xs:decimal".to_string()
    }

    fn from_xml_value(
        element: impl ToString,
        value: impl AsRef<str>,
    ) -> Result<Self, XmlReadError> {
        let value = value.as_ref();
        let value: f32 = value.parse().map_err(|_| XmlReadError::InvalidParseError {
            value: value.to_string(),
            data_type: Self::xml_type_display(),
            element: element.to_string(),
        })?;

        Ok(value)
    }
}

/// Reads a simple String tag.
///
/// ```xml
/// <description>Content</description>
/// ```
/// &
/// ```xml
/// <description />
/// ```
///
/// are valid XML tags. The first returns the string "Content", the latter is an empty string.
pub(crate) fn read_simple_tag<R: Read>(
    event_reader: &mut EventReader<R>,
    element: &OwnedName,
) -> Result<String, XmlReadError> {
    let element_display = element.to_string();
    let content = event_reader
        .next()
        .map_err(to_xml_read_error(&element_display))?;

    let content = match content {
        reader::XmlEvent::EndElement { .. } => String::new(),
        reader::XmlEvent::Characters(content) | reader::XmlEvent::CData(content) => {
            event_reader
                .next()
                .map_err(to_xml_read_error(&element_display))
                .and_then(closing_tag_or_error(element))?;
            content
        }
        unexpected => return Err(unexpected_element_error(element, unexpected)),
    };

    Ok(content)
}

pub(crate) fn read_optional_tag<R: Read>(
    event_reader: &mut EventReader<R>,
    element: &OwnedName,
) -> Result<Option<String>, XmlReadError> {
    let element_display = element.to_string();
    let content = event_reader
        .next()
        .map_err(to_xml_read_error(&element_display))
        .and_then(inner_text_or_none(&element_display))?;

    // If XML tag has content, read next element
    if content.is_some() {
        event_reader
            .next()
            .map_err(to_xml_read_error(&element_display))
            .and_then(closing_tag_or_error(element))?;
    }

    Ok(content)
}

pub(crate) fn read_u32_tag<R: Read>(
    event_reader: &mut EventReader<R>,
    element: &OwnedName,
) -> Result<u32, XmlReadError> {
    let element_display = element.to_string();
    let content = event_reader
        .next()
        .map_err(to_xml_read_error(&element_display))
        .and_then(inner_text_or_error(&element_display))?;

    let number = match content.trim().parse::<u32>() {
        Ok(n) => n,
        Err(_) => {
            return Err(XmlReadError::InvalidParseError {
                value: content,
                data_type: "u32".to_string(),
                element: element_display,
            })
        }
    };

    event_reader
        .next()
        .map_err(to_xml_read_error(&element_display))
        .and_then(closing_tag_or_error(element))?;

    Ok(number)
}

pub(crate) fn read_f32_tag<R: Read>(
    event_reader: &mut EventReader<R>,
    element: &OwnedName,
) -> Result<f32, XmlReadError> {
    let element_display = element.to_string();
    let content = event_reader
        .next()
        .map_err(to_xml_read_error(&element_display))
        .and_then(inner_text_or_error(&element_display))?;

    let number = match content.trim().parse::<f32>() {
        Ok(n) => n,
        Err(_) => {
            return Err(XmlReadError::InvalidParseError {
                value: content,
                data_type: "f32".to_string(),
                element: element_display,
            })
        }
    };

    event_reader
        .next()
        .map_err(to_xml_read_error(&element_display))
        .and_then(closing_tag_or_error(element))?;

    Ok(number)
}

pub(crate) fn read_boolean_tag<R: Read>(
    event_reader: &mut EventReader<R>,
    element: &OwnedName,
) -> Result<bool, XmlReadError> {
    read_simple_tag(event_reader, element)
        .and_then(|modified| bool::from_xml_value(element, modified))
}

impl FromXml for String {
    fn read_xml_element<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_simple_tag(event_reader, element_name)
    }
}

impl FromXml for u32 {
    fn read_xml_element<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_u32_tag(event_reader, element_name)
    }
}

impl FromXml for f32 {
    fn read_xml_element<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_f32_tag(event_reader, element_name)
    }
}

impl FromXml for bool {
    fn read_xml_element<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        _attributes: &[OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_boolean_tag(event_reader, element_name)
    }
}

pub(crate) fn read_list_tag<R: Read, X: FromXml>(
    event_reader: &mut EventReader<R>,
    element_name: &OwnedName,
    inner_element_tag: &str,
) -> Result<Vec<X>, XmlReadError> {
    let mut items = Vec::new();

    let mut got_end_tag = false;
    while !got_end_tag {
        let next_element = event_reader
            .next()
            .map_err(to_xml_read_error(&element_name.local_name))?;
        match next_element {
            reader::XmlEvent::StartElement {
                name, attributes, ..
            } if name.local_name == inner_element_tag => {
                items.push(X::read_xml_element(event_reader, &name, &attributes)?);
            }
            reader::XmlEvent::EndElement { name } if &name == element_name => {
                got_end_tag = true;
            }
            unexpected => return Err(unexpected_element_error(element_name, unexpected)),
        }
    }

    Ok(items)
}

pub(crate) fn read_lax_validation_tag<R: Read>(
    event_reader: &mut EventReader<R>,
    element: &OwnedName,
) -> Result<(), XmlReadError> {
    let mut got_end_tag = false;
    while !got_end_tag {
        let next_element = event_reader
            .next()
            .map_err(to_xml_read_error(&element.local_name))?;

        match next_element {
            reader::XmlEvent::StartElement { name, .. } => {
                read_lax_validation_tag(event_reader, &name)?
            }
            reader::XmlEvent::EndElement { name } if &name == element => {
                got_end_tag = true;
            }
            unexpected @ reader::XmlEvent::EndDocument => {
                return Err(unexpected_element_error(element, unexpected))
            }
            unexpected @ reader::XmlEvent::EndElement { .. } => {
                return Err(unexpected_element_error(element, unexpected))
            }
            _unknown => (),
        }
    }

    Ok(())
}

pub(crate) fn read_lax_validation_list_tag<R: Read, X: FromXml>(
    event_reader: &mut EventReader<R>,
    element_name: &OwnedName,
    inner_element_tag: &str,
) -> Result<Vec<X>, XmlReadError> {
    let mut items = Vec::new();

    let mut got_end_tag = false;
    while !got_end_tag {
        let next_element = event_reader
            .next()
            .map_err(to_xml_read_error(&element_name.local_name))?;
        match next_element {
            reader::XmlEvent::StartElement {
                name, attributes, ..
            } if name.local_name == inner_element_tag => {
                items.push(X::read_xml_element(event_reader, &name, &attributes)?);
            }
            reader::XmlEvent::StartElement { name, .. } => {
                read_lax_validation_tag(event_reader, &name)?
            }
            reader::XmlEvent::EndElement { name } if &name == element_name => {
                got_end_tag = true;
            }
            unexpected => return Err(unexpected_element_error(element_name, unexpected)),
        }
    }

    Ok(items)
}

pub(crate) fn unexpected_element_error(
    element: impl ToString,
    unexpected: reader::XmlEvent,
) -> XmlReadError {
    XmlReadError::UnexpectedElementReadError {
        error: format!("Got unexpected element {:?}", unexpected),
        element: element.to_string(),
    }
}

#[cfg(test)]
pub(crate) mod test {
    use xml::{EmitterConfig, ParserConfig};

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

    fn parser_config() -> ParserConfig {
        ParserConfig::default().trim_whitespace(true)
    }

    pub(crate) fn read_document_from_string<X: FromXmlDocument>(string: impl AsRef<str>) -> X {
        let mut event_reader =
            EventReader::new_with_config(string.as_ref().as_bytes(), parser_config());
        let output: X = X::read_xml_document(&mut event_reader)
            .expect("Failed to read the document from the string");

        // According to the documentation, an event reader that returns an
        // EndDocument event will continue to return that event for subsequent
        // requests
        let end_document = event_reader.next().expect("Expected to end the document");

        match end_document {
            reader::XmlEvent::EndDocument { .. } => (),
            other => panic!("Expected to end a document, but got {:?}", other),
        }

        output
    }

    pub(crate) fn read_element_from_string<X: FromXml>(string: impl AsRef<str>) -> X {
        let mut event_reader =
            EventReader::new_with_config(string.as_ref().as_bytes(), parser_config());

        let start_document = event_reader.next().expect("Expected to start the document");

        match start_document {
            reader::XmlEvent::StartDocument { .. } => (),
            other => panic!("Expected to start a document, but got {:?}", other),
        }

        let initial_event = event_reader
            .next()
            .expect("Failed to read from the XML input");
        let output = match initial_event {
            reader::XmlEvent::StartElement {
                name, attributes, ..
            } => X::read_xml_element(&mut event_reader, &name, &attributes)
                .expect("Failed to read the element from the string"),
            other => panic!("Expected to start an element, but got {:?}", other),
        };
        let end_document = event_reader.next().expect("Expected to end the document");

        match end_document {
            reader::XmlEvent::EndDocument { .. } => (),
            other => panic!("Expected to end a document, but got {:?}", other),
        }

        output
    }

    #[test]
    fn it_should_handle_invalid_lax_xml() {
        let input = r#"
<recursiveTag>
  <innerTag>
    <recursiveTag>
      Text
    </recursiveTag>
  </innerTag>
"#;
        let mut event_reader = EventReader::new_with_config(input.as_bytes(), parser_config());

        let start_document = event_reader.next().expect("Expected to start the document");

        match start_document {
            reader::XmlEvent::StartDocument { .. } => (),
            other => panic!("Expected to start a document, but got {:?}", other),
        }

        let start_lax_element = event_reader.next().expect("Expected to start the document");

        match start_lax_element {
            reader::XmlEvent::StartElement { name, .. } => {
                read_lax_validation_tag(&mut event_reader, &name)
                    .expect_err("Should have failed to parse invalid input");
            }
            other => panic!("Expected to start an element, but got {:?}", other),
        }

        // no end document, because it returns an error during the read_lax_validation_tag call
    }
}
