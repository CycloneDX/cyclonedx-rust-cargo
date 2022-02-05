use std::str::FromStr;

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
use crate::format::Format;
use serde::Deserialize;
use thiserror::Error;

pub fn config_from_toml(value: &toml::Value) -> Result<SbomConfig, ConfigError> {
    let wrapper: Result<ConfigWrapper, _> = value.clone().try_into();

    wrapper
        .map(|w| w.cyclonedx.unwrap_or(SbomConfig::empty_config()))
        .map_err(|e| ConfigError::TomlConfigError(format!("{}", e)))
}

#[derive(Debug, Deserialize, PartialEq)]
struct ConfigWrapper {
    cyclonedx: Option<SbomConfig>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SbomConfig {
    pub format: Option<Format>,
    pub included_dependencies: Option<IncludedDependencies>,
}

impl SbomConfig {
    pub fn empty_config() -> Self {
        Self {
            format: None,
            included_dependencies: None,
        }
    }

    pub fn merge(&self, other: &SbomConfig) -> SbomConfig {
        SbomConfig {
            format: other.format.or(self.format),
            included_dependencies: other.included_dependencies.or(self.included_dependencies),
        }
    }

    pub fn format(&self) -> Format {
        self.format.unwrap_or_default()
    }

    pub fn included_dependencies(&self) -> IncludedDependencies {
        self.included_dependencies.unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum IncludedDependencies {
    #[serde(rename(deserialize = "top-level"))]
    TopLevelDependencies,
    #[serde(rename(deserialize = "all"))]
    AllDependencies,
}

impl Default for IncludedDependencies {
    fn default() -> Self {
        Self::TopLevelDependencies
    }
}

impl FromStr for IncludedDependencies {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::AllDependencies),
            "top-level" => Ok(Self::TopLevelDependencies),
            _ => Err(format!("Expected all or top-level, got `{}`", s)),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to deserialize configuration from Toml: {0}")]
    TomlConfigError(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_deserialize_from_toml_value() {
        let toml = r#"
[cyclonedx]
format = "json"
included_dependencies = "top-level"
"#;

        let actual: ConfigWrapper = toml::from_str(toml).expect("Failed to parse toml");

        let expected = SbomConfig {
            format: Some(Format::Json),
            included_dependencies: Some(IncludedDependencies::TopLevelDependencies),
        };

        assert_eq!(actual.cyclonedx, Some(expected));
    }

    #[test]
    fn it_should_ignore_other_packages_from_toml_value() {
        let toml = r#"
[notourpackage]
format = "json"
included_dependencies = "top-level"
"#;

        let actual: ConfigWrapper = toml::from_str(toml).expect("Failed to parse toml");

        assert_eq!(actual.cyclonedx, None);
    }
}
