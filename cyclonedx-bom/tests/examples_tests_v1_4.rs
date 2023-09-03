mod examples {
    use cyclonedx_bom::models::bom::Bom;
    use cyclonedx_bom::validation::{Validate, ValidationResult};

    /*
    #[test]
    fn it_should_parse_all_of_the_valid_xml_examples() {
        insta::with_settings!({
        snapshot_path => "examples_snapshots/1.4",
        prepend_module_to_snapshot => false,
    }, {
        insta::glob!("examples/1.4/valid*.xml", |path| {
            let file = std::fs::File::open(path).expect(&format!("Failed to read file: {path:?}"));
            let bom = Bom::parse_from_xml_v1_4(file).expect(&format!(
                "Failed to parse the document as an SBOM: {path:?}"
            ));

            let validation_result = bom.validate().expect("Failed to validate BOM");
            assert_eq!(
                validation_result,
                ValidationResult::Passed,
                "{path:?} unexpectedly failed validation"
            );

            let mut output = Vec::new();
            bom.output_as_xml_v1_4(&mut output)
                .expect(&format!("Failed to output the file: {path:?}"));
            let bom_output = String::from_utf8_lossy(&output).to_string();

            insta::assert_snapshot!(bom_output);
        });
    });
    }
    */

    /*
    #[test]
    fn it_should_parse_all_of_the_valid_json_specifications() {
        insta::with_settings!({
        snapshot_path => "examples_snapshots/1.4",
        prepend_module_to_snapshot => false,
    }, {
        insta::glob!("examples/1.4/valid*.json", |path| {
            let file = std::fs::File::open(path).expect(&format!("Failed to read file: {path:?}"));
            let bom = Bom::parse_from_json_v1_4(file).expect(&format!(
                "Failed to parse the document as an SBOM: {path:?}"
            ));

            let validation_result = bom.validate().expect("Failed to validate BOM");
            assert_eq!(
                validation_result,
                ValidationResult::Passed,
                "{path:?} unexpectedly failed validation"
            );

            let mut output = Vec::new();
            bom.output_as_json_v1_4(&mut output)
                .expect(&format!("Failed to output the file: {path:?}"));
            let bom_output = String::from_utf8_lossy(&output).to_string();

            insta::assert_snapshot!(bom_output);
        });
    });
    }
    */

    /*
    #[test]
    fn it_should_fail_all_of_the_invalid_xml_specifications() {
        insta::with_settings!({
            snapshot_path => "examples/1.4",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("examples_snapshots/1.4/invalid*.xml", |path| {
                let file = std::fs::File::open(path).expect(&format!("Failed to read file: {path:?}"));
                if let Ok(bom) = Bom::parse_from_xml_v1_4(file) {
                    let validation_result = bom.validate().expect("Failed to validate BOM");
                    assert_ne!(
                        validation_result,
                        ValidationResult::Passed,
                        "{path:?} unexpectedly passed validation"
                    );
                }
            });
        });
    }
    */

    #[test]
    fn it_should_fail_all_of_the_invalid_json_specifications() {
        insta::with_settings!({
            snapshot_path => "examples_snapshots/1.4",
            prepend_module_to_snapshot => false,
        }, {
            insta::glob!("examples/1.4/invalid*.json", |path| {
                let file = std::fs::File::open(path).expect(&format!("Failed to read file: {path:?}"));
                let bom = Bom::parse_from_json_v1_4(file).expect(&format!(
                    "Failed to parse the document as an SBOM: {path:?}"
                ));

                let validation_result = bom.validate().expect("Failed to validate BOM");
                assert_ne!(
                    validation_result,
                    ValidationResult::Passed,
                    "{path:?} expectedly failed validation"
                );

                let mut output = Vec::new();
                bom.output_as_json_v1_4(&mut output)
                    .expect(&format!("Failed to output the file: {path:?}"));
                let bom_output = String::from_utf8_lossy(&output).to_string();

                insta::assert_snapshot!(bom_output);
            });
        });
    }
}
