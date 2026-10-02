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

//! Cryptographic asset properties, added in version 1.6.
//!
//! See <https://cyclonedx.org/docs/1.7/json/#components_items_cryptoProperties>

use crate::external_models::date_time::{validate_date_time, DateTime};
use crate::models::bom::{BomReference, SpecVersion};
use crate::models::hash::Hash;
use crate::validation::{Validate, ValidationContext, ValidationError, ValidationResult};

/// Implemented by the string enums of this module so they can be validated against a spec version.
trait VersionedEnum {
    fn is_unknown(&self) -> bool;
    fn since(&self) -> SpecVersion;
}

fn validate_versioned_enum<E: VersionedEnum>(
    value: &E,
    version: SpecVersion,
    name: &str,
) -> Result<(), ValidationError> {
    if value.is_unknown() || version < value.since() {
        return Err(ValidationError::new(format!("Unknown {name}")));
    }
    Ok(())
}

/// Declares a string enum with an `Unknown` catch-all, `new_unchecked` and `Display`.
///
/// The spec values contain characters (`+`, `_`, leading digits) that strum's case conversion
/// cannot produce, so every value is spelled out.
macro_rules! crypto_enum {
    (
        $(#[$meta:meta])*
        $name:ident, $unknown:ident, $label:literal {
            $($(#[$vmeta:meta])* $variant:ident => $value:literal $(since $since:ident)?,)+
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$vmeta])* $variant,)+
            #[doc(hidden)]
            $unknown(String),
        }

        impl $name {
            pub fn new_unchecked<A: AsRef<str>>(value: A) -> Self {
                match value.as_ref() {
                    $($value => Self::$variant,)+
                    unknown => Self::$unknown(unknown.to_string()),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant => f.write_str($value),)+
                    Self::$unknown(value) => f.write_str(value),
                }
            }
        }

        impl VersionedEnum for $name {
            fn is_unknown(&self) -> bool {
                matches!(self, Self::$unknown(_))
            }

            #[allow(unreachable_patterns)]
            fn since(&self) -> SpecVersion {
                match self {
                    $($(Self::$variant => SpecVersion::$since,)?)+
                    _ => SpecVersion::V1_6,
                }
            }
        }

        impl $name {
            pub(crate) fn validate_for(&self, version: SpecVersion) -> Result<(), ValidationError> {
                validate_versioned_enum(self, version, $label)
            }
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CryptoProperties {
    pub asset_type: CryptoAssetType,
    pub algorithm_properties: Option<AlgorithmProperties>,
    pub certificate_properties: Option<CertificateProperties>,
    pub related_crypto_material_properties: Option<RelatedCryptoMaterialProperties>,
    pub protocol_properties: Option<ProtocolProperties>,
    pub oid: Option<String>,
}

impl CryptoProperties {
    pub fn new(asset_type: CryptoAssetType) -> Self {
        Self {
            asset_type,
            algorithm_properties: None,
            certificate_properties: None,
            related_crypto_material_properties: None,
            protocol_properties: None,
            oid: None,
        }
    }
}

impl Validate for CryptoProperties {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_enum("asset_type", &self.asset_type, |v| v.validate_for(version))
            .add_struct_option(
                "algorithm_properties",
                self.algorithm_properties.as_ref(),
                version,
            )
            .add_struct_option(
                "certificate_properties",
                self.certificate_properties.as_ref(),
                version,
            )
            .add_struct_option(
                "related_crypto_material_properties",
                self.related_crypto_material_properties.as_ref(),
                version,
            )
            .add_struct_option(
                "protocol_properties",
                self.protocol_properties.as_ref(),
                version,
            )
            .into()
    }
}

crypto_enum! {
    CryptoAssetType, UnknownCryptoAssetType, "crypto asset type" {
        Algorithm => "algorithm",
        Certificate => "certificate",
        Protocol => "protocol",
        RelatedCryptoMaterial => "related-crypto-material",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct AlgorithmProperties {
    pub primitive: Option<CryptoPrimitive>,
    /// Added in version 1.7. Values come from the CycloneDX cryptography registry
    /// (`cryptography-defs.schema.json#/definitions/algorithmFamiliesEnum`).
    pub algorithm_family: Option<String>,
    pub parameter_set_identifier: Option<String>,
    /// Deprecated in version 1.7 in favour of `elliptic_curve`
    pub curve: Option<String>,
    /// Added in version 1.7. Values come from the CycloneDX cryptography registry
    /// (`cryptography-defs.schema.json#/definitions/ellipticCurvesEnum`).
    pub elliptic_curve: Option<String>,
    pub execution_environment: Option<CryptoExecutionEnvironment>,
    pub implementation_platform: Option<CryptoImplementationPlatform>,
    pub certification_level: Option<Vec<CryptoCertificationLevel>>,
    pub mode: Option<CryptoMode>,
    pub padding: Option<CryptoPadding>,
    pub crypto_functions: Option<Vec<CryptoFunction>>,
    pub classical_security_level: Option<u32>,
    pub nist_quantum_security_level: Option<u32>,
}

impl Validate for AlgorithmProperties {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_enum_option("primitive", self.primitive.as_ref(), |v| {
                v.validate_for(version)
            })
            .add_enum_option(
                "execution_environment",
                self.execution_environment.as_ref(),
                |v| v.validate_for(version),
            )
            .add_enum_option(
                "implementation_platform",
                self.implementation_platform.as_ref(),
                |v| v.validate_for(version),
            )
            .add_list_option(
                "certification_level",
                self.certification_level.as_ref(),
                |v: &CryptoCertificationLevel| v.validate_for(version),
            )
            .add_enum_option("mode", self.mode.as_ref(), |v| v.validate_for(version))
            .add_enum_option("padding", self.padding.as_ref(), |v| {
                v.validate_for(version)
            })
            .add_list_option(
                "crypto_functions",
                self.crypto_functions.as_ref(),
                |v: &CryptoFunction| v.validate_for(version),
            )
            .add_field_option(
                "nist_quantum_security_level",
                self.nist_quantum_security_level,
                validate_nist_quantum_security_level,
            )
            .into()
    }
}

fn validate_nist_quantum_security_level(level: u32) -> Result<(), ValidationError> {
    if level > 6 {
        return Err(ValidationError::new(
            "nistQuantumSecurityLevel must be between 0 and 6",
        ));
    }
    Ok(())
}

crypto_enum! {
    CryptoPrimitive, UnknownCryptoPrimitive, "cryptographic primitive" {
        Drbg => "drbg",
        Mac => "mac",
        BlockCipher => "block-cipher",
        StreamCipher => "stream-cipher",
        Signature => "signature",
        Hash => "hash",
        Pke => "pke",
        Xof => "xof",
        Kdf => "kdf",
        KeyAgree => "key-agree",
        Kem => "kem",
        Ae => "ae",
        Combiner => "combiner",
        /// Added in version 1.7
        KeyWrap => "key-wrap" since V1_7,
        Other => "other",
        Unknown => "unknown",
    }
}

crypto_enum! {
    CryptoExecutionEnvironment, UnknownCryptoExecutionEnvironment, "execution environment" {
        SoftwarePlainRam => "software-plain-ram",
        SoftwareEncryptedRam => "software-encrypted-ram",
        SoftwareTee => "software-tee",
        Hardware => "hardware",
        Other => "other",
        Unknown => "unknown",
    }
}

crypto_enum! {
    CryptoImplementationPlatform, UnknownCryptoImplementationPlatform, "implementation platform" {
        Generic => "generic",
        X86_32 => "x86_32",
        X86_64 => "x86_64",
        Armv7A => "armv7-a",
        Armv7M => "armv7-m",
        Armv8A => "armv8-a",
        Armv8M => "armv8-m",
        Armv9A => "armv9-a",
        Armv9M => "armv9-m",
        S390x => "s390x",
        Ppc64 => "ppc64",
        Ppc64le => "ppc64le",
        Other => "other",
        Unknown => "unknown",
    }
}

crypto_enum! {
    CryptoCertificationLevel, UnknownCryptoCertificationLevel, "certification level" {
        None => "none",
        Fips140_1L1 => "fips140-1-l1",
        Fips140_1L2 => "fips140-1-l2",
        Fips140_1L3 => "fips140-1-l3",
        Fips140_1L4 => "fips140-1-l4",
        Fips140_2L1 => "fips140-2-l1",
        Fips140_2L2 => "fips140-2-l2",
        Fips140_2L3 => "fips140-2-l3",
        Fips140_2L4 => "fips140-2-l4",
        Fips140_3L1 => "fips140-3-l1",
        Fips140_3L2 => "fips140-3-l2",
        Fips140_3L3 => "fips140-3-l3",
        Fips140_3L4 => "fips140-3-l4",
        CcEal1 => "cc-eal1",
        CcEal1Plus => "cc-eal1+",
        CcEal2 => "cc-eal2",
        CcEal2Plus => "cc-eal2+",
        CcEal3 => "cc-eal3",
        CcEal3Plus => "cc-eal3+",
        CcEal4 => "cc-eal4",
        CcEal4Plus => "cc-eal4+",
        CcEal5 => "cc-eal5",
        CcEal5Plus => "cc-eal5+",
        CcEal6 => "cc-eal6",
        CcEal6Plus => "cc-eal6+",
        CcEal7 => "cc-eal7",
        CcEal7Plus => "cc-eal7+",
        Other => "other",
        Unknown => "unknown",
    }
}

crypto_enum! {
    CryptoMode, UnknownCryptoMode, "mode" {
        Cbc => "cbc",
        Ecb => "ecb",
        Ccm => "ccm",
        Gcm => "gcm",
        Cfb => "cfb",
        Ofb => "ofb",
        Ctr => "ctr",
        Other => "other",
        Unknown => "unknown",
    }
}

crypto_enum! {
    CryptoPadding, UnknownCryptoPadding, "padding" {
        Pkcs5 => "pkcs5",
        Pkcs7 => "pkcs7",
        Pkcs1v15 => "pkcs1v15",
        Oaep => "oaep",
        Raw => "raw",
        Other => "other",
        Unknown => "unknown",
    }
}

crypto_enum! {
    CryptoFunction, UnknownCryptoFunction, "crypto function" {
        Generate => "generate",
        Keygen => "keygen",
        Encrypt => "encrypt",
        Decrypt => "decrypt",
        Digest => "digest",
        Tag => "tag",
        Keyderive => "keyderive",
        Sign => "sign",
        Verify => "verify",
        Encapsulate => "encapsulate",
        Decapsulate => "decapsulate",
        Other => "other",
        Unknown => "unknown",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct CertificateProperties {
    /// Added in version 1.7
    pub serial_number: Option<String>,
    pub subject_name: Option<String>,
    pub issuer_name: Option<String>,
    pub not_valid_before: Option<DateTime>,
    pub not_valid_after: Option<DateTime>,
    /// Deprecated in version 1.7 in favour of `related_cryptographic_assets`
    pub signature_algorithm_ref: Option<BomReference>,
    /// Deprecated in version 1.7 in favour of `related_cryptographic_assets`
    pub subject_public_key_ref: Option<BomReference>,
    pub certificate_format: Option<String>,
    /// Deprecated in version 1.7 in favour of `certificate_file_extension`
    pub certificate_extension: Option<String>,
    /// Added in version 1.7
    pub certificate_file_extension: Option<String>,
    /// Added in version 1.7
    pub fingerprint: Option<Hash>,
    /// Added in version 1.7
    pub certificate_state: Option<Vec<CertificateState>>,
    /// Added in version 1.7
    pub creation_date: Option<DateTime>,
    /// Added in version 1.7
    pub activation_date: Option<DateTime>,
    /// Added in version 1.7
    pub deactivation_date: Option<DateTime>,
    /// Added in version 1.7
    pub revocation_date: Option<DateTime>,
    /// Added in version 1.7
    pub destruction_date: Option<DateTime>,
    /// Added in version 1.7
    pub certificate_extensions: Option<Vec<CertificateExtension>>,
    /// Added in version 1.7
    pub related_cryptographic_assets: Option<Vec<RelatedCryptographicAsset>>,
}

impl Validate for CertificateProperties {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_field_option(
                "not_valid_before",
                self.not_valid_before.as_ref(),
                validate_date_time,
            )
            .add_field_option(
                "not_valid_after",
                self.not_valid_after.as_ref(),
                validate_date_time,
            )
            .add_struct_option("fingerprint", self.fingerprint.as_ref(), version)
            .add_list_option(
                "certificate_state",
                self.certificate_state.as_ref(),
                |state: &CertificateState| state.validate_version(version),
            )
            .add_field_option(
                "creation_date",
                self.creation_date.as_ref(),
                validate_date_time,
            )
            .add_field_option(
                "activation_date",
                self.activation_date.as_ref(),
                validate_date_time,
            )
            .add_field_option(
                "deactivation_date",
                self.deactivation_date.as_ref(),
                validate_date_time,
            )
            .add_field_option(
                "revocation_date",
                self.revocation_date.as_ref(),
                validate_date_time,
            )
            .add_field_option(
                "destruction_date",
                self.destruction_date.as_ref(),
                validate_date_time,
            )
            .add_list_option(
                "certificate_extensions",
                self.certificate_extensions.as_ref(),
                |extension: &CertificateExtension| extension.validate_version(version),
            )
            .into()
    }
}

/// Added in version 1.7
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CertificateState {
    Predefined {
        state: CertificateLifecycleState,
        reason: Option<String>,
    },
    Custom {
        name: String,
        description: Option<String>,
        reason: Option<String>,
    },
}

impl Validate for CertificateState {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut ctx = ValidationContext::new();
        if let Self::Predefined { state, .. } = self {
            ctx.add_enum("state", state, |v| v.validate_for(version));
        }
        ctx.into()
    }
}

crypto_enum! {
    /// Added in version 1.7
    CertificateLifecycleState, UnknownCertificateLifecycleState, "certificate state" {
        PreActivation => "pre-activation" since V1_7,
        Active => "active" since V1_7,
        Suspended => "suspended" since V1_7,
        Deactivated => "deactivated" since V1_7,
        Revoked => "revoked" since V1_7,
        Destroyed => "destroyed" since V1_7,
    }
}

/// Added in version 1.7
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CertificateExtension {
    Common {
        name: CommonCertificateExtensionName,
        value: String,
    },
    Custom {
        name: String,
        value: Option<String>,
    },
}

impl Validate for CertificateExtension {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        let mut ctx = ValidationContext::new();
        if let Self::Common { name, .. } = self {
            ctx.add_enum("name", name, |v| v.validate_for(version));
        }
        ctx.into()
    }
}

crypto_enum! {
    /// Added in version 1.7
    CommonCertificateExtensionName, UnknownCommonCertificateExtensionName, "certificate extension name" {
        BasicConstraints => "basicConstraints" since V1_7,
        KeyUsage => "keyUsage" since V1_7,
        ExtendedKeyUsage => "extendedKeyUsage" since V1_7,
        SubjectAlternativeName => "subjectAlternativeName" since V1_7,
        AuthorityKeyIdentifier => "authorityKeyIdentifier" since V1_7,
        SubjectKeyIdentifier => "subjectKeyIdentifier" since V1_7,
        AuthorityInformationAccess => "authorityInformationAccess" since V1_7,
        CertificatePolicies => "certificatePolicies" since V1_7,
        CrlDistributionPoints => "crlDistributionPoints" since V1_7,
        SignedCertificateTimestamp => "signedCertificateTimestamp" since V1_7,
    }
}

/// Added in version 1.7
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct RelatedCryptographicAsset {
    /// Free text in the schema, e.g. `publicKey` or `algorithm`.
    pub asset_type: Option<String>,
    pub reference: Option<BomReference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct RelatedCryptoMaterialProperties {
    pub material_type: Option<RelatedCryptoMaterialType>,
    pub id: Option<String>,
    pub state: Option<RelatedCryptoMaterialState>,
    /// Deprecated in version 1.7 in favour of `related_cryptographic_assets`
    pub algorithm_ref: Option<BomReference>,
    pub creation_date: Option<DateTime>,
    pub activation_date: Option<DateTime>,
    pub update_date: Option<DateTime>,
    pub expiration_date: Option<DateTime>,
    pub value: Option<String>,
    pub size: Option<i64>,
    pub format: Option<String>,
    pub secured_by: Option<SecuredBy>,
    /// Added in version 1.7
    pub fingerprint: Option<Hash>,
    /// Added in version 1.7
    pub related_cryptographic_assets: Option<Vec<RelatedCryptographicAsset>>,
}

impl Validate for RelatedCryptoMaterialProperties {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_enum_option("material_type", self.material_type.as_ref(), |v| {
                v.validate_for(version)
            })
            .add_enum_option("state", self.state.as_ref(), |v| v.validate_for(version))
            .add_field_option(
                "creation_date",
                self.creation_date.as_ref(),
                validate_date_time,
            )
            .add_field_option(
                "activation_date",
                self.activation_date.as_ref(),
                validate_date_time,
            )
            .add_field_option("update_date", self.update_date.as_ref(), validate_date_time)
            .add_field_option(
                "expiration_date",
                self.expiration_date.as_ref(),
                validate_date_time,
            )
            .add_struct_option("fingerprint", self.fingerprint.as_ref(), version)
            .into()
    }
}

crypto_enum! {
    RelatedCryptoMaterialType, UnknownRelatedCryptoMaterialType, "related crypto material type" {
        PrivateKey => "private-key",
        PublicKey => "public-key",
        SecretKey => "secret-key",
        Key => "key",
        Ciphertext => "ciphertext",
        Signature => "signature",
        Digest => "digest",
        InitializationVector => "initialization-vector",
        Nonce => "nonce",
        Seed => "seed",
        Salt => "salt",
        SharedSecret => "shared-secret",
        Tag => "tag",
        AdditionalData => "additional-data",
        Password => "password",
        Credential => "credential",
        Token => "token",
        Other => "other",
        Unknown => "unknown",
    }
}

crypto_enum! {
    RelatedCryptoMaterialState, UnknownRelatedCryptoMaterialState, "related crypto material state" {
        PreActivation => "pre-activation",
        Active => "active",
        Suspended => "suspended",
        Deactivated => "deactivated",
        Compromised => "compromised",
        Destroyed => "destroyed",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct SecuredBy {
    pub mechanism: Option<String>,
    pub algorithm_ref: Option<BomReference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ProtocolProperties {
    pub protocol_type: Option<ProtocolType>,
    pub version: Option<String>,
    pub cipher_suites: Option<Vec<CipherSuite>>,
    pub ikev2_transform_types: Option<Ikev2TransformTypes>,
    /// Deprecated in version 1.7 in favour of `related_cryptographic_assets`
    pub crypto_ref_array: Option<Vec<BomReference>>,
    /// Added in version 1.7
    pub related_cryptographic_assets: Option<Vec<RelatedCryptographicAsset>>,
}

impl Validate for ProtocolProperties {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_enum_option("protocol_type", self.protocol_type.as_ref(), |v| {
                v.validate_for(version)
            })
            .add_struct_option(
                "ikev2_transform_types",
                self.ikev2_transform_types.as_ref(),
                version,
            )
            .into()
    }
}

crypto_enum! {
    ProtocolType, UnknownProtocolType, "protocol type" {
        Tls => "tls",
        Ssh => "ssh",
        Ipsec => "ipsec",
        Ike => "ike",
        Sstp => "sstp",
        Wpa => "wpa",
        /// Added in version 1.7
        Dtls => "dtls" since V1_7,
        /// Added in version 1.7
        Quic => "quic" since V1_7,
        /// Added in version 1.7
        EapAka => "eap-aka" since V1_7,
        /// Added in version 1.7
        EapAkaPrime => "eap-aka-prime" since V1_7,
        /// Added in version 1.7
        Prins => "prins" since V1_7,
        /// Added in version 1.7
        FiveGAka => "5g-aka" since V1_7,
        Other => "other",
        Unknown => "unknown",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct CipherSuite {
    pub name: Option<String>,
    pub algorithms: Option<Vec<BomReference>>,
    pub identifiers: Option<Vec<String>>,
    /// Added in version 1.7
    pub tls_groups: Option<Vec<String>>,
    /// Added in version 1.7
    pub tls_signature_schemes: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Ikev2TransformTypes {
    pub encr: Option<Vec<Ikev2Transform<Ikev2Encryption>>>,
    pub prf: Option<Vec<Ikev2Transform<Ikev2NamedTransform>>>,
    pub integ: Option<Vec<Ikev2Transform<Ikev2NamedTransform>>>,
    pub ke: Option<Vec<Ikev2Transform<Ikev2KeyExchange>>>,
    pub esn: Option<bool>,
    pub auth: Option<Vec<Ikev2Transform<Ikev2NamedTransform>>>,
}

impl Validate for Ikev2TransformTypes {
    fn validate_version(&self, version: SpecVersion) -> ValidationResult {
        ValidationContext::new()
            .add_list_option("encr", self.encr.as_ref(), |t: &Ikev2Transform<_>| {
                t.validate_for(version)
            })
            .add_list_option("prf", self.prf.as_ref(), |t: &Ikev2Transform<_>| {
                t.validate_for(version)
            })
            .add_list_option("integ", self.integ.as_ref(), |t: &Ikev2Transform<_>| {
                t.validate_for(version)
            })
            .add_list_option("ke", self.ke.as_ref(), |t: &Ikev2Transform<_>| {
                t.validate_for(version)
            })
            .add_list_option("auth", self.auth.as_ref(), |t: &Ikev2Transform<_>| {
                t.validate_for(version)
            })
            .into()
    }
}

/// An IKEv2 transform entry.
///
/// Version 1.6 only allows a bare reference. Version 1.7 adds structured transforms and
/// deprecates the bare reference.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ikev2Transform<T> {
    Reference(BomReference),
    /// Added in version 1.7
    Transform(T),
}

impl<T> Ikev2Transform<T> {
    fn validate_for(&self, version: SpecVersion) -> Result<(), ValidationError> {
        if matches!(self, Self::Transform(_)) && version < SpecVersion::V1_7 {
            return Err(ValidationError::new(
                "Structured IKEv2 transforms require version 1.7 or later",
            ));
        }
        Ok(())
    }
}

/// Added in version 1.7
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Ikev2Encryption {
    pub name: Option<String>,
    pub key_length: Option<i64>,
    pub algorithm: Option<BomReference>,
}

/// Pseudorandom function, integrity or authentication transform. Added in version 1.7
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Ikev2NamedTransform {
    pub name: Option<String>,
    pub algorithm: Option<BomReference>,
}

/// Added in version 1.7
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Ikev2KeyExchange {
    pub group: Option<i64>,
    pub algorithm: Option<BomReference>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::hash::{HashAlgorithm, HashValue};
    use crate::validation::ValidationErrorsKind;

    #[test]
    fn enums_round_trip_through_strings() {
        for value in ["x86_32", "armv7-a", "ppc64le"] {
            assert_eq!(
                CryptoImplementationPlatform::new_unchecked(value).to_string(),
                value
            );
        }
        for value in ["fips140-2-l3", "cc-eal4+", "none"] {
            assert_eq!(
                CryptoCertificationLevel::new_unchecked(value).to_string(),
                value
            );
        }
        assert_eq!(
            ProtocolType::new_unchecked("5g-aka"),
            ProtocolType::FiveGAka
        );
        assert_eq!(
            CryptoPrimitive::new_unchecked("bogus"),
            CryptoPrimitive::UnknownCryptoPrimitive("bogus".to_string())
        );
    }

    #[test]
    fn valid_crypto_properties_should_pass_validation() {
        let properties = CryptoProperties {
            asset_type: CryptoAssetType::Algorithm,
            algorithm_properties: Some(AlgorithmProperties {
                primitive: Some(CryptoPrimitive::Ae),
                certification_level: Some(vec![CryptoCertificationLevel::Fips140_3L1]),
                crypto_functions: Some(vec![CryptoFunction::Encrypt]),
                nist_quantum_security_level: Some(1),
                ..Default::default()
            }),
            certificate_properties: Some(CertificateProperties {
                not_valid_before: Some(DateTime("2024-01-01T00:00:00Z".to_string())),
                fingerprint: Some(Hash {
                    alg: HashAlgorithm::SHA_256,
                    content: HashValue(
                        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
                            .to_string(),
                    ),
                }),
                certificate_state: Some(vec![CertificateState::Predefined {
                    state: CertificateLifecycleState::Active,
                    reason: None,
                }]),
                ..Default::default()
            }),
            related_crypto_material_properties: None,
            protocol_properties: Some(ProtocolProperties {
                protocol_type: Some(ProtocolType::Quic),
                ..Default::default()
            }),
            oid: None,
        };

        assert!(properties.validate_version(SpecVersion::V1_7).passed());
    }

    #[test]
    fn invalid_crypto_properties_should_fail_validation() {
        let properties = CryptoProperties {
            asset_type: CryptoAssetType::UnknownCryptoAssetType("bogus".to_string()),
            algorithm_properties: Some(AlgorithmProperties {
                primitive: Some(CryptoPrimitive::KeyWrap),
                nist_quantum_security_level: Some(7),
                ..Default::default()
            }),
            certificate_properties: None,
            related_crypto_material_properties: None,
            protocol_properties: Some(ProtocolProperties {
                protocol_type: Some(ProtocolType::Quic),
                ikev2_transform_types: Some(Ikev2TransformTypes {
                    encr: Some(vec![Ikev2Transform::Transform(Ikev2Encryption::default())]),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            oid: None,
        };

        let result = properties.validate_version(SpecVersion::V1_6);
        assert!(result.has_error("asset_type"));

        let Some(ValidationErrorsKind::Struct(algorithm)) = result.error("algorithm_properties")
        else {
            panic!("expected algorithm_properties errors");
        };
        assert!(algorithm.has_error("primitive"));
        assert!(algorithm.has_error("nist_quantum_security_level"));

        let Some(ValidationErrorsKind::Struct(protocol)) = result.error("protocol_properties")
        else {
            panic!("expected protocol_properties errors");
        };
        assert!(protocol.has_error("protocol_type"));
        assert!(protocol.has_error("ikev2_transform_types"));

        // key-wrap and quic are valid in 1.7
        let result = properties.validate_version(SpecVersion::V1_7);
        let Some(ValidationErrorsKind::Struct(algorithm)) = result.error("algorithm_properties")
        else {
            panic!("expected algorithm_properties errors");
        };
        assert!(!algorithm.has_error("primitive"));
        assert!(!result.has_error("protocol_properties"));
    }
}
