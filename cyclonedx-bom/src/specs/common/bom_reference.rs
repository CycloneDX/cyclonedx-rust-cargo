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

use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;

use crate::{
    errors::XmlReadError,
    models,
    xml::{
        attribute_or_error, closing_tag_or_error, to_xml_read_error, to_xml_write_error,
        write_close_tag, FromXml, ToInnerXml,
    },
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct BomReference(String);

impl BomReference {
    #[allow(unused)]
    pub fn new<T>(input: T) -> Self
    where
        T: ToString,
    {
        Self(input.to_string())
    }
}

impl AsRef<str> for BomReference {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<models::bom::BomReference> for BomReference {
    fn from(other: models::bom::BomReference) -> Self {
        Self(other.0)
    }
}

impl From<BomReference> for models::bom::BomReference {
    fn from(other: BomReference) -> Self {
        Self(other.0)
    }
}

const REF_ATTR: &str = "ref";

impl ToInnerXml for BomReference {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(tag).attr(REF_ATTR, &self.0))
            .map_err(to_xml_write_error(tag))?;

        write_close_tag(writer, tag)?;

        Ok(())
    }
}

impl FromXml for BomReference {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let reference = attribute_or_error(element_name, attributes, REF_ATTR)?;
        event_reader
            .next()
            .map_err(to_xml_read_error(&element_name.local_name))
            .and_then(closing_tag_or_error(element_name))?;

        Ok(Self(reference))
    }
}
