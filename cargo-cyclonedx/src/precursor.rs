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
use std::collections::{BTreeMap, BTreeSet, HashSet};

use cargo_metadata::{DependencyKind, PackageId};
use cargo_sbom_precursor::{Dependency, Precursor};

use crate::generator::PrivateDepKind;

#[derive(Debug)]
pub(crate) struct PackageGraph {
    pub(crate) root: PackageId,
    pub(crate) kinds: BTreeMap<PackageId, DependencyKind>,
    pub(crate) edges: BTreeMap<PackageId, BTreeSet<PackageId>>,
}

fn dep_kind(dep: &Dependency) -> PrivateDepKind {
    match dep.kind {
        cargo_sbom_precursor::DependencyKind::Build => PrivateDepKind::Build,
        _ => PrivateDepKind::Runtime,
    }
}

fn package_id(id: &str) -> PackageId {
    PackageId {
        repr: id.to_owned(),
    }
}

/// Collapses the precursor graph to one node per package.
/// If `only_normal` is set, build dependencies and everything only reachable through them are dropped.
pub(crate) fn package_graph(precursor: &Precursor, only_normal: bool) -> PackageGraph {
    let included = |dep: &Dependency| !only_normal || dep_kind(dep) == PrivateDepKind::Runtime;

    let mut node_kinds: Vec<Option<PrivateDepKind>> = vec![None; precursor.crates.len()];
    let mut nodes_to_visit = vec![(precursor.root, PrivateDepKind::Runtime)];
    while let Some((index, path_kind)) = nodes_to_visit.pop() {
        if let Some(previous_kind) = node_kinds[index] {
            if previous_kind >= path_kind {
                continue;
            }
        }
        node_kinds[index] = Some(path_kind);
        for dep in precursor.crates[index]
            .dependencies
            .iter()
            .filter(|d| included(d))
        {
            nodes_to_visit.push((dep.index, path_kind.min(dep_kind(dep))));
        }
    }

    let mut package_kinds: BTreeMap<PackageId, PrivateDepKind> = BTreeMap::new();
    let mut edges: BTreeMap<PackageId, BTreeSet<PackageId>> = BTreeMap::new();
    for (krate, node_kind) in precursor.crates.iter().zip(node_kinds) {
        if let Some(node_kind) = node_kind {
            let id = package_id(&krate.id);
            let package_kind = package_kinds.entry(id.clone()).or_insert(node_kind);
            *package_kind = (*package_kind).max(node_kind);

            let package_edges = edges.entry(id).or_default();
            for dep in krate.dependencies.iter().filter(|d| included(d)) {
                let dep_id = &precursor.crates[dep.index].id;
                if dep_id != &krate.id {
                    package_edges.insert(package_id(dep_id));
                }
            }
        }
    }

    PackageGraph {
        root: package_id(&precursor.root_crate().id),
        kinds: package_kinds
            .into_iter()
            .map(|(id, kind)| (id, DependencyKind::from(kind)))
            .collect(),
        edges,
    }
}

impl PackageGraph {
    pub(crate) fn top_level(mut self) -> Self {
        let direct = self.edges.remove(&self.root).unwrap_or_default();
        let keep: HashSet<&PackageId> = direct.iter().chain([&self.root]).collect();
        self.kinds.retain(|id, _| keep.contains(id));
        self.edges = direct
            .iter()
            .map(|id| (id.clone(), BTreeSet::new()))
            .collect();
        self.edges.insert(self.root.clone(), direct);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A binary `app` with a library target, a build script that uses `cc`,
    // a proc-macro `derive`, and `shared`, which is reached both at runtime and from the proc-macro.
    const FIXTURE: &str = r#"{
      "version": 1,
      "root": 0,
      "target": "x86_64-unknown-linux-gnu",
      "rustc": { "version": "1.95.0-nightly", "wrapper": null, "workspace_wrapper": null,
                 "commit_hash": null, "host": "x86_64-unknown-linux-gnu", "verbose_version": "" },
      "crates": [
        { "id": "path+file:///w/app#0.1.0", "kind": ["bin"], "features": [],
          "dependencies": [{"index": 1, "kind": "normal"}] },
        { "id": "path+file:///w/app#0.1.0", "kind": ["lib"], "features": [],
          "dependencies": [{"index": 2, "kind": "build"}, {"index": 3, "kind": "normal"},
                           {"index": 4, "kind": "build"}] },
        { "id": "path+file:///w/app#0.1.0", "kind": ["custom-build"], "features": [],
          "dependencies": [{"index": 5, "kind": "normal"}] },
        { "id": "registry+https://github.com/rust-lang/crates.io-index#shared@1.0.0",
          "kind": ["lib"], "features": ["std"], "dependencies": [] },
        { "id": "path+file:///w/derive#0.1.0", "kind": ["proc-macro"], "features": [],
          "dependencies": [{"index": 6, "kind": "normal"}] },
        { "id": "registry+https://github.com/rust-lang/crates.io-index#cc@1.0.0",
          "kind": ["lib"], "features": [], "dependencies": [] },
        { "id": "registry+https://github.com/rust-lang/crates.io-index#shared@1.0.0",
          "kind": ["lib"], "features": [], "dependencies": [] }
      ]
    }"#;

    const APP: &str = "path+file:///w/app#0.1.0";
    const SHARED: &str = "registry+https://github.com/rust-lang/crates.io-index#shared@1.0.0";
    const CC: &str = "registry+https://github.com/rust-lang/crates.io-index#cc@1.0.0";
    const DERIVE: &str = "path+file:///w/derive#0.1.0";

    fn id(s: &str) -> PackageId {
        package_id(s)
    }

    fn graph(only_normal: bool) -> PackageGraph {
        package_graph(
            &Precursor::from_slice(FIXTURE.as_bytes()).unwrap(),
            only_normal,
        )
    }

    #[test]
    fn collapses_nodes_to_packages() {
        let graph = graph(false);
        assert_eq!(graph.root, id(APP));
        assert_eq!(graph.kinds.len(), 4);
        assert_eq!(
            graph.edges[&id(APP)],
            BTreeSet::from([id(SHARED), id(DERIVE), id(CC)])
        );
    }

    #[test]
    fn runtime_wins_over_build() {
        let graph = graph(false);
        assert_eq!(graph.kinds[&id(SHARED)], DependencyKind::Normal);
        assert_eq!(graph.kinds[&id(DERIVE)], DependencyKind::Build);
        assert_eq!(graph.kinds[&id(CC)], DependencyKind::Build);
    }

    #[test]
    fn only_normal_drops_build_subgraph() {
        let graph = graph(true);
        assert_eq!(
            graph.kinds.keys().cloned().collect::<Vec<_>>(),
            vec![id(APP), id(SHARED)]
        );
        assert_eq!(graph.edges[&id(APP)], BTreeSet::from([id(SHARED)]));
    }

    #[test]
    fn top_level_keeps_direct_deps_only() {
        let graph = graph(false).top_level();
        assert_eq!(graph.edges.values().filter(|e| !e.is_empty()).count(), 1);
        assert!(graph.kinds.contains_key(&id(DERIVE)));
    }
}
