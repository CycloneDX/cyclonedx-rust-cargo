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

use cargo::core::Package;
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::traits::ToXml;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub struct License<'a> {
    pub expression: &'a str,
}

impl<'a> From<&'a Package> for License<'a> {
    fn from(pkg: &'a Package) -> Self {
        Self {
            expression: &pkg.manifest().metadata().license.as_ref().unwrap(),
        }
    }
}

impl<'a> From<&'a cargo_metadata::Package> for License<'a> {
    fn from(pkg: &'a cargo_metadata::Package) -> Self {
        Self {
            expression: &pkg.license.as_ref().unwrap(),
        }
    }
}

impl ToXml for License<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("license")?;
        match self {
            expr => {
                xml.begin_elem("expression")?;
                xml.text(expr.expression.trim())?;
                xml.end_elem()?;
            }
        }
        xml.end_elem()
    }
}

impl ToXml for Vec<License<'_>> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        if self.len() > 0 {
            xml.begin_elem("licenses")?;

            for license in self {
                license.to_xml(xml)?;
            }

            xml.end_elem()?;
        }

        Ok(())
    }
}
