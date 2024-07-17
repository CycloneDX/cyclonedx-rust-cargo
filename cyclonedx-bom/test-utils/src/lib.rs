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

 use jsonschema::{error::ValidationErrorKind, paths::JSONPointer, JSONSchema};

 use cyclonedx_bom::models::bom::SpecVersion;
 
 #[derive(Debug)]
 pub struct ValidationError {
     pub instance: String,
     pub kind: ValidationErrorKind,
     pub hierarchy: Vec<String>,
 }
 
 impl ValidationError {
     pub fn new(instance: String, kind: ValidationErrorKind, hierarchy: JSONPointer) -> Self {
         let hierarchy = hierarchy
             .iter()
             .map(|chunk| format!("{:?}", chunk))
             .collect::<Vec<_>>();
         dbg!(&hierarchy);
 
         Self {
             instance,
             kind,
             hierarchy,
         }
     }
 }
 
 /// Validates a [`serde_json::Value`] against a JSON schema defined by the given [`SpecVersion`].
 ///
 /// This function returns the list of validation errors if something fails.
 ///
 /// ## Example
 ///
 /// ```rust
 /// use cyclonedx_bom::prelude::*;
 /// use cyclonedx_bom::schema::validate_json_with_schema;
 ///
 /// let bom_json = r#"{
 ///   "bomFormat": "CycloneDX",
 ///   "specVersion": "1.3",
 ///   "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
 ///   "version": 1
 /// }"#;
 /// let json: serde_json::Value = serde_json::from_str(bom_json).expect("Failed to parse JSON");
 /// let result = validate_json_with_schema(&json, SpecVersion::V1_3);
 /// assert!(result.is_ok());
 /// ```
 ///
 pub fn validate_json_with_schema(
     json: &serde_json::Value,
     version: SpecVersion,
 ) -> Result<(), Vec<ValidationError>> {
     let spdx_schema = include_str!("../schema/spdx.schema.json");
     let spdx_schema: serde_json::Value =
         serde_json::from_str(spdx_schema).expect("Failed to read spdx.schema.json");
     let jsf_schema = include_str!("../schema/jsf-0.82.schema.json");
     let jsf_schema: serde_json::Value =
         serde_json::from_str(jsf_schema).expect("Failed to load jsf-0.82.schema.json");
 
     let schema = match version {
         SpecVersion::V1_3 => include_str!("../schema/bom-1.3.schema.json"),
         SpecVersion::V1_4 => include_str!("../schema/bom-1.4.schema.json"),
         SpecVersion::V1_5 => include_str!("../schema/bom-1.5.schema.json"),
     };
     let schema: serde_json::Value =
         serde_json::from_str(schema).expect("Failed to parse JSON schema file");
 
     // Fill in external schema files, handle unknown format(s)
     let compiled_schema = JSONSchema::options()
         .with_draft(jsonschema::Draft::Draft7)
         .with_document(
             "http://cyclonedx.org/schema/spdx.schema.json".to_string(),
             spdx_schema,
         )
         .with_document(
             "http://cyclonedx.org/schema/jsf-0.82.schema.json".to_string(),
             jsf_schema,
         )
         .with_format("idn-email", with_idn_email)
         .compile(&schema)
         .expect("Failed to compile JSON schema file");
 
     let result = compiled_schema.validate(json);
     if let Err(errors) = result {
         let errors = errors.collect::<Vec<_>>();
         dbg!(&errors);
     }
     compiled_schema.validate(json).map_err(|iter| {
         iter.map(|err| ValidationError::new(err.instance.to_string(), err.kind, err.instance_path))
             .collect::<Vec<_>>()
     })
 }
 
 /// For now ignore the content of the given email string.
 fn with_idn_email(_s: &str) -> bool {
     true
 }
 
 #[cfg(test)]
 mod test {
    use super::validate_json_with_schema;
    use cyclonedx_bom::models::bom::SpecVersion;
 
     #[test]
     fn it_should_validate_version_13() {
         let input = r#"
 {
   "bomFormat": "CycloneDX",
   "specVersion": "1.3",
   "version": 1,
   "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
   "components": [
     {
       "type": "library",
       "name": "acme-library-a",
       "version": "1.0.0",
       "components": [
         {
           "type": "library",
           "name": "acme-library-b",
           "version": "2.0.0"
         }
       ]
     }
   ],
   "services": [
     {
       "name": "acme-service-a",
       "services": [
         {
           "name": "acme-service-b"
         }
       ]
     }
   ]
 }"#;
         let json = serde_json::from_str(input).expect("Failed to parse JSON");
         assert!(validate_json_with_schema(&json, SpecVersion::V1_3).is_ok());
     }
 }
 