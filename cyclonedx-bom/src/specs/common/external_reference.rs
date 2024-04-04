/*
 * This file is part of CycloneDX Rust Cargo.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use cyclonedx_bom_macros::versioned;

#[versioned("1.3", "1.4", "1.5")]
pub(crate) mod base {
    use crate::{
        errors::XmlReadError,
        external_models, models,
        specs::common::hash::Hashes,
        utilities::{convert_optional, convert_vec},
        xml::to_xml_write_error,
        xml::{
            attribute_or_error, read_list_tag, read_simple_tag, to_xml_read_error,
            unexpected_element_error, write_simple_tag, FromXml, ToXml,
        },
    };
    use serde::{Deserialize, Serialize};
    use xml::{reader, writer::XmlEvent};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(transparent)]
    pub(crate) struct ExternalReferences(Vec<ExternalReference>);

    impl From<models::external_reference::ExternalReferences> for ExternalReferences {
        fn from(other: models::external_reference::ExternalReferences) -> Self {
            ExternalReferences(convert_vec(other.0))
        }
    }

    impl From<ExternalReferences> for models::external_reference::ExternalReferences {
        fn from(other: ExternalReferences) -> Self {
            models::external_reference::ExternalReferences(convert_vec(other.0))
        }
    }

    const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";

    impl ToXml for ExternalReferences {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            writer
                .write(XmlEvent::start_element(EXTERNAL_REFERENCES_TAG))
                .map_err(to_xml_write_error(EXTERNAL_REFERENCES_TAG))?;

            for external_reference in &self.0 {
                external_reference.write_xml_element(writer)?;
            }

            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(EXTERNAL_REFERENCES_TAG))?;
            Ok(())
        }
    }

    impl FromXml for ExternalReferences {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            _attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, crate::errors::XmlReadError>
        where
            Self: Sized,
        {
            read_list_tag(event_reader, element_name, REFERENCE_TAG).map(ExternalReferences)
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct ExternalReference {
        #[serde(rename = "type")]
        external_reference_type: String,
        url: Uri,
        #[serde(skip_serializing_if = "Option::is_none")]
        comment: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hashes: Option<Hashes>,
    }

    impl From<models::external_reference::ExternalReference> for ExternalReference {
        fn from(other: models::external_reference::ExternalReference) -> Self {
            Self {
                external_reference_type: other.external_reference_type.to_string(),
                url: other.url.into(),
                comment: other.comment,
                hashes: convert_optional(other.hashes),
            }
        }
    }

    impl From<ExternalReference> for models::external_reference::ExternalReference {
        fn from(other: ExternalReference) -> Self {
            Self {
                external_reference_type:
                    models::external_reference::ExternalReferenceType::new_unchecked(
                        other.external_reference_type,
                    ),
                url: other.url.into(),
                comment: other.comment,
                hashes: convert_optional(other.hashes),
            }
        }
    }

    const REFERENCE_TAG: &str = "reference";
    const TYPE_ATTR: &str = "type";
    const URL_TAG: &str = "url";
    const COMMENT_TAG: &str = "comment";

    impl ToXml for ExternalReference {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            writer
                .write(
                    XmlEvent::start_element(REFERENCE_TAG)
                        .attr(TYPE_ATTR, &self.external_reference_type),
                )
                .map_err(to_xml_write_error(REFERENCE_TAG))?;

            self.url.write_xml_element(writer)?;

            if let Some(comment) = &self.comment {
                write_simple_tag(writer, COMMENT_TAG, comment)?;
            }

            if let Some(hashes) = &self.hashes {
                hashes.write_xml_element(writer)?;
            }

            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(REFERENCE_TAG))?;

            Ok(())
        }
    }

    const HASHES_TAG: &str = "hashes";

    impl FromXml for ExternalReference {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let reference_type = attribute_or_error(element_name, attributes, TYPE_ATTR)?;
            let mut url: Option<Uri> = None;
            let mut comment: Option<String> = None;
            let mut hashes: Option<Hashes> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(REFERENCE_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement { name, .. } if name.local_name == URL_TAG => {
                        url = Some(Uri::read_xml_element(event_reader, &name, &attributes)?)
                    }
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == COMMENT_TAG =>
                    {
                        comment = Some(read_simple_tag(event_reader, &name)?)
                    }
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == HASHES_TAG => {
                        hashes = Some(Hashes::read_xml_element(event_reader, &name, &attributes)?)
                    }
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            let url = url.ok_or_else(|| XmlReadError::RequiredDataMissing {
                required_field: URL_TAG.to_string(),
                element: element_name.local_name.to_string(),
            })?;

            Ok(Self {
                external_reference_type: reference_type,
                url,
                comment,
                hashes,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase", untagged)]
    pub(crate) enum Uri {
        Url(String),
    }

    impl From<models::external_reference::Uri> for Uri {
        fn from(value: models::external_reference::Uri) -> Self {
            match value {
                models::external_reference::Uri::Url(url) => Self::Url(url.0),
            }
        }
    }

    impl From<Uri> for models::external_reference::Uri {
        fn from(value: Uri) -> Self {
            match value {
                Uri::Url(url) => Self::Url(external_models::uri::Uri(url)),
            }
        }
    }

    impl ToXml for Uri {
        fn write_xml_element<W: std::io::prelude::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            match self {
                Self::Url(url) => write_simple_tag(writer, URL_TAG, &url),
            }
        }
    }

    impl FromXml for Uri {
        fn read_xml_element<R: std::io::prelude::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            _attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            read_simple_tag(event_reader, element_name).map(Self::Url)
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;
        use crate::{
            specs::common::hash::test::{corresponding_hashes, example_hashes},
            xml::test::{read_element_from_string, write_element_to_string},
        };

        pub(crate) fn example_external_references() -> ExternalReferences {
            ExternalReferences(vec![example_external_reference()])
        }

        pub(crate) fn corresponding_external_references(
        ) -> models::external_reference::ExternalReferences {
            models::external_reference::ExternalReferences(vec![corresponding_external_reference()])
        }

        pub(crate) fn example_uri() -> Uri {
            Uri::Url("url".to_string())
        }

        pub(crate) fn example_external_reference() -> ExternalReference {
            ExternalReference {
                external_reference_type: "external reference type".to_string(),
                url: example_uri(),
                comment: Some("comment".to_string()),
                hashes: Some(example_hashes()),
            }
        }

        pub(crate) fn corresponding_external_reference(
        ) -> models::external_reference::ExternalReference {
            models::external_reference::ExternalReference {
                external_reference_type:
                    models::external_reference::ExternalReferenceType::UnknownExternalReferenceType(
                        "external reference type".to_string(),
                    ),
                url: models::external_reference::Uri::Url(external_models::uri::Uri(
                    "url".to_string(),
                )),
                comment: Some("comment".to_string()),
                hashes: Some(corresponding_hashes()),
            }
        }

        #[test]
        fn it_should_write_xml_full() {
            let xml_output = write_element_to_string(example_external_references());
            insta::assert_snapshot!(xml_output);
        }

        #[test]
        fn it_should_read_xml_full() {
            let input = r#"
<externalReferences>
  <reference type="external reference type">
    <url>url</url>
    <comment>comment</comment>
    <hashes>
      <hash alg="algorithm">hash value</hash>
    </hashes>
  </reference>
</externalReferences>
"#;
            let actual: ExternalReferences = read_element_from_string(input);
            let expected = example_external_references();
            assert_eq!(actual, expected);
        }
    }
}
