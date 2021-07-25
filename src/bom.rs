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
use std::{io, iter::FromIterator};

use cargo::core::Package;
use serde::{Serialize, Serializer};
use uuid::Uuid;
use xml_writer::XmlWriter;

use crate::{Component, ToXml};

static SPEC_VERSION: &'static str = "1.3";

#[derive(Clone, Copy, Serialize)]
enum BomFormat {
    CycloneDX,
}

fn uuid_to_urn<S: Serializer>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&uuid.to_urn())
}

/// A software bill of materials for a Rust crate.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bom<'a> {
    bom_format: BomFormat,
    spec_version: &'static str,
    #[serde(serialize_with = "uuid_to_urn")]
    serial_number: Uuid,
    version: u32,
    components: Vec<Component<'a>>,
}

/// Create a new BOM from a sequence of cargo package references.
impl<'a> FromIterator<&'a Package> for Bom<'a> {
    fn from_iter<T: IntoIterator<Item = &'a Package>>(iter: T) -> Self {
        Self {
            bom_format: BomFormat::CycloneDX,
            spec_version: SPEC_VERSION,
            version: 1,
            serial_number: Uuid::new_v4(),
            components: iter.into_iter().map(Component::library).collect(),
        }
    }
}

impl ToXml for Bom<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        let namespace = format!("http://cyclonedx.org/schema/bom/{}", SPEC_VERSION);
        xml.dtd("UTF-8")?;
        xml.begin_elem("bom")?;
        xml.attr("serialNumber", &self.serial_number.to_urn().to_string())?;
        xml.attr("version", "1")?;
        xml.attr("xmlns", namespace.as_str())?;

        xml.begin_elem("components")?;
        for component in &self.components {
            component.to_xml(xml)?;
        }
        xml.end_elem()?;

        xml.end_elem()
    }
}