use serde::{Deserialize, Serialize};
use xml::reader;

use crate::{
    errors::XmlReadError,
    models,
    specs::v1_5::external_reference::ExternalReference,
    xml::{
        read_lax_validation_tag, read_simple_tag, to_xml_read_error, unexpected_element_error,
        write_close_tag, write_simple_tag, write_start_tag, FromXml, ToInnerXml, ToXml,
    },
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct ResourceReferences(pub(crate) Vec<ResourceReference>);

const RESOURCE_REFERENCES_TAG: &str = "resourceReferences";

impl ToXml for ResourceReferences {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, RESOURCE_REFERENCES_TAG)?;

        for reference in &self.0 {
            reference.write_xml_named_element(writer, RESOURCE_REFERENCE_TAG)?;
        }

        write_close_tag(writer, RESOURCE_REFERENCES_TAG)?;

        Ok(())
    }
}

impl FromXml for ResourceReferences {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut resource_references = vec![];

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(RESOURCE_REFERENCE_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == RESOURCE_REFERENCE_TAG => {
                    resource_references.push(ResourceReference::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?);
                }
                // lax validation of any elements from a different schema
                reader::XmlEvent::StartElement { name, .. } => {
                    read_lax_validation_tag(event_reader, &name)?
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self(resource_references))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, rename_all_fields = "camelCase")]
pub(crate) enum ResourceReference {
    Ref {
        r#ref: String,
    },
    ExternalReference {
        external_reference: ExternalReference,
    },
}

impl From<models::formulation::workflow::resource_reference::ResourceReference>
    for ResourceReference
{
    fn from(
        resource_reference: models::formulation::workflow::resource_reference::ResourceReference,
    ) -> Self {
        match resource_reference {
            models::formulation::workflow::resource_reference::ResourceReference::Ref(r#ref) => Self::Ref { r#ref },
            models::formulation::workflow::resource_reference::ResourceReference::ExternalReference(external_reference) => Self::ExternalReference { external_reference: external_reference.into() },
        }
    }
}

impl From<ResourceReference>
    for models::formulation::workflow::resource_reference::ResourceReference
{
    fn from(resource_reference: ResourceReference) -> Self {
        match resource_reference {
            ResourceReference::Ref { r#ref } => Self::Ref(r#ref),
            ResourceReference::ExternalReference { external_reference } => {
                Self::ExternalReference(external_reference.into())
            }
        }
    }
}

const RESOURCE_REFERENCE_TAG: &str = "resourceReference";
const REF_TAG: &str = "ref";
const EXTERNAL_REFERENCE_TAG: &str = "externalReference";

impl ToInnerXml for ResourceReference {
    fn write_xml_named_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_start_tag(writer, tag)?;

        match self {
            Self::Ref { r#ref } => write_simple_tag(writer, REF_TAG, r#ref)?,
            Self::ExternalReference { external_reference } => {
                external_reference.write_xml_element(writer)?
            }
        }

        write_close_tag(writer, tag)?;

        Ok(())
    }
}

impl FromXml for ResourceReference {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let mut resource_reference = None;

        let mut got_end_tag = false;

        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(RESOURCE_REFERENCE_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name: ref elem_name,
                    ref attributes,
                    ..
                } => match elem_name.local_name.as_str() {
                    REF_TAG => {
                        resource_reference = Some(Self::Ref {
                            r#ref: read_simple_tag(event_reader, elem_name)?,
                        })
                    }
                    EXTERNAL_REFERENCE_TAG => {
                        resource_reference = Some(Self::ExternalReference {
                            external_reference: ExternalReference::read_xml_element(
                                event_reader,
                                elem_name,
                                attributes,
                            )?,
                        })
                    }
                    _ => {
                        return Err(unexpected_element_error(
                            elem_name.to_string(),
                            next_element,
                        ))
                    }
                },
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        resource_reference.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: RESOURCE_REFERENCE_TAG.into(),
            element: element_name.local_name.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::xml::test::{read_element_from_string, write_element_to_string};

    use super::*;

    fn example_resource_references() -> ResourceReferences {
        ResourceReferences(vec![ResourceReference::Ref {
            r#ref: "component-a".into(),
        }])
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_resource_references());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
 <resourceReferences>
    <resourceReference>
        <ref>component-a</ref>
    </resourceReference>
</resourceReferences>
"#;
        let actual: ResourceReferences = read_element_from_string(input);
        let expected = example_resource_references();
        assert_eq!(actual, expected);
    }
}
