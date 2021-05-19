use std::io;

use cargo::core::{Package, TargetKind};
use chrono::{DateTime, Utc};
use serde::Serialize;
use xml_writer::XmlWriter;

use crate::{author::Authors, Component, IsEmpty, ToXml};

#[derive(Serialize)]
pub struct Metadata<'a> {
    timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    authors: Option<Authors>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<Component<'a>>,
}

impl<'a> Default for Metadata<'a> {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            authors: None,
            component: None,
        }
    }
}

impl<'a> From<&'a Package> for Metadata<'a> {
    fn from(pkg: &'a Package) -> Self {
        Self {
            authors: Some(Authors::from(pkg)),
            component: Some(if could_be_application(pkg) {
                Component::application(pkg).without_scope()
            } else {
                Component::library(pkg).without_scope()
            }),
            ..Default::default()
        }
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

        if let Some(authors) = &self.authors {
            authors.to_xml(xml)?;
        }

        if let Some(component) = &self.component {
            component.to_xml(xml)?;
        }

        xml.end_elem()
    }
}

/// Check if `pkg` might be an executable application based on the presence of binary targets.
fn could_be_application(pkg: &Package) -> bool {
    pkg.targets()
        .iter()
        .any(|tgt| *tgt.kind() == TargetKind::Bin)
}
