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

use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::traits::ToXml;

pub struct ExternalReferenceError;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"^([a-z0-9+.-]+):(?://(?:((?:[a-z0-9-._~!$&'()*+,;=:]|%[0-9A-F]{2})*)@)?((?:[a-z0-9-._~!$&'()*+,;=]|%[0-9A-F]{2})*)(?::(\d*))?(/(?:[a-z0-9-._~!$&'()*+,;=:@/]|%[0-9A-F]{2})*)?|(/?(?:[a-z0-9-._~!$&'()*+,;=:@]|%[0-9A-F]{2})+(?:[a-z0-9-._~!$&'()*+,;=:@/]|%[0-9A-F]{2})*)?)(?:\?((?:[a-z0-9-._~!$&'()*+,;=:/?@]|%[0-9A-F]{2})*))?(?:#((?:[a-z0-9-._~!$&'()*+,;=:/?@]|%[0-9A-F]{2})*))?$").unwrap();
}

#[derive(Serialize)]
/// A reference to external materials, such as documentation.
pub struct ExternalReference {
    #[serde(rename = "type")]
    pub ref_type: String,
    pub url: String,
}

impl<'a> ExternalReference {
    pub fn new(ref_type: &'a str, url: &'a str) -> Result<Self, ExternalReferenceError> {
        if URL_REGEX.is_match(url) {
            Ok(Self {
                ref_type: ref_type.to_string(),
                url: url.to_string(),
            })
        } else {
            Err(ExternalReferenceError)
        }
    }
}

impl ToXml for ExternalReference {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("reference")?;
        xml.attr("type", &self.ref_type)?;
        xml.begin_elem("url")?;
        // XXX is this trim() needed? The regex doesn't permit leading or trailing whitespace.
        xml.text(self.url.trim())?;
        xml.end_elem()?;
        xml.end_elem()
    }
}

impl ToXml for Vec<ExternalReference> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        if self.len() > 0 {
            xml.begin_elem("externalReferences")?;
            for reference in self {
                reference.to_xml(xml)?;
            }
            xml.end_elem()?;
        }

        Ok(())
    }
}
