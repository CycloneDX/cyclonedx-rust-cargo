mod v1_5 {
    use cyclonedx_bom::models::bom::{Bom, SpecVersion};
    use cyclonedx_bom::validation::Validate;
    use test_utils::validate_json_with_schema;

    #[test]
    fn it_should_parse_all_of_the_valid_xml_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.5",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.5/valid*.xml", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                let bom = Bom::parse_from_xml_v1_5(file).unwrap_or_else(|e| panic!("Failed to parse the document as an BOM: {path:?} {:#?}", e));

                let validation_result = bom.validate_version(SpecVersion::V1_5);
                if !validation_result.passed() {
                    dbg!(&validation_result);
                }
                assert!(
                    validation_result.passed(),
                    "{path:?} unexpectedly failed validation"
                );

                let mut output = Vec::new();
                bom.output_as_xml_v1_5(&mut output)
                    .unwrap_or_else(|_| panic!("Failed to output the file: {path:?}"));
                let bom_output = String::from_utf8_lossy(&output).to_string();

                insta::assert_snapshot!(bom_output);
            });
        });
    }

    #[test]
    fn it_should_parse_all_of_the_valid_json_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.5",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.5/valid*.json", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                let bom = Bom::parse_from_json_v1_5(file).unwrap_or_else(|e| panic!("Failed to parse the document as an BOM: {path:?} {:#?}", e));

                let validation_result = bom.validate_version(SpecVersion::V1_5);
                assert!(
                    validation_result.passed(),
                    "{path:?} unexpectedly failed validation"
                );

                let mut output = Vec::new();
                bom.output_as_json_v1_5(&mut output)
                    .unwrap_or_else(|_| panic!("Failed to output the file: {path:?}"));
                let bom_output = String::from_utf8_lossy(&output).to_string();

                // Check that the written JSON file validates against its schema.
                let json = serde_json::from_str(&bom_output).expect("Failed to parse JSON");
                validate_json_with_schema(&json, SpecVersion::V1_5)
                    .unwrap_or_else(|errors| panic!("Failed to validate output {path:?}, errors: {errors:?}"));

                insta::assert_snapshot!(bom_output);
            });
        });
    }

    #[test]
    fn it_should_fail_to_parse_all_of_the_invalid_xml_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.5",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.5/invalid*.xml", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                if let Ok(bom) = Bom::parse_from_xml_v1_5(file) {
                    let validation_result = bom.validate_version(SpecVersion::V1_5);
                    assert!(
                        validation_result.has_errors(),
                        "{path:?} unexpectedly passed validation"
                    );
                }
            });
        });
    }

    #[test]
    fn it_should_fail_to_parse_all_of_the_invalid_json_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.5",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.5/invalid*.json", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                if let Ok(bom) = Bom::parse_from_json_v1_5(file) {
                    let validation_result = bom.validate_version(SpecVersion::V1_5);
                    assert!(
                        validation_result.has_errors(),
                        "{path:?} unexpectedly passed validation"
                    );
                }
            });
        });
    }
}
