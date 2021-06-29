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
use std::io;
use xml_writer::XmlWriter;

use cargo::core::{Package, TargetKind};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::str::FromStr;
use log::{debug};

use crate::author::Author;
use crate::component::Component;
use crate::traits::ToXml;

#[derive(Serialize)]
pub struct Metadata<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<Component<'a>>,
}

impl<'a> Default for Metadata<'a> {
    fn default() -> Self {
        Self {
            timestamp: Some(Utc::now()),
            authors: None::<Vec<Author>>,
            component: None::<Component>,
        }
    }
}

impl<'a> From<&'a Package> for Metadata<'a> {
    fn from(pkg: &'a Package) -> Self {
        Self {
            authors: get_authors_from_package(pkg),
            component: Some(if could_be_application(pkg) {
                Component::application(pkg).without_scope()
            } else {
                Component::library(pkg).without_scope()
            }),
            ..Default::default()
        }
    }
}

impl<'a> From<&'a cargo_metadata::Package> for Metadata<'a> {
    fn from(package: &'a cargo_metadata::Package) -> Self {
        Self {
            authors: get_authors_from_package_cm(package),
            component: Some(if is_an_application(package) {
                Component::application_cm(package).without_scope()
            } else {
                Component::library_cm(package).without_scope()
            }),
            ..Default::default()
        }
    }
}

impl ToXml for Metadata<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("metadata")?;

        if let Some(timestamp) = &self.timestamp {
            xml.elem_text(
                "timestamp",
                &timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )?;
        }

        if let Some(authors) = &self.authors {
            authors.to_xml(xml)?;
        }

        if let Some(component) = &self.component {
            component.to_xml(xml)?;
        }

        xml.end_elem()
    }
}

fn get_authors_from_package<'a>(package: &'a Package) -> Option<Vec<Author>> {
    let mut authors = Vec::new();
    let mut invalid_authors = Vec::new();
    
    for author in package.authors() {
        match Author::from_str(&author) {
            Ok(author) => authors.push(author),
            Err(e) => invalid_authors.push((author, e)),
        }
    }
    invalid_authors.into_iter().for_each(|(author, error)| debug!("Invalid author {}: {:?}", author, error));

    if authors.len() > 0 {
        return Some(authors);
    }

    None
}

fn get_authors_from_package_cm<'a>(package: &'a cargo_metadata::Package) -> Option<Vec<Author>> {
    let mut authors = Vec::new();
    let mut invalid_authors = Vec::new();
    
    for author in &package.authors {
        match Author::from_str(&author) {
            Ok(author) => authors.push(author),
            Err(e) => invalid_authors.push((author, e)),
        }
    }
    invalid_authors.into_iter().for_each(|(author, error)| debug!("Invalid author {}: {:?}", author, error));

    if authors.len() > 0 {
        return Some(authors);
    }

    None
}

/// Check if `pkg` might be an executable application based on the presence of binary targets.
fn could_be_application(pkg: &Package) -> bool {
    pkg.targets()
        .iter()
        .any(|tgt| *tgt.kind() == TargetKind::Bin)
}

/// Is the `package` an application? This will tell us!
fn is_an_application(pkg: &cargo_metadata::Package) -> bool {
    let mut kinds = pkg
        .targets
        .iter()
        .map(|target| target.clone().kind)
        .flatten();

    kinds.any(|kind| kind == "bin")
}
