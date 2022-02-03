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
use serde::Deserialize;
use std::{fmt, str::FromStr};

/// Output format for CycloneDX BOM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
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
