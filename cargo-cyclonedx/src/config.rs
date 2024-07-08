use cyclonedx_bom::models::bom::SpecVersion;
use serde::Deserialize;
use std::collections::HashSet;
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SbomConfig {
    pub format: Option<Format>,
    pub included_dependencies: Option<IncludedDependencies>,
    pub output_options: Option<OutputOptions>,
    pub features: Option<Features>,
    pub target: Option<Target>,
    pub license_parser: Option<LicenseParserOptions>,
    pub describe: Option<Describe>,
    pub spec_version: Option<SpecVersion>,
    pub only_normal_deps: Option<bool>,
}

impl SbomConfig {
    pub fn empty_config() -> Self {
        Default::default()
    }

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
            license_parser: other
                .license_parser
                .clone()
                .map(|other| self.license_parser.clone().unwrap_or_default().merge(other))
                .or_else(|| self.license_parser.clone()),
            describe: other.describe.or(self.describe),
            spec_version: other.spec_version.or(self.spec_version),
            only_normal_deps: other.only_normal_deps.or(self.only_normal_deps),
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

    pub fn license_parser(&self) -> LicenseParserOptions {
        self.license_parser.clone().unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum IncludedDependencies {
    TopLevelDependencies,
    #[default]
    AllDependencies,
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OutputOptions {
    pub filename: FilenamePattern,
    pub platform_suffix: PlatformSuffix,
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

impl Target {
    pub fn as_str(&self) -> &str {
        match self {
            Target::AllTargets => "all",
            Target::SingleTarget(target) => target.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilenamePattern {
    CrateName,
    Custom(FilenameOverride),
}

impl Default for FilenamePattern {
    fn default() -> Self {
        Self::CrateName
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    #[default]
    Bom,
    Package,
    Binary,
    /// Not to be confused with a compilation target:
    /// https://doc.rust-lang.org/cargo/reference/cargo-targets.html
    CargoTarget,
}

impl FromStr for Pattern {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bom" => Ok(Self::Bom),
            "package" => Ok(Self::Package),
            "binary" => Ok(Self::Binary),
            "cargo-target" => Ok(Self::CargoTarget),
            _ => Err(format!("Expected bom or package, got `{}`", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilenameOverride(String);

impl FilenameOverride {
    pub fn new(custom_prefix: impl Into<String>) -> Result<Self, FilenameOverrideError> {
        let prefix = custom_prefix.into();

        if prefix.contains(std::path::MAIN_SEPARATOR) {
            Err(FilenameOverrideError::TheOne(
                std::path::MAIN_SEPARATOR.to_string(),
            ))
        } else {
            Ok(Self(prefix))
        }
    }
}

impl std::fmt::Display for FilenameOverride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FilenameOverrideError {
    #[error("Illegal characters in custom prefix string: {0}")]
    TheOne(String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum PlatformSuffix {
    Included,
    #[default]
    NotIncluded,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct LicenseParserOptions {
    /// Use lax or strict parsing
    #[serde(default)]
    pub mode: ParseMode,

    /// Silently accept the named licenses
    #[serde(default)]
    pub accept_named: HashSet<String>,
}

impl LicenseParserOptions {
    pub fn merge(mut self, other: Self) -> Self {
        Self {
            mode: other.mode,
            accept_named: {
                self.accept_named.extend(other.accept_named);
                self.accept_named
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub enum ParseMode {
    /// Parse licenses in strict mode
    Strict,
    /// Parse licenses in lax mode
    #[default]
    Lax,
}

/// What does the SBOM describe?
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Describe {
    /// Describe the entire crate in a single SBOM file, with Cargo targets as subcomponents. (default)
    #[default]
    Crate,
    /// A separate SBOM is emitted for each binary (bin, cdylib) while all other targets are ignored
    Binaries,
    /// A separate SBOM is emitted for each Cargo target, including things that aren't directly executable (e.g rlib)
    AllCargoTargets,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_error_for_a_filename_with_a_path_separator() {
        let filename = format!("directory{}filename", std::path::MAIN_SEPARATOR);

        let actual = FilenameOverride::new(filename)
            .expect_err("Should not have been able to create Customfilename with path separator");

        let expected = FilenameOverrideError::TheOne(std::path::MAIN_SEPARATOR.to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_create_a_custom_filename_from_a_valid_string() {
        let filename = "customfilename".to_string();

        let actual = FilenameOverride::new(filename.clone())
            .expect("Should have been able to create Customfilename");

        let expected = FilenameOverride(filename);

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_merge_license_names() {
        let config_1 = SbomConfig {
            license_parser: Some(LicenseParserOptions {
                mode: ParseMode::Strict,
                accept_named: ["Foo".into()].into(),
            }),
            ..Default::default()
        };
        let config_2 = SbomConfig {
            license_parser: Some(LicenseParserOptions {
                mode: ParseMode::Lax,
                accept_named: ["Bar".into()].into(),
            }),
            ..Default::default()
        };

        let config = config_1.merge(&config_2);

        assert_eq!(
            config,
            SbomConfig {
                license_parser: Some(LicenseParserOptions {
                    mode: ParseMode::Lax,
                    accept_named: ["Foo".into(), "Bar".into()].into(),
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn it_should_keep_strict() {
        let config_1 = SbomConfig {
            license_parser: Some(LicenseParserOptions {
                mode: ParseMode::Strict,
                accept_named: ["Foo".into()].into(),
            }),
            ..Default::default()
        };
        let config_2 = SbomConfig::default();

        let config = config_1.merge(&config_2);

        assert_eq!(
            config,
            SbomConfig {
                license_parser: Some(LicenseParserOptions {
                    mode: ParseMode::Strict,
                    accept_named: ["Foo".into()].into(),
                }),
                ..Default::default()
            }
        );
    }
}
