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

use std::collections::HashSet;

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct Dependencies(Vec<Dependency>);

impl From<models::Dependencies> for Dependencies {
    fn from(other: models::Dependencies) -> Self {
        let mut dependencies_to_process = other.0;
        let mut flat_dependencies = HashSet::new();

        while let Some(dependency) = dependencies_to_process.pop() {
            flat_dependencies.insert(Dependency {
                dependency_ref: dependency.dependency_ref,
                depends_on: dependency
                    .dependencies
                    .iter()
                    .map(|d| d.dependency_ref.clone())
                    .collect(),
            });
            for sub_dependency in dependency.dependencies {
                if !sub_dependency.dependencies.is_empty() {
                    dependencies_to_process.push(sub_dependency)
                }
            }
        }

        let mut flat_dependencies: Vec<_> = flat_dependencies.into_iter().collect();
        flat_dependencies.sort_by_key(|d| d.dependency_ref.clone());

        Self(flat_dependencies)
    }
}

impl From<Dependencies> for models::Dependencies {
    fn from(other: Dependencies) -> Self {
        Self(other.0.into_iter().map(std::convert::Into::into).collect())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Dependency {
    #[serde(rename = "ref")]
    dependency_ref: String,
    depends_on: Vec<String>,
}

impl From<Dependency> for models::Dependency {
    fn from(other: Dependency) -> Self {
        Self {
            dependency_ref: other.dependency_ref,
            dependencies: other
                .depends_on
                .into_iter()
                .map(|d| models::Dependency {
                    dependency_ref: d,
                    dependencies: Vec::new(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub(crate) fn example_dependencies() -> Dependencies {
        Dependencies(vec![Dependency {
            dependency_ref: "ref".to_string(),
            depends_on: vec!["depends on".to_string()],
        }])
    }

    pub(crate) fn corresponding_dependencies() -> models::Dependencies {
        models::Dependencies(vec![models::Dependency {
            dependency_ref: "ref".to_string(),
            dependencies: vec![models::Dependency {
                dependency_ref: "depends on".to_string(),
                dependencies: Vec::new(),
            }],
        }])
    }

    #[test]
    fn it_flattens_dependencies() {
        let actual: Dependencies = models::Dependencies(vec![models::Dependency {
            dependency_ref: "a".to_string(),
            dependencies: vec![
                models::Dependency {
                    dependency_ref: "b".to_string(),
                    dependencies: Vec::new(),
                },
                models::Dependency {
                    dependency_ref: "c".to_string(),
                    dependencies: Vec::new(),
                },
            ],
        }])
        .into();
        let expected = Dependencies(vec![Dependency {
            dependency_ref: "a".to_string(),
            depends_on: vec!["b".to_string(), "c".to_string()],
        }]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_deduplicates_when_flattening_dependencies() {
        let actual: Dependencies = models::Dependencies(vec![
            models::Dependency {
                dependency_ref: "a".to_string(),
                dependencies: vec![models::Dependency {
                    dependency_ref: "common".to_string(),
                    dependencies: vec![models::Dependency {
                        dependency_ref: "common_transitive".to_string(),
                        dependencies: Vec::new(),
                    }],
                }],
            },
            models::Dependency {
                dependency_ref: "b".to_string(),
                dependencies: vec![models::Dependency {
                    dependency_ref: "common".to_string(),
                    dependencies: vec![models::Dependency {
                        dependency_ref: "common_transitive".to_string(),
                        dependencies: Vec::new(),
                    }],
                }],
            },
        ])
        .into();
        let expected = Dependencies(vec![
            Dependency {
                dependency_ref: "a".to_string(),
                depends_on: vec!["common".to_string()],
            },
            Dependency {
                dependency_ref: "b".to_string(),
                depends_on: vec!["common".to_string()],
            },
            Dependency {
                dependency_ref: "common".to_string(),
                depends_on: vec!["common_transitive".to_string()],
            },
        ]);
        assert_eq!(actual, expected);
    }
}
