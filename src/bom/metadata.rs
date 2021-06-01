use std::{io, marker::PhantomData};

use cargo::core::Package;
use chrono::{DateTime, Utc};
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::ToXml;

#[derive(Serialize)]
pub struct Metadata<'a> {
    timestamp: DateTime<Utc>,
    #[serde(skip)]
    temp: PhantomData<&'a ()>,
}

impl<'a> Default for Metadata<'a> {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            temp: PhantomData::default(),
        }
    }
}

impl<'a> From<&'a Package> for Metadata<'a> {
    fn from(_pkg: &'a Package) -> Self {
        Default::default()
    }
}

impl ToXml for Metadata<'_> {
    fn to_xml<W: io::Write>(&self, xml: &mut XmlWriter<W>) -> io::Result<()> {
        xml.begin_elem("metadata")?;

        xml.elem_text(
            "timestamp",
            &self
                .timestamp
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        )?;

        xml.end_elem()
    }
}
