use std::{fmt, str::FromStr};

/// Output format for CycloneDX BOM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Xml,
}

impl Default for Format {
    fn default() -> Self {
        Self::Xml
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Json => "json".fmt(f),
            Format::Xml => "xml".fmt(f),
        }
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "xml" => Ok(Self::Xml),
            "json" => Ok(Self::Json),
            _ => Err(format!("Expected xml or json, got `{}`", s)),
        }
    }
}
