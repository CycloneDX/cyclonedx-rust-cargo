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

#[derive(Debug, PartialEq)]
pub struct Composition {
    pub aggregate: AggregateType,
    pub assemblies: Option<Vec<BomReference>>,
    pub dependencies: Option<Vec<BomReference>>,
}

#[derive(Debug, PartialEq)]
pub struct Compositions(pub Vec<Composition>);

#[derive(Debug, PartialEq)]
pub enum AggregateType {
    Complete,
    Incomplete,
    IncompleteFirstPartyOnly,
    IncompleteThirdPartyOnly,
    Unknown,
    NotSpecified,
    #[doc(hidden)]
    UnknownAggregateType(String),
}

impl ToString for AggregateType {
    fn to_string(&self) -> String {
        match self {
            AggregateType::Complete => "complete",
            AggregateType::Incomplete => "incomplete",
            AggregateType::IncompleteFirstPartyOnly => "incomplete_first_party_only",
            AggregateType::IncompleteThirdPartyOnly => "incomplete_third_party_only",
            AggregateType::Unknown => "unknown",
            AggregateType::NotSpecified => "not_specified",
            AggregateType::UnknownAggregateType(uat) => uat,
        }
        .to_string()
    }
}

impl AggregateType {
    pub(crate) fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
        match value.as_ref() {
            "complete" => Self::Complete,
            "incomplete" => Self::Incomplete,
            "incomplete_first_party_only" => Self::IncompleteFirstPartyOnly,
            "incomplete_third_party_only" => Self::IncompleteThirdPartyOnly,
            "unknown" => Self::Unknown,
            "not_specified" => Self::NotSpecified,
            unknown => Self::UnknownAggregateType(unknown.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BomReference(pub(crate) String);
