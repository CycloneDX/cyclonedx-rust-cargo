mod v1_7 {
    use cyclonedx_bom::models::bom::{Bom, SpecVersion};
    use cyclonedx_bom::validation::Validate;
    use test_utils::validate_json_with_schema;

    #[test]
    fn it_should_parse_all_of_the_valid_xml_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.7",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.7/valid*.xml", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                let bom = Bom::parse_from_xml_v1_7(file).unwrap_or_else(|e| panic!("Failed to parse the document as an BOM: {path:?} {:#?}", e));

                let validation_result = bom.validate_version(SpecVersion::V1_7);
                if !validation_result.passed() {
                    dbg!(&validation_result);
                }
                assert!(
                    validation_result.passed(),
                    "{path:?} unexpectedly failed validation"
                );

                let mut output = Vec::new();
                bom.output_as_xml_v1_7(&mut output)
                    .unwrap_or_else(|_| panic!("Failed to output the file: {path:?}"));
                let bom_output = String::from_utf8_lossy(&output).to_string();

                insta::assert_snapshot!(bom_output);
            });
        });
    }

    #[test]
    fn it_should_parse_all_of_the_valid_json_specifications() {
        insta::with_settings!({
            snapshot_path => "spec/snapshots/1.7",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("spec/1.7/valid*.json", |path| {
                let file = std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read file: {path:?}"));
                let bom = Bom::parse_from_json_v1_7(file).unwrap_or_else(|e| panic!("Failed to parse the document as an BOM: {path:?} {:#?}", e));

                let validation_result = bom.validate_version(SpecVersion::V1_7);
                assert!(
                    validation_result.passed(),
                    "{path:?} unexpectedly failed validation"
                );

                let mut output = Vec::new();
                bom.output_as_json_v1_7(&mut output)
                    .unwrap_or_else(|_| panic!("Failed to output the file: {path:?}"));
                let bom_output = String::from_utf8_lossy(&output).to_string();

                let json = serde_json::from_str(&bom_output).expect("Failed to parse JSON");
                validate_json_with_schema(&json, SpecVersion::V1_7)
                    .unwrap_or_else(|errors| panic!("Failed to validate output {path:?}, errors: {errors:?}"));

                insta::assert_snapshot!(bom_output);
            });
        });
    }
}
