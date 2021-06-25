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
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::traits::ToXml;

#[derive(Serialize)]
pub struct ExternalReferences<'a>(Vec<ExternalReference<'a>>);

impl<'a> ExternalReferences<'a> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> From<&'a Package> for ExternalReferences<'a> {
    fn from(v: &'a Package) -> Self {
        fn ext_ref<'a>(
            ref_type: &'a str,
            uri: &'a Option<String>,
        ) -> Option<ExternalReference<'a>> {
            ExternalReference::new(ref_type, uri.as_ref()?).ok()
        }

        let metadata = v.manifest().metadata();
        Self(
            ext_ref("documentation", &metadata.documentation)
                .into_iter()
                .chain(ext_ref("website", &metadata.homepage))
                .chain(ext_ref("other", &metadata.links))
                .chain(ext_ref("vcs", &metadata.repository))
                .collect(),
        )
    }
}

impl ToXml for ExternalReferences<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        if !self.0.is_empty() {
            xml.begin_elem("externalReferences")?;
            for reference in &self.0 {
                reference.to_xml(xml)?;
            }
            xml.end_elem()?;
        }

        Ok(())
    }
}

pub struct ExternalReferenceError;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"^([a-z0-9+.-]+):(?://(?:((?:[a-z0-9-._~!$&'()*+,;=:]|%[0-9A-F]{2})*)@)?((?:[a-z0-9-._~!$&'()*+,;=]|%[0-9A-F]{2})*)(?::(\d*))?(/(?:[a-z0-9-._~!$&'()*+,;=:@/]|%[0-9A-F]{2})*)?|(/?(?:[a-z0-9-._~!$&'()*+,;=:@]|%[0-9A-F]{2})+(?:[a-z0-9-._~!$&'()*+,;=:@/]|%[0-9A-F]{2})*)?)(?:\?((?:[a-z0-9-._~!$&'()*+,;=:/?@]|%[0-9A-F]{2})*))?(?:#((?:[a-z0-9-._~!$&'()*+,;=:/?@]|%[0-9A-F]{2})*))?$").unwrap();
}

#[derive(Serialize)]
/// A reference to external materials, such as documentation.
pub struct ExternalReference<'a> {
    #[serde(rename = "type")]
    ref_type: &'a str,
    url: &'a str,
}

impl<'a> ExternalReference<'a> {
    pub fn new(ref_type: &'a str, url: &'a str) -> Result<Self, ExternalReferenceError> {
        if URL_REGEX.is_match(url) {
            Ok(Self { ref_type, url })
        } else {
            Err(ExternalReferenceError)
        }
    }
}

impl ToXml for ExternalReference<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("reference")?;
        xml.attr("type", self.ref_type)?;
        xml.begin_elem("url")?;
        // XXX is this trim() needed? The regex doesn't permit leading or trailing whitespace.
        xml.text(self.url.trim())?;
        xml.end_elem()?;
        xml.end_elem()
    }
}
