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
use std::{convert::TryFrom, io};

use cargo::core::Package;
use serde::Serialize;
use thiserror::Error;
use xml_writer::XmlWriter;

use crate::traits::ToXml;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub struct License {
    pub expression: String,
}

impl<'a> TryFrom<&'a Package> for License {
    type Error = LicenseError;

    fn try_from(pkg: &'a Package) -> Result<Self, Self::Error> {
        let expression = pkg
            .manifest()
            .metadata()
            .license
            .as_ref()
            .ok_or(LicenseError::NoLicenseProvidedError)?
            .to_string();
        Ok(Self { expression })
    }
}

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("No license was found in the package manifest")]
    NoLicenseProvidedError,
}

impl<'a> TryFrom<&'a cargo_metadata::Package> for License {
    type Error = LicenseError;

    fn try_from(pkg: &'a cargo_metadata::Package) -> Result<Self, Self::Error> {
        let expression = pkg
            .license
            .as_ref()
            .ok_or(LicenseError::NoLicenseProvidedError)?
            .to_string();
        Ok(Self { expression })
    }
}

impl ToXml for License {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("license")?;
        xml.begin_elem("expression")?;
        xml.text(self.expression.trim())?;
        xml.end_elem()?;
        xml.end_elem()
    }
}

impl ToXml for Vec<License> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        if !self.is_empty() {
            xml.begin_elem("licenses")?;

            for license in self {
                license.to_xml(xml)?;
            }

            xml.end_elem()?;
        }

        Ok(())
    }
}
