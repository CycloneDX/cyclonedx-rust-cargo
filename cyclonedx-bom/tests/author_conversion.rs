use cyclonedx_bom::models::bom::{Bom, SpecVersion};
use serde_json::{json, Value};
use test_utils::validate_json_with_schema;

fn write_json(bom: &Bom, version: SpecVersion) -> Value {
    let mut output = Vec::new();
    bom.clone().output_as_json(&mut output, version).unwrap();
    serde_json::from_slice(&output).unwrap()
}

fn write_xml(bom: &Bom, version: SpecVersion) -> String {
    let mut output = Vec::new();
    bom.clone().output_as_xml(&mut output, version).unwrap();
    String::from_utf8(output).unwrap()
}

#[test]
fn formats_authors_for_legacy_versions() {
    let cases = [
        ("absent", json!({}), None),
        ("empty list", json!({"authors": []}), None),
        ("empty contact", json!({"authors": [{}]}), None),
        (
            "name",
            json!({"authors": [{"name": "Alice"}]}),
            Some("Alice"),
        ),
        (
            "email",
            json!({"authors": [{"email": "alice@example.com"}]}),
            Some("alice@example.com"),
        ),
        (
            "name and email",
            json!({"authors": [{"name": "Alice", "email": "alice@example.com"}]}),
            Some("Alice <alice@example.com>"),
        ),
        (
            "order and duplicates",
            json!({"authors": [{"name": "Bob"}, {"name": "Alice"}, {"name": "Bob"}]}),
            Some("Bob, Alice, Bob"),
        ),
        (
            "trim and skip blanks",
            json!({"authors": [
                {"name": "  Alice  ", "email": " alice@example.com "},
                {"name": "\t", "email": " "},
                {"name": "", "email": "bob@example.com"},
                {"name": "Carol", "email": ""}
            ]}),
            Some("Alice <alice@example.com>, bob@example.com, Carol"),
        ),
        (
            "no usable fields",
            json!({"authors": [{"phone": "123"}, {"bom-ref": "contact"}, {"name": " "}]}),
            None,
        ),
        (
            "ignore phone and reference",
            json!({"authors": [{"name": "Alice", "phone": "123", "bom-ref": "contact"}]}),
            Some("Alice"),
        ),
        (
            "normalize synthesized text",
            json!({"authors": [{"name": "Alice\r\nA.\tSmith\rB.\nJones"}]}),
            Some("Alice A. Smith B. Jones"),
        ),
        (
            "legacy precedence",
            json!({"author": " Legacy <legacy@example.com> ", "authors": [{"name": "Alice"}]}),
            Some(" Legacy <legacy@example.com> "),
        ),
        (
            "empty legacy precedence",
            json!({"author": "", "authors": [{"name": "Alice"}]}),
            Some(""),
        ),
    ];

    for (label, fields, expected) in cases {
        let mut component = json!({"type": "library", "name": "demo", "version": "1.0"});
        component
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        let bom = Bom::parse_json_value(json!({
            "bomFormat": "CycloneDX", "specVersion": "1.6", "version": 1,
            "components": [component]
        }))
        .unwrap();

        for version in [SpecVersion::V1_3, SpecVersion::V1_4, SpecVersion::V1_5] {
            let output = write_json(&bom, version);
            assert_eq!(
                output["components"][0]["author"].as_str(),
                expected,
                "{label}: {version:?}"
            );
            assert!(output["components"][0].get("authors").is_none());
            if expected.is_none() {
                assert!(output["components"][0].get("author").is_none());
            }
            validate_json_with_schema(&output, version).unwrap();
            let reparsed = Bom::parse_json_value(output).unwrap();
            assert_eq!(
                reparsed.components.as_ref().unwrap().0[0].author.as_deref(),
                expected
            );
            let xml = write_xml(&bom, version);
            assert!(!xml.contains("<authors>"));
            let from_xml = Bom::parse_from_xml_with_version(xml.as_bytes(), version).unwrap();
            // The existing XML reader trims text, including explicit legacy author values.
            assert_eq!(
                write_json(&from_xml, version)["components"][0]["author"].as_str(),
                expected.map(str::trim),
                "{label}: {version:?} XML"
            );
        }
    }
}

const LEGACY_AUTHOR: &str = "Alice & Co <alice@example.com>, Bob";
const COMPONENT_PATHS: &[&str] = &[
    "/components/0",
    "/components/0/components/0",
    "/components/0/pedigree/ancestors/0",
    "/components/0/pedigree/descendants/0",
    "/components/0/pedigree/variants/0",
    "/metadata/component",
    "/metadata/tools/components/0",
    "/formulation/0/components/0",
];

fn structured_fixtures() -> [Bom; 2] {
    let component = json!({
        "type": "library", "name": "demo", "version": "1.0",
        "authors": [{"name": "Alice & Co", "email": "alice@example.com"}, {"name": "Bob"}]
    });
    let mut parent = component.clone();
    parent["components"] = json!([component]);
    parent["pedigree"] = json!({
        "ancestors": [component], "descendants": [component], "variants": [component]
    });
    let input = json!({
        "bomFormat": "CycloneDX", "specVersion": "1.6", "version": 1,
        "metadata": {
            "authors": [{"name": "BOM Creator"}],
            "component": component,
            "tools": {"components": [component]}
        },
        "components": [parent],
        "formulation": [{"components": [component]}]
    });
    validate_json_with_schema(&input, SpecVersion::V1_6).unwrap();

    let component_body = r#"<authors>
        <author><name>Alice &amp; Co</name><email>alice@example.com</email></author>
        <author><name>Bob</name></author>
        </authors><name>demo</name><version>1.0</version>"#;
    let component_xml = format!(r#"<component type="library">{component_body}</component>"#);
    let xml = format!(
        r#"<bom xmlns="http://cyclonedx.org/schema/bom/1.6" version="1">
        <metadata>
            <tools><components>{component_xml}</components></tools>
            <authors><author><name>BOM Creator</name></author></authors>
            {component_xml}
        </metadata>
        <components><component type="library">{component_body}
            <pedigree>
                <ancestors>{component_xml}</ancestors>
                <descendants>{component_xml}</descendants>
                <variants>{component_xml}</variants>
            </pedigree>
            <components>{component_xml}</components>
        </component></components>
        <formulation><formula><components>{component_xml}</components></formula></formulation>
        </bom>"#
    );
    [
        Bom::parse_json_value(input).unwrap(),
        Bom::parse_from_xml_with_version(xml.as_bytes(), SpecVersion::V1_6).unwrap(),
    ]
}

fn assert_legacy_authors(output: &Value) {
    validate_json_with_schema(output, SpecVersion::V1_5).unwrap();
    for path in COMPONENT_PATHS {
        let component = output.pointer(path).unwrap();
        assert_eq!(component["author"], LEGACY_AUTHOR, "{path}");
        assert!(component.get("authors").is_none(), "{path}");
    }
    assert_eq!(
        output["metadata"]["authors"],
        json!([{"name": "BOM Creator"}])
    );
}

#[test]
fn downgrades_json_and_xml_authors_in_all_component_locations() {
    for bom in structured_fixtures() {
        // Reading preserves structured authors without populating the legacy field.
        assert!(bom.components.as_ref().unwrap().0[0].author.is_none());
        assert_eq!(
            bom.components.as_ref().unwrap().0[0]
                .authors
                .as_ref()
                .unwrap()
                .len(),
            2
        );

        let json = write_json(&bom, SpecVersion::V1_5);
        assert_legacy_authors(&json);
        let reparsed = Bom::parse_json_value(json).unwrap();
        assert_legacy_authors(&write_json(&reparsed, SpecVersion::V1_5));

        let xml = write_xml(&bom, SpecVersion::V1_5);
        // Only metadata.authors remains structured in 1.5.
        assert_eq!(xml.matches("<authors>").count(), 1);
        let reparsed = Bom::parse_from_xml_with_version(xml.as_bytes(), SpecVersion::V1_5).unwrap();
        assert_legacy_authors(&write_json(&reparsed, SpecVersion::V1_5));
    }
}

#[test]
fn modern_versions_preserve_author_fields_independently() {
    for mut bom in structured_fixtures() {
        for legacy in [None, Some("Legacy".into()), Some("".into())] {
            bom.components.as_mut().unwrap().0[0].author = legacy;
            for version in [SpecVersion::V1_6, SpecVersion::V1_7] {
                let json = write_json(&bom, version);
                validate_json_with_schema(&json, version).unwrap();
                let from_json = Bom::parse_json_value(json).unwrap();
                let xml = write_xml(&bom, version);
                let from_xml = Bom::parse_from_xml_with_version(xml.as_bytes(), version).unwrap();
                for reparsed in [from_json, from_xml] {
                    assert_eq!(reparsed.components, bom.components);
                    assert_eq!(reparsed.metadata, bom.metadata);
                    assert_eq!(reparsed.formulation, bom.formulation);
                }
            }
        }
    }
}
