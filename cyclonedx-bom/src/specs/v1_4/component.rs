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

use crate::{
    errors::XmlReadError,
    external_models::{
        normalized_string::NormalizedString,
        uri::{Purl, Uri},
    },
    specs::v1_4::{
        attached_text::AttachedText, code::Commits, code::Patches,
        external_reference::ExternalReferences, hash::Hashes, license::Licenses,
        organization::OrganizationalEntity, property::Properties,
    },
    xml::{
        attribute_or_error, optional_attribute, read_boolean_tag, read_lax_validation_list_tag,
        read_lax_validation_tag, read_list_tag, read_simple_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_simple_tag, FromXml, FromXmlType,
        ToInnerXml, ToXml,
    },
};
use crate::{
    models,
    utilities::{convert_optional, convert_vec},
};
use serde::{Deserialize, Serialize};
use xml::{reader, writer::XmlEvent};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Components(Vec<Component>);

impl From<models::component::Components> for Components {
    fn from(other: models::component::Components) -> Self {
        Components(convert_vec(other.0))
    }
}

impl From<Components> for models::component::Components {
    fn from(other: Components) -> Self {
        models::component::Components(convert_vec(other.0))
    }
}

impl ToInnerXml for Components {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(tag))
            .map_err(to_xml_write_error(tag))?;

        for component in &self.0 {
            component.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(tag))?;
        Ok(())
    }
}

const COMPONENTS_TAG: &str = "components";

impl ToXml for Components {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        self.write_xml_named_element(writer, COMPONENTS_TAG)
    }
}

impl FromXml for Components {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        read_lax_validation_list_tag(event_reader, element_name, COMPONENT_TAG).map(Components)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Component {
    #[serde(rename = "type")]
    component_type: String,
    #[serde(rename = "mime-type", skip_serializing_if = "Option::is_none")]
    mime_type: Option<MimeType>,
    #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
    bom_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supplier: Option<OrganizationalEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    name: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hashes: Option<Hashes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    licenses: Option<Licenses>,
    #[serde(skip_serializing_if = "Option::is_none")]
    copyright: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpe: Option<Cpe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    purl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    swid: Option<Swid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pedigree: Option<Pedigree>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_references: Option<ExternalReferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence: Option<ComponentEvidence>,
}

impl From<models::component::Component> for Component {
    fn from(other: models::component::Component) -> Self {
        Self {
            component_type: other.component_type.to_string(),
            mime_type: other.mime_type.map(|m| MimeType(m.0)),
            bom_ref: other.bom_ref,
            supplier: convert_optional(other.supplier),
            author: other.author.map(|a| a.to_string()),
            publisher: other.publisher.map(|p| p.to_string()),
            group: other.group.map(|g| g.to_string()),
            name: other.name.to_string(),
            version: other.version.to_string(),
            description: other.description.map(|d| d.to_string()),
            scope: other.scope.map(|s| s.to_string()),
            hashes: convert_optional(other.hashes),
            licenses: convert_optional(other.licenses),
            copyright: other.copyright.map(|c| c.to_string()),
            cpe: convert_optional(other.cpe),
            purl: other.purl.map(|p| p.0),
            swid: convert_optional(other.swid),
            modified: other.modified,
            pedigree: convert_optional(other.pedigree),
            external_references: convert_optional(other.external_references),
            properties: convert_optional(other.properties),
            components: convert_optional(other.components),
            evidence: convert_optional(other.evidence),
        }
    }
}

impl From<Component> for models::component::Component {
    fn from(other: Component) -> Self {
        Self {
            component_type: models::component::Classification::new_unchecked(other.component_type),
            mime_type: other.mime_type.map(|m| models::component::MimeType(m.0)),
            bom_ref: other.bom_ref,
            supplier: convert_optional(other.supplier),
            author: other.author.map(NormalizedString::new_unchecked),
            publisher: other.publisher.map(NormalizedString::new_unchecked),
            group: other.group.map(NormalizedString::new_unchecked),
            name: NormalizedString::new_unchecked(other.name),
            version: NormalizedString::new_unchecked(other.version),
            description: other.description.map(NormalizedString::new_unchecked),
            scope: other.scope.map(models::component::Scope::new_unchecked),
            hashes: convert_optional(other.hashes),
            licenses: convert_optional(other.licenses),
            copyright: other.copyright.map(NormalizedString::new_unchecked),
            cpe: convert_optional(other.cpe),
            purl: other.purl.map(Purl),
            swid: convert_optional(other.swid),
            modified: other.modified,
            pedigree: convert_optional(other.pedigree),
            external_references: convert_optional(other.external_references),
            properties: convert_optional(other.properties),
            components: convert_optional(other.components),
            evidence: convert_optional(other.evidence),
        }
    }
}

const COMPONENT_TAG: &str = "component";
const TYPE_ATTR: &str = "type";
const MIME_TYPE_ATTR: &str = "mime-type";
const BOM_REF_ATTR: &str = "bom-ref";
const SUPPLIER_TAG: &str = "supplier";
const AUTHOR_TAG: &str = "author";
const PUBLISHER_TAG: &str = "publisher";
const GROUP_TAG: &str = "group";
const NAME_TAG: &str = "name";
const VERSION_TAG: &str = "version";
const DESCRIPTION_TAG: &str = "description";
const SCOPE_TAG: &str = "scope";
const COPYRIGHT_TAG: &str = "copyright";
const PURL_TAG: &str = "purl";
const MODIFIED_TAG: &str = "modified";

impl ToXml for Component {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let mut component_start_tag =
            XmlEvent::start_element(COMPONENT_TAG).attr(TYPE_ATTR, &self.component_type);

        if let Some(mime_type) = &self.mime_type {
            component_start_tag = component_start_tag.attr(MIME_TYPE_ATTR, &mime_type.0);
        }

        if let Some(bom_ref) = &self.bom_ref {
            component_start_tag = component_start_tag.attr(BOM_REF_ATTR, bom_ref);
        }

        writer
            .write(component_start_tag)
            .map_err(to_xml_write_error(COMPONENT_TAG))?;

        if let Some(supplier) = &self.supplier {
            if supplier.will_write() {
                supplier.write_xml_named_element(writer, SUPPLIER_TAG)?;
            }
        }

        if let Some(author) = &self.author {
            write_simple_tag(writer, AUTHOR_TAG, author)?;
        }

        if let Some(publisher) = &self.publisher {
            write_simple_tag(writer, PUBLISHER_TAG, publisher)?;
        }

        if let Some(group) = &self.group {
            write_simple_tag(writer, GROUP_TAG, group)?;
        }

        write_simple_tag(writer, NAME_TAG, &self.name)?;

        write_simple_tag(writer, VERSION_TAG, &self.version)?;

        if let Some(description) = &self.description {
            write_simple_tag(writer, DESCRIPTION_TAG, description)?;
        }

        if let Some(scope) = &self.scope {
            write_simple_tag(writer, SCOPE_TAG, scope)?;
        }

        if let Some(hashes) = &self.hashes {
            hashes.write_xml_element(writer)?;
        }

        if let Some(licenses) = &self.licenses {
            licenses.write_xml_element(writer)?;
        }

        if let Some(copyright) = &self.copyright {
            write_simple_tag(writer, COPYRIGHT_TAG, copyright)?;
        }

        if let Some(cpe) = &self.cpe {
            cpe.write_xml_element(writer)?;
        }

        if let Some(purl) = &self.purl {
            write_simple_tag(writer, PURL_TAG, purl)?;
        }

        if let Some(swid) = &self.swid {
            swid.write_xml_element(writer)?;
        }

        if let Some(modified) = &self.modified {
            write_simple_tag(writer, MODIFIED_TAG, &format!("{}", modified))?;
        }

        if let Some(pedigree) = &self.pedigree {
            pedigree.write_xml_element(writer)?;
        }

        if let Some(external_references) = &self.external_references {
            external_references.write_xml_element(writer)?;
        }

        if let Some(properties) = &self.properties {
            properties.write_xml_element(writer)?;
        }

        if let Some(components) = &self.components {
            components.write_xml_element(writer)?;
        }

        if let Some(evidence) = &self.evidence {
            if evidence.will_write() {
                evidence.write_xml_element(writer)?;
            }
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(COMPONENT_TAG))?;

        Ok(())
    }
}

const HASHES_TAG: &str = "hashes";
const LICENSES_TAG: &str = "licenses";
const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";
const PROPERTIES_TAG: &str = "properties";

impl FromXml for Component {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let component_type = attribute_or_error(element_name, attributes, TYPE_ATTR)?;
        let mime_type = optional_attribute(attributes, MIME_TYPE_ATTR).map(MimeType);
        let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);

        let mut supplier: Option<OrganizationalEntity> = None;
        let mut author: Option<String> = None;
        let mut publisher: Option<String> = None;
        let mut group: Option<String> = None;
        let mut component_name: Option<String> = None;
        let mut version: Option<String> = None;
        let mut description: Option<String> = None;
        let mut scope: Option<String> = None;
        let mut hashes: Option<Hashes> = None;
        let mut licenses: Option<Licenses> = None;
        let mut copyright: Option<String> = None;
        let mut cpe: Option<Cpe> = None;
        let mut purl: Option<String> = None;
        let mut swid: Option<Swid> = None;
        let mut modified: Option<bool> = None;
        let mut pedigree: Option<Pedigree> = None;
        let mut external_references: Option<ExternalReferences> = None;
        let mut properties: Option<Properties> = None;
        let mut components: Option<Components> = None;
        let mut evidence: Option<ComponentEvidence> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(COMPONENT_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SUPPLIER_TAG => {
                    supplier = Some(OrganizationalEntity::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == AUTHOR_TAG => {
                    author = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == PUBLISHER_TAG => {
                    publisher = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == GROUP_TAG => {
                    group = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    component_name = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == VERSION_TAG => {
                    version = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == DESCRIPTION_TAG =>
                {
                    description = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == SCOPE_TAG => {
                    scope = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == HASHES_TAG => {
                    hashes = Some(Hashes::read_xml_element(event_reader, &name, &attributes)?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == LICENSES_TAG => {
                    licenses = Some(Licenses::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == COPYRIGHT_TAG => {
                    copyright = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == CPE_TAG => {
                    cpe = Some(Cpe::read_xml_element(event_reader, &name, &attributes)?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == PURL_TAG => {
                    purl = Some(read_simple_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SWID_TAG => {
                    swid = Some(Swid::read_xml_element(event_reader, &name, &attributes)?)
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == MODIFIED_TAG => {
                    modified = Some(read_boolean_tag(event_reader, &name)?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PEDIGREE_TAG => {
                    pedigree = Some(Pedigree::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == EXTERNAL_REFERENCES_TAG => {
                    external_references = Some(ExternalReferences::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PROPERTIES_TAG => {
                    properties = Some(Properties::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COMPONENTS_TAG => {
                    components = Some(Components::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == EVIDENCE_TAG => {
                    evidence = Some(ComponentEvidence::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
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

        let component_name = component_name.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: NAME_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;
        let version = version.ok_or_else(|| XmlReadError::RequiredDataMissing {
            required_field: VERSION_TAG.to_string(),
            element: element_name.local_name.to_string(),
        })?;

        Ok(Self {
            component_type,
            mime_type,
            bom_ref,
            supplier,
            author,
            publisher,
            group,
            name: component_name,
            version,
            description,
            scope,
            hashes,
            licenses,
            copyright,
            cpe,
            purl,
            swid,
            modified,
            pedigree,
            external_references,
            properties,
            components,
            evidence,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Swid {
    tag_id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<AttachedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl From<models::component::Swid> for Swid {
    fn from(other: models::component::Swid) -> Self {
        Self {
            tag_id: other.tag_id,
            name: other.name,
            version: other.version,
            tag_version: other.tag_version,
            patch: other.patch,
            text: convert_optional(other.text),
            url: other.url.map(|u| u.to_string()),
        }
    }
}

impl From<Swid> for models::component::Swid {
    fn from(other: Swid) -> Self {
        Self {
            tag_id: other.tag_id,
            name: other.name,
            version: other.version,
            tag_version: other.tag_version,
            patch: other.patch,
            text: convert_optional(other.text),
            url: other.url.map(Uri),
        }
    }
}

const SWID_TAG: &str = "swid";
const TAG_ID_ATTR: &str = "tagId";
const NAME_ATTR: &str = "name";
const VERSION_ATTR: &str = "version";
const TAG_VERSION_ATTR: &str = "tagVersion";
const PATCH_ATTR: &str = "patch";
const TEXT_TAG: &str = "text";
const URL_TAG: &str = "url";

impl ToXml for Swid {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let tag_version = self.tag_version.map(|tv| format!("{}", tv));
        let patch = self.patch.map(|p| format!("{}", p));

        let mut swid_start_tag = XmlEvent::start_element(SWID_TAG)
            .attr(TAG_ID_ATTR, &self.tag_id)
            .attr(NAME_ATTR, &self.name);

        if let Some(version) = &self.version {
            swid_start_tag = swid_start_tag.attr(VERSION_ATTR, version);
        }

        if let Some(tag_version) = &tag_version {
            swid_start_tag = swid_start_tag.attr(TAG_VERSION_ATTR, tag_version);
        }

        if let Some(patch) = &patch {
            swid_start_tag = swid_start_tag.attr(PATCH_ATTR, patch);
        }

        writer
            .write(swid_start_tag)
            .map_err(to_xml_write_error(SWID_TAG))?;

        if let Some(text) = &self.text {
            text.write_xml_named_element(writer, TEXT_TAG)?;
        }

        if let Some(url) = &self.url {
            write_simple_tag(writer, URL_TAG, url)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(SWID_TAG))?;

        Ok(())
    }
}

impl FromXml for Swid {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let tag_id = attribute_or_error(element_name, attributes, TAG_ID_ATTR)?;
        let name = attribute_or_error(element_name, attributes, NAME_ATTR)?;
        let version = optional_attribute(attributes, VERSION_ATTR);
        let tag_version =
            if let Some(tag_version) = optional_attribute(attributes, TAG_VERSION_ATTR) {
                let tag_version = u32::from_xml_value(TAG_VERSION_ATTR, &tag_version)?;
                Some(tag_version)
            } else {
                None
            };
        let patch = if let Some(patch) = optional_attribute(attributes, PATCH_ATTR) {
            let patch = bool::from_xml_value(PATCH_ATTR, patch)?;
            Some(patch)
        } else {
            None
        };
        let mut text: Option<AttachedText> = None;
        let mut url: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(SWID_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == TEXT_TAG => {
                    text = Some(AttachedText::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == URL_TAG => {
                    url = Some(read_simple_tag(event_reader, &name)?)
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

        Ok(Self {
            tag_id,
            name,
            version,
            tag_version,
            patch,
            text,
            url,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Cpe(String);

impl From<models::component::Cpe> for Cpe {
    fn from(other: models::component::Cpe) -> Self {
        Self(other.0)
    }
}

impl From<Cpe> for models::component::Cpe {
    fn from(other: Cpe) -> Self {
        Self(other.0)
    }
}

const CPE_TAG: &str = "cpe";

impl ToXml for Cpe {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        write_simple_tag(writer, CPE_TAG, &self.0)
    }
}

impl FromXml for Cpe {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_simple_tag(event_reader, element_name).map(Cpe)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ComponentEvidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    licenses: Option<Licenses>,
    #[serde(skip_serializing_if = "Option::is_none")]
    copyright: Option<CopyrightTexts>,
}

impl From<models::component::ComponentEvidence> for ComponentEvidence {
    fn from(other: models::component::ComponentEvidence) -> Self {
        Self {
            licenses: convert_optional(other.licenses),
            copyright: convert_optional(other.copyright),
        }
    }
}

impl From<ComponentEvidence> for models::component::ComponentEvidence {
    fn from(other: ComponentEvidence) -> Self {
        Self {
            licenses: convert_optional(other.licenses),
            copyright: convert_optional(other.copyright),
        }
    }
}

const EVIDENCE_TAG: &str = "evidence";

impl ToXml for ComponentEvidence {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(EVIDENCE_TAG))
            .map_err(to_xml_write_error(EVIDENCE_TAG))?;

        if let Some(licenses) = &self.licenses {
            licenses.write_xml_element(writer)?;
        }

        if let Some(copyright) = &self.copyright {
            copyright.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(EVIDENCE_TAG))?;

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.licenses.is_some() || self.copyright.is_some()
    }
}

impl FromXml for ComponentEvidence {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut licenses: Option<Licenses> = None;
        let mut copyright: Option<CopyrightTexts> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(EVIDENCE_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == LICENSES_TAG => {
                    licenses = Some(Licenses::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COPYRIGHT_TAG => {
                    copyright = Some(CopyrightTexts::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            licenses,
            copyright,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Pedigree {
    #[serde(skip_serializing_if = "Option::is_none")]
    ancestors: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    descendants: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variants: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commits: Option<Commits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patches: Option<Patches>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
}

impl From<models::component::Pedigree> for Pedigree {
    fn from(other: models::component::Pedigree) -> Self {
        Self {
            ancestors: convert_optional(other.ancestors),
            descendants: convert_optional(other.descendants),
            variants: convert_optional(other.variants),
            commits: convert_optional(other.commits),
            patches: convert_optional(other.patches),
            notes: other.notes,
        }
    }
}

impl From<Pedigree> for models::component::Pedigree {
    fn from(other: Pedigree) -> Self {
        Self {
            ancestors: convert_optional(other.ancestors),
            descendants: convert_optional(other.descendants),
            variants: convert_optional(other.variants),
            commits: convert_optional(other.commits),
            patches: convert_optional(other.patches),
            notes: other.notes,
        }
    }
}

const PEDIGREE_TAG: &str = "pedigree";
const ANCESTORS_TAG: &str = "ancestors";
const DESCENDANTS_TAG: &str = "descendants";
const VARIANTS_TAG: &str = "variants";
const NOTES_TAG: &str = "notes";

impl ToXml for Pedigree {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(PEDIGREE_TAG))
            .map_err(to_xml_write_error(PEDIGREE_TAG))?;

        if let Some(ancestors) = &self.ancestors {
            ancestors.write_xml_named_element(writer, ANCESTORS_TAG)?;
        }

        if let Some(descendants) = &self.descendants {
            descendants.write_xml_named_element(writer, DESCENDANTS_TAG)?;
        }

        if let Some(variants) = &self.variants {
            variants.write_xml_named_element(writer, VARIANTS_TAG)?;
        }

        if let Some(commits) = &self.commits {
            commits.write_xml_element(writer)?;
        }

        if let Some(patches) = &self.patches {
            patches.write_xml_element(writer)?;
        }

        if let Some(notes) = &self.notes {
            write_simple_tag(writer, NOTES_TAG, notes)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(PEDIGREE_TAG))?;

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.ancestors.is_some()
            || self.descendants.is_some()
            || self.variants.is_some()
            || self.commits.is_some()
            || self.patches.is_some()
            || self.notes.is_some()
    }
}

const COMMITS_TAG: &str = "commits";
const PATCHES_TAG: &str = "patches";

impl FromXml for Pedigree {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut ancestors: Option<Components> = None;
        let mut descendants: Option<Components> = None;
        let mut variants: Option<Components> = None;
        let mut commits: Option<Commits> = None;
        let mut patches: Option<Patches> = None;
        let mut notes: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(PEDIGREE_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == ANCESTORS_TAG => {
                    ancestors = Some(Components::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == DESCENDANTS_TAG => {
                    descendants = Some(Components::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == VARIANTS_TAG => {
                    variants = Some(Components::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COMMITS_TAG => {
                    commits = Some(Commits::read_xml_element(event_reader, &name, &attributes)?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PATCHES_TAG => {
                    patches = Some(Patches::read_xml_element(event_reader, &name, &attributes)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NOTES_TAG => {
                    notes = Some(read_simple_tag(event_reader, &name)?)
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

        Ok(Self {
            ancestors,
            descendants,
            variants,
            commits,
            patches,
            notes,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Copyright {
    text: String,
}

impl From<models::component::Copyright> for Copyright {
    fn from(other: models::component::Copyright) -> Self {
        Self { text: other.0 }
    }
}

impl From<Copyright> for models::component::Copyright {
    fn from(other: Copyright) -> Self {
        Self(other.text)
    }
}

impl ToXml for Copyright {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(TEXT_TAG))
            .map_err(to_xml_write_error(TEXT_TAG))?;

        writer
            .write(XmlEvent::cdata(&self.text))
            .map_err(to_xml_write_error(TEXT_TAG))?;

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(TEXT_TAG))?;

        Ok(())
    }
}

impl FromXml for Copyright {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_simple_tag(event_reader, element_name).map(|text| Self { text })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
struct CopyrightTexts(Vec<Copyright>);

impl From<models::component::CopyrightTexts> for CopyrightTexts {
    fn from(other: models::component::CopyrightTexts) -> Self {
        CopyrightTexts(convert_vec(other.0))
    }
}

impl From<CopyrightTexts> for models::component::CopyrightTexts {
    fn from(other: CopyrightTexts) -> Self {
        models::component::CopyrightTexts(convert_vec(other.0))
    }
}

impl ToXml for CopyrightTexts {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(XmlEvent::start_element(COPYRIGHT_TAG))
            .map_err(to_xml_write_error(COPYRIGHT_TAG))?;

        for copyright in &self.0 {
            copyright.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(COPYRIGHT_TAG))?;
        Ok(())
    }
}

impl FromXml for CopyrightTexts {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_list_tag(event_reader, element_name, TEXT_TAG).map(CopyrightTexts)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct MimeType(String);

impl From<models::component::MimeType> for MimeType {
    fn from(other: models::component::MimeType) -> Self {
        Self(other.0)
    }
}

impl From<MimeType> for models::component::MimeType {
    fn from(other: MimeType) -> Self {
        Self(other.0)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        specs::v1_4::{
            attached_text::test::{corresponding_attached_text, example_attached_text},
            code::test::{
                corresponding_commits, corresponding_patches, example_commits, example_patches,
            },
            external_reference::test::{
                corresponding_external_references, example_external_references,
            },
            hash::test::{corresponding_hashes, example_hashes},
            license::test::{corresponding_licenses, example_licenses},
            organization::test::{corresponding_entity, example_entity},
            property::test::{corresponding_properties, example_properties},
        },
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::*;

    pub(crate) fn example_components() -> Components {
        Components(vec![example_component()])
    }

    pub(crate) fn corresponding_components() -> models::component::Components {
        models::component::Components(vec![corresponding_component()])
    }

    pub(crate) fn example_component() -> Component {
        Component {
            component_type: "component type".to_string(),
            mime_type: Some(MimeType("mime type".to_string())),
            bom_ref: Some("bom ref".to_string()),
            supplier: Some(example_entity()),
            author: Some("author".to_string()),
            publisher: Some("publisher".to_string()),
            group: Some("group".to_string()),
            name: "name".to_string(),
            version: "version".to_string(),
            description: Some("description".to_string()),
            scope: Some("scope".to_string()),
            hashes: Some(example_hashes()),
            licenses: Some(example_licenses()),
            copyright: Some("copyright".to_string()),
            cpe: Some(example_cpe()),
            purl: Some("purl".to_string()),
            swid: Some(example_swid()),
            modified: Some(true),
            pedigree: Some(example_pedigree()),
            external_references: Some(example_external_references()),
            properties: Some(example_properties()),
            components: Some(example_empty_components()),
            evidence: Some(example_evidence()),
        }
    }

    pub(crate) fn corresponding_component() -> models::component::Component {
        models::component::Component {
            component_type: models::component::Classification::UnknownClassification(
                "component type".to_string(),
            ),
            mime_type: Some(models::component::MimeType("mime type".to_string())),
            bom_ref: Some("bom ref".to_string()),
            supplier: Some(corresponding_entity()),
            author: Some(NormalizedString::new_unchecked("author".to_string())),
            publisher: Some(NormalizedString::new_unchecked("publisher".to_string())),
            group: Some(NormalizedString::new_unchecked("group".to_string())),
            name: NormalizedString::new_unchecked("name".to_string()),
            version: NormalizedString::new_unchecked("version".to_string()),
            description: Some(NormalizedString::new_unchecked("description".to_string())),
            scope: Some(models::component::Scope::UnknownScope("scope".to_string())),
            hashes: Some(corresponding_hashes()),
            licenses: Some(corresponding_licenses()),
            copyright: Some(NormalizedString::new_unchecked("copyright".to_string())),
            cpe: Some(corresponding_cpe()),
            purl: Some(Purl("purl".to_string())),
            swid: Some(corresponding_swid()),
            modified: Some(true),
            pedigree: Some(corresponding_pedigree()),
            external_references: Some(corresponding_external_references()),
            properties: Some(corresponding_properties()),
            components: Some(corresponding_empty_components()),
            evidence: Some(corresponding_evidence()),
        }
    }

    fn example_empty_components() -> Components {
        Components(Vec::new())
    }

    fn corresponding_empty_components() -> models::component::Components {
        models::component::Components(Vec::new())
    }

    fn example_cpe() -> Cpe {
        Cpe("cpe".to_string())
    }

    fn corresponding_cpe() -> models::component::Cpe {
        models::component::Cpe("cpe".to_string())
    }

    fn example_swid() -> Swid {
        Swid {
            tag_id: "tag id".to_string(),
            name: "name".to_string(),
            version: Some("version".to_string()),
            tag_version: Some(1),
            patch: Some(true),
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        }
    }

    fn corresponding_swid() -> models::component::Swid {
        models::component::Swid {
            tag_id: "tag id".to_string(),
            name: "name".to_string(),
            version: Some("version".to_string()),
            tag_version: Some(1),
            patch: Some(true),
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        }
    }

    fn example_pedigree() -> Pedigree {
        Pedigree {
            ancestors: Some(example_empty_components()),
            descendants: Some(example_empty_components()),
            variants: Some(example_empty_components()),
            commits: Some(example_commits()),
            patches: Some(example_patches()),
            notes: Some("notes".to_string()),
        }
    }

    fn corresponding_pedigree() -> models::component::Pedigree {
        models::component::Pedigree {
            ancestors: Some(corresponding_empty_components()),
            descendants: Some(corresponding_empty_components()),
            variants: Some(corresponding_empty_components()),
            commits: Some(corresponding_commits()),
            patches: Some(corresponding_patches()),
            notes: Some("notes".to_string()),
        }
    }

    fn example_evidence() -> ComponentEvidence {
        ComponentEvidence {
            licenses: Some(example_licenses()),
            copyright: Some(example_copyright_texts()),
        }
    }

    fn corresponding_evidence() -> models::component::ComponentEvidence {
        models::component::ComponentEvidence {
            licenses: Some(corresponding_licenses()),
            copyright: Some(corresponding_copyright_texts()),
        }
    }

    fn example_copyright_texts() -> CopyrightTexts {
        CopyrightTexts(vec![example_copyright()])
    }

    fn corresponding_copyright_texts() -> models::component::CopyrightTexts {
        models::component::CopyrightTexts(vec![corresponding_copyright()])
    }

    fn example_copyright() -> Copyright {
        Copyright {
            text: "copyright".to_string(),
        }
    }

    fn corresponding_copyright() -> models::component::Copyright {
        models::component::Copyright("copyright".to_string())
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_components());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<components>
  <component type="component type" mime-type="mime type" bom-ref="bom ref">
    <supplier>
      <name>name</name>
      <url>url</url>
      <contact>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </contact>
    </supplier>
    <author>author</author>
    <publisher>publisher</publisher>
    <group>group</group>
    <name>name</name>
    <version>version</version>
    <description>description</description>
    <scope>scope</scope>
    <hashes>
      <hash alg="algorithm">hash value</hash>
    </hashes>
    <licenses>
      <expression>expression</expression>
    </licenses>
    <copyright>copyright</copyright>
    <cpe>cpe</cpe>
    <purl>purl</purl>
    <swid tagId="tag id" name="name" version="version" tagVersion="1" patch="true">
      <text content-type="content type" encoding="encoding">content</text>
      <url>url</url>
    </swid>
    <modified>true</modified>
    <pedigree>
      <ancestors />
      <descendants />
      <variants />
      <commits>
        <commit>
          <uid>uid</uid>
          <url>url</url>
          <author>
            <timestamp>timestamp</timestamp>
            <name>name</name>
            <email>email</email>
          </author>
          <committer>
            <timestamp>timestamp</timestamp>
            <name>name</name>
            <email>email</email>
          </committer>
          <message>message</message>
        </commit>
      </commits>
      <patches>
        <patch type="patch type">
          <diff>
            <text content-type="content type" encoding="encoding">content</text>
            <url>url</url>
          </diff>
          <resolves>
            <issue type="issue type">
              <id>id</id>
              <name>name</name>
              <description>description</description>
              <source>
                <name>name</name>
                <url>url</url>
              </source>
              <references>
                <url>reference</url>
              </references>
            </issue>
          </resolves>
        </patch>
      </patches>
      <notes>notes</notes>
    </pedigree>
    <externalReferences>
      <reference type="external reference type">
        <url>url</url>
        <comment>comment</comment>
        <hashes>
          <hash alg="algorithm">hash value</hash>
        </hashes>
      </reference>
    </externalReferences>
    <properties>
      <property name="name">value</property>
    </properties>
    <components />
    <evidence>
      <licenses>
        <expression>expression</expression>
      </licenses>
      <copyright>
        <text><![CDATA[copyright]]></text>
      </copyright>
    </evidence>
  </component>
</components>
"#;
        let actual: Components = read_element_from_string(input);
        let expected = example_components();
        assert_eq!(actual, expected);
    }
}
