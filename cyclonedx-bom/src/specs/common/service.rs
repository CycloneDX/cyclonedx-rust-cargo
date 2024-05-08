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

#[versioned("1.3", "1.4", "1.5")]
pub(crate) mod base {
    #[versioned("1.3", "1.4")]
    use crate::{
        errors::BomError,
        utilities::{try_convert_optional, try_convert_vec},
    };
    use crate::{
        errors::XmlReadError,
        external_models::{normalized_string::NormalizedString, uri::Uri},
        models,
        utilities::{convert_optional, convert_vec},
        xml::{
            attribute_or_error, optional_attribute, read_boolean_tag, read_lax_validation_list_tag,
            read_lax_validation_tag, read_list_tag, read_simple_tag, to_xml_read_error,
            to_xml_write_error, unexpected_element_error, write_close_tag, write_simple_tag,
            write_start_tag, FromXml, ToInnerXml, ToXml,
        },
    };
    use serde::{Deserialize, Serialize};
    use xml::{reader, writer::XmlEvent};

    #[versioned("1.4", "1.5")]
    use crate::specs::common::signature::Signature;
    use crate::specs::common::{organization::OrganizationalEntity, property::Properties};
    #[versioned("1.3")]
    use crate::specs::v1_3::{external_reference::ExternalReferences, license::Licenses};
    #[versioned("1.4")]
    use crate::specs::v1_4::{external_reference::ExternalReferences, license::Licenses};
    #[versioned("1.5")]
    use crate::specs::v1_5::{
        external_reference::ExternalReferences, license::Licenses, service_data::ServiceData,
    };

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(transparent)]
    pub(crate) struct Services(pub Vec<Service>);

    #[versioned("1.3", "1.4")]
    impl TryFrom<models::service::Services> for Services {
        type Error = BomError;

        fn try_from(other: models::service::Services) -> Result<Self, Self::Error> {
            try_convert_vec(other.0).map(Services)
        }
    }

    #[versioned("1.5")]
    impl From<models::service::Services> for Services {
        fn from(other: models::service::Services) -> Self {
            Services(convert_vec(other.0))
        }
    }

    impl From<Services> for models::service::Services {
        fn from(other: Services) -> Self {
            models::service::Services(convert_vec(other.0))
        }
    }

    const SERVICES_TAG: &str = "services";

    impl ToXml for Services {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            write_start_tag(writer, SERVICES_TAG)?;

            for service in &self.0 {
                service.write_xml_element(writer)?;
            }

            write_close_tag(writer, SERVICES_TAG)?;

            Ok(())
        }
    }

    impl FromXml for Services {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            _attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            read_lax_validation_list_tag(event_reader, element_name, SERVICE_TAG).map(Services)
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Service {
        #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
        pub(crate) bom_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) provider: Option<OrganizationalEntity>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) group: Option<String>,
        pub(crate) name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) endpoints: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) authenticated: Option<bool>,
        #[serde(rename = "x-trust-boundary", skip_serializing_if = "Option::is_none")]
        pub(crate) x_trust_boundary: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) data: Option<Data>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) licenses: Option<Licenses>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) external_references: Option<ExternalReferences>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) properties: Option<Properties>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) services: Option<Services>,
        #[versioned("1.4", "1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) signature: Option<Signature>,
        #[versioned("1.5")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) trust_zone: Option<String>,
    }

    #[versioned("1.3", "1.4")]
    impl TryFrom<models::service::Service> for Service {
        type Error = BomError;

        fn try_from(other: models::service::Service) -> Result<Self, Self::Error> {
            Ok(Self {
                bom_ref: other.bom_ref,
                provider: convert_optional(other.provider),
                group: other.group.map(|g| g.to_string()),
                name: other.name.to_string(),
                version: other.version.map(|v| v.to_string()),
                description: other.description.map(|d| d.to_string()),
                endpoints: other
                    .endpoints
                    .map(|endpoints| endpoints.into_iter().map(|e| e.to_string()).collect()),
                authenticated: other.authenticated,
                x_trust_boundary: other.x_trust_boundary,
                data: convert_optional(other.data),
                licenses: convert_optional(other.licenses),
                external_references: try_convert_optional(other.external_references)?,
                properties: convert_optional(other.properties),
                services: try_convert_optional(other.services)?,
                #[versioned("1.4", "1.5")]
                signature: convert_optional(other.signature),
                #[versioned("1.5")]
                trust_zone: other.trust_zone.map(|tz| tz.to_string()),
            })
        }
    }

    #[versioned("1.5")]
    impl From<models::service::Service> for Service {
        fn from(other: models::service::Service) -> Self {
            Self {
                bom_ref: other.bom_ref,
                provider: convert_optional(other.provider),
                group: other.group.map(|g| g.to_string()),
                name: other.name.to_string(),
                version: other.version.map(|v| v.to_string()),
                description: other.description.map(|d| d.to_string()),
                endpoints: other
                    .endpoints
                    .map(|endpoints| endpoints.into_iter().map(|e| e.to_string()).collect()),
                authenticated: other.authenticated,
                x_trust_boundary: other.x_trust_boundary,
                data: convert_optional(other.data),
                licenses: convert_optional(other.licenses),
                external_references: convert_optional(other.external_references),
                properties: convert_optional(other.properties),
                services: convert_optional(other.services),
                #[versioned("1.4", "1.5")]
                signature: convert_optional(other.signature),
                #[versioned("1.5")]
                trust_zone: other.trust_zone.map(|tz| tz.to_string()),
            }
        }
    }

    impl From<Service> for models::service::Service {
        fn from(other: Service) -> Self {
            Self {
                bom_ref: other.bom_ref,
                provider: convert_optional(other.provider),
                group: other.group.map(NormalizedString::new_unchecked),
                name: NormalizedString::new_unchecked(other.name),
                version: other.version.map(NormalizedString::new_unchecked),
                description: other.description.map(NormalizedString::new_unchecked),
                endpoints: other
                    .endpoints
                    .map(|endpoints| endpoints.into_iter().map(Uri).collect()),
                authenticated: other.authenticated,
                x_trust_boundary: other.x_trust_boundary,
                data: convert_optional(other.data),
                licenses: convert_optional(other.licenses),
                external_references: convert_optional(other.external_references),
                properties: convert_optional(other.properties),
                services: convert_optional(other.services),
                #[versioned("1.3")]
                signature: None,
                #[versioned("1.4", "1.5")]
                signature: convert_optional(other.signature),
                #[versioned("1.3", "1.4")]
                trust_zone: None,
                #[versioned("1.5")]
                trust_zone: other.trust_zone.map(NormalizedString::new_unchecked),
            }
        }
    }

    const SERVICE_TAG: &str = "service";
    const BOM_REF_ATTR: &str = "bom-ref";
    const PROVIDER_TAG: &str = "provider";
    const GROUP_TAG: &str = "group";
    const NAME_TAG: &str = "name";
    const VERSION_TAG: &str = "version";
    const DESCRIPTION_TAG: &str = "description";
    const ENDPOINTS_TAG: &str = "endpoints";
    const ENDPOINT_TAG: &str = "endpoint";
    const AUTHENTICATED_TAG: &str = "authenticated";
    const X_TRUST_BOUNDARY_TAG: &str = "x-trust-boundary";
    const DATA_TAG: &str = "data";
    #[versioned("1.4", "1.5")]
    const SIGNATURE_TAG: &str = "signature";
    #[versioned("1.5")]
    const TRUST_ZONE_TAG: &str = "trustZone";

    impl ToXml for Service {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            let mut service_start_tag = XmlEvent::start_element(SERVICE_TAG);

            if let Some(bom_ref) = &self.bom_ref {
                service_start_tag = service_start_tag.attr(BOM_REF_ATTR, bom_ref);
            }

            writer
                .write(service_start_tag)
                .map_err(to_xml_write_error(SERVICE_TAG))?;

            if let Some(provider) = &self.provider {
                provider.write_xml_named_element(writer, PROVIDER_TAG)?;
            }

            if let Some(group) = &self.group {
                write_simple_tag(writer, GROUP_TAG, group)?;
            }

            write_simple_tag(writer, NAME_TAG, &self.name)?;

            if let Some(version) = &self.version {
                write_simple_tag(writer, VERSION_TAG, version)?;
            }

            if let Some(description) = &self.description {
                write_simple_tag(writer, DESCRIPTION_TAG, description)?;
            }

            if let Some(endpoints) = &self.endpoints {
                write_start_tag(writer, ENDPOINTS_TAG)?;

                for endpoint in endpoints {
                    write_simple_tag(writer, ENDPOINT_TAG, endpoint)?;
                }

                write_close_tag(writer, ENDPOINTS_TAG)?;
            }

            if let Some(authenticated) = &self.authenticated {
                write_simple_tag(writer, AUTHENTICATED_TAG, &format!("{}", authenticated))?;
            }

            if let Some(x_trust_boundary) = &self.x_trust_boundary {
                write_simple_tag(
                    writer,
                    X_TRUST_BOUNDARY_TAG,
                    &format!("{}", x_trust_boundary),
                )?;
            }

            if let Some(data) = &self.data {
                write_start_tag(writer, DATA_TAG)?;
                data.write_xml_element(writer)?;
                write_close_tag(writer, DATA_TAG)?;
            }

            if let Some(licenses) = &self.licenses {
                licenses.write_xml_element(writer)?;
            }

            if let Some(external_references) = &self.external_references {
                external_references.write_xml_element(writer)?;
            }

            if let Some(properties) = &self.properties {
                properties.write_xml_element(writer)?;
            }

            if let Some(services) = &self.services {
                services.write_xml_element(writer)?;
            }

            #[versioned("1.4", "1.5")]
            if let Some(signature) = &self.signature {
                signature.write_xml_element(writer)?;
            }

            #[versioned("1.5")]
            if let Some(trust_zone) = &self.trust_zone {
                write_simple_tag(writer, TRUST_ZONE_TAG, trust_zone)?;
            }

            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(SERVICE_TAG))?;

            Ok(())
        }
    }

    const LICENSES_TAG: &str = "licenses";
    const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";
    const PROPERTIES_TAG: &str = "properties";

    impl FromXml for Service {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let bom_ref = optional_attribute(attributes, BOM_REF_ATTR);

            let mut provider: Option<OrganizationalEntity> = None;
            let mut group: Option<String> = None;
            let mut service_name: Option<String> = None;
            let mut version: Option<String> = None;
            let mut description: Option<String> = None;
            let mut endpoints: Option<Vec<String>> = None;
            let mut authenticated: Option<bool> = None;
            let mut x_trust_boundary: Option<bool> = None;
            let mut data: Option<Data> = None;
            let mut licenses: Option<Licenses> = None;
            let mut external_references: Option<ExternalReferences> = None;
            let mut properties: Option<Properties> = None;
            let mut services: Option<Services> = None;
            #[versioned("1.4", "1.5")]
            let mut signature: Option<Signature> = None;
            #[versioned("1.5")]
            let mut trust_zone: Option<String> = None;

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(SERVICE_TAG))?;
                match next_element {
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == PROVIDER_TAG => {
                        provider = Some(OrganizationalEntity::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }

                    reader::XmlEvent::StartElement { name, .. } if name.local_name == GROUP_TAG => {
                        group = Some(read_simple_tag(event_reader, &name)?);
                    }

                    reader::XmlEvent::StartElement { name, .. } if name.local_name == NAME_TAG => {
                        service_name = Some(read_simple_tag(event_reader, &name)?);
                    }

                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == VERSION_TAG =>
                    {
                        version = Some(read_simple_tag(event_reader, &name)?);
                    }

                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == DESCRIPTION_TAG =>
                    {
                        description = Some(read_simple_tag(event_reader, &name)?);
                    }

                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == ENDPOINTS_TAG =>
                    {
                        endpoints = Some(read_list_tag(event_reader, &name, ENDPOINT_TAG)?);
                    }

                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == AUTHENTICATED_TAG =>
                    {
                        authenticated = Some(read_boolean_tag(event_reader, &name)?)
                    }

                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == X_TRUST_BOUNDARY_TAG =>
                    {
                        x_trust_boundary = Some(read_boolean_tag(event_reader, &name)?)
                    }

                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == DATA_TAG => {
                        data = Some(Data::read_xml_element(event_reader, &name, &attributes)?);
                    }

                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == LICENSES_TAG => {
                        licenses = Some(Licenses::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }

                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == EXTERNAL_REFERENCES_TAG => {
                        external_references = Some(ExternalReferences::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }

                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == PROPERTIES_TAG => {
                        properties = Some(Properties::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }

                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == SERVICES_TAG => {
                        services = Some(Services::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }
                    #[versioned("1.4", "1.5")]
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == SIGNATURE_TAG => {
                        signature = Some(Signature::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?)
                    }
                    #[versioned("1.5")]
                    reader::XmlEvent::StartElement { name, .. }
                        if name.local_name == TRUST_ZONE_TAG =>
                    {
                        trust_zone = Some(read_simple_tag(event_reader, &name)?)
                    }

                    // lax validation of any elements from a different schema
                    reader::XmlEvent::StartElement { name, .. } => {
                        read_lax_validation_tag(event_reader, &name)?
                    }
                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    unexpected => return Err(unexpected_element_error(element_name, unexpected)),
                }
            }

            let name = service_name.ok_or_else(|| XmlReadError::RequiredDataMissing {
                required_field: NAME_TAG.to_string(),
                element: element_name.local_name.to_string(),
            })?;

            Ok(Self {
                bom_ref,
                provider,
                group,
                name,
                version,
                description,
                endpoints,
                authenticated,
                x_trust_boundary,
                data,
                licenses,
                external_references,
                properties,
                services,
                #[versioned("1.4", "1.5")]
                signature,
                #[versioned("1.5")]
                trust_zone,
            })
        }
    }

    #[versioned("1.3", "1.4", "1.5")]
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase", untagged)]
    pub(crate) enum Data {
        /// Legacy entry type until version 1.4
        Classification(Vec<DataClassification>),
        #[versioned("1.5")]
        ServiceData(Vec<ServiceData>),
    }

    impl From<models::service::Data> for Data {
        fn from(other: models::service::Data) -> Self {
            match other {
                models::service::Data::Classification(classification) => {
                    Self::Classification(convert_vec(classification))
                }
                #[versioned("1.3", "1.4")]
                models::service::Data::ServiceData(data) => {
                    let classifications =
                        data.into_iter().map(|d| d.classification.into()).collect();
                    Self::Classification(classifications)
                }
                #[versioned("1.5")]
                models::service::Data::ServiceData(data) => Self::ServiceData(convert_vec(data)),
            }
        }
    }

    impl From<Data> for models::service::Data {
        fn from(other: Data) -> Self {
            match other {
                Data::Classification(classification) => {
                    Self::Classification(convert_vec(classification))
                }
                #[versioned("1.5")]
                Data::ServiceData(data) => Self::ServiceData(convert_vec(data)),
            }
        }
    }

    #[versioned("1.5")]
    const DATAFLOW_TAG: &str = "dataflow";

    #[versioned("1.3", "1.4")]
    impl FromXml for Data {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            _attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let mut classifications: Vec<DataClassification> = Vec::new();

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(&element_name.local_name))?;

                match next_element {
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == CLASSIFICATION_TAG => {
                        classifications.push(DataClassification::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }

                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    _ => (),
                }
            }

            Ok(Self::Classification(classifications))
        }
    }

    #[versioned("1.5")]
    impl FromXml for Data {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            _attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let mut classifications: Vec<DataClassification> = Vec::new();
            let mut service_data: Vec<ServiceData> = Vec::new();

            let mut got_end_tag = false;
            while !got_end_tag {
                let next_element = event_reader
                    .next()
                    .map_err(to_xml_read_error(&element_name.local_name))?;

                match next_element {
                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == CLASSIFICATION_TAG => {
                        classifications.push(DataClassification::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }

                    reader::XmlEvent::StartElement {
                        name, attributes, ..
                    } if name.local_name == DATAFLOW_TAG => {
                        service_data.push(ServiceData::read_xml_element(
                            event_reader,
                            &name,
                            &attributes,
                        )?);
                    }

                    reader::XmlEvent::EndElement { name } if &name == element_name => {
                        got_end_tag = true;
                    }
                    _ => (),
                }
            }

            if !service_data.is_empty() {
                Ok(Self::ServiceData(service_data))
            } else {
                Ok(Self::Classification(classifications))
            }
        }
    }

    impl ToXml for Data {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            match self {
                Self::Classification(classifications) => {
                    for classification in classifications {
                        classification.write_xml_element(writer)?;
                    }
                    Ok(())
                }
                #[versioned("1.5")]
                Self::ServiceData(services) => {
                    for service in services {
                        service.write_xml_element(writer)?;
                    }
                    Ok(())
                }
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct DataClassification {
        pub(crate) flow: String,
        pub(crate) classification: String,
    }

    impl DataClassification {
        #[allow(unused)]
        pub fn new(flow: &str, classification: &str) -> Self {
            Self {
                flow: flow.to_string(),
                classification: classification.to_string(),
            }
        }
    }

    impl From<models::service::DataClassification> for DataClassification {
        fn from(other: models::service::DataClassification) -> Self {
            Self {
                flow: other.flow.to_string(),
                classification: other.classification.to_string(),
            }
        }
    }

    impl From<DataClassification> for models::service::DataClassification {
        fn from(other: DataClassification) -> Self {
            Self {
                flow: models::service::DataFlowType::new_unchecked(&other.flow),
                classification: NormalizedString::new_unchecked(other.classification),
            }
        }
    }

    const CLASSIFICATION_TAG: &str = "classification";
    const FLOW_ATTR: &str = "flow";

    impl ToXml for DataClassification {
        fn write_xml_element<W: std::io::Write>(
            &self,
            writer: &mut xml::EventWriter<W>,
        ) -> Result<(), crate::errors::XmlWriteError> {
            writer
                .write(XmlEvent::start_element(CLASSIFICATION_TAG).attr(FLOW_ATTR, &self.flow))
                .map_err(to_xml_write_error(CLASSIFICATION_TAG))?;

            writer
                .write(XmlEvent::characters(&self.classification))
                .map_err(to_xml_write_error(CLASSIFICATION_TAG))?;

            write_close_tag(writer, CLASSIFICATION_TAG)?;

            Ok(())
        }
    }

    impl FromXml for DataClassification {
        fn read_xml_element<R: std::io::Read>(
            event_reader: &mut xml::EventReader<R>,
            element_name: &xml::name::OwnedName,
            attributes: &[xml::attribute::OwnedAttribute],
        ) -> Result<Self, XmlReadError>
        where
            Self: Sized,
        {
            let flow = attribute_or_error(element_name, attributes, FLOW_ATTR)?;
            let classification = read_simple_tag(event_reader, element_name)?;

            Ok(Self {
                flow,
                classification,
            })
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;
        use pretty_assertions::assert_eq;

        #[versioned("1.4", "1.5")]
        use crate::specs::common::signature::test::{corresponding_signature, example_signature};
        #[versioned("1.3")]
        use crate::specs::v1_3::{
            external_reference::test::{
                corresponding_external_references, example_external_references,
            },
            license::test::{corresponding_licenses, example_licenses},
        };
        #[versioned("1.4")]
        use crate::specs::v1_4::{
            external_reference::test::{
                corresponding_external_references, example_external_references,
            },
            license::test::{corresponding_licenses, example_licenses},
        };
        #[versioned("1.5")]
        use crate::specs::v1_5::{
            data_governance::{DataGovernance, DataGovernanceResponsibleParty},
            external_reference::test::{
                corresponding_external_references, example_external_references,
            },
            license::test::{corresponding_licenses, example_licenses},
        };
        use crate::{
            specs::common::{
                organization::test::{corresponding_entity, example_entity},
                property::test::{corresponding_properties, example_properties},
            },
            xml::test::{read_element_from_string, write_element_to_string},
        };

        pub(crate) fn example_services() -> Services {
            Services(vec![example_service()])
        }

        pub(crate) fn corresponding_services() -> models::service::Services {
            models::service::Services(vec![corresponding_service()])
        }

        pub(crate) fn example_service() -> Service {
            Service {
                bom_ref: Some("bom-ref".to_string()),
                provider: Some(example_entity()),
                group: Some("group".to_string()),
                name: "name".to_string(),
                version: Some("version".to_string()),
                description: Some("description".to_string()),
                endpoints: Some(vec!["endpoint".to_string()]),
                authenticated: Some(true),
                x_trust_boundary: Some(true),
                data: Some(example_data_classification()),
                licenses: Some(example_licenses()),
                external_references: Some(example_external_references()),
                properties: Some(example_properties()),
                services: Some(Services(vec![])),
                #[versioned("1.4", "1.5")]
                signature: Some(example_signature()),
                #[versioned("1.5")]
                trust_zone: Some("trust zone".to_string()),
            }
        }

        pub(crate) fn corresponding_service() -> models::service::Service {
            models::service::Service {
                bom_ref: Some("bom-ref".to_string()),
                provider: Some(corresponding_entity()),
                group: Some(NormalizedString::new_unchecked("group".to_string())),
                name: NormalizedString::new_unchecked("name".to_string()),
                version: Some(NormalizedString::new_unchecked("version".to_string())),
                description: Some(NormalizedString::new_unchecked("description".to_string())),
                endpoints: Some(vec![Uri("endpoint".to_string())]),
                authenticated: Some(true),
                x_trust_boundary: Some(true),
                data: Some(corresponding_data_classification()),
                licenses: Some(corresponding_licenses()),
                external_references: Some(corresponding_external_references()),
                properties: Some(corresponding_properties()),
                services: Some(models::service::Services(vec![])),
                #[versioned("1.3")]
                signature: None,
                #[versioned("1.4", "1.5")]
                signature: Some(corresponding_signature()),
                #[versioned("1.3", "1.4")]
                trust_zone: None,
                #[versioned("1.5")]
                trust_zone: Some("trust zone".into()),
            }
        }

        #[versioned("1.3", "1.4")]
        fn example_data_classification() -> Data {
            Data::Classification(vec![DataClassification {
                flow: "flow".to_string(),
                classification: "classification".to_string(),
            }])
        }

        #[versioned("1.5")]
        fn example_data_classification() -> Data {
            Data::ServiceData(vec![ServiceData {
                name: Some("Consumer to Stock Service".to_string()),
                description: Some("Traffic to/from consumer to service".to_string()),
                classification: DataClassification {
                    flow: "flow".to_string(),
                    classification: "classification".to_string(),
                },
                governance: Some(DataGovernance {
                    custodians: None,
                    stewards: None,
                    owners: Some(vec![DataGovernanceResponsibleParty::Organization(
                        OrganizationalEntity {
                            bom_ref: None,
                            name: Some("Organization 1".to_string()),
                            url: None,
                            contact: None,
                        },
                    )]),
                }),
                source: Some(vec!["https://0.0.0.0".to_string()]),
                destination: Some(vec!["https://0.0.0.0".to_string()]),
            }])
        }

        #[versioned("1.3", "1.4")]
        fn corresponding_data_classification() -> models::service::Data {
            models::service::Data::Classification(vec![models::service::DataClassification {
                flow: models::service::DataFlowType::UnknownDataFlow("flow".to_string()),
                classification: NormalizedString::new_unchecked("classification".to_string()),
            }])
        }

        #[versioned("1.5")]
        fn corresponding_data_classification() -> models::service::Data {
            models::service::Data::ServiceData(vec![models::service::ServiceData {
                name: Some(NormalizedString::new_unchecked(
                    "Consumer to Stock Service".to_string(),
                )),
                description: Some(NormalizedString::new_unchecked(
                    "Traffic to/from consumer to service".to_string(),
                )),
                classification: models::service::DataClassification {
                    flow: models::service::DataFlowType::UnknownDataFlow("flow".to_string()),
                    classification: NormalizedString::new_unchecked("classification".to_string()),
                },
                governance: Some(models::data_governance::DataGovernance {
                    custodians: None,
                    stewards: None,
                    owners: Some(vec![
                        models::data_governance::DataGovernanceResponsibleParty::Organization(
                            models::organization::OrganizationalEntity {
                                bom_ref: None,
                                name: Some(NormalizedString::new_unchecked(
                                    "Organization 1".to_string(),
                                )),
                                url: None,
                                contact: None,
                            },
                        ),
                    ]),
                }),
                source: Some(vec![Uri("https://0.0.0.0".to_string())]),
                destination: Some(vec![Uri("https://0.0.0.0".to_string())]),
            }])
        }

        #[test]
        fn it_should_write_xml_full() {
            // NOTE: this only tests version 1.3 currently
            let xml_output = write_element_to_string(example_services());
            insta::assert_snapshot!(xml_output);
        }

        #[test]
        fn it_should_read_xml_data_classifications() {
            let input = r#"
<data>
  <classification flow="inbound">PII</classification>
  <classification flow="outbound">PIFI</classification>
  <classification flow="bi-directional">public</classification>
  <classification flow="unknown">partner-data</classification>
</data>
            "#;

            let actual: Data = read_element_from_string(input);
            let expected = Data::Classification(vec![
                DataClassification::new("inbound", "PII"),
                DataClassification::new("outbound", "PIFI"),
                DataClassification::new("bi-directional", "public"),
                DataClassification::new("unknown", "partner-data"),
            ]);
            assert_eq!(actual, expected);
        }

        #[test]
        fn it_should_read_xml_full() {
            #[versioned("1.3")]
            let input = r#"
<services>
  <service bom-ref="bom-ref">
    <provider>
      <name>name</name>
      <url>url</url>
      <contact>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </contact>
    </provider>
    <group>group</group>
    <name>name</name>
    <version>version</version>
    <description>description</description>
    <endpoints>
      <endpoint>endpoint</endpoint>
    </endpoints>
    <authenticated>true</authenticated>
    <x-trust-boundary>true</x-trust-boundary>
    <data>
      <classification flow="flow">classification</classification>
    </data>
    <licenses>
      <expression>expression</expression>
    </licenses>
    <externalReferences>
      <reference type="external reference type">
        <url>url</url>
        <comment>comment</comment>
        <hashes>
          <hash alg="algorithm">hash value</hash>
        </hashes>
      </reference>
    </externalReferences>
    <properties>
      <property name="name">value</property>
    </properties>
    <services />
  </service>
</services>
"#;
            #[versioned("1.4")]
            let input = r#"
<services>
  <service bom-ref="bom-ref">
    <provider>
      <name>name</name>
      <url>url</url>
      <contact>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </contact>
    </provider>
    <group>group</group>
    <name>name</name>
    <version>version</version>
    <description>description</description>
    <endpoints>
      <endpoint>endpoint</endpoint>
    </endpoints>
    <authenticated>true</authenticated>
    <x-trust-boundary>true</x-trust-boundary>
    <data>
      <classification flow="flow">classification</classification>
    </data>
    <licenses>
      <expression>expression</expression>
    </licenses>
    <externalReferences>
      <reference type="external reference type">
        <url>url</url>
        <comment>comment</comment>
        <hashes>
          <hash alg="algorithm">hash value</hash>
        </hashes>
      </reference>
    </externalReferences>
    <properties>
      <property name="name">value</property>
    </properties>
    <services />
    <signature>
      <algorithm>HS512</algorithm>
     <value>1234567890</value>
    </signature>
    <trustZone>trust zone</trustZone>
  </service>
</services>
"#;
            #[versioned("1.5")]
            let input = r#"
<services>
  <service bom-ref="bom-ref">
    <provider>
      <name>name</name>
      <url>url</url>
      <contact>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </contact>
    </provider>
    <group>group</group>
    <name>name</name>
    <version>version</version>
    <description>description</description>
    <endpoints>
      <endpoint>endpoint</endpoint>
    </endpoints>
    <authenticated>true</authenticated>
    <x-trust-boundary>true</x-trust-boundary>
    <data>
      <dataflow name="Consumer to Stock Service" description="Traffic to/from consumer to service">
        <classification flow="flow">classification</classification>
        <governance>
          <owners>
            <owner>
              <organization>
                <name>Organization 1</name>
              </organization>
            </owner>
          </owners>
        </governance>
        <source>
          <url>https://0.0.0.0</url>
        </source>
        <destination>
          <url>https://0.0.0.0</url>
        </destination>
      </dataflow>
    </data>
    <licenses>
      <expression>expression</expression>
    </licenses>
    <externalReferences>
      <reference type="external reference type">
        <url>url</url>
        <comment>comment</comment>
        <hashes>
          <hash alg="algorithm">hash value</hash>
        </hashes>
      </reference>
    </externalReferences>
    <properties>
      <property name="name">value</property>
    </properties>
    <services />
    <signature>
      <algorithm>HS512</algorithm>
     <value>1234567890</value>
    </signature>
    <trustZone>trust zone</trustZone>
  </service>
</services>
"#;
            let actual: Services = read_element_from_string(input);
            let expected = example_services();
            assert_eq!(actual, expected);
        }
    }
}
