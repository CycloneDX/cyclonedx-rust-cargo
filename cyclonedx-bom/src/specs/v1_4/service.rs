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

use crate::{
    errors::XmlReadError,
    external_models::{
        normalized_string::NormalizedString,
        uri::Uri,
    },
    models,
    utilities::{convert_optional, convert_optional_vec, convert_vec},
    xml::{
        attribute_or_error, optional_attribute, read_boolean_tag, read_lax_validation_list_tag,
        read_lax_validation_tag, read_list_tag, read_simple_tag, to_xml_read_error,
        to_xml_write_error, unexpected_element_error, write_simple_tag, FromXml, ToInnerXml, ToXml,
    },
};
use serde::{Deserialize, Serialize};
use xml::{reader, writer::XmlEvent};

use crate::specs::v1_4::{
    external_reference::ExternalReferences,
    license::Licenses,
    organization::OrganizationalEntity,
    property::Properties,
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct Services(Vec<Service>);

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
        writer
            .write(XmlEvent::start_element(SERVICES_TAG))
            .map_err(to_xml_write_error(SERVICES_TAG))?;

        for service in &self.0 {
            service.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(SERVICES_TAG))?;

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
    bom_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<OrganizationalEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    authenticated: Option<bool>,
    #[serde(rename = "x-trust-boundary", skip_serializing_if = "Option::is_none")]
    x_trust_boundary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Vec<DataClassification>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    licenses: Option<Licenses>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_references: Option<ExternalReferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    services: Option<Services>,
}

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
            data: convert_optional_vec(other.data),
            licenses: convert_optional(other.licenses),
            external_references: convert_optional(other.external_references),
            properties: convert_optional(other.properties),
            services: convert_optional(other.services),
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
            data: convert_optional_vec(other.data),
            licenses: convert_optional(other.licenses),
            external_references: convert_optional(other.external_references),
            properties: convert_optional(other.properties),
            services: convert_optional(other.services),
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
            writer
                .write(XmlEvent::start_element(ENDPOINTS_TAG))
                .map_err(to_xml_write_error(ENDPOINTS_TAG))?;
            for endpoint in endpoints {
                write_simple_tag(writer, ENDPOINT_TAG, endpoint)?;
            }
            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(ENDPOINTS_TAG))?;
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
            writer
                .write(XmlEvent::start_element(DATA_TAG))
                .map_err(to_xml_write_error(DATA_TAG))?;
            for d in data {
                d.write_xml_element(writer)?;
            }
            writer
                .write(XmlEvent::end_element())
                .map_err(to_xml_write_error(DATA_TAG))?;
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
        let mut data: Option<Vec<DataClassification>> = None;
        let mut licenses: Option<Licenses> = None;
        let mut external_references: Option<ExternalReferences> = None;
        let mut properties: Option<Properties> = None;
        let mut services: Option<Services> = None;

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

                reader::XmlEvent::StartElement { name, .. } if name.local_name == VERSION_TAG => {
                    version = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. }
                    if name.local_name == DESCRIPTION_TAG =>
                {
                    description = Some(read_simple_tag(event_reader, &name)?);
                }

                reader::XmlEvent::StartElement { name, .. } if name.local_name == ENDPOINTS_TAG => {
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

                reader::XmlEvent::StartElement { name, .. } if name.local_name == DATA_TAG => {
                    data = Some(read_list_tag(event_reader, &name, CLASSIFICATION_TAG)?);
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
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DataClassification {
    flow: String,
    classification: String,
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

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(CLASSIFICATION_TAG))?;

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
    use crate::{
        specs::v1_4::{
            external_reference::test::{
                corresponding_external_references, example_external_references,
            },
            license::test::{corresponding_licenses, example_licenses},
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
            data: Some(vec![example_data_classification()]),
            licenses: Some(example_licenses()),
            external_references: Some(example_external_references()),
            properties: Some(example_properties()),
            services: Some(Services(vec![])),
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
            data: Some(vec![corresponding_data_classification()]),
            licenses: Some(corresponding_licenses()),
            external_references: Some(corresponding_external_references()),
            properties: Some(corresponding_properties()),
            services: Some(models::service::Services(vec![])),
        }
    }

    fn example_data_classification() -> DataClassification {
        DataClassification {
            flow: "flow".to_string(),
            classification: "classification".to_string(),
        }
    }

    fn corresponding_data_classification() -> models::service::DataClassification {
        models::service::DataClassification {
            flow: models::service::DataFlowType::UnknownDataFlow("flow".to_string()),
            classification: NormalizedString::new_unchecked("classification".to_string()),
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_services());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
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
        let actual: Services = read_element_from_string(input);
        let expected = example_services();
        assert_eq!(actual, expected);
    }
}
