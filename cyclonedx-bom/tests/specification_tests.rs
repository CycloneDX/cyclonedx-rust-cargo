use cyclonedx_bom::models::bom::Bom;

#[test]
fn it_should_parse_all_of_the_valid_xml_specifications() {
    insta::glob!("data/1.3/valid*.xml", |path| {
        let file = std::fs::File::open(path).expect("Failed to read file: {path:?}");
        let bom = Bom::parse_from_xml_v1_3(file)
            .expect("Failed to parse the document as an SBOM: {path:?}");

        // TODO: validate here

        let mut output = Vec::new();
        bom.output_as_xml_v1_3(&mut output)
            .expect("Failed to output the file: {path:?}");
        let bom_output = String::from_utf8_lossy(&output).to_string();

        insta::assert_snapshot!(bom_output);
    });
}

#[test]
fn it_should_parse_all_of_the_valid_json_specifications() {
    insta::glob!("data/1.3/valid*.json", |path| {
        let file = std::fs::File::open(path).expect("Failed to read file: {path:?}");
        let bom = Bom::parse_from_json_v1_3(file)
            .expect("Failed to parse the document as an SBOM: {path:?}");

        // TODO: validate here

        let mut output = Vec::new();
        bom.output_as_json_v1_3(&mut output)
            .expect("Failed to output the file: {path:?}");
        let bom_output = String::from_utf8_lossy(&output).to_string();

        insta::assert_snapshot!(bom_output);
    });
}
