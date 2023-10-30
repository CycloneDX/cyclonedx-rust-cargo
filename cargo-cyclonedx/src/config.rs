use std::str::FromStr;

use thiserror::Error;

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

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SbomConfig {
    pub format: Option<Format>,
    pub included_dependencies: Option<IncludedDependencies>,
    pub output_options: Option<OutputOptions>,
    pub features: Option<Features>,
    pub target: Option<Target>,
}

impl SbomConfig {
    pub fn empty_config() -> Self {
        Default::default()
    }

    /// The config passed as an argument takes priority over `Self`
    pub fn merge(&self, other: &SbomConfig) -> SbomConfig {
        SbomConfig {
            format: other.format.or(self.format),
            included_dependencies: other.included_dependencies.or(self.included_dependencies),
            output_options: other
                .output_options
                .clone()
                .or_else(|| self.output_options.clone()),
            features: other.features.clone().or_else(|| self.features.clone()),
            target: other.target.clone().or_else(|| self.target.clone()),
        }
    }

    pub fn format(&self) -> Format {
        self.format.unwrap_or_default()
    }

    pub fn included_dependencies(&self) -> IncludedDependencies {
        self.included_dependencies.unwrap_or_default()
    }

    pub fn output_options(&self) -> OutputOptions {
        self.output_options.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncludedDependencies {
    TopLevelDependencies,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputOptions {
    pub cdx_extension: CdxExtension,
    pub prefix: Prefix,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            cdx_extension: CdxExtension::default(),
            prefix: Prefix::Pattern(Pattern::Bom),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdxExtension {
    Included,
    NotIncluded,
}

impl CdxExtension {
    pub fn extension(&self) -> String {
        match &self {
            CdxExtension::Included => ".cdx".to_string(),
            CdxExtension::NotIncluded => "".to_string(),
        }
    }
}

impl Default for CdxExtension {
    fn default() -> Self {
        Self::NotIncluded
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Features {
    pub all_features: bool,
    pub no_default_features: bool,
    pub features: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Target {
    #[default]
    AllTargets,
    SingleTarget(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prefix {
    Pattern(Pattern),
    Custom(CustomPrefix),
}

impl Default for Prefix {
    fn default() -> Self {
        Self::Pattern(Pattern::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    Bom,
    Package,
}

impl Default for Pattern {
    fn default() -> Self {
        Self::Bom
    }
}

impl FromStr for Pattern {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bom" => Ok(Self::Bom),
            "package" => Ok(Self::Package),
            _ => Err(format!("Expected bom or package, got `{}`", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomPrefix(String);

impl CustomPrefix {
    pub fn new(custom_prefix: impl Into<String>) -> Result<Self, PrefixError> {
        let prefix = custom_prefix.into();

        if prefix.contains(std::path::MAIN_SEPARATOR) {
            Err(PrefixError::CustomPrefixError(
                std::path::MAIN_SEPARATOR.to_string(),
            ))
        } else {
            Ok(Self(prefix))
        }
    }
}

impl ToString for CustomPrefix {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PrefixError {
    #[error("Illegal characters in custom prefix string: {0}")]
    CustomPrefixError(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_error_for_a_prefix_with_a_path_separator() {
        let prefix = format!("directory{}prefix", std::path::MAIN_SEPARATOR);

        let actual = CustomPrefix::new(prefix)
            .expect_err("Should not have been able to create CustomPrefix with path separator");

        let expected = PrefixError::CustomPrefixError(std::path::MAIN_SEPARATOR.to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_create_a_custom_prefix_from_a_valid_string() {
        let prefix = "customprefix".to_string();

        let actual = CustomPrefix::new(prefix.clone())
            .expect("Should have been able to create CustomPrefix");

        let expected = CustomPrefix(prefix);

        assert_eq!(actual, expected);
    }
}
