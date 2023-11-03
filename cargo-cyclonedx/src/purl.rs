use std::str::FromStr;

use cargo_metadata::{camino::Utf8Path, Package};
use cyclonedx_bom::prelude::Purl;

use crate::urlencode::urlencode;

pub fn get_purl(package: &Package, subpath: Option<&Utf8Path>) -> Result<Purl, purl::PackageError> {
    let mut builder = purl::PurlBuilder::new(purl::PackageType::Cargo, &package.name)
        .with_version(package.version.to_string());

    if let Some(source) = &package.source {
        if !source.is_crates_io() {
            match source.repr.split_once('+') {
                // qualifier names are taken from the spec, which defines these two for all PURL types:
                // https://github.com/package-url/purl-spec/blob/master/PURL-SPECIFICATION.rst#known-qualifiers-keyvalue-pairs
                Some(("git", _git_path)) => {
                    builder = builder.with_qualifier("vcs_url", source_to_vcs_url(&source))?
                }
                Some(("registry", registry_url)) => {
                    builder = builder.with_qualifier("repository_url", urlencode(registry_url))?
                }
                Some((source, _path)) => log::error!("Unknown source kind {}", source),
                None => {
                    log::error!("No '+' separator found in source field from `cargo metadata`")
                }
            }
        }
    } else {
        // source is None for packages from the local filesystem.
        // The manifest path ends with a `Cargo.toml`, so the package directory is its parent
        let package_dir = package.manifest_path.parent().unwrap();
        // url-encode the path to the package manifest to make it a valid URL
        let manifest_url = format!("file://{}", urlencode(package_dir.as_str()));
        // url-encode the whole URL *again* because we are embedding this URL inside another URL (PURL)
        builder = builder.with_qualifier("download_url", urlencode(&manifest_url))?
    }

    if let Some(subpath) = subpath {
        builder = builder.with_subpath(to_purl_subpath(subpath));
    }

    let purl = builder.build()?;
    Ok(Purl::from_str(&purl.to_string()).unwrap())
}

/// Converts the `cargo metadata`'s `source` field to a valid PURL `vcs_url`.
/// Assumes that the source kind is `git`, panics if it isn't.
fn source_to_vcs_url(source: &cargo_metadata::Source) -> String {
    assert!(source.repr.starts_with("git+"));
    urlencode(&source.repr.replace("#", "@"))
}

/// Converts a relative path to PURL subpath
fn to_purl_subpath(path: &Utf8Path) -> String {
    assert!(path.is_relative());
    let parts: Vec<String> = path.components().map(|c| urlencode(c.as_str())).collect();
    parts.join("/")
}
