use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    specs::common::property::Properties,
    xml::{
        read_boolean_tag, read_lax_validation_tag, read_simple_tag, to_xml_read_error,
        unexpected_element_error, write_close_tag, write_simple_tag, write_start_tag, FromXml,
        ToXml,
    },
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Volume {
    uid: Option<String>,
    name: Option<String>,
    mode: Option<String>,
    path: Option<String>,
    size_allocated: Option<String>,
    persistent: Option<bool>,
    remote: Option<bool>,
    properties: Option<Properties>,
}

const VOLUME_TAG: &str = "volume";
const UID_TAG: &str = "uid";
const NAME_TAG: &str = "name";
const MODE_TAG: &str = "mode";
const PATH_TAG: &str = "path";
const SIZE_ALLOCATED_TAG: &str = "sizeAllocated";
const PERSISTENT_TAG: &str = "persistent";
const REMOTE_TAG: &str = "remote";
const PROPERTIES_TAG: &str = "properties";

impl ToXml for Volume {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, VOLUME_TAG)?;

        if let Some(uid) = &self.uid {
            write_simple_tag(writer, UID_TAG, uid)?;
        }

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(mode) = &self.mode {
            write_simple_tag(writer, MODE_TAG, mode)?;
        }

        if let Some(path) = &self.path {
            write_simple_tag(writer, PATH_TAG, path)?;
        }

        if let Some(size_allocated) = &self.size_allocated {
            write_simple_tag(writer, SIZE_ALLOCATED_TAG, size_allocated)?;
        }

        if let Some(persistent) = &self.persistent {
            write_simple_tag(writer, PERSISTENT_TAG, &persistent.to_string())?
        }

        if let Some(remote) = &self.remote {
            write_simple_tag(writer, REMOTE_TAG, &remote.to_string())?
        }

        if let Some(properties) = &self.properties {
            properties.write_xml_element(writer)?;
        }

        write_close_tag(writer, VOLUME_TAG)?;

        Ok(())
    }
}

impl FromXml for Volume {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut uid: Option<String> = None;
        let mut name: Option<String> = None;
        let mut mode: Option<String> = None;
        let mut path: Option<String> = None;
        let mut size_allocated: Option<String> = None;
        let mut persistent: Option<bool> = None;
        let mut remote: Option<bool> = None;
        let mut properties: Option<Properties> = None;

        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(VOLUME_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name: elem_name,
                    attributes,
                    ..
                } => {
                    match elem_name.local_name.as_str() {
                        UID_TAG => uid = Some(read_simple_tag(event_reader, &elem_name)?),
                        NAME_TAG => name = Some(read_simple_tag(event_reader, &elem_name)?),
                        MODE_TAG => mode = Some(read_simple_tag(event_reader, &elem_name)?),
                        PATH_TAG => path = Some(read_simple_tag(event_reader, &elem_name)?),
                        SIZE_ALLOCATED_TAG => {
                            size_allocated = Some(read_simple_tag(event_reader, &elem_name)?)
                        }
                        PERSISTENT_TAG => {
                            persistent = Some(read_boolean_tag(event_reader, &elem_name)?)
                        }
                        REMOTE_TAG => remote = Some(read_boolean_tag(event_reader, &elem_name)?),
                        PROPERTIES_TAG => {
                            properties = Some(Properties::read_xml_element(
                                event_reader,
                                &elem_name,
                                &attributes,
                            )?)
                        }
                        _ => read_lax_validation_tag(event_reader, &elem_name)?,
                    };
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            uid,
            name,
            mode,
            path,
            size_allocated,
            persistent,
            remote,
            properties,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;

    fn example_volume() -> Volume {
        Volume {
            uid: Some("volume-1".into()),
            name: Some("My volume".into()),
            mode: Some("filesystem".into()),
            path: Some("/".into()),
            size_allocated: Some("10GB".into()),
            persistent: Some(true),
            remote: Some(false),
            properties: None,
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_volume());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<volume>
  <uid>volume-1</uid>
  <name>My volume</name>
  <mode>filesystem</mode>
  <path>/</path>
  <sizeAllocated>10GB</sizeAllocated>
  <persistent>true</persistent>
  <remote>false</remote>
</volume>
"#;
        let actual: Volume = read_element_from_string(input);
        let expected = example_volume();
        assert_eq!(actual, expected);
    }
}
