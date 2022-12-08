use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn find_content_in_help_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("cyclonedx").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cargo cyclonedx [OPTIONS]"));

    Ok(())
}

#[test]
fn manifest_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(tmp_dir.path()).arg("cyclonedx");

    cmd.assert()
        .failure()
        .stdout("")
        .stderr(predicate::str::contains(format!(
            "Error: failed to read `{}`",
            tmp_dir.path().join("Cargo.toml").display(),
        )));

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn manifest_is_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_file = assert_fs::NamedTempFile::new("Cargo.toml")?;
    tmp_file.touch()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.arg("cyclonedx")
        .arg("--manifest-path")
        .arg(tmp_file.path());

    cmd.assert()
        .failure()
        .stdout("")
        .stderr(predicate::str::contains(format!(
            "Error: failed to parse manifest at `{}`",
            tmp_file.path().display(),
        )));

    tmp_file.close()?;

    Ok(())
}

#[test]
fn find_content_in_bom_files() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_rust_project()?;
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--top-level");

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
    pkg_dir.child("src/main.rs").touch()?;

    pkg_dir.child("Cargo.toml").write_str(&format!(
        r#"
        [package]
        name = "{}"
        version = "0.0.0"
        license = "{}"
        "#,
        pkg_name, license,
    ))?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--all")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains(format!(
            "Outputting {}",
            tmp_dir.path().join("bom.xml").display(),
        )))
        .stderr(predicate::str::contains(format!(
            "Package {} has an invalid license expression, trying lax parsing ({})",
            pkg_name, license,
        )));

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--quiet");

    cmd.assert().success().stdout("").stderr("");

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn bom_file_name_extension_is_prepended_with_cdx() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_rust_project()?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--output-cdx");

    cmd.assert().success().stdout("");

    tmp_dir.child("bom.xml").assert(predicate::path::missing());

    tmp_dir
        .child("bom.cdx.xml")
        .assert(predicate::path::exists());

    tmp_dir.close()?;

    Ok(())
}

#[test]
fn bom_file_contains_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = assert_fs::TempDir::new()?;
    tmp_dir.child("src/main.rs").touch()?;

    tmp_dir.child("Cargo.toml").write_str(
        r#"[package]
name = "pkg"
version = "0.0.0"

[dependencies]
child-1 = { path = "child-1" }
"#,
    )?;

    tmp_dir.child("child-1/src/main.rs").touch()?;

    tmp_dir.child("child-1/Cargo.toml").write_str(
        r#"[package]
name = "child-1"
version = "1.0.0"

[dependencies]
child-2 = { path = "../child-2" }
child-3 = { path = "../child-3" }
"#,
    )?;

    tmp_dir.child("child-2/src/main.rs").touch()?;

    tmp_dir
        .child("child-2/Cargo.toml")
        .write_str(r#"package = { name = "child-2", version = "2.0.0" }"#)?;

    tmp_dir.child("child-3/src/main.rs").touch()?;

    tmp_dir
        .child("child-3/Cargo.toml")
        .write_str(r#"package = { name = "child-3", version = "3.0.0" }"#)?;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path()).arg("cyclonedx");

    cmd.assert().success().stdout("");

    tmp_dir
        .child("bom.xml")
        .assert(predicate::str::contains(r#"<dependencies>"#).not());

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(tmp_dir.path())
        .arg("cyclonedx")
        .arg("--all");

    cmd.assert().success().stdout("");

    let patterns = [
        r#"<dependencies>"#,
        r#"<dependency ref="pkg:cargo/child-1@1.0.0">"#,
        r#"<dependency ref="pkg:cargo/child-2@2.0.0" />"#,
        r#"<dependency ref="pkg:cargo/child-3@3.0.0" />"#,
    ];

    for pattern in patterns {
        tmp_dir
            .child("bom.xml")
            .assert(predicate::str::contains(pattern));
    }

    Ok(())
}

fn make_temp_rust_project() -> Result<assert_fs::TempDir, assert_fs::fixture::FixtureError> {
    let tmp_dir = assert_fs::TempDir::new()?;
    tmp_dir.child("src/main.rs").touch()?;

    tmp_dir
        .child("Cargo.toml")
        .write_str(r#"package = { name = "pkg", version = "0.0.0" }"#)?;

    Ok(tmp_dir)
}
