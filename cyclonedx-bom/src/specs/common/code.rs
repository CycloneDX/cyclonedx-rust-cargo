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
    external_models::{date_time::DateTime, normalized_string::NormalizedString, uri::Uri},
    models,
    specs::common::attached_text::AttachedText,
    utilities::{convert_optional, convert_optional_vec, convert_vec},
    xml::{
        attribute_or_error, read_lax_validation_list_tag, read_lax_validation_tag, read_list_tag,
        read_simple_tag, to_xml_read_error, to_xml_write_error, unexpected_element_error,
        write_simple_tag, FromXml, ToInnerXml, ToXml,
    },
};
use serde::{Deserialize, Serialize};
use xml::{reader, writer};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Commits(Vec<Commit>);

impl From<models::code::Commits> for Commits {
    fn from(other: models::code::Commits) -> Self {
        Commits(convert_vec(other.0))
    }
}

impl From<Commits> for models::code::Commits {
    fn from(other: Commits) -> Self {
        models::code::Commits(convert_vec(other.0))
    }
}

const COMMITS_TAG: &str = "commits";

impl ToXml for Commits {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(COMMITS_TAG))
            .map_err(to_xml_write_error(COMMITS_TAG))?;

        for commit in &self.0 {
            commit.write_xml_element(writer)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(COMMITS_TAG))?;
        Ok(())
    }
}

impl FromXml for Commits {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_lax_validation_list_tag(event_reader, element_name, COMMIT_TAG).map(Commits)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Commit {
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<IdentifiableAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    committer: Option<IdentifiableAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl From<models::code::Commit> for Commit {
    fn from(other: models::code::Commit) -> Self {
        Self {
            uid: other.uid.map(|uid| uid.to_string()),
            url: other.url.map(|url| url.to_string()),
            author: convert_optional(other.author),
            committer: convert_optional(other.committer),
            message: other.message.map(|m| m.to_string()),
        }
    }
}

impl From<Commit> for models::code::Commit {
    fn from(other: Commit) -> Self {
        Self {
            uid: other.uid.map(NormalizedString::new_unchecked),
            url: other.url.map(Uri),
            author: convert_optional(other.author),
            committer: convert_optional(other.committer),
            message: other.message.map(NormalizedString::new_unchecked),
        }
    }
}

const COMMIT_TAG: &str = "commit";
const UID_TAG: &str = "uid";
const URL_TAG: &str = "url";
const AUTHOR_TAG: &str = "author";
const COMMITTER_TAG: &str = "committer";
const MESSAGE_TAG: &str = "message";

impl ToXml for Commit {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(COMMIT_TAG))
            .map_err(to_xml_write_error(COMMIT_TAG))?;

        if let Some(uid) = &self.uid {
            write_simple_tag(writer, UID_TAG, uid)?;
        }

        if let Some(url) = &self.url {
            write_simple_tag(writer, URL_TAG, url)?;
        }

        if let Some(author) = &self.author {
            author.write_xml_named_element(writer, AUTHOR_TAG)?;
        }

        if let Some(committer) = &self.committer {
            committer.write_xml_named_element(writer, COMMITTER_TAG)?;
        }

        if let Some(message) = &self.message {
            write_simple_tag(writer, MESSAGE_TAG, message)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(COMMIT_TAG))?;

        Ok(())
    }
}

impl FromXml for Commit {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut uid: Option<String> = None;
        let mut url: Option<String> = None;
        let mut author: Option<IdentifiableAction> = None;
        let mut committer: Option<IdentifiableAction> = None;
        let mut message: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(COMMIT_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == UID_TAG => {
                    uid = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == URL_TAG => {
                    url = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == AUTHOR_TAG => {
                    author = Some(IdentifiableAction::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COMMITTER_TAG => {
                    committer = Some(IdentifiableAction::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == MESSAGE_TAG => {
                    message = Some(read_simple_tag(event_reader, &name)?)
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

        Ok(Self {
            uid,
            url,
            author,
            committer,
            message,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct IdentifiableAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
}

impl From<models::code::IdentifiableAction> for IdentifiableAction {
    fn from(other: models::code::IdentifiableAction) -> Self {
        Self {
            timestamp: other.timestamp.map(|t| t.to_string()),
            name: other.name.map(|n| n.to_string()),
            email: other.email.map(|e| e.to_string()),
        }
    }
}

impl From<IdentifiableAction> for models::code::IdentifiableAction {
    fn from(other: IdentifiableAction) -> Self {
        Self {
            timestamp: other.timestamp.map(DateTime),
            name: other.name.map(NormalizedString::new_unchecked),
            email: other.email.map(NormalizedString::new_unchecked),
        }
    }
}

const TIMESTAMP_TAG: &str = "timestamp";
const NAME_TAG: &str = "name";
const EMAIL_TAG: &str = "email";

impl ToInnerXml for IdentifiableAction {
    fn write_xml_named_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
        tag: &str,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(tag))
            .map_err(to_xml_write_error(tag))?;

        if let Some(timestamp) = &self.timestamp {
            write_simple_tag(writer, TIMESTAMP_TAG, timestamp)?;
        }

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(email) = &self.email {
            write_simple_tag(writer, EMAIL_TAG, email)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(tag))?;

        Ok(())
    }
}

impl FromXml for IdentifiableAction {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut timestamp: Option<String> = None;
        let mut identity_name: Option<String> = None;
        let mut email: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader
                .next()
                .map_err(to_xml_read_error(element_name.local_name.as_str()))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == TIMESTAMP_TAG => {
                    timestamp = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    identity_name = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == EMAIL_TAG => {
                    email = Some(read_simple_tag(event_reader, &name)?)
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
            timestamp,
            name: identity_name,
            email,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Patches(Vec<Patch>);

impl From<models::code::Patches> for Patches {
    fn from(other: models::code::Patches) -> Self {
        Patches(convert_vec(other.0))
    }
}

impl From<Patches> for models::code::Patches {
    fn from(other: Patches) -> Self {
        models::code::Patches(convert_vec(other.0))
    }
}

const PATCHES_TAG: &str = "patches";

impl ToXml for Patches {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(PATCHES_TAG))
            .map_err(to_xml_write_error(PATCHES_TAG))?;

        for patch in &self.0 {
            patch.write_xml_element(writer)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(PATCHES_TAG))?;
        Ok(())
    }
}

impl FromXml for Patches {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        read_lax_validation_list_tag(event_reader, element_name, PATCH_TAG).map(Patches)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Patch {
    #[serde(rename = "type")]
    patch_type: String,
    diff: Option<Diff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolves: Option<Vec<Issue>>,
}

impl From<models::code::Patch> for Patch {
    fn from(other: models::code::Patch) -> Self {
        Self {
            patch_type: other.patch_type.to_string(),
            diff: convert_optional(other.diff),
            resolves: convert_optional_vec(other.resolves),
        }
    }
}

impl From<Patch> for models::code::Patch {
    fn from(other: Patch) -> Self {
        Self {
            patch_type: models::code::PatchClassification::new_unchecked(other.patch_type),
            diff: convert_optional(other.diff),
            resolves: convert_optional_vec(other.resolves),
        }
    }
}

const PATCH_TAG: &str = "patch";
const TYPE_ATTR: &str = "type";
const RESOLVES_TAG: &str = "resolves";

impl ToXml for Patch {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(PATCH_TAG).attr(TYPE_ATTR, &self.patch_type))
            .map_err(to_xml_write_error(PATCH_TAG))?;

        if let Some(diff) = &self.diff {
            if diff.will_write() {
                diff.write_xml_element(writer)?;
            }
        }

        if let Some(resolves) = &self.resolves {
            writer
                .write(writer::XmlEvent::start_element(RESOLVES_TAG))
                .map_err(to_xml_write_error(PATCH_TAG))?;

            for issue in resolves {
                issue.write_xml_element(writer)?;
            }

            writer
                .write(writer::XmlEvent::end_element())
                .map_err(to_xml_write_error(RESOLVES_TAG))?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(PATCH_TAG))?;

        Ok(())
    }
}

impl FromXml for Patch {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let patch_type = attribute_or_error(element_name, attributes, TYPE_ATTR)?;
        let mut diff: Option<Diff> = None;
        let mut resolves: Option<Vec<Issue>> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(PATCH_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == DIFF_TAG => {
                    diff = Some(Diff::read_xml_element(event_reader, &name, &attributes)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == RESOLVES_TAG => {
                    resolves = Some(read_list_tag(event_reader, &name, ISSUE_TAG)?)
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
            patch_type,
            diff,
            resolves,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Diff {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<AttachedText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl From<models::code::Diff> for Diff {
    fn from(other: models::code::Diff) -> Self {
        Self {
            text: convert_optional(other.text),
            url: other.url.map(|u| u.to_string()),
        }
    }
}

impl From<Diff> for models::code::Diff {
    fn from(other: Diff) -> Self {
        Self {
            text: convert_optional(other.text),
            url: other.url.map(Uri),
        }
    }
}

const DIFF_TAG: &str = "diff";
const TEXT_TAG: &str = "text";

impl ToXml for Diff {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(DIFF_TAG))
            .map_err(to_xml_write_error(DIFF_TAG))?;

        if let Some(text) = &self.text {
            text.write_xml_named_element(writer, TEXT_TAG)?;
        }

        if let Some(url) = &self.url {
            write_simple_tag(writer, URL_TAG, url)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(DIFF_TAG))?;

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.text.is_some() || self.url.is_some()
    }
}

impl FromXml for Diff {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut text: Option<AttachedText> = None;
        let mut url: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(DIFF_TAG))?;
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

        Ok(Self { text, url })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Issue {
    #[serde(rename = "type")]
    issue_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    references: Option<Vec<String>>,
}

impl From<models::code::Issue> for Issue {
    fn from(other: models::code::Issue) -> Self {
        Self {
            issue_type: other.issue_type.to_string(),
            id: other.id.map(|i| i.to_string()),
            name: other.name.map(|n| n.to_string()),
            description: other.description.map(|d| d.to_string()),
            source: convert_optional(other.source),
            references: other
                .references
                .map(|references| references.into_iter().map(|r| r.to_string()).collect()),
        }
    }
}

impl From<Issue> for models::code::Issue {
    fn from(other: Issue) -> Self {
        Self {
            issue_type: models::code::IssueClassification::new_unchecked(other.issue_type),
            id: other.id.map(NormalizedString::new_unchecked),
            name: other.name.map(NormalizedString::new_unchecked),
            description: other.description.map(NormalizedString::new_unchecked),
            source: convert_optional(other.source),
            references: other
                .references
                .map(|references| references.into_iter().map(Uri).collect()),
        }
    }
}

const ISSUE_TAG: &str = "issue";
const ID_TAG: &str = "id";
const DESCRIPTION_TAG: &str = "description";
const REFERENCES_TAG: &str = "references";

impl ToXml for Issue {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(ISSUE_TAG).attr(TYPE_ATTR, &self.issue_type))
            .map_err(to_xml_write_error(ISSUE_TAG))?;

        if let Some(id) = &self.id {
            write_simple_tag(writer, ID_TAG, id)?;
        }

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(description) = &self.description {
            write_simple_tag(writer, DESCRIPTION_TAG, description)?;
        }

        if let Some(source) = &self.source {
            if source.will_write() {
                source.write_xml_element(writer)?;
            }
        }

        if let Some(references) = &self.references {
            writer
                .write(writer::XmlEvent::start_element(REFERENCES_TAG))
                .map_err(to_xml_write_error(REFERENCES_TAG))?;

            for reference in references {
                write_simple_tag(writer, URL_TAG, reference)?;
            }

            writer
                .write(writer::XmlEvent::end_element())
                .map_err(to_xml_write_error(REFERENCES_TAG))?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(ISSUE_TAG))?;

        Ok(())
    }
}

impl FromXml for Issue {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let issue_type = attribute_or_error(element_name, attributes, TYPE_ATTR)?;
        let mut id: Option<String> = None;
        let mut issue_name: Option<String> = None;
        let mut description: Option<String> = None;
        let mut source: Option<Source> = None;
        let mut references: Option<Vec<String>> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(DIFF_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == ID_TAG => {
                    id = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    issue_name = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == DESCRIPTION_TAG =>
                {
                    description = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SOURCE_TAG => {
                    source = Some(Source::read_xml_element(event_reader, &name, &attributes)?)
                }
                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == REFERENCES_TAG =>
                {
                    references = Some(read_list_tag(event_reader, &name, URL_TAG)?)
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
            issue_type,
            id,
            name: issue_name,
            description,
            source,
            references,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl From<models::code::Source> for Source {
    fn from(other: models::code::Source) -> Self {
        Self {
            name: other.name.map(|n| n.to_string()),
            url: other.url.map(|u| u.to_string()),
        }
    }
}

impl From<Source> for models::code::Source {
    fn from(other: Source) -> Self {
        Self {
            name: other.name.map(NormalizedString::new_unchecked),
            url: other.url.map(Uri),
        }
    }
}

const SOURCE_TAG: &str = "source";

impl ToXml for Source {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(SOURCE_TAG))
            .map_err(to_xml_write_error(SOURCE_TAG))?;

        if let Some(name) = &self.name {
            write_simple_tag(writer, NAME_TAG, name)?;
        }

        if let Some(url) = &self.url {
            write_simple_tag(writer, URL_TAG, url)?;
        }

        writer
            .write(writer::XmlEvent::end_element())
            .map_err(to_xml_write_error(SOURCE_TAG))?;

        Ok(())
    }

    fn will_write(&self) -> bool {
        self.name.is_some() || self.url.is_some()
    }
}

impl FromXml for Source {
    fn read_xml_element<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        _attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let mut source_name: Option<String> = None;
        let mut url: Option<String> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(DIFF_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                    source_name = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::StartElement { name, .. } if name.local_name == URL_TAG => {
                    url = Some(read_simple_tag(event_reader, &name)?)
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        Ok(Self {
            name: source_name,
            url,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        specs::common::attached_text::test::{corresponding_attached_text, example_attached_text},
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::*;

    pub(crate) fn example_commits() -> Commits {
        Commits(vec![example_commit()])
    }

    pub(crate) fn corresponding_commits() -> models::code::Commits {
        models::code::Commits(vec![corresponding_commit()])
    }

    pub(crate) fn example_commit() -> Commit {
        Commit {
            uid: Some("uid".to_string()),
            url: Some("url".to_string()),
            author: Some(example_identifiable_action()),
            committer: Some(example_identifiable_action()),
            message: Some("message".to_string()),
        }
    }

    pub(crate) fn corresponding_commit() -> models::code::Commit {
        models::code::Commit {
            uid: Some(NormalizedString::new_unchecked("uid".to_string())),
            url: Some(Uri("url".to_string())),
            author: Some(corresponding_identifiable_action()),
            committer: Some(corresponding_identifiable_action()),
            message: Some(NormalizedString::new_unchecked("message".to_string())),
        }
    }

    fn example_identifiable_action() -> IdentifiableAction {
        IdentifiableAction {
            timestamp: Some("timestamp".to_string()),
            name: Some("name".to_string()),
            email: Some("email".to_string()),
        }
    }

    fn corresponding_identifiable_action() -> models::code::IdentifiableAction {
        models::code::IdentifiableAction {
            timestamp: Some(DateTime("timestamp".to_string())),
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            email: Some(NormalizedString::new_unchecked("email".to_string())),
        }
    }

    pub(crate) fn example_patches() -> Patches {
        Patches(vec![example_patch()])
    }

    pub(crate) fn corresponding_patches() -> models::code::Patches {
        models::code::Patches(vec![corresponding_patch()])
    }

    pub(crate) fn example_patch() -> Patch {
        Patch {
            patch_type: "patch type".to_string(),
            diff: Some(example_diff()),
            resolves: Some(vec![example_issue()]),
        }
    }

    pub(crate) fn corresponding_patch() -> models::code::Patch {
        models::code::Patch {
            patch_type: models::code::PatchClassification::UnknownPatchClassification(
                "patch type".to_string(),
            ),
            diff: Some(corresponding_diff()),
            resolves: Some(vec![corresponding_issue()]),
        }
    }

    fn example_diff() -> Diff {
        Diff {
            text: Some(example_attached_text()),
            url: Some("url".to_string()),
        }
    }

    fn corresponding_diff() -> models::code::Diff {
        models::code::Diff {
            text: Some(corresponding_attached_text()),
            url: Some(Uri("url".to_string())),
        }
    }

    fn example_issue() -> Issue {
        Issue {
            issue_type: "issue type".to_string(),
            id: Some("id".to_string()),
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            source: Some(example_source()),
            references: Some(vec!["reference".to_string()]),
        }
    }

    fn corresponding_issue() -> models::code::Issue {
        models::code::Issue {
            issue_type: models::code::IssueClassification::UnknownIssueClassification(
                "issue type".to_string(),
            ),
            id: Some(NormalizedString::new_unchecked("id".to_string())),
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            description: Some(NormalizedString::new_unchecked("description".to_string())),
            source: Some(corresponding_source()),
            references: Some(vec![Uri("reference".to_string())]),
        }
    }

    fn example_source() -> Source {
        Source {
            name: Some("name".to_string()),
            url: Some("url".to_string()),
        }
    }

    fn corresponding_source() -> models::code::Source {
        models::code::Source {
            name: Some(NormalizedString::new_unchecked("name".to_string())),
            url: Some(Uri("url".to_string())),
        }
    }

    #[test]
    fn it_should_write_commits_xml_full() {
        let xml_output = write_element_to_string(example_commits());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_write_patches_xml_full() {
        let xml_output = write_element_to_string(example_patches());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_commits_xml_full() {
        let input = r#"
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

"#;
        let actual: Commits = read_element_from_string(input);
        let expected = example_commits();
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_patches_xml_full() {
        let input = r#"
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

"#;
        let actual: Patches = read_element_from_string(input);
        let expected = example_patches();
        assert_eq!(actual, expected);
    }
}
