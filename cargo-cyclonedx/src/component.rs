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
 * Copyright (c) OWASP Foundation. All Rights Reserved.
 */
use std::{convert::TryFrom, fmt, io};

use cargo::core::Package;
use packageurl::PackageUrl;
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::{license::LicenseError, traits::ToXml};

use crate::license::License;
use crate::reference::ExternalReference;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComponentType {
    Application,
    Library,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Application => "application".fmt(f),
            Self::Library => "library".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    Required,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Required => "required".fmt(f),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    #[serde(flatten)]
    pub metadata: ComponentCommon,
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licenses: Option<Vec<License>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_references: Option<Vec<ExternalReference>>,
}

impl<'a> Component {
    /// Create a component which describes the package as a library.
    pub fn library(pkg: &'a Package) -> Self {
        let licenses = match License::try_from(pkg) {
            Ok(license) => vec![license],
            Err(LicenseError::NoLicenseProvidedError) => {
                log::trace!("Library did not have a license");
                Vec::new()
            }
        };
        Self {
            component_type: ComponentType::Library,
            scope: Some(Scope::Required),
            metadata: ComponentCommon::from(pkg),
            licenses: Some(licenses),
            external_references: get_external_references(pkg),
        }
    }

    pub fn library_cm(package: &'a cargo_metadata::Package) -> Self {
        let licenses = match License::try_from(package) {
            Ok(license) => vec![license],
            Err(LicenseError::NoLicenseProvidedError) => {
                log::trace!("Library CM did not have a license");
                Vec::new()
            }
        };
        Self {
            component_type: ComponentType::Library,
            external_references: get_external_references_cm(package),
            licenses: Some(licenses),
            metadata: ComponentCommon::from(package),
            scope: Some(Scope::Required),
        }
    }

    /// Create a component which describes the package as an application.
    pub fn application(pkg: &'a Package) -> Self {
        let licenses = match License::try_from(pkg) {
            Ok(license) => vec![license],
            Err(LicenseError::NoLicenseProvidedError) => {
                log::trace!("Application did not have a license");
                Vec::new()
            }
        };
        Self {
            component_type: ComponentType::Application,
            scope: Some(Scope::Required),
            metadata: ComponentCommon::from(pkg),
            licenses: Some(licenses),
            external_references: get_external_references(pkg),
        }
    }

    pub fn application_cm(package: &'a cargo_metadata::Package) -> Self {
        let licenses = match License::try_from(package) {
            Ok(license) => vec![license],
            Err(LicenseError::NoLicenseProvidedError) => {
                log::trace!("Application CM did not have a license");
                Vec::new()
            }
        };
        Self {
            component_type: ComponentType::Application,
            external_references: get_external_references_cm(package),
            licenses: Some(licenses),
            metadata: ComponentCommon::from(package),
            scope: Some(Scope::Required),
        }
    }

    /// Remove the `scope` value.
    pub fn without_scope(mut self) -> Self {
        self.scope = None;
        self
    }
}

impl<'a> From<&'a Package> for Component {
    fn from(package: &'a Package) -> Self {
        let licenses = match License::try_from(package) {
            Ok(license) => vec![license],
            Err(LicenseError::NoLicenseProvidedError) => {
                log::trace!("Package did not have a license");
                Vec::new()
            }
        };
        Self {
            component_type: ComponentType::Library,
            scope: Some(Scope::Required),
            metadata: ComponentCommon::from(package),
            licenses: Some(licenses),
            external_references: get_external_references(package),
        }
    }
}

impl<'a> From<&'a cargo_metadata::Package> for Component {
    fn from(package: &'a cargo_metadata::Package) -> Self {
        let licenses = match License::try_from(package) {
            Ok(license) => vec![license],
            Err(LicenseError::NoLicenseProvidedError) => {
                log::trace!("Package did not have a license");
                Vec::new()
            }
        };
        Self {
            component_type: ComponentType::Library,
            external_references: get_external_references_cm(package),
            licenses: Some(licenses),
            metadata: ComponentCommon::from(package),
            scope: Some(Scope::Required),
        }
    }
}

impl ToXml for Component {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("component")?;
        xml.attr("type", &self.component_type.to_string())?;

        self.metadata.to_xml(xml)?;

        if let Some(scope) = &self.scope {
            xml.elem_text("scope", &scope.to_string())?;
        }

        //TODO: Add hashes. May require file components and manual calculation of all files

        if let Some(licenses) = &self.licenses {
            licenses.to_xml(xml)?;
        }

        if let Some(external_references) = &self.external_references {
            external_references.to_xml(xml)?;
        }

        xml.end_elem()
    }
}

#[derive(Serialize)]
pub struct ComponentCommon {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub purl: Option<String>,
}

impl<'a> From<&'a Package> for ComponentCommon {
    fn from(package: &'a Package) -> Self {
        let name = package.name().to_owned().trim().to_string();
        let version = package.version().to_string();
        let purl = create_purl(&name, &version);

        Self {
            name,
            purl,
            version,
            description: package
                .manifest()
                .metadata()
                .description
                .as_ref()
                .map(|s| s.to_string()),
        }
    }
}

impl<'a> From<&'a cargo_metadata::Package> for ComponentCommon {
    fn from(package: &'a cargo_metadata::Package) -> Self {
        let name = package.name.trim().to_string();
        let version = package.version.to_string();
        let purl = create_purl(&name, &version);

        Self {
            name,
            purl,
            version,
            description: package.description.as_ref().map(|s| s.to_string()),
        }
    }
}

fn create_purl(name: &str, version: &str) -> Option<String> {
    // PackageUrl::new performs validation on the "type" value passed in. Since we hard-code this value to "cargo", this should always pass.
    // Setting this to gracefully log and handle the potential future case where the "name" field is validated as well.
    // Reference: https://github.com/althonos/packageurl.rs/blob/0.3.0/src/validation.rs#L2
    let purl = PackageUrl::new("cargo", name);
    match purl {
        Ok(mut purl) => {
            let output = purl.with_version(version.trim()).to_string();
            Some(output)
        }
        Err(e) => {
            log::debug!("Error processing purl: {}", e);
            None
        }
    }
}

impl ToXml for ComponentCommon {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("name")?;
        xml.text(&self.name)?;
        xml.end_elem()?;

        xml.begin_elem("version")?;
        xml.text(self.version.trim())?;
        xml.end_elem()?;

        if let Some(x) = &self.description {
            xml.begin_elem("description")?;
            xml.cdata(x.trim())?;
            xml.end_elem()?;
        }
        if let Some(purl) = &self.purl {
            xml.begin_elem("purl")?;
            xml.text(purl)?;
            xml.end_elem()?;
        }

        Ok(())
    }
}

// Moved to component.rs because references need not be aware of a package, but a component that wants to create external references can be
fn get_external_references(package: &Package) -> Option<Vec<ExternalReference>> {
    let mut references = Vec::new();

    let metadata = package.manifest().metadata();

    if let Some(documentation) = &metadata.documentation {
        references.push(ExternalReference {
            ref_type: "documentation".to_string(),
            url: documentation.to_string(),
        });
    }

    if let Some(website) = &metadata.homepage {
        references.push(ExternalReference {
            ref_type: "website".to_string(),
            url: website.to_string(),
        });
    }

    if let Some(other) = &metadata.links {
        references.push(ExternalReference {
            ref_type: "other".to_string(),
            url: other.to_string(),
        });
    }

    if let Some(vcs) = &metadata.repository {
        references.push(ExternalReference {
            ref_type: "vcs".to_string(),
            url: vcs.to_string(),
        });
    }

    if !references.is_empty() {
        return Some(references);
    }

    None
}

// Duplicate of the above fn get_external_references, largely just for parsing `cargo_metadata::Package`
fn get_external_references_cm(package: &cargo_metadata::Package) -> Option<Vec<ExternalReference>> {
    let mut references = Vec::new();

    if let Some(documentation) = &package.documentation {
        references.push(ExternalReference {
            ref_type: "documentation".to_string(),
            url: documentation.to_string(),
        });
    }

    if let Some(website) = &package.homepage {
        references.push(ExternalReference {
            ref_type: "website".to_string(),
            url: website.to_string(),
        });
    }

    if let Some(other) = &package.links {
        references.push(ExternalReference {
            ref_type: "other".to_string(),
            url: other.to_string(),
        });
    }

    if let Some(vcs) = &package.repository {
        references.push(ExternalReference {
            ref_type: "vcs".to_string(),
            url: vcs.to_string(),
        });
    }

    if !references.is_empty() {
        return Some(references);
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn purl_construction() {
        assert_eq!(
            create_purl("test-library", " 0.0.1 "),
            Some("pkg:cargo/test-library@0.0.1".to_string())
        )
    }
}
