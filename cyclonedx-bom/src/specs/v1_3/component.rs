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
    external_models::{
        normalized_string::NormalizedString,
        uri::{Purl, Uri},
    },
    specs::v1_3::{
        attached_text::AttachedText, code::Commit, code::Patch,
        external_reference::ExternalReference, hash::Hashes, license::Licenses,
        organization::OrganizationalEntity, property::Properties,
    },
};
use crate::{
    models,
    utilities::{convert_optional, convert_optional_vec},
};
use serde::{Deserialize, Serialize};

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
    external_references: Option<Vec<ExternalReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence: Option<ComponentEvidence>,
}

impl From<models::Component> for Component {
    fn from(other: models::Component) -> Self {
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
            purl: other.purl.map(|p| p.0.to_string()),
            swid: convert_optional(other.swid),
            modified: other.modified,
            pedigree: convert_optional(other.pedigree),
            external_references: convert_optional_vec(other.external_references),
            properties: convert_optional(other.properties),
            components: convert_optional_vec(other.components),
            evidence: convert_optional(other.evidence),
        }
    }
}

impl From<Component> for models::Component {
    fn from(other: Component) -> Self {
        Self {
            component_type: models::Classification::new_unchecked(other.component_type),
            mime_type: other.mime_type.map(|m| models::MimeType(m.0)),
            bom_ref: other.bom_ref,
            supplier: convert_optional(other.supplier),
            author: other.author.map(NormalizedString::new_unchecked),
            publisher: other.publisher.map(NormalizedString::new_unchecked),
            group: other.group.map(NormalizedString::new_unchecked),
            name: NormalizedString::new_unchecked(other.name),
            version: NormalizedString::new_unchecked(other.version),
            description: other.description.map(NormalizedString::new_unchecked),
            scope: other.scope.map(models::Scope::new_unchecked),
            hashes: convert_optional(other.hashes),
            licenses: convert_optional(other.licenses),
            copyright: other.copyright.map(NormalizedString::new_unchecked),
            cpe: convert_optional(other.cpe),
            purl: other.purl.map(|p| Purl(Uri(p))),
            swid: convert_optional(other.swid),
            modified: other.modified,
            pedigree: convert_optional(other.pedigree),
            external_references: convert_optional_vec(other.external_references),
            properties: convert_optional(other.properties),
            components: convert_optional_vec(other.components),
            evidence: convert_optional(other.evidence),
        }
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

impl From<models::Swid> for Swid {
    fn from(other: models::Swid) -> Self {
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

impl From<Swid> for models::Swid {
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Cpe(String);

impl From<models::Cpe> for Cpe {
    fn from(other: models::Cpe) -> Self {
        Self(other.0)
    }
}

impl From<Cpe> for models::Cpe {
    fn from(other: Cpe) -> Self {
        Self(other.0)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ComponentEvidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    licenses: Option<Licenses>,
    #[serde(skip_serializing_if = "Option::is_none")]
    copyright: Option<Vec<Copyright>>,
}

impl From<models::ComponentEvidence> for ComponentEvidence {
    fn from(other: models::ComponentEvidence) -> Self {
        Self {
            licenses: convert_optional(other.licenses),
            copyright: convert_optional_vec(other.copyright),
        }
    }
}

impl From<ComponentEvidence> for models::ComponentEvidence {
    fn from(other: ComponentEvidence) -> Self {
        Self {
            licenses: convert_optional(other.licenses),
            copyright: convert_optional_vec(other.copyright),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Pedigree {
    #[serde(skip_serializing_if = "Option::is_none")]
    ancestors: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    descendants: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variants: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commits: Option<Vec<Commit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patches: Option<Vec<Patch>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
}

impl From<models::Pedigree> for Pedigree {
    fn from(other: models::Pedigree) -> Self {
        Self {
            ancestors: convert_optional_vec(other.ancestors),
            descendants: convert_optional_vec(other.descendants),
            variants: convert_optional_vec(other.variants),
            commits: convert_optional_vec(other.commits),
            patches: convert_optional_vec(other.patches),
            notes: other.notes,
        }
    }
}

impl From<Pedigree> for models::Pedigree {
    fn from(other: Pedigree) -> Self {
        Self {
            ancestors: convert_optional_vec(other.ancestors),
            descendants: convert_optional_vec(other.descendants),
            variants: convert_optional_vec(other.variants),
            commits: convert_optional_vec(other.commits),
            patches: convert_optional_vec(other.patches),
            notes: other.notes,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Copyright {
    text: String,
}

impl From<models::Copyright> for Copyright {
    fn from(other: models::Copyright) -> Self {
        Self { text: other.0 }
    }
}

impl From<Copyright> for models::Copyright {
    fn from(other: Copyright) -> Self {
        Self(other.text)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct MimeType(String);

impl From<models::MimeType> for MimeType {
    fn from(other: models::MimeType) -> Self {
        Self(other.0)
    }
}

impl From<MimeType> for models::MimeType {
    fn from(other: MimeType) -> Self {
        Self(other.0)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::specs::v1_3::{
        attached_text::test::{corresponding_attached_text, example_attached_text},
        code::test::{corresponding_commit, corresponding_patch, example_commit, example_patch},
        external_reference::test::{corresponding_external_reference, example_external_reference},
        hash::test::{corresponding_hashes, example_hashes},
        license::test::{corresponding_licenses, example_licenses},
        organization::test::{corresponding_entity, example_entity},
        property::test::{corresponding_properties, example_properties},
    };

    use super::*;

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
            external_references: Some(vec![example_external_reference()]),
            properties: Some(example_properties()),
            components: Some(vec![]),
            evidence: Some(example_evidence()),
        }
    }

    pub(crate) fn corresponding_component() -> models::Component {
        models::Component {
            component_type: models::Classification::UnknownClassification(
                "component type".to_string(),
            ),
            mime_type: Some(models::MimeType("mime type".to_string())),
            bom_ref: Some("bom ref".to_string()),
            supplier: Some(corresponding_entity()),
            author: Some(NormalizedString::new_unchecked("author".to_string())),
            publisher: Some(NormalizedString::new_unchecked("publisher".to_string())),
            group: Some(NormalizedString::new_unchecked("group".to_string())),
            name: NormalizedString::new_unchecked("name".to_string()),
            version: NormalizedString::new_unchecked("version".to_string()),
            description: Some(NormalizedString::new_unchecked("description".to_string())),
            scope: Some(models::Scope::UnknownScope("scope".to_string())),
            hashes: Some(corresponding_hashes()),
            licenses: Some(corresponding_licenses()),
            copyright: Some(NormalizedString::new_unchecked("copyright".to_string())),
            cpe: Some(corresponding_cpe()),
            purl: Some(Purl(Uri("purl".to_string()))),
            swid: Some(corresponding_swid()),
            modified: Some(true),
            pedigree: Some(corresponding_pedigree()),
            external_references: Some(vec![corresponding_external_reference()]),
            properties: Some(corresponding_properties()),
            components: Some(vec![]),
            evidence: Some(corresponding_evidence()),
        }
    }

    fn example_cpe() -> Cpe {
        Cpe("cpe".to_string())
    }

    fn corresponding_cpe() -> models::Cpe {
        models::Cpe("cpe".to_string())
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

    fn corresponding_swid() -> models::Swid {
        models::Swid {
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
            ancestors: Some(vec![]),
            descendants: Some(vec![]),
            variants: Some(vec![]),
            commits: Some(vec![example_commit()]),
            patches: Some(vec![example_patch()]),
            notes: Some("notes".to_string()),
        }
    }

    fn corresponding_pedigree() -> models::Pedigree {
        models::Pedigree {
            ancestors: Some(vec![]),
            descendants: Some(vec![]),
            variants: Some(vec![]),
            commits: Some(vec![corresponding_commit()]),
            patches: Some(vec![corresponding_patch()]),
            notes: Some("notes".to_string()),
        }
    }

    fn example_evidence() -> ComponentEvidence {
        ComponentEvidence {
            licenses: Some(example_licenses()),
            copyright: Some(vec![example_copyright()]),
        }
    }

    fn corresponding_evidence() -> models::ComponentEvidence {
        models::ComponentEvidence {
            licenses: Some(corresponding_licenses()),
            copyright: Some(vec![corresponding_copyright()]),
        }
    }

    fn example_copyright() -> Copyright {
        Copyright {
            text: "copyright".to_string(),
        }
    }

    fn corresponding_copyright() -> models::Copyright {
        models::Copyright("copyright".to_string())
    }
}
