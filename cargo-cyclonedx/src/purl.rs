use std::str::FromStr;

use cargo_metadata::{camino::Utf8Path, Package};
use cyclonedx_bom::{external_models::uri::validate_purl, prelude::Purl as CdxPurl};
use pathdiff::diff_utf8_paths;
use purl::{PackageError, PackageType, PurlBuilder};

pub fn get_purl(
    package: &Package,
    root_package: &Package,
    workspace_root: &Utf8Path,
    subpath: Option<&Utf8Path>,
) -> Result<CdxPurl, PackageError> {
    let mut builder = PurlBuilder::new(PackageType::Cargo, &package.name)
        .with_version(package.version.to_string());

    if let Some(source) = &package.source {
        if !source.is_crates_io() {
            match source.repr.split_once('+') {
                // qualifier names are taken from the spec, which defines these two for all PURL types:
                // https://github.com/package-url/purl-spec/blob/master/PURL-SPECIFICATION.rst#known-qualifiers-keyvalue-pairs
                Some(("git", _git_path)) => {
                    builder = builder.with_qualifier("vcs_url", source_to_vcs_url(source))?
                }
                Some(("registry" | "sparse", registry_url)) => {
                    builder = builder.with_qualifier("repository_url", registry_url)?
                }
                Some((source, _path)) => log::warn!("Unknown source kind {}", source),
                None => {
                    log::warn!("No '+' separator found in source field from `cargo metadata`")
                }
            }
        }
    } else {
        // source is None for packages from the local filesystem.
        // The manifest path ends with a `Cargo.toml`, so the package directory is its parent
        let mut package_dir = package.manifest_path.parent().unwrap().to_owned();
        // If the package is within the workspace, encode the relative path instead of the absolute one
        // to make the SBOM reproducible(ish) and more clearly signal first-party dependencies.
        if package_dir.starts_with(workspace_root) {
            let root_package_dir = root_package.manifest_path.parent().unwrap();
            debug_assert!(root_package_dir.starts_with(workspace_root));
            package_dir = diff_utf8_paths(package_dir, root_package_dir).unwrap();
            if package_dir.as_str() == "" {
                // if the diff is empty, we are in the current directory
                package_dir = ".".into();
            }
        }
        // url-encode the path to the package manifest to make it a valid URL
        let manifest_url = format!("file://{}", package_dir.as_str());
        // url-encode the whole URL *again* because we are embedding this URL inside another URL (PURL)
        builder = builder.with_qualifier("download_url", &manifest_url)?
    }

    if let Some(subpath) = subpath {
        builder = builder.with_subpath(to_purl_subpath(subpath));
    }

    let purl = builder.build()?;
    let cdx_purl = CdxPurl::from_str(&purl.to_string()).unwrap();
    if cfg!(debug_assertions) {
        assert_validation_passes(&cdx_purl);
    }
    Ok(CdxPurl::from_str(&purl.to_string()).unwrap())
}

/// Converts the `cargo metadata`'s `source` field to a valid PURL `vcs_url`.
///
/// The `vcs_url` qualifier is specified to use the SPDX Package Download Location format:
/// `<vcs_tool>+<transport>://<host_name>[/<path_to_repository>][@<revision_tag_or_branch>][#<sub_path>]`
///
/// Cargo metadata uses a different format:
/// `git+<url>[?branch=<branch>|?tag=<tag>|?rev=<rev>]#<commit_hash>`
///
/// This function strips the query parameters (since the commit hash already identifies the code)
/// and converts the `#commit_hash` to `@commit_hash` per the SPDX format.
///
/// Assumes that the source kind is `git`, panics if it isn't.
fn source_to_vcs_url(source: &cargo_metadata::Source) -> String {
    assert!(source.repr.starts_with("git+"));
    let url = &source.repr;
    // Find where query parameters start (if any) and where the commit hash fragment starts
    let query_start = url.find('?');
    let fragment_start = url.find('#');
    match (query_start, fragment_start) {
        // Has both query params and commit hash: strip query, keep commit as @
        (Some(q), Some(f)) => {
            let base = &url[..q];
            let commit = &url[f + 1..];
            format!("{}@{}", base, commit)
        }
        // No query params, has commit hash: just replace # with @
        (None, Some(_)) => url.replace('#', "@"),
        // Has query params but no commit hash: extract the ref value as @
        (Some(q), None) => {
            let base = &url[..q];
            let query = &url[q + 1..];
            // Extract the value from branch=X, tag=X, or rev=X
            let ref_value = query
                .split('&')
                .find_map(|param| {
                    param
                        .strip_prefix("branch=")
                        .or_else(|| param.strip_prefix("tag="))
                        .or_else(|| param.strip_prefix("rev="))
                })
                .unwrap_or(query);
            format!("{}@{}", base, ref_value)
        }
        // No query params, no commit hash: return as-is
        (None, None) => url.to_string(),
    }
}

/// Converts a relative path to PURL subpath
fn to_purl_subpath(path: &Utf8Path) -> String {
    assert!(path.is_relative());
    let parts: Vec<&str> = path.components().map(|c| c.as_str()).collect();
    parts.join("/")
}

fn assert_validation_passes(purl: &CdxPurl) {
    assert!(validate_purl(purl).is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;
    use percent_encoding::percent_decode;
    use purl::Purl;
    use serde_json;

    const CRATES_IO_PACKAGE_JSON: &str = include_str!("../tests/fixtures/crates_io_package.json");
    const GIT_PACKAGE_JSON: &str = include_str!("../tests/fixtures/git_package.json");
    const GIT_PACKAGE_WITH_BRANCH_JSON: &str =
        include_str!("../tests/fixtures/git_package_with_branch.json");
    const ROOT_PACKAGE_JSON: &str = include_str!("../tests/fixtures/root_package.json");
    const WORKSPACE_PACKAGE_JSON: &str = include_str!("../tests/fixtures/workspace_package.json");

    #[test]
    fn crates_io_purl() {
        let crates_io_package: Package = serde_json::from_str(CRATES_IO_PACKAGE_JSON).unwrap();
        let purl = get_purl(
            &crates_io_package,
            &crates_io_package,
            Utf8Path::new("/foo/bar"),
            None,
        )
        .unwrap();
        // Validate that data roundtripped correctly
        let parsed_purl = Purl::from_str(purl.as_ref()).unwrap();
        assert_eq!(parsed_purl.name(), "aho-corasick");
        assert_eq!(parsed_purl.version(), Some("1.1.2"));
        assert!(parsed_purl.qualifiers().is_empty());
        assert!(parsed_purl.subpath().is_none());
        assert!(parsed_purl.namespace().is_none());
    }

    #[test]
    fn git_purl() {
        let git_package: Package = serde_json::from_str(GIT_PACKAGE_JSON).unwrap();
        let purl = get_purl(&git_package, &git_package, Utf8Path::new("/foo/bar"), None).unwrap();
        // Validate that data roundtripped correctly
        let parsed_purl = Purl::from_str(purl.as_ref()).unwrap();
        assert_eq!(parsed_purl.name(), "auditable-extract");
        assert_eq!(parsed_purl.version(), Some("0.3.2"));
        assert_eq!(parsed_purl.qualifiers().len(), 1);
        let (qualifier, value) = parsed_purl.qualifiers().iter().next().unwrap();
        assert_eq!(qualifier.as_str(), "vcs_url");
        assert_eq!(value, "git+https://github.com/rust-secure-code/cargo-auditable.git@da85607fb1a09435d77288ccf05a92b2e8ec3f71");
        assert!(parsed_purl.subpath().is_none());
        assert!(parsed_purl.namespace().is_none());
    }

    #[test]
    fn git_purl_with_branch() {
        let git_package: Package = serde_json::from_str(GIT_PACKAGE_WITH_BRANCH_JSON).unwrap();
        let purl = get_purl(&git_package, &git_package, Utf8Path::new("/foo/bar"), None).unwrap();
        // Validate that data roundtripped correctly
        let parsed_purl = Purl::from_str(purl.as_ref()).unwrap();
        assert_eq!(parsed_purl.name(), "rav1d");
        assert_eq!(parsed_purl.version(), Some("1.1.0"));
        assert_eq!(parsed_purl.qualifiers().len(), 1);
        let (qualifier, value) = parsed_purl.qualifiers().iter().next().unwrap();
        assert_eq!(qualifier.as_str(), "vcs_url");
        // The ?branch= query param must be stripped; only the commit hash remains after @
        let decoded_value = percent_decode(value.as_bytes())
            .decode_utf8()
            .unwrap()
            .to_string();
        assert_eq!(
            decoded_value,
            "git+https://github.com/leo030303/rav1d.git@3a50834ce3743bc580f340ba3bfbdbf6a46ab783"
        );
        // Ensure ?branch= is NOT present in the vcs_url
        assert!(!decoded_value.contains("?branch="));
        assert!(parsed_purl.subpath().is_none());
        assert!(parsed_purl.namespace().is_none());
    }

    #[test]
    fn toplevel_package_purl() {
        let root_package: Package = serde_json::from_str(ROOT_PACKAGE_JSON).unwrap();
        let purl = get_purl(
            &root_package,
            &root_package,
            Utf8Path::new("/home/shnatsel/Code/cargo-cyclonedx/"),
            None,
        )
        .unwrap();
        // Validate that data roundtripped correctly
        let parsed_purl = Purl::from_str(purl.as_ref()).unwrap();
        assert_eq!(parsed_purl.name(), "cargo-cyclonedx");
        assert_eq!(parsed_purl.version(), Some("0.3.8"));
        assert_eq!(parsed_purl.qualifiers().len(), 1);
        let (qualifier, value) = parsed_purl.qualifiers().iter().next().unwrap();
        assert_eq!(qualifier.as_str(), "download_url");
        let decoded_path = percent_decode(value.as_bytes()).decode_utf8().unwrap();
        assert_eq!(decoded_path, "file://.");
        assert!(parsed_purl.subpath().is_none());
        assert!(parsed_purl.namespace().is_none());
    }

    #[test]
    fn toplevel_package_with_subpath() {
        let root_package: Package = serde_json::from_str(ROOT_PACKAGE_JSON).unwrap();
        let purl = get_purl(
            &root_package,
            &root_package,
            Utf8Path::new("/home/shnatsel/Code/cargo-cyclonedx/"),
            Some("src/кириллица/lib.rs".into()),
        )
        .unwrap();
        // Validate that data roundtripped correctly
        let parsed_purl = Purl::from_str(purl.as_ref()).unwrap();
        assert_eq!(parsed_purl.name(), "cargo-cyclonedx");
        assert_eq!(parsed_purl.version(), Some("0.3.8"));
        assert_eq!(parsed_purl.qualifiers().len(), 1);
        let (qualifier, value) = parsed_purl.qualifiers().iter().next().unwrap();
        assert_eq!(qualifier.as_str(), "download_url");
        let decoded_path = percent_decode(value.as_bytes()).decode_utf8().unwrap();
        assert_eq!(decoded_path, "file://.");
        assert_eq!(parsed_purl.subpath().unwrap(), "src/кириллица/lib.rs");
        assert!(parsed_purl.namespace().is_none());
    }

    #[test]
    fn workspace_package() {
        let root_package: Package = serde_json::from_str(ROOT_PACKAGE_JSON).unwrap();
        let workspace_package: Package = serde_json::from_str(WORKSPACE_PACKAGE_JSON).unwrap();
        let purl = get_purl(
            &workspace_package,
            &root_package,
            Utf8Path::new("/home/shnatsel/Code/cargo-cyclonedx/"),
            None,
        )
        .unwrap();
        // Validate that data roundtripped correctly
        let parsed_purl = Purl::from_str(purl.as_ref()).unwrap();
        assert_eq!(parsed_purl.name(), "cyclonedx-bom");
        assert_eq!(parsed_purl.version(), Some("0.4.1"));
        assert_eq!(parsed_purl.qualifiers().len(), 1);
        let (qualifier, value) = parsed_purl.qualifiers().iter().next().unwrap();
        assert_eq!(qualifier.as_str(), "download_url");
        let decoded_path = percent_decode(value.as_bytes()).decode_utf8().unwrap();
        assert_eq!(decoded_path, "file://../cyclonedx-bom");
        assert!(parsed_purl.subpath().is_none());
        assert!(parsed_purl.namespace().is_none());
    }

    #[test]
    fn local_package() {
        let root_package: Package = serde_json::from_str(ROOT_PACKAGE_JSON).unwrap();
        let workspace_package: Package = serde_json::from_str(WORKSPACE_PACKAGE_JSON).unwrap();
        let purl = get_purl(
            &workspace_package,
            &root_package,
            Utf8Path::new("/foo/bar/"),
            None,
        )
        .unwrap();
        // Validate that data roundtripped correctly
        let parsed_purl = Purl::from_str(purl.as_ref()).unwrap();
        assert_eq!(parsed_purl.name(), "cyclonedx-bom");
        assert_eq!(parsed_purl.version(), Some("0.4.1"));
        assert_eq!(parsed_purl.qualifiers().len(), 1);
        let (qualifier, value) = parsed_purl.qualifiers().iter().next().unwrap();
        assert_eq!(qualifier.as_str(), "download_url");
        let decoded_path = percent_decode(value.as_bytes()).decode_utf8().unwrap();
        assert_eq!(
            decoded_path,
            "file:///home/shnatsel/Code/cargo-cyclonedx/cyclonedx-bom"
        );
        assert!(parsed_purl.subpath().is_none());
        assert!(parsed_purl.namespace().is_none());
    }
}
