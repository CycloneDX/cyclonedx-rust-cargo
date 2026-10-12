use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use cyclonedx_bom::models::bom::{Bom, SpecVersion};
use predicates::prelude::*;
use serde_json::{json, Value};
use std::process::Command;

#[test]
fn find_content_in_help_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(assert_cmd::cargo_bin!());
    cmd.arg("cyclonedx").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cargo cyclonedx [OPTIONS]"));

    Ok(())
}

#[test]
fn manifest_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::new(assert_cmd::cargo_bin!());
    cmd.current_dir(tmp_dir.path()).arg("cyclonedx");

    cmd.assert().failure().stdout("");

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn manifest_is_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_file = assert_fs::NamedTempFile::new("Cargo.toml")?;
    tmp_file.touch()?;

    let mut cmd = Command::new(assert_cmd::cargo_bin!());

    cmd.arg("cyclonedx")
        .arg("--manifest-path")
        .arg(tmp_file.path());

    cmd.assert().failure().stdout("");

    tmp_file.close()?;

    Ok(())
}

#[test]
fn find_content_in_bom_files() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_rust_project()?;
    let mut cmd = Command::new(assert_cmd::cargo_bin!());

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--top-level")
        .arg("--override-filename=bom");

    cmd.assert().success().stdout("");

    tmp_dir
        .child("bom.xml")
        .assert(predicate::str::contains("<vendor>CycloneDX</vendor>"));

    cmd.arg("--format").arg("json");
    cmd.assert().success().stdout("");

    tmp_dir
        .child("bom.json")
        .assert(predicate::str::contains(r#""vendor": "CycloneDX"#));

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn find_content_in_stderr() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_rust_project()?;

    let pkg_name = "nested-pkg";

    tmp_dir.child("Cargo.toml").write_str(&format!(
        r#"
        [package]
        name = "test"
        version = "0.0.0"

        [dependencies.{0}]
        path = "{0}"
        "#,
        pkg_name,
    ))?;

    let license = "TEST";
    let pkg_dir = tmp_dir.child(pkg_name);
    pkg_dir.child("src/lib.rs").touch()?;

    pkg_dir.child("Cargo.toml").write_str(&format!(
        r#"
        [package]
        name = "{}"
        version = "0.0.0"
        license = "{}"
        "#,
        pkg_name, license,
    ))?;

    let mut cmd = Command::new(assert_cmd::cargo_bin!());

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--all")
        .arg("--license-strict")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains(format!(
            "Package {} has an invalid license expression ({}), using as named license: Invalid SPDX expression: unknown term",
            pkg_name, license,
        )));

    let mut cmd = Command::new(assert_cmd::cargo_bin!());

    cmd.current_dir(tmp_dir.path()).arg("cyclonedx").arg("-qq");

    cmd.assert().success().stdout("");

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn emits_package_authors_for_each_spec_version() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_rust_project()?;
    tmp_dir.child("src/lib.rs").touch()?;
    tmp_dir.child("Cargo.toml").write_str(
        r#"
        [workspace]
        members = ["inherited", "missing", "empty", "blank"]
        resolver = "2"

        [workspace.package]
        authors = ["Workspace Author <workspace@example.com>"]

        [package]
        name = "pkg"
        version = "0.0.0"
        edition = "2021"
        authors = [
            " Alice & Co < alice@example.com > ",
            "Example, Inc.",
            "Bad <invalid>",
            "<email-only@example.com>",
            "bare@example.com",
            "Example, Inc.",
            " "
        ]

        [dependencies]
        inherited = { path = "inherited" }
        missing = { path = "missing" }
        empty = { path = "empty" }
        blank = { path = "blank" }
    "#,
    )?;
    for (name, authors) in [
        ("inherited", "authors.workspace = true"),
        ("missing", ""),
        ("empty", "authors = []"),
        ("blank", "authors = [\"\", \" \"]"),
    ] {
        let package_dir = tmp_dir.child(name);
        package_dir.child("src/lib.rs").touch()?;
        package_dir.child("Cargo.toml").write_str(&format!(
            r#"
            [package]
            name = "{name}"
            version = "0.0.0"
            edition = "2021"
            {authors}
        "#
        ))?;
    }

    for version in [
        SpecVersion::V1_3,
        SpecVersion::V1_4,
        SpecVersion::V1_5,
        SpecVersion::V1_6,
        SpecVersion::V1_7,
    ] {
        for format in ["json", "xml"] {
            Command::new(assert_cmd::cargo_bin!())
                .current_dir(tmp_dir.path())
                .env("CARGO_NET_OFFLINE", "true")
                .args([
                    "cyclonedx",
                    "--target",
                    "all",
                    "--override-filename=bom",
                    "--format",
                    format,
                    "--spec-version",
                    &version.to_string(),
                ])
                .assert()
                .success();

            for package_dir in ["", "inherited", "missing", "empty", "blank"] {
                let path = tmp_dir
                    .path()
                    .join(package_dir)
                    .join(format!("bom.{format}"));
                let bytes = std::fs::read(path)?;
                let output: Value = if format == "json" {
                    serde_json::from_slice(&bytes)?
                } else {
                    let bom = Bom::parse_from_xml_with_version(bytes.as_slice(), version)?;
                    let mut json = Vec::new();
                    // Inspect the parsed fields without invoking the older-version fallback again.
                    bom.output_as_json(&mut json, SpecVersion::V1_7)?;
                    serde_json::from_slice(&json)?
                };
                let metadata = &output["metadata"];
                let root = &metadata["component"];
                let (expected_authors, _) =
                    expected_package_authors(root["name"].as_str().unwrap());
                assert_eq!(metadata.get("authors"), expected_authors.as_ref());
                assert_package_authors(root, version);
                let targets = root["components"].as_array().unwrap();
                assert_eq!(targets.len(), if package_dir.is_empty() { 2 } else { 1 });
                for target in targets {
                    assert!(target.get("authors").is_none());
                    assert!(target.get("author").is_none());
                }
                for dependency in output["components"].as_array().unwrap() {
                    assert_package_authors(dependency, version);
                }
                if package_dir.is_empty() {
                    assert_eq!(output["components"].as_array().unwrap().len(), 4);
                }
                if format == "xml" && version < SpecVersion::V1_6 {
                    // Component authors must not use the structured XML representation before 1.6.
                    assert_eq!(
                        std::str::from_utf8(&bytes)?.matches("<authors>").count(),
                        usize::from(expected_authors.is_some())
                    );
                }
            }
        }
    }
    tmp_dir.close()?;
    Ok(())
}

fn expected_package_authors(name: &str) -> (Option<Value>, Option<&'static str>) {
    match name {
        "pkg" => (Some(json!([
            {"name": "Alice & Co", "email": "alice@example.com"},
            {"name": "Example, Inc."},
            {"name": "Bad <invalid>"},
            {"email": "email-only@example.com"},
            {"name": "bare@example.com"},
            {"name": "Example, Inc."}
        ])), Some("Alice & Co <alice@example.com>, Example, Inc., Bad <invalid>, email-only@example.com, bare@example.com, Example, Inc.")),
        "inherited" => (Some(json!([{"name": "Workspace Author", "email": "workspace@example.com"}])),
            Some("Workspace Author <workspace@example.com>")),
        "missing" | "empty" | "blank" => (None, None),
        other => panic!("Unexpected package {other}"),
    }
}

fn assert_package_authors(component: &Value, version: SpecVersion) {
    let name = component["name"].as_str().unwrap();
    let (authors, author) = expected_package_authors(name);
    if version >= SpecVersion::V1_6 {
        assert_eq!(
            component.get("authors"),
            authors.as_ref(),
            "{name}: {version}"
        );
        assert!(component.get("author").is_none());
    } else {
        assert_eq!(
            component.get("author"),
            author.map(|a| json!(a)).as_ref(),
            "{name}: {version}"
        );
        assert!(component.get("authors").is_none());
    }
}

fn make_temp_rust_project() -> Result<assert_fs::TempDir, assert_fs::fixture::FixtureError> {
    let tmp_dir = assert_fs::TempDir::new()?;
    tmp_dir.child("src/main.rs").touch()?;

    tmp_dir
        .child("Cargo.toml")
        .write_str(r#"package = { name = "pkg", version = "0.0.0" }"#)?;

    Ok(tmp_dir)
}
