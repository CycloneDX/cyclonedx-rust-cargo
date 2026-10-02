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

use cyclonedx_bom_macros::versioned;

#[versioned("1.6", "1.7")]
pub(crate) mod base {
    #[versioned("1.7")]
    use crate::specs::common::hash::{Hash, HashValue};
    #[versioned("1.7")]
    use crate::xml::{attribute_or_error, to_xml_write_error};
    use crate::{
        errors::{XmlReadError, XmlWriteError},
        external_models::date_time::DateTime,
        models::{self, bom::BomReference},
        utilities::{convert_optional, convert_optional_vec},
        xml::{
            read_lax_validation_tag, read_list_tag, read_simple_tag, to_xml_read_error,
            unexpected_element_error, write_close_tag, write_list_string_tag, write_list_tag,
            write_simple_option_tag, write_simple_tag, write_start_tag, FromXml, FromXmlType,
            ToXml,
        },
    };
    use serde::{Deserialize, Serialize};
    use std::io::{Read, Write};
    #[versioned("1.7")]
    use xml::writer;
    use xml::{attribute::OwnedAttribute, name::OwnedName, reader, EventReader, EventWriter};

    /// bom-1.6.schema.json #definitions/cryptoProperties
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CryptoProperties {
        asset_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        algorithm_properties: Option<AlgorithmProperties>,
        #[serde(skip_serializing_if = "Option::is_none")]
        certificate_properties: Option<CertificateProperties>,
        #[serde(skip_serializing_if = "Option::is_none")]
        related_crypto_material_properties: Option<RelatedCryptoMaterialProperties>,
        #[serde(skip_serializing_if = "Option::is_none")]
        protocol_properties: Option<ProtocolProperties>,
        #[serde(skip_serializing_if = "Option::is_none")]
        oid: Option<String>,
    }

    impl From<models::crypto_properties::CryptoProperties> for CryptoProperties {
        fn from(other: models::crypto_properties::CryptoProperties) -> Self {
            Self {
                asset_type: other.asset_type.to_string(),
                algorithm_properties: convert_optional(other.algorithm_properties),
                certificate_properties: convert_optional(other.certificate_properties),
                related_crypto_material_properties: convert_optional(
                    other.related_crypto_material_properties,
                ),
                protocol_properties: convert_optional(other.protocol_properties),
                oid: other.oid,
            }
        }
    }

    impl From<CryptoProperties> for models::crypto_properties::CryptoProperties {
        fn from(other: CryptoProperties) -> Self {
            Self {
                asset_type: models::crypto_properties::CryptoAssetType::new_unchecked(
                    other.asset_type,
                ),
                algorithm_properties: convert_optional(other.algorithm_properties),
                certificate_properties: convert_optional(other.certificate_properties),
                related_crypto_material_properties: convert_optional(
                    other.related_crypto_material_properties,
                ),
                protocol_properties: convert_optional(other.protocol_properties),
                oid: other.oid,
            }
        }
    }

    const CRYPTO_PROPERTIES_TAG: &str = "cryptoProperties";
    const ASSET_TYPE_TAG: &str = "assetType";
    const ALGORITHM_PROPERTIES_TAG: &str = "algorithmProperties";
    const CERTIFICATE_PROPERTIES_TAG: &str = "certificateProperties";
    const RELATED_CRYPTO_MATERIAL_PROPERTIES_TAG: &str = "relatedCryptoMaterialProperties";
    const PROTOCOL_PROPERTIES_TAG: &str = "protocolProperties";
    const OID_TAG: &str = "oid";

    impl ToXml for CryptoProperties {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, CRYPTO_PROPERTIES_TAG)?;
            write_simple_tag(writer, ASSET_TYPE_TAG, &self.asset_type)?;
            self.algorithm_properties.write_xml_element(writer)?;
            self.certificate_properties.write_xml_element(writer)?;
            self.related_crypto_material_properties
                .write_xml_element(writer)?;
            self.protocol_properties.write_xml_element(writer)?;
            write_simple_option_tag(writer, OID_TAG, &self.oid)?;
            write_close_tag(writer, CRYPTO_PROPERTIES_TAG)
        }
    }

    impl FromXml for CryptoProperties {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut asset_type: Option<String> = None;
            let mut algorithm_properties: Option<AlgorithmProperties> = None;
            let mut certificate_properties: Option<CertificateProperties> = None;
            let mut related_crypto_material_properties: Option<RelatedCryptoMaterialProperties> =
                None;
            let mut protocol_properties: Option<ProtocolProperties> = None;
            let mut oid: Option<String> = None;

            read_children(event_reader, element_name, |reader, name, attributes| {
                match name.local_name.as_str() {
                    ASSET_TYPE_TAG => asset_type = Some(read_simple_tag(reader, name)?),
                    ALGORITHM_PROPERTIES_TAG => {
                        algorithm_properties = Some(AlgorithmProperties::read_xml_element(
                            reader, name, attributes,
                        )?)
                    }
                    CERTIFICATE_PROPERTIES_TAG => {
                        certificate_properties = Some(CertificateProperties::read_xml_element(
                            reader, name, attributes,
                        )?)
                    }
                    RELATED_CRYPTO_MATERIAL_PROPERTIES_TAG => {
                        related_crypto_material_properties =
                            Some(RelatedCryptoMaterialProperties::read_xml_element(
                                reader, name, attributes,
                            )?)
                    }
                    PROTOCOL_PROPERTIES_TAG => {
                        protocol_properties = Some(ProtocolProperties::read_xml_element(
                            reader, name, attributes,
                        )?)
                    }
                    OID_TAG => oid = Some(read_simple_tag(reader, name)?),
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            Ok(Self {
                asset_type: asset_type.ok_or_else(|| {
                    XmlReadError::required_data_missing(ASSET_TYPE_TAG, element_name)
                })?,
                algorithm_properties,
                certificate_properties,
                related_crypto_material_properties,
                protocol_properties,
                oid,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct AlgorithmProperties {
        #[serde(skip_serializing_if = "Option::is_none")]
        primitive: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        algorithm_family: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameter_set_identifier: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        curve: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        elliptic_curve: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        execution_environment: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        implementation_platform: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        certification_level: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mode: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        padding: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        crypto_functions: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        classical_security_level: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        nist_quantum_security_level: Option<u32>,
    }

    impl From<models::crypto_properties::AlgorithmProperties> for AlgorithmProperties {
        fn from(other: models::crypto_properties::AlgorithmProperties) -> Self {
            Self {
                primitive: other.primitive.map(|v| v.to_string()),
                #[versioned("1.7")]
                algorithm_family: other.algorithm_family,
                parameter_set_identifier: other.parameter_set_identifier,
                curve: other.curve,
                #[versioned("1.7")]
                elliptic_curve: other.elliptic_curve,
                execution_environment: other.execution_environment.map(|v| v.to_string()),
                implementation_platform: other.implementation_platform.map(|v| v.to_string()),
                certification_level: other
                    .certification_level
                    .map(|levels| levels.into_iter().map(|v| v.to_string()).collect()),
                mode: other.mode.map(|v| v.to_string()),
                padding: other.padding.map(|v| v.to_string()),
                crypto_functions: other
                    .crypto_functions
                    .map(|functions| functions.into_iter().map(|v| v.to_string()).collect()),
                classical_security_level: other.classical_security_level,
                nist_quantum_security_level: other.nist_quantum_security_level,
            }
        }
    }

    impl From<AlgorithmProperties> for models::crypto_properties::AlgorithmProperties {
        fn from(other: AlgorithmProperties) -> Self {
            use models::crypto_properties as m;
            Self {
                primitive: other.primitive.map(m::CryptoPrimitive::new_unchecked),
                #[versioned("1.6")]
                algorithm_family: None,
                #[versioned("1.7")]
                algorithm_family: other.algorithm_family,
                parameter_set_identifier: other.parameter_set_identifier,
                curve: other.curve,
                #[versioned("1.6")]
                elliptic_curve: None,
                #[versioned("1.7")]
                elliptic_curve: other.elliptic_curve,
                execution_environment: other
                    .execution_environment
                    .map(m::CryptoExecutionEnvironment::new_unchecked),
                implementation_platform: other
                    .implementation_platform
                    .map(m::CryptoImplementationPlatform::new_unchecked),
                certification_level: other.certification_level.map(|levels| {
                    levels
                        .into_iter()
                        .map(m::CryptoCertificationLevel::new_unchecked)
                        .collect()
                }),
                mode: other.mode.map(m::CryptoMode::new_unchecked),
                padding: other.padding.map(m::CryptoPadding::new_unchecked),
                crypto_functions: other.crypto_functions.map(|functions| {
                    functions
                        .into_iter()
                        .map(m::CryptoFunction::new_unchecked)
                        .collect()
                }),
                classical_security_level: other.classical_security_level,
                nist_quantum_security_level: other.nist_quantum_security_level,
            }
        }
    }

    const PRIMITIVE_TAG: &str = "primitive";
    #[versioned("1.7")]
    const ALGORITHM_FAMILY_TAG: &str = "algorithmFamily";
    const PARAMETER_SET_IDENTIFIER_TAG: &str = "parameterSetIdentifier";
    const CURVE_TAG: &str = "curve";
    #[versioned("1.7")]
    const ELLIPTIC_CURVE_TAG: &str = "ellipticCurve";
    const EXECUTION_ENVIRONMENT_TAG: &str = "executionEnvironment";
    const IMPLEMENTATION_PLATFORM_TAG: &str = "implementationPlatform";
    const CERTIFICATION_LEVEL_TAG: &str = "certificationLevel";
    const MODE_TAG: &str = "mode";
    const PADDING_TAG: &str = "padding";
    const CRYPTO_FUNCTIONS_TAG: &str = "cryptoFunctions";
    const CRYPTO_FUNCTION_TAG: &str = "cryptoFunction";
    const CLASSICAL_SECURITY_LEVEL_TAG: &str = "classicalSecurityLevel";
    const NIST_QUANTUM_SECURITY_LEVEL_TAG: &str = "nistQuantumSecurityLevel";

    impl ToXml for AlgorithmProperties {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, ALGORITHM_PROPERTIES_TAG)?;
            write_simple_option_tag(writer, PRIMITIVE_TAG, &self.primitive)?;
            #[versioned("1.7")]
            write_simple_option_tag(writer, ALGORITHM_FAMILY_TAG, &self.algorithm_family)?;
            write_simple_option_tag(
                writer,
                PARAMETER_SET_IDENTIFIER_TAG,
                &self.parameter_set_identifier,
            )?;
            write_simple_option_tag(writer, CURVE_TAG, &self.curve)?;
            #[versioned("1.7")]
            write_simple_option_tag(writer, ELLIPTIC_CURVE_TAG, &self.elliptic_curve)?;
            write_simple_option_tag(
                writer,
                EXECUTION_ENVIRONMENT_TAG,
                &self.execution_environment,
            )?;
            write_simple_option_tag(
                writer,
                IMPLEMENTATION_PLATFORM_TAG,
                &self.implementation_platform,
            )?;
            write_repeated_tag(writer, CERTIFICATION_LEVEL_TAG, &self.certification_level)?;
            write_simple_option_tag(writer, MODE_TAG, &self.mode)?;
            write_simple_option_tag(writer, PADDING_TAG, &self.padding)?;
            if let Some(crypto_functions) = &self.crypto_functions {
                write_list_string_tag(
                    writer,
                    CRYPTO_FUNCTIONS_TAG,
                    CRYPTO_FUNCTION_TAG,
                    crypto_functions,
                )?;
            }
            write_integer_option_tag(
                writer,
                CLASSICAL_SECURITY_LEVEL_TAG,
                self.classical_security_level,
            )?;
            write_integer_option_tag(
                writer,
                NIST_QUANTUM_SECURITY_LEVEL_TAG,
                self.nist_quantum_security_level,
            )?;
            write_close_tag(writer, ALGORITHM_PROPERTIES_TAG)
        }
    }

    impl FromXml for AlgorithmProperties {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut primitive: Option<String> = None;
            #[versioned("1.7")]
            let mut algorithm_family: Option<String> = None;
            let mut parameter_set_identifier: Option<String> = None;
            let mut curve: Option<String> = None;
            #[versioned("1.7")]
            let mut elliptic_curve: Option<String> = None;
            let mut execution_environment: Option<String> = None;
            let mut implementation_platform: Option<String> = None;
            let mut certification_level: Option<Vec<String>> = None;
            let mut mode: Option<String> = None;
            let mut padding: Option<String> = None;
            let mut crypto_functions: Option<Vec<String>> = None;
            let mut classical_security_level: Option<u32> = None;
            let mut nist_quantum_security_level: Option<u32> = None;

            read_children(event_reader, element_name, |reader, name, _| {
                match name.local_name.as_str() {
                    PRIMITIVE_TAG => primitive = Some(read_simple_tag(reader, name)?),
                    #[versioned("1.7")]
                    ALGORITHM_FAMILY_TAG => algorithm_family = Some(read_simple_tag(reader, name)?),
                    PARAMETER_SET_IDENTIFIER_TAG => {
                        parameter_set_identifier = Some(read_simple_tag(reader, name)?)
                    }
                    CURVE_TAG => curve = Some(read_simple_tag(reader, name)?),
                    #[versioned("1.7")]
                    ELLIPTIC_CURVE_TAG => elliptic_curve = Some(read_simple_tag(reader, name)?),
                    EXECUTION_ENVIRONMENT_TAG => {
                        execution_environment = Some(read_simple_tag(reader, name)?)
                    }
                    IMPLEMENTATION_PLATFORM_TAG => {
                        implementation_platform = Some(read_simple_tag(reader, name)?)
                    }
                    CERTIFICATION_LEVEL_TAG => certification_level
                        .get_or_insert_with(Vec::new)
                        .push(read_simple_tag(reader, name)?),
                    MODE_TAG => mode = Some(read_simple_tag(reader, name)?),
                    PADDING_TAG => padding = Some(read_simple_tag(reader, name)?),
                    CRYPTO_FUNCTIONS_TAG => {
                        crypto_functions = Some(read_list_tag(reader, name, CRYPTO_FUNCTION_TAG)?)
                    }
                    CLASSICAL_SECURITY_LEVEL_TAG => {
                        classical_security_level = Some(read_u32(reader, name)?)
                    }
                    NIST_QUANTUM_SECURITY_LEVEL_TAG => {
                        nist_quantum_security_level = Some(read_u32(reader, name)?)
                    }
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            Ok(Self {
                primitive,
                #[versioned("1.7")]
                algorithm_family,
                parameter_set_identifier,
                curve,
                #[versioned("1.7")]
                elliptic_curve,
                execution_environment,
                implementation_platform,
                certification_level,
                mode,
                padding,
                crypto_functions,
                classical_security_level,
                nist_quantum_security_level,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CertificateProperties {
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        serial_number: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subject_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        issuer_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        not_valid_before: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        not_valid_after: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature_algorithm_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subject_public_key_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        certificate_format: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        certificate_extension: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        certificate_file_extension: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        fingerprint: Option<Hash>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        certificate_state: Option<Vec<CertificateState>>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        creation_date: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        activation_date: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        deactivation_date: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        revocation_date: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        destruction_date: Option<String>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        certificate_extensions: Option<Vec<CertificateExtension>>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        related_cryptographic_assets: Option<Vec<RelatedCryptographicAsset>>,
    }

    impl From<models::crypto_properties::CertificateProperties> for CertificateProperties {
        fn from(other: models::crypto_properties::CertificateProperties) -> Self {
            Self {
                #[versioned("1.7")]
                serial_number: other.serial_number,
                subject_name: other.subject_name,
                issuer_name: other.issuer_name,
                not_valid_before: other.not_valid_before.map(|d| d.0),
                not_valid_after: other.not_valid_after.map(|d| d.0),
                signature_algorithm_ref: other.signature_algorithm_ref.map(|r| r.0),
                subject_public_key_ref: other.subject_public_key_ref.map(|r| r.0),
                certificate_format: other.certificate_format,
                certificate_extension: other.certificate_extension,
                #[versioned("1.7")]
                certificate_file_extension: other.certificate_file_extension,
                #[versioned("1.7")]
                fingerprint: convert_optional(other.fingerprint),
                #[versioned("1.7")]
                certificate_state: convert_optional_vec(other.certificate_state),
                #[versioned("1.7")]
                creation_date: other.creation_date.map(|d| d.0),
                #[versioned("1.7")]
                activation_date: other.activation_date.map(|d| d.0),
                #[versioned("1.7")]
                deactivation_date: other.deactivation_date.map(|d| d.0),
                #[versioned("1.7")]
                revocation_date: other.revocation_date.map(|d| d.0),
                #[versioned("1.7")]
                destruction_date: other.destruction_date.map(|d| d.0),
                #[versioned("1.7")]
                certificate_extensions: convert_optional_vec(other.certificate_extensions),
                #[versioned("1.7")]
                related_cryptographic_assets: convert_optional_vec(
                    other.related_cryptographic_assets,
                ),
            }
        }
    }

    impl From<CertificateProperties> for models::crypto_properties::CertificateProperties {
        fn from(other: CertificateProperties) -> Self {
            Self {
                #[versioned("1.6")]
                serial_number: None,
                #[versioned("1.7")]
                serial_number: other.serial_number,
                subject_name: other.subject_name,
                issuer_name: other.issuer_name,
                not_valid_before: other.not_valid_before.map(DateTime),
                not_valid_after: other.not_valid_after.map(DateTime),
                signature_algorithm_ref: other.signature_algorithm_ref.map(BomReference),
                subject_public_key_ref: other.subject_public_key_ref.map(BomReference),
                certificate_format: other.certificate_format,
                certificate_extension: other.certificate_extension,
                #[versioned("1.6")]
                certificate_file_extension: None,
                #[versioned("1.7")]
                certificate_file_extension: other.certificate_file_extension,
                #[versioned("1.6")]
                fingerprint: None,
                #[versioned("1.7")]
                fingerprint: convert_optional(other.fingerprint),
                #[versioned("1.6")]
                certificate_state: None,
                #[versioned("1.7")]
                certificate_state: convert_optional_vec(other.certificate_state),
                #[versioned("1.6")]
                creation_date: None,
                #[versioned("1.7")]
                creation_date: other.creation_date.map(DateTime),
                #[versioned("1.6")]
                activation_date: None,
                #[versioned("1.7")]
                activation_date: other.activation_date.map(DateTime),
                #[versioned("1.6")]
                deactivation_date: None,
                #[versioned("1.7")]
                deactivation_date: other.deactivation_date.map(DateTime),
                #[versioned("1.6")]
                revocation_date: None,
                #[versioned("1.7")]
                revocation_date: other.revocation_date.map(DateTime),
                #[versioned("1.6")]
                destruction_date: None,
                #[versioned("1.7")]
                destruction_date: other.destruction_date.map(DateTime),
                #[versioned("1.6")]
                certificate_extensions: None,
                #[versioned("1.7")]
                certificate_extensions: convert_optional_vec(other.certificate_extensions),
                #[versioned("1.6")]
                related_cryptographic_assets: None,
                #[versioned("1.7")]
                related_cryptographic_assets: convert_optional_vec(
                    other.related_cryptographic_assets,
                ),
            }
        }
    }

    #[versioned("1.7")]
    const SERIAL_NUMBER_TAG: &str = "serialNumber";
    const SUBJECT_NAME_TAG: &str = "subjectName";
    const ISSUER_NAME_TAG: &str = "issuerName";
    const NOT_VALID_BEFORE_TAG: &str = "notValidBefore";
    const NOT_VALID_AFTER_TAG: &str = "notValidAfter";
    const SIGNATURE_ALGORITHM_REF_TAG: &str = "signatureAlgorithmRef";
    const SUBJECT_PUBLIC_KEY_REF_TAG: &str = "subjectPublicKeyRef";
    const CERTIFICATE_FORMAT_TAG: &str = "certificateFormat";
    const CERTIFICATE_EXTENSION_TAG: &str = "certificateExtension";
    #[versioned("1.7")]
    const CERTIFICATE_FILE_EXTENSION_TAG: &str = "certificateFileExtension";
    #[versioned("1.7")]
    const FINGERPRINT_TAG: &str = "fingerprint";
    #[versioned("1.7")]
    const CERTIFICATE_STATE_TAG: &str = "certificateState";
    const CREATION_DATE_TAG: &str = "creationDate";
    const ACTIVATION_DATE_TAG: &str = "activationDate";
    #[versioned("1.7")]
    const DEACTIVATION_DATE_TAG: &str = "deactivationDate";
    #[versioned("1.7")]
    const REVOCATION_DATE_TAG: &str = "revocationDate";
    #[versioned("1.7")]
    const DESTRUCTION_DATE_TAG: &str = "destructionDate";
    #[versioned("1.7")]
    const CERTIFICATE_EXTENSIONS_TAG: &str = "certificateExtensions";
    #[versioned("1.7")]
    const RELATED_CRYPTOGRAPHIC_ASSETS_TAG: &str = "relatedCryptographicAssets";
    #[versioned("1.7")]
    const RELATED_CRYPTOGRAPHIC_ASSET_TAG: &str = "relatedCryptographicAsset";

    impl ToXml for CertificateProperties {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, CERTIFICATE_PROPERTIES_TAG)?;
            #[versioned("1.7")]
            write_simple_option_tag(writer, SERIAL_NUMBER_TAG, &self.serial_number)?;
            write_simple_option_tag(writer, SUBJECT_NAME_TAG, &self.subject_name)?;
            write_simple_option_tag(writer, ISSUER_NAME_TAG, &self.issuer_name)?;
            write_simple_option_tag(writer, NOT_VALID_BEFORE_TAG, &self.not_valid_before)?;
            write_simple_option_tag(writer, NOT_VALID_AFTER_TAG, &self.not_valid_after)?;
            write_simple_option_tag(
                writer,
                SIGNATURE_ALGORITHM_REF_TAG,
                &self.signature_algorithm_ref,
            )?;
            write_simple_option_tag(
                writer,
                SUBJECT_PUBLIC_KEY_REF_TAG,
                &self.subject_public_key_ref,
            )?;
            write_simple_option_tag(writer, CERTIFICATE_FORMAT_TAG, &self.certificate_format)?;
            write_simple_option_tag(
                writer,
                CERTIFICATE_EXTENSION_TAG,
                &self.certificate_extension,
            )?;
            #[versioned("1.7")]
            write_simple_option_tag(
                writer,
                CERTIFICATE_FILE_EXTENSION_TAG,
                &self.certificate_file_extension,
            )?;
            #[versioned("1.7")]
            if let Some(fingerprint) = &self.fingerprint {
                write_fingerprint(writer, fingerprint)?;
            }
            #[versioned("1.7")]
            for state in self.certificate_state.iter().flatten() {
                state.write_xml_element(writer)?;
            }
            #[versioned("1.7")]
            write_simple_option_tag(writer, CREATION_DATE_TAG, &self.creation_date)?;
            #[versioned("1.7")]
            write_simple_option_tag(writer, ACTIVATION_DATE_TAG, &self.activation_date)?;
            #[versioned("1.7")]
            write_simple_option_tag(writer, DEACTIVATION_DATE_TAG, &self.deactivation_date)?;
            #[versioned("1.7")]
            write_simple_option_tag(writer, REVOCATION_DATE_TAG, &self.revocation_date)?;
            #[versioned("1.7")]
            write_simple_option_tag(writer, DESTRUCTION_DATE_TAG, &self.destruction_date)?;
            #[versioned("1.7")]
            if let Some(extensions) = &self.certificate_extensions {
                write_list_tag(writer, CERTIFICATE_EXTENSIONS_TAG, extensions)?;
            }
            #[versioned("1.7")]
            if let Some(assets) = &self.related_cryptographic_assets {
                write_list_tag(writer, RELATED_CRYPTOGRAPHIC_ASSETS_TAG, assets)?;
            }
            write_close_tag(writer, CERTIFICATE_PROPERTIES_TAG)
        }
    }

    impl FromXml for CertificateProperties {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            #[versioned("1.7")]
            let mut serial_number: Option<String> = None;
            let mut subject_name: Option<String> = None;
            let mut issuer_name: Option<String> = None;
            let mut not_valid_before: Option<String> = None;
            let mut not_valid_after: Option<String> = None;
            let mut signature_algorithm_ref: Option<String> = None;
            let mut subject_public_key_ref: Option<String> = None;
            let mut certificate_format: Option<String> = None;
            let mut certificate_extension: Option<String> = None;
            #[versioned("1.7")]
            let mut certificate_file_extension: Option<String> = None;
            #[versioned("1.7")]
            let mut fingerprint: Option<Hash> = None;
            #[versioned("1.7")]
            let mut certificate_state: Option<Vec<CertificateState>> = None;
            #[versioned("1.7")]
            let mut creation_date: Option<String> = None;
            #[versioned("1.7")]
            let mut activation_date: Option<String> = None;
            #[versioned("1.7")]
            let mut deactivation_date: Option<String> = None;
            #[versioned("1.7")]
            let mut revocation_date: Option<String> = None;
            #[versioned("1.7")]
            let mut destruction_date: Option<String> = None;
            #[versioned("1.7")]
            let mut certificate_extensions: Option<Vec<CertificateExtension>> = None;
            #[versioned("1.7")]
            let mut related_cryptographic_assets: Option<
                Vec<RelatedCryptographicAsset>,
            > = None;

            read_children(event_reader, element_name, |reader, name, _attributes| {
                match name.local_name.as_str() {
                    #[versioned("1.7")]
                    SERIAL_NUMBER_TAG => serial_number = Some(read_simple_tag(reader, name)?),
                    SUBJECT_NAME_TAG => subject_name = Some(read_simple_tag(reader, name)?),
                    ISSUER_NAME_TAG => issuer_name = Some(read_simple_tag(reader, name)?),
                    NOT_VALID_BEFORE_TAG => not_valid_before = Some(read_simple_tag(reader, name)?),
                    NOT_VALID_AFTER_TAG => not_valid_after = Some(read_simple_tag(reader, name)?),
                    SIGNATURE_ALGORITHM_REF_TAG => {
                        signature_algorithm_ref = Some(read_simple_tag(reader, name)?)
                    }
                    SUBJECT_PUBLIC_KEY_REF_TAG => {
                        subject_public_key_ref = Some(read_simple_tag(reader, name)?)
                    }
                    CERTIFICATE_FORMAT_TAG => {
                        certificate_format = Some(read_simple_tag(reader, name)?)
                    }
                    CERTIFICATE_EXTENSION_TAG => {
                        certificate_extension = Some(read_simple_tag(reader, name)?)
                    }
                    #[versioned("1.7")]
                    CERTIFICATE_FILE_EXTENSION_TAG => {
                        certificate_file_extension = Some(read_simple_tag(reader, name)?)
                    }
                    #[versioned("1.7")]
                    FINGERPRINT_TAG => {
                        fingerprint = Some(read_fingerprint(reader, name, _attributes)?)
                    }
                    #[versioned("1.7")]
                    CERTIFICATE_STATE_TAG => certificate_state.get_or_insert_with(Vec::new).push(
                        CertificateState::read_xml_element(reader, name, _attributes)?,
                    ),
                    #[versioned("1.7")]
                    CREATION_DATE_TAG => creation_date = Some(read_simple_tag(reader, name)?),
                    #[versioned("1.7")]
                    ACTIVATION_DATE_TAG => activation_date = Some(read_simple_tag(reader, name)?),
                    #[versioned("1.7")]
                    DEACTIVATION_DATE_TAG => {
                        deactivation_date = Some(read_simple_tag(reader, name)?)
                    }
                    #[versioned("1.7")]
                    REVOCATION_DATE_TAG => revocation_date = Some(read_simple_tag(reader, name)?),
                    #[versioned("1.7")]
                    DESTRUCTION_DATE_TAG => destruction_date = Some(read_simple_tag(reader, name)?),
                    #[versioned("1.7")]
                    CERTIFICATE_EXTENSIONS_TAG => {
                        certificate_extensions =
                            Some(read_list_tag(reader, name, CERTIFICATE_EXTENSION_TAG)?)
                    }
                    #[versioned("1.7")]
                    RELATED_CRYPTOGRAPHIC_ASSETS_TAG => {
                        related_cryptographic_assets = Some(read_list_tag(
                            reader,
                            name,
                            RELATED_CRYPTOGRAPHIC_ASSET_TAG,
                        )?)
                    }
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            Ok(Self {
                #[versioned("1.7")]
                serial_number,
                subject_name,
                issuer_name,
                not_valid_before,
                not_valid_after,
                signature_algorithm_ref,
                subject_public_key_ref,
                certificate_format,
                certificate_extension,
                #[versioned("1.7")]
                certificate_file_extension,
                #[versioned("1.7")]
                fingerprint,
                #[versioned("1.7")]
                certificate_state,
                #[versioned("1.7")]
                creation_date,
                #[versioned("1.7")]
                activation_date,
                #[versioned("1.7")]
                deactivation_date,
                #[versioned("1.7")]
                revocation_date,
                #[versioned("1.7")]
                destruction_date,
                #[versioned("1.7")]
                certificate_extensions,
                #[versioned("1.7")]
                related_cryptographic_assets,
            })
        }
    }

    /// The JSON schema `oneOf` distinguishes the variants by their required property.
    #[versioned("1.7")]
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(untagged)]
    pub(crate) enum CertificateState {
        Predefined {
            state: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            reason: Option<String>,
        },
        Custom {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            reason: Option<String>,
        },
    }

    #[versioned("1.7")]
    impl From<models::crypto_properties::CertificateState> for CertificateState {
        fn from(other: models::crypto_properties::CertificateState) -> Self {
            use models::crypto_properties::CertificateState as M;
            match other {
                M::Predefined { state, reason } => Self::Predefined {
                    state: state.to_string(),
                    reason,
                },
                M::Custom {
                    name,
                    description,
                    reason,
                } => Self::Custom {
                    name,
                    description,
                    reason,
                },
            }
        }
    }

    #[versioned("1.7")]
    impl From<CertificateState> for models::crypto_properties::CertificateState {
        fn from(other: CertificateState) -> Self {
            match other {
                CertificateState::Predefined { state, reason } => Self::Predefined {
                    state: models::crypto_properties::CertificateLifecycleState::new_unchecked(
                        state,
                    ),
                    reason,
                },
                CertificateState::Custom {
                    name,
                    description,
                    reason,
                } => Self::Custom {
                    name,
                    description,
                    reason,
                },
            }
        }
    }

    #[versioned("1.7")]
    const STATE_TAG: &str = "state";
    #[versioned("1.7")]
    const REASON_TAG: &str = "reason";
    const NAME_TAG: &str = "name";
    #[versioned("1.7")]
    const DESCRIPTION_TAG: &str = "description";

    #[versioned("1.7")]
    impl ToXml for CertificateState {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, CERTIFICATE_STATE_TAG)?;
            match self {
                Self::Predefined { state, reason } => {
                    write_simple_tag(writer, STATE_TAG, state)?;
                    write_simple_option_tag(writer, REASON_TAG, reason)?;
                }
                Self::Custom {
                    name,
                    description,
                    reason,
                } => {
                    write_simple_tag(writer, NAME_TAG, name)?;
                    write_simple_option_tag(writer, DESCRIPTION_TAG, description)?;
                    write_simple_option_tag(writer, REASON_TAG, reason)?;
                }
            }
            write_close_tag(writer, CERTIFICATE_STATE_TAG)
        }
    }

    #[versioned("1.7")]
    impl FromXml for CertificateState {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut state: Option<String> = None;
            let mut name: Option<String> = None;
            let mut description: Option<String> = None;
            let mut reason: Option<String> = None;

            read_children(event_reader, element_name, |reader, child, _| {
                match child.local_name.as_str() {
                    STATE_TAG => state = Some(read_simple_tag(reader, child)?),
                    NAME_TAG => name = Some(read_simple_tag(reader, child)?),
                    DESCRIPTION_TAG => description = Some(read_simple_tag(reader, child)?),
                    REASON_TAG => reason = Some(read_simple_tag(reader, child)?),
                    _ => read_lax_validation_tag(reader, child)?,
                }
                Ok(())
            })?;

            match (state, name) {
                (Some(state), _) => Ok(Self::Predefined { state, reason }),
                (None, Some(name)) => Ok(Self::Custom {
                    name,
                    description,
                    reason,
                }),
                (None, None) => Err(XmlReadError::required_data_missing(STATE_TAG, element_name)),
            }
        }
    }

    /// The JSON schema `oneOf` distinguishes the variants by their property names.
    #[versioned("1.7")]
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(untagged)]
    pub(crate) enum CertificateExtension {
        #[serde(rename_all = "camelCase")]
        Common {
            common_extension_name: String,
            common_extension_value: String,
        },
        #[serde(rename_all = "camelCase")]
        Custom {
            custom_extension_name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            custom_extension_value: Option<String>,
        },
    }

    #[versioned("1.7")]
    impl From<models::crypto_properties::CertificateExtension> for CertificateExtension {
        fn from(other: models::crypto_properties::CertificateExtension) -> Self {
            use models::crypto_properties::CertificateExtension as M;
            match other {
                M::Common { name, value } => Self::Common {
                    common_extension_name: name.to_string(),
                    common_extension_value: value,
                },
                M::Custom { name, value } => Self::Custom {
                    custom_extension_name: name,
                    custom_extension_value: value,
                },
            }
        }
    }

    #[versioned("1.7")]
    impl From<CertificateExtension> for models::crypto_properties::CertificateExtension {
        fn from(other: CertificateExtension) -> Self {
            match other {
                CertificateExtension::Common {
                    common_extension_name,
                    common_extension_value,
                } => Self::Common {
                    name: models::crypto_properties::CommonCertificateExtensionName::new_unchecked(
                        common_extension_name,
                    ),
                    value: common_extension_value,
                },
                CertificateExtension::Custom {
                    custom_extension_name,
                    custom_extension_value,
                } => Self::Custom {
                    name: custom_extension_name,
                    value: custom_extension_value,
                },
            }
        }
    }

    #[versioned("1.7")]
    const COMMON_EXTENSION_NAME_TAG: &str = "commonExtensionName";
    #[versioned("1.7")]
    const COMMON_EXTENSION_VALUE_TAG: &str = "commonExtensionValue";
    #[versioned("1.7")]
    const CUSTOM_EXTENSION_NAME_TAG: &str = "customExtensionName";
    #[versioned("1.7")]
    const CUSTOM_EXTENSION_VALUE_TAG: &str = "customExtensionValue";

    #[versioned("1.7")]
    impl ToXml for CertificateExtension {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, CERTIFICATE_EXTENSION_TAG)?;
            match self {
                Self::Common {
                    common_extension_name,
                    common_extension_value,
                } => {
                    write_simple_tag(writer, COMMON_EXTENSION_NAME_TAG, common_extension_name)?;
                    write_simple_tag(writer, COMMON_EXTENSION_VALUE_TAG, common_extension_value)?;
                }
                Self::Custom {
                    custom_extension_name,
                    custom_extension_value,
                } => {
                    write_simple_tag(writer, CUSTOM_EXTENSION_NAME_TAG, custom_extension_name)?;
                    write_simple_option_tag(
                        writer,
                        CUSTOM_EXTENSION_VALUE_TAG,
                        custom_extension_value,
                    )?;
                }
            }
            write_close_tag(writer, CERTIFICATE_EXTENSION_TAG)
        }
    }

    #[versioned("1.7")]
    impl FromXml for CertificateExtension {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut common_name: Option<String> = None;
            let mut common_value: Option<String> = None;
            let mut custom_name: Option<String> = None;
            let mut custom_value: Option<String> = None;

            read_children(event_reader, element_name, |reader, name, _| {
                match name.local_name.as_str() {
                    COMMON_EXTENSION_NAME_TAG => common_name = Some(read_simple_tag(reader, name)?),
                    COMMON_EXTENSION_VALUE_TAG => {
                        common_value = Some(read_simple_tag(reader, name)?)
                    }
                    CUSTOM_EXTENSION_NAME_TAG => custom_name = Some(read_simple_tag(reader, name)?),
                    CUSTOM_EXTENSION_VALUE_TAG => {
                        custom_value = Some(read_simple_tag(reader, name)?)
                    }
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            match (common_name, custom_name) {
                (Some(common_extension_name), _) => Ok(Self::Common {
                    common_extension_name,
                    common_extension_value: common_value.ok_or_else(|| {
                        XmlReadError::required_data_missing(
                            COMMON_EXTENSION_VALUE_TAG,
                            element_name,
                        )
                    })?,
                }),
                (None, Some(custom_extension_name)) => Ok(Self::Custom {
                    custom_extension_name,
                    custom_extension_value: custom_value,
                }),
                (None, None) => Err(XmlReadError::required_data_missing(
                    CUSTOM_EXTENSION_NAME_TAG,
                    element_name,
                )),
            }
        }
    }

    #[versioned("1.7")]
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub(crate) struct RelatedCryptographicAsset {
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        asset_type: Option<String>,
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        reference: Option<String>,
    }

    #[versioned("1.7")]
    impl From<models::crypto_properties::RelatedCryptographicAsset> for RelatedCryptographicAsset {
        fn from(other: models::crypto_properties::RelatedCryptographicAsset) -> Self {
            Self {
                asset_type: other.asset_type,
                reference: other.reference.map(|r| r.0),
            }
        }
    }

    #[versioned("1.7")]
    impl From<RelatedCryptographicAsset> for models::crypto_properties::RelatedCryptographicAsset {
        fn from(other: RelatedCryptographicAsset) -> Self {
            Self {
                asset_type: other.asset_type,
                reference: other.reference.map(BomReference),
            }
        }
    }

    const TYPE_TAG: &str = "type";
    #[versioned("1.7")]
    const REF_TAG: &str = "ref";

    #[versioned("1.7")]
    impl ToXml for RelatedCryptographicAsset {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, RELATED_CRYPTOGRAPHIC_ASSET_TAG)?;
            write_simple_option_tag(writer, TYPE_TAG, &self.asset_type)?;
            write_simple_option_tag(writer, REF_TAG, &self.reference)?;
            write_close_tag(writer, RELATED_CRYPTOGRAPHIC_ASSET_TAG)
        }
    }

    #[versioned("1.7")]
    impl FromXml for RelatedCryptographicAsset {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut asset_type: Option<String> = None;
            let mut reference: Option<String> = None;

            read_children(event_reader, element_name, |reader, name, _| {
                match name.local_name.as_str() {
                    TYPE_TAG => asset_type = Some(read_simple_tag(reader, name)?),
                    REF_TAG => reference = Some(read_simple_tag(reader, name)?),
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            Ok(Self {
                asset_type,
                reference,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct RelatedCryptoMaterialProperties {
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        material_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        algorithm_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        creation_date: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        activation_date: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        update_date: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expiration_date: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        secured_by: Option<SecuredBy>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        fingerprint: Option<Hash>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        related_cryptographic_assets: Option<Vec<RelatedCryptographicAsset>>,
    }

    impl From<models::crypto_properties::RelatedCryptoMaterialProperties>
        for RelatedCryptoMaterialProperties
    {
        fn from(other: models::crypto_properties::RelatedCryptoMaterialProperties) -> Self {
            Self {
                material_type: other.material_type.map(|v| v.to_string()),
                id: other.id,
                state: other.state.map(|v| v.to_string()),
                algorithm_ref: other.algorithm_ref.map(|r| r.0),
                creation_date: other.creation_date.map(|d| d.0),
                activation_date: other.activation_date.map(|d| d.0),
                update_date: other.update_date.map(|d| d.0),
                expiration_date: other.expiration_date.map(|d| d.0),
                value: other.value,
                size: other.size,
                format: other.format,
                secured_by: convert_optional(other.secured_by),
                #[versioned("1.7")]
                fingerprint: convert_optional(other.fingerprint),
                #[versioned("1.7")]
                related_cryptographic_assets: convert_optional_vec(
                    other.related_cryptographic_assets,
                ),
            }
        }
    }

    impl From<RelatedCryptoMaterialProperties>
        for models::crypto_properties::RelatedCryptoMaterialProperties
    {
        fn from(other: RelatedCryptoMaterialProperties) -> Self {
            use models::crypto_properties as m;
            Self {
                material_type: other
                    .material_type
                    .map(m::RelatedCryptoMaterialType::new_unchecked),
                id: other.id,
                state: other
                    .state
                    .map(m::RelatedCryptoMaterialState::new_unchecked),
                algorithm_ref: other.algorithm_ref.map(BomReference),
                creation_date: other.creation_date.map(DateTime),
                activation_date: other.activation_date.map(DateTime),
                update_date: other.update_date.map(DateTime),
                expiration_date: other.expiration_date.map(DateTime),
                value: other.value,
                size: other.size,
                format: other.format,
                secured_by: convert_optional(other.secured_by),
                #[versioned("1.6")]
                fingerprint: None,
                #[versioned("1.7")]
                fingerprint: convert_optional(other.fingerprint),
                #[versioned("1.6")]
                related_cryptographic_assets: None,
                #[versioned("1.7")]
                related_cryptographic_assets: convert_optional_vec(
                    other.related_cryptographic_assets,
                ),
            }
        }
    }

    const ID_TAG: &str = "id";
    const MATERIAL_STATE_TAG: &str = "state";
    const ALGORITHM_REF_TAG: &str = "algorithmRef";
    const UPDATE_DATE_TAG: &str = "updateDate";
    const EXPIRATION_DATE_TAG: &str = "expirationDate";
    const VALUE_TAG: &str = "value";
    const SIZE_TAG: &str = "size";
    const FORMAT_TAG: &str = "format";
    const SECURED_BY_TAG: &str = "securedBy";

    impl ToXml for RelatedCryptoMaterialProperties {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, RELATED_CRYPTO_MATERIAL_PROPERTIES_TAG)?;
            write_simple_option_tag(writer, TYPE_TAG, &self.material_type)?;
            write_simple_option_tag(writer, ID_TAG, &self.id)?;
            write_simple_option_tag(writer, MATERIAL_STATE_TAG, &self.state)?;
            write_simple_option_tag(writer, ALGORITHM_REF_TAG, &self.algorithm_ref)?;
            write_simple_option_tag(writer, CREATION_DATE_TAG, &self.creation_date)?;
            write_simple_option_tag(writer, ACTIVATION_DATE_TAG, &self.activation_date)?;
            write_simple_option_tag(writer, UPDATE_DATE_TAG, &self.update_date)?;
            write_simple_option_tag(writer, EXPIRATION_DATE_TAG, &self.expiration_date)?;
            write_simple_option_tag(writer, VALUE_TAG, &self.value)?;
            write_integer_option_tag(writer, SIZE_TAG, self.size)?;
            write_simple_option_tag(writer, FORMAT_TAG, &self.format)?;
            self.secured_by.write_xml_element(writer)?;
            #[versioned("1.7")]
            if let Some(fingerprint) = &self.fingerprint {
                write_fingerprint(writer, fingerprint)?;
            }
            #[versioned("1.7")]
            if let Some(assets) = &self.related_cryptographic_assets {
                write_list_tag(writer, RELATED_CRYPTOGRAPHIC_ASSETS_TAG, assets)?;
            }
            write_close_tag(writer, RELATED_CRYPTO_MATERIAL_PROPERTIES_TAG)
        }
    }

    impl FromXml for RelatedCryptoMaterialProperties {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut material_type: Option<String> = None;
            let mut id: Option<String> = None;
            let mut state: Option<String> = None;
            let mut algorithm_ref: Option<String> = None;
            let mut creation_date: Option<String> = None;
            let mut activation_date: Option<String> = None;
            let mut update_date: Option<String> = None;
            let mut expiration_date: Option<String> = None;
            let mut value: Option<String> = None;
            let mut size: Option<i64> = None;
            let mut format: Option<String> = None;
            let mut secured_by: Option<SecuredBy> = None;
            #[versioned("1.7")]
            let mut fingerprint: Option<Hash> = None;
            #[versioned("1.7")]
            let mut related_cryptographic_assets: Option<
                Vec<RelatedCryptographicAsset>,
            > = None;

            read_children(event_reader, element_name, |reader, name, attributes| {
                match name.local_name.as_str() {
                    TYPE_TAG => material_type = Some(read_simple_tag(reader, name)?),
                    ID_TAG => id = Some(read_simple_tag(reader, name)?),
                    MATERIAL_STATE_TAG => state = Some(read_simple_tag(reader, name)?),
                    ALGORITHM_REF_TAG => algorithm_ref = Some(read_simple_tag(reader, name)?),
                    CREATION_DATE_TAG => creation_date = Some(read_simple_tag(reader, name)?),
                    ACTIVATION_DATE_TAG => activation_date = Some(read_simple_tag(reader, name)?),
                    UPDATE_DATE_TAG => update_date = Some(read_simple_tag(reader, name)?),
                    EXPIRATION_DATE_TAG => expiration_date = Some(read_simple_tag(reader, name)?),
                    VALUE_TAG => value = Some(read_simple_tag(reader, name)?),
                    SIZE_TAG => size = Some(read_i64(reader, name)?),
                    FORMAT_TAG => format = Some(read_simple_tag(reader, name)?),
                    SECURED_BY_TAG => {
                        secured_by = Some(SecuredBy::read_xml_element(reader, name, attributes)?)
                    }
                    #[versioned("1.7")]
                    FINGERPRINT_TAG => {
                        fingerprint = Some(read_fingerprint(reader, name, attributes)?)
                    }
                    #[versioned("1.7")]
                    RELATED_CRYPTOGRAPHIC_ASSETS_TAG => {
                        related_cryptographic_assets = Some(read_list_tag(
                            reader,
                            name,
                            RELATED_CRYPTOGRAPHIC_ASSET_TAG,
                        )?)
                    }
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            Ok(Self {
                material_type,
                id,
                state,
                algorithm_ref,
                creation_date,
                activation_date,
                update_date,
                expiration_date,
                value,
                size,
                format,
                secured_by,
                #[versioned("1.7")]
                fingerprint,
                #[versioned("1.7")]
                related_cryptographic_assets,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct SecuredBy {
        #[serde(skip_serializing_if = "Option::is_none")]
        mechanism: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        algorithm_ref: Option<String>,
    }

    impl From<models::crypto_properties::SecuredBy> for SecuredBy {
        fn from(other: models::crypto_properties::SecuredBy) -> Self {
            Self {
                mechanism: other.mechanism,
                algorithm_ref: other.algorithm_ref.map(|r| r.0),
            }
        }
    }

    impl From<SecuredBy> for models::crypto_properties::SecuredBy {
        fn from(other: SecuredBy) -> Self {
            Self {
                mechanism: other.mechanism,
                algorithm_ref: other.algorithm_ref.map(BomReference),
            }
        }
    }

    const MECHANISM_TAG: &str = "mechanism";

    impl ToXml for SecuredBy {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, SECURED_BY_TAG)?;
            write_simple_option_tag(writer, MECHANISM_TAG, &self.mechanism)?;
            write_simple_option_tag(writer, ALGORITHM_REF_TAG, &self.algorithm_ref)?;
            write_close_tag(writer, SECURED_BY_TAG)
        }
    }

    impl FromXml for SecuredBy {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut mechanism: Option<String> = None;
            let mut algorithm_ref: Option<String> = None;

            read_children(event_reader, element_name, |reader, name, _| {
                match name.local_name.as_str() {
                    MECHANISM_TAG => mechanism = Some(read_simple_tag(reader, name)?),
                    ALGORITHM_REF_TAG => algorithm_ref = Some(read_simple_tag(reader, name)?),
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            Ok(Self {
                mechanism,
                algorithm_ref,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct ProtocolProperties {
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        protocol_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cipher_suites: Option<Vec<CipherSuite>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ikev2_transform_types: Option<Ikev2TransformTypes>,
        #[serde(skip_serializing_if = "Option::is_none")]
        crypto_ref_array: Option<Vec<String>>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        related_cryptographic_assets: Option<Vec<RelatedCryptographicAsset>>,
    }

    impl From<models::crypto_properties::ProtocolProperties> for ProtocolProperties {
        fn from(other: models::crypto_properties::ProtocolProperties) -> Self {
            Self {
                protocol_type: other.protocol_type.map(|v| v.to_string()),
                version: other.version,
                cipher_suites: convert_optional_vec(other.cipher_suites),
                ikev2_transform_types: convert_optional(other.ikev2_transform_types),
                crypto_ref_array: other
                    .crypto_ref_array
                    .map(|refs| refs.into_iter().map(|r| r.0).collect()),
                #[versioned("1.7")]
                related_cryptographic_assets: convert_optional_vec(
                    other.related_cryptographic_assets,
                ),
            }
        }
    }

    impl From<ProtocolProperties> for models::crypto_properties::ProtocolProperties {
        fn from(other: ProtocolProperties) -> Self {
            Self {
                protocol_type: other
                    .protocol_type
                    .map(models::crypto_properties::ProtocolType::new_unchecked),
                version: other.version,
                cipher_suites: convert_optional_vec(other.cipher_suites),
                ikev2_transform_types: convert_optional(other.ikev2_transform_types),
                crypto_ref_array: other
                    .crypto_ref_array
                    .map(|refs| refs.into_iter().map(BomReference).collect()),
                #[versioned("1.6")]
                related_cryptographic_assets: None,
                #[versioned("1.7")]
                related_cryptographic_assets: convert_optional_vec(
                    other.related_cryptographic_assets,
                ),
            }
        }
    }

    const VERSION_TAG: &str = "version";
    const CIPHER_SUITES_TAG: &str = "cipherSuites";
    const CIPHER_SUITE_TAG: &str = "cipherSuite";
    const IKEV2_TRANSFORM_TYPES_TAG: &str = "ikev2TransformTypes";
    const CRYPTO_REF_TAG: &str = "cryptoRef";

    impl ToXml for ProtocolProperties {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, PROTOCOL_PROPERTIES_TAG)?;
            write_simple_option_tag(writer, TYPE_TAG, &self.protocol_type)?;
            write_simple_option_tag(writer, VERSION_TAG, &self.version)?;
            if let Some(cipher_suites) = &self.cipher_suites {
                write_list_tag(writer, CIPHER_SUITES_TAG, cipher_suites)?;
            }
            self.ikev2_transform_types.write_xml_element(writer)?;
            write_repeated_tag(writer, CRYPTO_REF_TAG, &self.crypto_ref_array)?;
            #[versioned("1.7")]
            if let Some(assets) = &self.related_cryptographic_assets {
                write_list_tag(writer, RELATED_CRYPTOGRAPHIC_ASSETS_TAG, assets)?;
            }
            write_close_tag(writer, PROTOCOL_PROPERTIES_TAG)
        }
    }

    impl FromXml for ProtocolProperties {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut protocol_type: Option<String> = None;
            let mut version: Option<String> = None;
            let mut cipher_suites: Option<Vec<CipherSuite>> = None;
            let mut ikev2_transform_types: Option<Ikev2TransformTypes> = None;
            let mut crypto_ref_array: Option<Vec<String>> = None;
            #[versioned("1.7")]
            let mut related_cryptographic_assets: Option<
                Vec<RelatedCryptographicAsset>,
            > = None;

            read_children(event_reader, element_name, |reader, name, attributes| {
                match name.local_name.as_str() {
                    TYPE_TAG => protocol_type = Some(read_simple_tag(reader, name)?),
                    VERSION_TAG => version = Some(read_simple_tag(reader, name)?),
                    CIPHER_SUITES_TAG => {
                        cipher_suites = Some(read_list_tag(reader, name, CIPHER_SUITE_TAG)?)
                    }
                    IKEV2_TRANSFORM_TYPES_TAG => {
                        ikev2_transform_types = Some(Ikev2TransformTypes::read_xml_element(
                            reader, name, attributes,
                        )?)
                    }
                    CRYPTO_REF_TAG => crypto_ref_array
                        .get_or_insert_with(Vec::new)
                        .push(read_simple_tag(reader, name)?),
                    #[versioned("1.7")]
                    RELATED_CRYPTOGRAPHIC_ASSETS_TAG => {
                        related_cryptographic_assets = Some(read_list_tag(
                            reader,
                            name,
                            RELATED_CRYPTOGRAPHIC_ASSET_TAG,
                        )?)
                    }
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            Ok(Self {
                protocol_type,
                version,
                cipher_suites,
                ikev2_transform_types,
                crypto_ref_array,
                #[versioned("1.7")]
                related_cryptographic_assets,
            })
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CipherSuite {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        algorithms: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        identifiers: Option<Vec<String>>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        tls_groups: Option<Vec<String>>,
        #[versioned("1.7")]
        #[serde(skip_serializing_if = "Option::is_none")]
        tls_signature_schemes: Option<Vec<String>>,
    }

    impl From<models::crypto_properties::CipherSuite> for CipherSuite {
        fn from(other: models::crypto_properties::CipherSuite) -> Self {
            Self {
                name: other.name,
                algorithms: other
                    .algorithms
                    .map(|refs| refs.into_iter().map(|r| r.0).collect()),
                identifiers: other.identifiers,
                #[versioned("1.7")]
                tls_groups: other.tls_groups,
                #[versioned("1.7")]
                tls_signature_schemes: other.tls_signature_schemes,
            }
        }
    }

    impl From<CipherSuite> for models::crypto_properties::CipherSuite {
        fn from(other: CipherSuite) -> Self {
            Self {
                name: other.name,
                algorithms: other
                    .algorithms
                    .map(|refs| refs.into_iter().map(BomReference).collect()),
                identifiers: other.identifiers,
                #[versioned("1.6")]
                tls_groups: None,
                #[versioned("1.7")]
                tls_groups: other.tls_groups,
                #[versioned("1.6")]
                tls_signature_schemes: None,
                #[versioned("1.7")]
                tls_signature_schemes: other.tls_signature_schemes,
            }
        }
    }

    const ALGORITHMS_TAG: &str = "algorithms";
    const ALGORITHM_TAG: &str = "algorithm";
    const IDENTIFIERS_TAG: &str = "identifiers";
    const IDENTIFIER_TAG: &str = "identifier";
    #[versioned("1.7")]
    const TLS_GROUPS_TAG: &str = "tlsGroups";
    #[versioned("1.7")]
    const GROUP_TAG: &str = "group";
    #[versioned("1.7")]
    const TLS_SIGNATURE_SCHEMES_TAG: &str = "tlsSignatureSchemes";
    #[versioned("1.7")]
    const SIGNATURE_SCHEME_TAG: &str = "signatureScheme";

    impl ToXml for CipherSuite {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, CIPHER_SUITE_TAG)?;
            write_simple_option_tag(writer, NAME_TAG, &self.name)?;
            if let Some(algorithms) = &self.algorithms {
                write_list_string_tag(writer, ALGORITHMS_TAG, ALGORITHM_TAG, algorithms)?;
            }
            if let Some(identifiers) = &self.identifiers {
                write_list_string_tag(writer, IDENTIFIERS_TAG, IDENTIFIER_TAG, identifiers)?;
            }
            #[versioned("1.7")]
            if let Some(tls_groups) = &self.tls_groups {
                write_list_string_tag(writer, TLS_GROUPS_TAG, GROUP_TAG, tls_groups)?;
            }
            #[versioned("1.7")]
            if let Some(schemes) = &self.tls_signature_schemes {
                write_list_string_tag(
                    writer,
                    TLS_SIGNATURE_SCHEMES_TAG,
                    SIGNATURE_SCHEME_TAG,
                    schemes,
                )?;
            }
            write_close_tag(writer, CIPHER_SUITE_TAG)
        }
    }

    impl FromXml for CipherSuite {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut name: Option<String> = None;
            let mut algorithms: Option<Vec<String>> = None;
            let mut identifiers: Option<Vec<String>> = None;
            #[versioned("1.7")]
            let mut tls_groups: Option<Vec<String>> = None;
            #[versioned("1.7")]
            let mut tls_signature_schemes: Option<Vec<String>> = None;

            read_children(event_reader, element_name, |reader, child, _| {
                match child.local_name.as_str() {
                    NAME_TAG => name = Some(read_simple_tag(reader, child)?),
                    ALGORITHMS_TAG => {
                        algorithms = Some(read_list_tag(reader, child, ALGORITHM_TAG)?)
                    }
                    IDENTIFIERS_TAG => {
                        identifiers = Some(read_list_tag(reader, child, IDENTIFIER_TAG)?)
                    }
                    #[versioned("1.7")]
                    TLS_GROUPS_TAG => tls_groups = Some(read_list_tag(reader, child, GROUP_TAG)?),
                    #[versioned("1.7")]
                    TLS_SIGNATURE_SCHEMES_TAG => {
                        tls_signature_schemes =
                            Some(read_list_tag(reader, child, SIGNATURE_SCHEME_TAG)?)
                    }
                    _ => read_lax_validation_tag(reader, child)?,
                }
                Ok(())
            })?;

            Ok(Self {
                name,
                algorithms,
                identifiers,
                #[versioned("1.7")]
                tls_groups,
                #[versioned("1.7")]
                tls_signature_schemes,
            })
        }
    }

    #[versioned("1.6")]
    type Ikev2Entries<T> = Option<Vec<T>>;
    #[versioned("1.7")]
    type Ikev2Entries<T> = Option<Vec<Ikev2Transform<T>>>;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub(crate) struct Ikev2TransformTypes {
        #[serde(skip_serializing_if = "Option::is_none")]
        encr: Ikev2Entries<Ikev2Encryption>,
        #[serde(skip_serializing_if = "Option::is_none")]
        prf: Ikev2Entries<Ikev2NamedTransform>,
        #[serde(skip_serializing_if = "Option::is_none")]
        integ: Ikev2Entries<Ikev2NamedTransform>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ke: Ikev2Entries<Ikev2KeyExchange>,
        #[serde(skip_serializing_if = "Option::is_none")]
        esn: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        auth: Ikev2Entries<Ikev2NamedTransform>,
    }

    impl From<models::crypto_properties::Ikev2TransformTypes> for Ikev2TransformTypes {
        fn from(other: models::crypto_properties::Ikev2TransformTypes) -> Self {
            Self {
                encr: convert_ikev2_to_spec(other.encr),
                prf: convert_ikev2_to_spec(other.prf),
                integ: convert_ikev2_to_spec(other.integ),
                ke: convert_ikev2_to_spec(other.ke),
                esn: other.esn,
                auth: convert_ikev2_to_spec(other.auth),
            }
        }
    }

    impl From<Ikev2TransformTypes> for models::crypto_properties::Ikev2TransformTypes {
        fn from(other: Ikev2TransformTypes) -> Self {
            Self {
                encr: convert_ikev2_from_spec(other.encr),
                prf: convert_ikev2_from_spec(other.prf),
                integ: convert_ikev2_from_spec(other.integ),
                ke: convert_ikev2_from_spec(other.ke),
                esn: other.esn,
                auth: convert_ikev2_from_spec(other.auth),
            }
        }
    }

    const ENCR_TAG: &str = "encr";
    const PRF_TAG: &str = "prf";
    const INTEG_TAG: &str = "integ";
    const KE_TAG: &str = "ke";
    const ESN_TAG: &str = "esn";
    const AUTH_TAG: &str = "auth";

    impl ToXml for Ikev2TransformTypes {
        fn write_xml_element<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_start_tag(writer, IKEV2_TRANSFORM_TYPES_TAG)?;
            write_ikev2_entries(writer, ENCR_TAG, &self.encr)?;
            write_ikev2_entries(writer, PRF_TAG, &self.prf)?;
            write_ikev2_entries(writer, INTEG_TAG, &self.integ)?;
            write_ikev2_entries(writer, KE_TAG, &self.ke)?;
            if let Some(esn) = self.esn {
                write_simple_tag(writer, ESN_TAG, &esn.to_string())?;
            }
            write_ikev2_entries(writer, AUTH_TAG, &self.auth)?;
            write_close_tag(writer, IKEV2_TRANSFORM_TYPES_TAG)
        }
    }

    impl FromXml for Ikev2TransformTypes {
        fn read_xml_element<R: Read>(
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
            _attributes: &[OwnedAttribute],
        ) -> Result<Self, XmlReadError> {
            let mut encr: Ikev2Entries<Ikev2Encryption> = None;
            let mut prf: Ikev2Entries<Ikev2NamedTransform> = None;
            let mut integ: Ikev2Entries<Ikev2NamedTransform> = None;
            let mut ke: Ikev2Entries<Ikev2KeyExchange> = None;
            let mut esn: Option<bool> = None;
            let mut auth: Ikev2Entries<Ikev2NamedTransform> = None;

            read_children(event_reader, element_name, |reader, name, _| {
                match name.local_name.as_str() {
                    ENCR_TAG => read_ikev2_entry(reader, name, &mut encr)?,
                    PRF_TAG => read_ikev2_entry(reader, name, &mut prf)?,
                    INTEG_TAG => read_ikev2_entry(reader, name, &mut integ)?,
                    KE_TAG => read_ikev2_entry(reader, name, &mut ke)?,
                    ESN_TAG => {
                        esn = Some(bool::from_xml_value(
                            ESN_TAG,
                            read_simple_tag(reader, name)?,
                        )?)
                    }
                    AUTH_TAG => read_ikev2_entry(reader, name, &mut auth)?,
                    _ => read_lax_validation_tag(reader, name)?,
                }
                Ok(())
            })?;

            Ok(Self {
                encr,
                prf,
                integ,
                ke,
                esn,
                auth,
            })
        }
    }

    /// In 1.6 every IKEv2 transform is a bare reference, so the transform types collapse to
    /// `String` and the structured variants of the model are dropped (validation rejects them).
    #[versioned("1.6")]
    type Ikev2Encryption = String;
    #[versioned("1.6")]
    type Ikev2NamedTransform = String;
    #[versioned("1.6")]
    type Ikev2KeyExchange = String;

    #[versioned("1.6")]
    fn ikev2_reference<T>(entry: models::crypto_properties::Ikev2Transform<T>) -> Option<String> {
        match entry {
            models::crypto_properties::Ikev2Transform::Reference(reference) => Some(reference.0),
            models::crypto_properties::Ikev2Transform::Transform(_) => None,
        }
    }

    #[versioned("1.6")]
    fn convert_ikev2_to_spec<T>(
        entries: Option<Vec<models::crypto_properties::Ikev2Transform<T>>>,
    ) -> Option<Vec<String>> {
        entries.map(|entries| entries.into_iter().filter_map(ikev2_reference).collect())
    }

    #[versioned("1.6")]
    fn convert_ikev2_from_spec<T>(
        entries: Option<Vec<String>>,
    ) -> Option<Vec<models::crypto_properties::Ikev2Transform<T>>> {
        entries.map(|entries| {
            entries
                .into_iter()
                .map(|reference| {
                    models::crypto_properties::Ikev2Transform::Reference(BomReference(reference))
                })
                .collect()
        })
    }

    #[versioned("1.7")]
    fn convert_ikev2_to_spec<M, S: From<M>>(
        entries: Option<Vec<models::crypto_properties::Ikev2Transform<M>>>,
    ) -> Ikev2Entries<S> {
        convert_optional_vec(entries)
    }

    #[versioned("1.7")]
    fn convert_ikev2_from_spec<S, M: From<S>>(
        entries: Ikev2Entries<S>,
    ) -> Option<Vec<models::crypto_properties::Ikev2Transform<M>>> {
        convert_optional_vec(entries)
    }

    #[versioned("1.6")]
    fn write_ikev2_entries<W: Write>(
        writer: &mut EventWriter<W>,
        tag: &str,
        entries: &Option<Vec<String>>,
    ) -> Result<(), XmlWriteError> {
        write_repeated_tag(writer, tag, entries)
    }

    #[versioned("1.6")]
    fn read_ikev2_entry<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        entries: &mut Option<Vec<String>>,
    ) -> Result<(), XmlReadError> {
        entries
            .get_or_insert_with(Vec::new)
            .push(read_simple_tag(event_reader, element_name)?);
        Ok(())
    }

    /// The JSON schema allows either an array of references (deprecated) or an array of
    /// structured transforms for each IKEv2 transform type.
    #[versioned("1.7")]
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(untagged)]
    pub(crate) enum Ikev2Transform<T> {
        Reference(String),
        Transform(T),
    }

    #[versioned("1.7")]
    impl<M, S: From<M>> From<models::crypto_properties::Ikev2Transform<M>> for Ikev2Transform<S> {
        fn from(other: models::crypto_properties::Ikev2Transform<M>) -> Self {
            match other {
                models::crypto_properties::Ikev2Transform::Reference(reference) => {
                    Self::Reference(reference.0)
                }
                models::crypto_properties::Ikev2Transform::Transform(transform) => {
                    Self::Transform(transform.into())
                }
            }
        }
    }

    #[versioned("1.7")]
    impl<S, M: From<S>> From<Ikev2Transform<S>> for models::crypto_properties::Ikev2Transform<M> {
        fn from(other: Ikev2Transform<S>) -> Self {
            match other {
                Ikev2Transform::Reference(reference) => Self::Reference(BomReference(reference)),
                Ikev2Transform::Transform(transform) => Self::Transform(transform.into()),
            }
        }
    }

    /// Child elements of a structured IKEv2 transform.
    #[versioned("1.7")]
    trait Ikev2TransformFields: Default {
        fn read_child<R: Read>(
            &mut self,
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
        ) -> Result<(), XmlReadError>;

        fn write_children<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError>;
    }

    #[versioned("1.7")]
    fn write_ikev2_entries<W: Write, T: Ikev2TransformFields>(
        writer: &mut EventWriter<W>,
        tag: &str,
        entries: &Ikev2Entries<T>,
    ) -> Result<(), XmlWriteError> {
        for entry in entries.iter().flatten() {
            match entry {
                Ikev2Transform::Reference(reference) => write_simple_tag(writer, tag, reference)?,
                Ikev2Transform::Transform(transform) => {
                    write_start_tag(writer, tag)?;
                    transform.write_children(writer)?;
                    write_close_tag(writer, tag)?;
                }
            }
        }
        Ok(())
    }

    /// The XSD declares IKEv2 transforms as mixed content: text holds the deprecated reference,
    /// child elements hold the structured transform.
    #[versioned("1.7")]
    fn read_ikev2_entry<R: Read, T: Ikev2TransformFields>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        entries: &mut Ikev2Entries<T>,
    ) -> Result<(), XmlReadError> {
        let mut transform = T::default();
        let mut has_children = false;
        let mut reference: Option<String> = None;

        loop {
            match event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?
            {
                reader::XmlEvent::Characters(text) | reader::XmlEvent::CData(text) => {
                    reference = Some(text)
                }
                reader::XmlEvent::StartElement { name, .. } => {
                    has_children = true;
                    transform.read_child(event_reader, &name)?;
                }
                reader::XmlEvent::EndElement { name } if &name == element_name => break,
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }

        let entry = if has_children {
            Ikev2Transform::Transform(transform)
        } else {
            Ikev2Transform::Reference(reference.unwrap_or_default())
        };
        entries.get_or_insert_with(Vec::new).push(entry);
        Ok(())
    }

    #[versioned("1.7")]
    const KEY_LENGTH_TAG: &str = "keyLength";

    #[versioned("1.7")]
    #[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Ikev2Encryption {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        key_length: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        algorithm: Option<String>,
    }

    #[versioned("1.7")]
    impl From<models::crypto_properties::Ikev2Encryption> for Ikev2Encryption {
        fn from(other: models::crypto_properties::Ikev2Encryption) -> Self {
            Self {
                name: other.name,
                key_length: other.key_length,
                algorithm: other.algorithm.map(|r| r.0),
            }
        }
    }

    #[versioned("1.7")]
    impl From<Ikev2Encryption> for models::crypto_properties::Ikev2Encryption {
        fn from(other: Ikev2Encryption) -> Self {
            Self {
                name: other.name,
                key_length: other.key_length,
                algorithm: other.algorithm.map(BomReference),
            }
        }
    }

    #[versioned("1.7")]
    impl Ikev2TransformFields for Ikev2Encryption {
        fn read_child<R: Read>(
            &mut self,
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
        ) -> Result<(), XmlReadError> {
            match element_name.local_name.as_str() {
                NAME_TAG => self.name = Some(read_simple_tag(event_reader, element_name)?),
                KEY_LENGTH_TAG => self.key_length = Some(read_i64(event_reader, element_name)?),
                ALGORITHM_TAG => {
                    self.algorithm = Some(read_simple_tag(event_reader, element_name)?)
                }
                _ => read_lax_validation_tag(event_reader, element_name)?,
            }
            Ok(())
        }

        fn write_children<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_simple_option_tag(writer, NAME_TAG, &self.name)?;
            write_integer_option_tag(writer, KEY_LENGTH_TAG, self.key_length)?;
            write_simple_option_tag(writer, ALGORITHM_TAG, &self.algorithm)
        }
    }

    #[versioned("1.7")]
    #[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
    pub(crate) struct Ikev2NamedTransform {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        algorithm: Option<String>,
    }

    #[versioned("1.7")]
    impl From<models::crypto_properties::Ikev2NamedTransform> for Ikev2NamedTransform {
        fn from(other: models::crypto_properties::Ikev2NamedTransform) -> Self {
            Self {
                name: other.name,
                algorithm: other.algorithm.map(|r| r.0),
            }
        }
    }

    #[versioned("1.7")]
    impl From<Ikev2NamedTransform> for models::crypto_properties::Ikev2NamedTransform {
        fn from(other: Ikev2NamedTransform) -> Self {
            Self {
                name: other.name,
                algorithm: other.algorithm.map(BomReference),
            }
        }
    }

    #[versioned("1.7")]
    impl Ikev2TransformFields for Ikev2NamedTransform {
        fn read_child<R: Read>(
            &mut self,
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
        ) -> Result<(), XmlReadError> {
            match element_name.local_name.as_str() {
                NAME_TAG => self.name = Some(read_simple_tag(event_reader, element_name)?),
                ALGORITHM_TAG => {
                    self.algorithm = Some(read_simple_tag(event_reader, element_name)?)
                }
                _ => read_lax_validation_tag(event_reader, element_name)?,
            }
            Ok(())
        }

        fn write_children<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_simple_option_tag(writer, NAME_TAG, &self.name)?;
            write_simple_option_tag(writer, ALGORITHM_TAG, &self.algorithm)
        }
    }

    #[versioned("1.7")]
    #[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
    pub(crate) struct Ikev2KeyExchange {
        #[serde(skip_serializing_if = "Option::is_none")]
        group: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        algorithm: Option<String>,
    }

    #[versioned("1.7")]
    impl From<models::crypto_properties::Ikev2KeyExchange> for Ikev2KeyExchange {
        fn from(other: models::crypto_properties::Ikev2KeyExchange) -> Self {
            Self {
                group: other.group,
                algorithm: other.algorithm.map(|r| r.0),
            }
        }
    }

    #[versioned("1.7")]
    impl From<Ikev2KeyExchange> for models::crypto_properties::Ikev2KeyExchange {
        fn from(other: Ikev2KeyExchange) -> Self {
            Self {
                group: other.group,
                algorithm: other.algorithm.map(BomReference),
            }
        }
    }

    #[versioned("1.7")]
    impl Ikev2TransformFields for Ikev2KeyExchange {
        fn read_child<R: Read>(
            &mut self,
            event_reader: &mut EventReader<R>,
            element_name: &OwnedName,
        ) -> Result<(), XmlReadError> {
            match element_name.local_name.as_str() {
                GROUP_TAG => self.group = Some(read_i64(event_reader, element_name)?),
                ALGORITHM_TAG => {
                    self.algorithm = Some(read_simple_tag(event_reader, element_name)?)
                }
                _ => read_lax_validation_tag(event_reader, element_name)?,
            }
            Ok(())
        }

        fn write_children<W: Write>(
            &self,
            writer: &mut EventWriter<W>,
        ) -> Result<(), XmlWriteError> {
            write_integer_option_tag(writer, GROUP_TAG, self.group)?;
            write_simple_option_tag(writer, ALGORITHM_TAG, &self.algorithm)
        }
    }

    /// Writes `<fingerprint alg="...">value</fingerprint>` (XSD `bom:hashType`).
    #[versioned("1.7")]
    fn write_fingerprint<W: Write>(
        writer: &mut EventWriter<W>,
        fingerprint: &Hash,
    ) -> Result<(), XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(FINGERPRINT_TAG).attr("alg", &fingerprint.alg))
            .map_err(to_xml_write_error(FINGERPRINT_TAG))?;
        writer
            .write(writer::XmlEvent::characters(&fingerprint.content.0))
            .map_err(to_xml_write_error(FINGERPRINT_TAG))?;
        write_close_tag(writer, FINGERPRINT_TAG)
    }

    #[versioned("1.7")]
    fn read_fingerprint<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        attributes: &[OwnedAttribute],
    ) -> Result<Hash, XmlReadError> {
        let alg = attribute_or_error(element_name, attributes, "alg")?;
        let content = read_simple_tag(event_reader, element_name)?;
        Ok(Hash {
            alg,
            content: HashValue(content),
        })
    }

    /// Calls `on_child` for every child element until the closing tag of `element_name`.
    fn read_children<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
        mut on_child: impl FnMut(
            &mut EventReader<R>,
            &OwnedName,
            &[OwnedAttribute],
        ) -> Result<(), XmlReadError>,
    ) -> Result<(), XmlReadError> {
        loop {
            match event_reader
                .next()
                .map_err(to_xml_read_error(&element_name.local_name))?
            {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } => on_child(event_reader, &name, &attributes)?,
                reader::XmlEvent::EndElement { name } if &name == element_name => return Ok(()),
                unexpected => return Err(unexpected_element_error(element_name, unexpected)),
            }
        }
    }

    /// Writes one element per item, without a wrapper element (XSD `maxOccurs="unbounded"`).
    fn write_repeated_tag<W: Write>(
        writer: &mut EventWriter<W>,
        tag: &str,
        items: &Option<Vec<String>>,
    ) -> Result<(), XmlWriteError> {
        for item in items.iter().flatten() {
            write_simple_tag(writer, tag, item)?;
        }
        Ok(())
    }

    fn write_integer_option_tag<W: Write>(
        writer: &mut EventWriter<W>,
        tag: &str,
        value: Option<impl ToString>,
    ) -> Result<(), XmlWriteError> {
        match value {
            Some(value) => write_simple_tag(writer, tag, &value.to_string()),
            None => Ok(()),
        }
    }

    fn read_u32<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
    ) -> Result<u32, XmlReadError> {
        let value = read_simple_tag(event_reader, element_name)?;
        u32::from_xml_value(&element_name.local_name, value.trim())
    }

    fn read_i64<R: Read>(
        event_reader: &mut EventReader<R>,
        element_name: &OwnedName,
    ) -> Result<i64, XmlReadError> {
        let value = read_simple_tag(event_reader, element_name)?;
        value
            .trim()
            .parse()
            .map_err(|_| XmlReadError::InvalidParseError {
                value,
                data_type: "xs:integer".to_string(),
                element: element_name.local_name.clone(),
            })
    }

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;
        use crate::models::crypto_properties as m;
        use crate::xml::test::{read_element_from_string, write_element_to_string};

        pub(crate) fn example_crypto_properties() -> CryptoProperties {
            #[versioned("1.6")]
            let ikev2_transform_types = Ikev2TransformTypes {
                encr: Some(vec!["encr-ref".to_string()]),
                prf: Some(vec!["prf-ref".to_string()]),
                integ: Some(vec!["integ-ref".to_string()]),
                ke: Some(vec!["ke-ref".to_string()]),
                esn: Some(true),
                auth: Some(vec!["auth-ref".to_string()]),
            };
            #[versioned("1.7")]
            let ikev2_transform_types = Ikev2TransformTypes {
                encr: Some(vec![
                    Ikev2Transform::Transform(Ikev2Encryption {
                        name: Some("AES-128-GCM".to_string()),
                        key_length: Some(128),
                        algorithm: Some("encr-ref".to_string()),
                    }),
                    Ikev2Transform::Reference("encr-ref-2".to_string()),
                ]),
                prf: Some(vec![Ikev2Transform::Transform(Ikev2NamedTransform {
                    name: Some("PRF_HMAC_SHA2_256".to_string()),
                    algorithm: Some("prf-ref".to_string()),
                })]),
                integ: Some(vec![Ikev2Transform::Reference("integ-ref".to_string())]),
                ke: Some(vec![Ikev2Transform::Transform(Ikev2KeyExchange {
                    group: Some(19),
                    algorithm: Some("ke-ref".to_string()),
                })]),
                esn: Some(true),
                auth: Some(vec![Ikev2Transform::Reference("auth-ref".to_string())]),
            };

            let cipher_suite = CipherSuite {
                name: Some("TLS_AES_128_GCM_SHA256".to_string()),
                algorithms: Some(vec!["aes-ref".to_string()]),
                identifiers: Some(vec!["0x13".to_string(), "0x01".to_string()]),
                #[versioned("1.7")]
                tls_groups: Some(vec!["x25519".to_string()]),
                #[versioned("1.7")]
                tls_signature_schemes: Some(vec!["ecdsa_secp256r1_sha256".to_string()]),
            };

            #[versioned("1.7")]
            let certificate_states = vec![
                CertificateState::Predefined {
                    state: "active".to_string(),
                    reason: Some("issued".to_string()),
                },
                CertificateState::Custom {
                    name: "monitored".to_string(),
                    description: Some("under monitoring".to_string()),
                    reason: None,
                },
            ];
            #[versioned("1.7")]
            let certificate_extensions = vec![
                CertificateExtension::Common {
                    common_extension_name: "keyUsage".to_string(),
                    common_extension_value: "digitalSignature".to_string(),
                },
                CertificateExtension::Custom {
                    custom_extension_name: "1.3.6.1.4.1.99999".to_string(),
                    custom_extension_value: None,
                },
            ];
            #[versioned("1.7")]
            let related_assets = vec![RelatedCryptographicAsset {
                asset_type: Some("publicKey".to_string()),
                reference: Some("key-ref".to_string()),
            }];

            CryptoProperties {
                asset_type: "algorithm".to_string(),
                algorithm_properties: Some(AlgorithmProperties {
                    primitive: Some("ae".to_string()),
                    #[versioned("1.7")]
                    algorithm_family: Some("AES".to_string()),
                    parameter_set_identifier: Some("128".to_string()),
                    curve: Some("secp256r1".to_string()),
                    #[versioned("1.7")]
                    elliptic_curve: Some("secg/secp256r1".to_string()),
                    execution_environment: Some("software-plain-ram".to_string()),
                    implementation_platform: Some("x86_64".to_string()),
                    certification_level: Some(vec!["none".to_string(), "cc-eal4+".to_string()]),
                    mode: Some("gcm".to_string()),
                    padding: Some("pkcs7".to_string()),
                    crypto_functions: Some(vec!["encrypt".to_string(), "decrypt".to_string()]),
                    classical_security_level: Some(128),
                    nist_quantum_security_level: Some(1),
                }),
                certificate_properties: Some(CertificateProperties {
                    #[versioned("1.7")]
                    serial_number: Some("01".to_string()),
                    subject_name: Some("CN = example.com".to_string()),
                    issuer_name: Some("CN = Example CA".to_string()),
                    not_valid_before: Some("2024-01-01T00:00:00Z".to_string()),
                    not_valid_after: Some("2025-01-01T00:00:00Z".to_string()),
                    signature_algorithm_ref: Some("sig-ref".to_string()),
                    subject_public_key_ref: Some("key-ref".to_string()),
                    certificate_format: Some("X.509".to_string()),
                    certificate_extension: Some("crt".to_string()),
                    #[versioned("1.7")]
                    certificate_file_extension: Some("pem".to_string()),
                    #[versioned("1.7")]
                    fingerprint: Some(Hash {
                        alg: "SHA-256".to_string(),
                        content: HashValue(
                            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
                                .to_string(),
                        ),
                    }),
                    #[versioned("1.7")]
                    certificate_state: Some(certificate_states),
                    #[versioned("1.7")]
                    creation_date: Some("2023-12-01T00:00:00Z".to_string()),
                    #[versioned("1.7")]
                    activation_date: Some("2024-01-01T00:00:00Z".to_string()),
                    #[versioned("1.7")]
                    deactivation_date: Some("2024-06-01T00:00:00Z".to_string()),
                    #[versioned("1.7")]
                    revocation_date: Some("2024-07-01T00:00:00Z".to_string()),
                    #[versioned("1.7")]
                    destruction_date: Some("2024-08-01T00:00:00Z".to_string()),
                    #[versioned("1.7")]
                    certificate_extensions: Some(certificate_extensions),
                    #[versioned("1.7")]
                    related_cryptographic_assets: Some(vec![RelatedCryptographicAsset {
                        asset_type: Some("signature".to_string()),
                        reference: Some("sig-ref".to_string()),
                    }]),
                }),
                related_crypto_material_properties: Some(RelatedCryptoMaterialProperties {
                    material_type: Some("public-key".to_string()),
                    id: Some("key-id".to_string()),
                    state: Some("active".to_string()),
                    algorithm_ref: Some("rsa-ref".to_string()),
                    creation_date: Some("2024-01-01T00:00:00Z".to_string()),
                    activation_date: Some("2024-01-02T00:00:00Z".to_string()),
                    update_date: Some("2024-01-03T00:00:00Z".to_string()),
                    expiration_date: Some("2025-01-01T00:00:00Z".to_string()),
                    value: Some("key value".to_string()),
                    size: Some(2048),
                    format: Some("PEM".to_string()),
                    secured_by: Some(SecuredBy {
                        mechanism: Some("Software".to_string()),
                        algorithm_ref: Some("aes-ref".to_string()),
                    }),
                    #[versioned("1.7")]
                    fingerprint: Some(Hash {
                        alg: "SHA-256".to_string(),
                        content: HashValue(
                            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
                                .to_string(),
                        ),
                    }),
                    #[versioned("1.7")]
                    related_cryptographic_assets: Some(related_assets),
                }),
                protocol_properties: Some(ProtocolProperties {
                    protocol_type: Some("ike".to_string()),
                    version: Some("2".to_string()),
                    cipher_suites: Some(vec![cipher_suite]),
                    ikev2_transform_types: Some(ikev2_transform_types),
                    crypto_ref_array: Some(vec!["ref-1".to_string(), "ref-2".to_string()]),
                    #[versioned("1.7")]
                    related_cryptographic_assets: Some(vec![RelatedCryptographicAsset {
                        asset_type: Some("algorithm".to_string()),
                        reference: Some("aes-ref".to_string()),
                    }]),
                }),
                oid: Some("2.16.840.1.101.3.4.1.6".to_string()),
            }
        }

        pub(crate) fn corresponding_crypto_properties() -> m::CryptoProperties {
            #[versioned("1.6")]
            let ikev2_transform_types = m::Ikev2TransformTypes {
                encr: Some(vec![m::Ikev2Transform::Reference(BomReference::new(
                    "encr-ref",
                ))]),
                prf: Some(vec![m::Ikev2Transform::Reference(BomReference::new(
                    "prf-ref",
                ))]),
                integ: Some(vec![m::Ikev2Transform::Reference(BomReference::new(
                    "integ-ref",
                ))]),
                ke: Some(vec![m::Ikev2Transform::Reference(BomReference::new(
                    "ke-ref",
                ))]),
                esn: Some(true),
                auth: Some(vec![m::Ikev2Transform::Reference(BomReference::new(
                    "auth-ref",
                ))]),
            };
            #[versioned("1.7")]
            let ikev2_transform_types = m::Ikev2TransformTypes {
                encr: Some(vec![
                    m::Ikev2Transform::Transform(m::Ikev2Encryption {
                        name: Some("AES-128-GCM".to_string()),
                        key_length: Some(128),
                        algorithm: Some(BomReference::new("encr-ref")),
                    }),
                    m::Ikev2Transform::Reference(BomReference::new("encr-ref-2")),
                ]),
                prf: Some(vec![m::Ikev2Transform::Transform(m::Ikev2NamedTransform {
                    name: Some("PRF_HMAC_SHA2_256".to_string()),
                    algorithm: Some(BomReference::new("prf-ref")),
                })]),
                integ: Some(vec![m::Ikev2Transform::Reference(BomReference::new(
                    "integ-ref",
                ))]),
                ke: Some(vec![m::Ikev2Transform::Transform(m::Ikev2KeyExchange {
                    group: Some(19),
                    algorithm: Some(BomReference::new("ke-ref")),
                })]),
                esn: Some(true),
                auth: Some(vec![m::Ikev2Transform::Reference(BomReference::new(
                    "auth-ref",
                ))]),
            };

            #[versioned("1.6")]
            let (tls_groups, tls_signature_schemes) = (None, None);
            #[versioned("1.7")]
            let (tls_groups, tls_signature_schemes) = (
                Some(vec!["x25519".to_string()]),
                Some(vec!["ecdsa_secp256r1_sha256".to_string()]),
            );

            #[versioned("1.6")]
            let certificate_properties = m::CertificateProperties {
                subject_name: Some("CN = example.com".to_string()),
                issuer_name: Some("CN = Example CA".to_string()),
                not_valid_before: Some(DateTime("2024-01-01T00:00:00Z".to_string())),
                not_valid_after: Some(DateTime("2025-01-01T00:00:00Z".to_string())),
                signature_algorithm_ref: Some(BomReference::new("sig-ref")),
                subject_public_key_ref: Some(BomReference::new("key-ref")),
                certificate_format: Some("X.509".to_string()),
                certificate_extension: Some("crt".to_string()),
                ..Default::default()
            };
            #[versioned("1.7")]
            let certificate_properties = m::CertificateProperties {
                serial_number: Some("01".to_string()),
                subject_name: Some("CN = example.com".to_string()),
                issuer_name: Some("CN = Example CA".to_string()),
                not_valid_before: Some(DateTime("2024-01-01T00:00:00Z".to_string())),
                not_valid_after: Some(DateTime("2025-01-01T00:00:00Z".to_string())),
                signature_algorithm_ref: Some(BomReference::new("sig-ref")),
                subject_public_key_ref: Some(BomReference::new("key-ref")),
                certificate_format: Some("X.509".to_string()),
                certificate_extension: Some("crt".to_string()),
                certificate_file_extension: Some("pem".to_string()),
                fingerprint: Some(models::hash::Hash {
                    alg: models::hash::HashAlgorithm::SHA_256,
                    content: models::hash::HashValue(
                        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
                            .to_string(),
                    ),
                }),
                certificate_state: Some(vec![
                    m::CertificateState::Predefined {
                        state: m::CertificateLifecycleState::Active,
                        reason: Some("issued".to_string()),
                    },
                    m::CertificateState::Custom {
                        name: "monitored".to_string(),
                        description: Some("under monitoring".to_string()),
                        reason: None,
                    },
                ]),
                creation_date: Some(DateTime("2023-12-01T00:00:00Z".to_string())),
                activation_date: Some(DateTime("2024-01-01T00:00:00Z".to_string())),
                deactivation_date: Some(DateTime("2024-06-01T00:00:00Z".to_string())),
                revocation_date: Some(DateTime("2024-07-01T00:00:00Z".to_string())),
                destruction_date: Some(DateTime("2024-08-01T00:00:00Z".to_string())),
                certificate_extensions: Some(vec![
                    m::CertificateExtension::Common {
                        name: m::CommonCertificateExtensionName::KeyUsage,
                        value: "digitalSignature".to_string(),
                    },
                    m::CertificateExtension::Custom {
                        name: "1.3.6.1.4.1.99999".to_string(),
                        value: None,
                    },
                ]),
                related_cryptographic_assets: Some(vec![m::RelatedCryptographicAsset {
                    asset_type: Some("signature".to_string()),
                    reference: Some(BomReference::new("sig-ref")),
                }]),
            };

            #[versioned("1.6")]
            let (material_fingerprint, material_assets, protocol_assets) = (None, None, None);
            #[versioned("1.7")]
            let (material_fingerprint, material_assets, protocol_assets) = (
                Some(models::hash::Hash {
                    alg: models::hash::HashAlgorithm::SHA_256,
                    content: models::hash::HashValue(
                        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
                            .to_string(),
                    ),
                }),
                Some(vec![m::RelatedCryptographicAsset {
                    asset_type: Some("publicKey".to_string()),
                    reference: Some(BomReference::new("key-ref")),
                }]),
                Some(vec![m::RelatedCryptographicAsset {
                    asset_type: Some("algorithm".to_string()),
                    reference: Some(BomReference::new("aes-ref")),
                }]),
            );

            #[versioned("1.6")]
            let (algorithm_family, elliptic_curve) = (None, None);
            #[versioned("1.7")]
            let (algorithm_family, elliptic_curve) =
                (Some("AES".to_string()), Some("secg/secp256r1".to_string()));

            m::CryptoProperties {
                asset_type: m::CryptoAssetType::Algorithm,
                algorithm_properties: Some(m::AlgorithmProperties {
                    primitive: Some(m::CryptoPrimitive::Ae),
                    algorithm_family,
                    parameter_set_identifier: Some("128".to_string()),
                    curve: Some("secp256r1".to_string()),
                    elliptic_curve,
                    execution_environment: Some(m::CryptoExecutionEnvironment::SoftwarePlainRam),
                    implementation_platform: Some(m::CryptoImplementationPlatform::X86_64),
                    certification_level: Some(vec![
                        m::CryptoCertificationLevel::None,
                        m::CryptoCertificationLevel::CcEal4Plus,
                    ]),
                    mode: Some(m::CryptoMode::Gcm),
                    padding: Some(m::CryptoPadding::Pkcs7),
                    crypto_functions: Some(vec![
                        m::CryptoFunction::Encrypt,
                        m::CryptoFunction::Decrypt,
                    ]),
                    classical_security_level: Some(128),
                    nist_quantum_security_level: Some(1),
                }),
                certificate_properties: Some(certificate_properties),
                related_crypto_material_properties: Some(m::RelatedCryptoMaterialProperties {
                    material_type: Some(m::RelatedCryptoMaterialType::PublicKey),
                    id: Some("key-id".to_string()),
                    state: Some(m::RelatedCryptoMaterialState::Active),
                    algorithm_ref: Some(BomReference::new("rsa-ref")),
                    creation_date: Some(DateTime("2024-01-01T00:00:00Z".to_string())),
                    activation_date: Some(DateTime("2024-01-02T00:00:00Z".to_string())),
                    update_date: Some(DateTime("2024-01-03T00:00:00Z".to_string())),
                    expiration_date: Some(DateTime("2025-01-01T00:00:00Z".to_string())),
                    value: Some("key value".to_string()),
                    size: Some(2048),
                    format: Some("PEM".to_string()),
                    secured_by: Some(m::SecuredBy {
                        mechanism: Some("Software".to_string()),
                        algorithm_ref: Some(BomReference::new("aes-ref")),
                    }),
                    fingerprint: material_fingerprint,
                    related_cryptographic_assets: material_assets,
                }),
                protocol_properties: Some(m::ProtocolProperties {
                    protocol_type: Some(m::ProtocolType::Ike),
                    version: Some("2".to_string()),
                    cipher_suites: Some(vec![m::CipherSuite {
                        name: Some("TLS_AES_128_GCM_SHA256".to_string()),
                        algorithms: Some(vec![BomReference::new("aes-ref")]),
                        identifiers: Some(vec!["0x13".to_string(), "0x01".to_string()]),
                        tls_groups,
                        tls_signature_schemes,
                    }]),
                    ikev2_transform_types: Some(ikev2_transform_types),
                    crypto_ref_array: Some(vec![
                        BomReference::new("ref-1"),
                        BomReference::new("ref-2"),
                    ]),
                    related_cryptographic_assets: protocol_assets,
                }),
                oid: Some("2.16.840.1.101.3.4.1.6".to_string()),
            }
        }

        #[test]
        fn it_should_convert_to_and_from_the_model() {
            let model: m::CryptoProperties = example_crypto_properties().into();
            assert_eq!(model, corresponding_crypto_properties());

            let spec: CryptoProperties = corresponding_crypto_properties().into();
            assert_eq!(spec, example_crypto_properties());
        }

        #[test]
        fn it_should_round_trip_json() {
            let json = serde_json::to_string(&example_crypto_properties()).unwrap();
            let actual: CryptoProperties = serde_json::from_str(&json).unwrap();
            assert_eq!(actual, example_crypto_properties());
        }

        #[test]
        fn it_should_write_xml_full() {
            let xml_output = write_element_to_string(example_crypto_properties());
            insta::assert_snapshot!(xml_output);
        }

        #[test]
        fn it_should_read_xml_full() {
            let xml_output = write_element_to_string(example_crypto_properties());
            let actual: CryptoProperties = read_element_from_string(xml_output);
            assert_eq!(actual, example_crypto_properties());
        }

        #[test]
        fn it_should_read_xml_with_unbounded_elements() {
            let input = r#"
<cryptoProperties>
  <assetType>algorithm</assetType>
  <algorithmProperties>
    <certificationLevel>fips140-3-l1</certificationLevel>
    <certificationLevel>cc-eal2</certificationLevel>
    <classicalSecurityLevel>256</classicalSecurityLevel>
  </algorithmProperties>
  <protocolProperties>
    <ikev2TransformTypes>
      <encr>ref-a</encr>
      <encr>ref-b</encr>
      <esn>false</esn>
    </ikev2TransformTypes>
    <cryptoRef>ref-c</cryptoRef>
    <cryptoRef>ref-d</cryptoRef>
  </protocolProperties>
</cryptoProperties>
"#;
            let actual: CryptoProperties = read_element_from_string(input);
            let algorithm = actual.algorithm_properties.unwrap();
            assert_eq!(
                algorithm.certification_level,
                Some(vec!["fips140-3-l1".to_string(), "cc-eal2".to_string()])
            );
            assert_eq!(algorithm.classical_security_level, Some(256));

            let protocol = actual.protocol_properties.unwrap();
            assert_eq!(
                protocol.crypto_ref_array,
                Some(vec!["ref-c".to_string(), "ref-d".to_string()])
            );
            let ikev2 = protocol.ikev2_transform_types.unwrap();
            assert_eq!(ikev2.esn, Some(false));
            assert_eq!(ikev2.encr.map(|e| e.len()), Some(2));
        }

        #[versioned("1.7")]
        #[test]
        fn it_should_read_json_with_deprecated_and_structured_ikev2_transforms() {
            let input = r#"{
                "encr": ["encr-ref"],
                "ke": [{ "group": 19, "algorithm": "ke-ref" }]
            }"#;
            let actual: Ikev2TransformTypes = serde_json::from_str(input).unwrap();
            assert_eq!(
                actual.encr,
                Some(vec![Ikev2Transform::Reference("encr-ref".to_string())])
            );
            assert_eq!(
                actual.ke,
                Some(vec![Ikev2Transform::Transform(Ikev2KeyExchange {
                    group: Some(19),
                    algorithm: Some("ke-ref".to_string()),
                })])
            );
        }

        #[versioned("1.7")]
        #[test]
        fn it_should_read_json_certificate_state_and_extension_variants() {
            let input = r#"[
                { "state": "revoked", "reason": "key compromise" },
                { "name": "monitored", "description": "watched" }
            ]"#;
            let actual: Vec<CertificateState> = serde_json::from_str(input).unwrap();
            assert_eq!(
                actual,
                vec![
                    CertificateState::Predefined {
                        state: "revoked".to_string(),
                        reason: Some("key compromise".to_string()),
                    },
                    CertificateState::Custom {
                        name: "monitored".to_string(),
                        description: Some("watched".to_string()),
                        reason: None,
                    },
                ]
            );

            let input = r#"[
                { "commonExtensionName": "keyUsage", "commonExtensionValue": "keyCertSign" },
                { "customExtensionName": "1.2.3" }
            ]"#;
            let actual: Vec<CertificateExtension> = serde_json::from_str(input).unwrap();
            assert_eq!(
                actual,
                vec![
                    CertificateExtension::Common {
                        common_extension_name: "keyUsage".to_string(),
                        common_extension_value: "keyCertSign".to_string(),
                    },
                    CertificateExtension::Custom {
                        custom_extension_name: "1.2.3".to_string(),
                        custom_extension_value: None,
                    },
                ]
            );
        }
    }
}
