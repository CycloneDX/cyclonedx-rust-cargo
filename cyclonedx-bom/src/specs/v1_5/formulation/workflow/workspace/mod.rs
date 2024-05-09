pub(crate) mod volume;

use serde::{Deserialize, Serialize};
use xml::writer;

use crate::{
    elem_tag,
    errors::XmlReadError,
    get_elements_lax, models,
    specs::common::property::Properties,
    utilities::{convert_optional, convert_optional_vec},
    xml::{
        attribute_or_error, to_xml_write_error, write_close_tag, write_list_string_tag,
        write_simple_option_tag, write_simple_tag, FromXml, ToXml, VecElemTag, VecXmlReader,
    },
};

use super::resource_reference::ResourceReferences;
use volume::Volume;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Workspace {
    #[serde(rename = "bom-ref")]
    pub(crate) bom_ref: String,
    pub(crate) uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resource_references: Option<ResourceReferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) access_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mount_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) managed_data_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) volume_request: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) volume: Option<Volume>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<Properties>,
}

impl From<models::formulation::workflow::workspace::Workspace> for Workspace {
    fn from(workspace: models::formulation::workflow::workspace::Workspace) -> Self {
        Self {
            bom_ref: workspace.bom_ref.0,
            uid: workspace.uid,
            name: workspace.name,
            aliases: workspace.aliases,
            description: workspace.description,
            resource_references: convert_optional_vec(workspace.resource_references)
                .map(ResourceReferences),
            access_mode: workspace
                .access_mode
                .map(|access_mode| access_mode.to_string()),
            mount_path: workspace.mount_path,
            managed_data_type: workspace.managed_data_type,
            volume_request: workspace.volume_request,
            volume: convert_optional(workspace.volume),
            properties: convert_optional(workspace.properties),
        }
    }
}

impl From<Workspace> for models::formulation::workflow::workspace::Workspace {
    fn from(workspace: Workspace) -> Self {
        Self {
            bom_ref: models::bom::BomReference::new(workspace.bom_ref),
            uid: workspace.uid,
            name: workspace.name,
            aliases: workspace.aliases,
            description: workspace.description,
            resource_references: convert_optional_vec(workspace.resource_references.map(|rs| rs.0)),
            access_mode: workspace
                .access_mode
                .map(models::formulation::workflow::workspace::AccessMode::new_unchecked),
            mount_path: workspace.mount_path,
            managed_data_type: workspace.managed_data_type,
            volume_request: workspace.volume_request,
            volume: convert_optional(workspace.volume),
            properties: convert_optional(workspace.properties),
        }
    }
}

const WORKSPACE_TAG: &str = "workspace";
const BOM_REF_ATTR: &str = "bom-ref";
const UID_TAG: &str = "uid";
const NAME_TAG: &str = "name";
const ALIASES_TAG: &str = "aliases";
const DESCRIPTION_TAG: &str = "description";
const RESOURCE_REFERENCES_TAG: &str = "resourceReferences";
const ACCESS_MODE_TAG: &str = "accessMode";
const MOUNT_PATH_TAG: &str = "mountPath";
const MANAGED_DATA_TYPE_TAG: &str = "managedDataType";
const VOLUME_REQUEST_TAG: &str = "volumeRequest";
const VOLUME_TAG: &str = "volume";
const PROPERTIES_TAG: &str = "properties";
elem_tag!(AliasTag = "alias");

impl ToXml for Workspace {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        writer
            .write(writer::XmlEvent::start_element(WORKSPACE_TAG).attr(BOM_REF_ATTR, &self.bom_ref))
            .map_err(to_xml_write_error(WORKSPACE_TAG))?;

        write_simple_tag(writer, UID_TAG, &self.uid)?;
        write_simple_option_tag(writer, NAME_TAG, &self.name)?;

        if let Some(aliases) = &self.aliases {
            write_list_string_tag(writer, ALIASES_TAG, AliasTag::VALUE, aliases)?
        }

        write_simple_option_tag(writer, DESCRIPTION_TAG, &self.description)?;
        self.resource_references.write_xml_element(writer)?;
        write_simple_option_tag(writer, ACCESS_MODE_TAG, &self.access_mode)?;
        write_simple_option_tag(writer, MOUNT_PATH_TAG, &self.mount_path)?;
        write_simple_option_tag(writer, MANAGED_DATA_TYPE_TAG, &self.managed_data_type)?;
        write_simple_option_tag(writer, VOLUME_REQUEST_TAG, &self.volume_request)?;
        self.volume.write_xml_element(writer)?;
        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, WORKSPACE_TAG)
    }
}

impl FromXml for Workspace {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = attribute_or_error(element_name, attributes, BOM_REF_ATTR)?;

        get_elements_lax! {
            event_reader, element_name,
            UID_TAG => uid: String,
            NAME_TAG => name: String,
            ALIASES_TAG => aliases: VecXmlReader<String, AliasTag>,
            DESCRIPTION_TAG => description: String,
            RESOURCE_REFERENCES_TAG => resource_references: ResourceReferences,
            ACCESS_MODE_TAG => access_mode: String,
            MOUNT_PATH_TAG => mount_path: String,
            MANAGED_DATA_TYPE_TAG => managed_data_type: String,
            VOLUME_REQUEST_TAG => volume_request: String,
            VOLUME_TAG => volume: Volume,
            PROPERTIES_TAG => properties: Properties,
        };

        Ok(Self {
            bom_ref,
            uid: uid.ok_or_else(|| XmlReadError::required_data_missing(UID_TAG, element_name))?,
            name,
            aliases: aliases.map(Vec::from),
            description,
            resource_references,
            access_mode,
            mount_path,
            managed_data_type,
            volume_request,
            volume,
            properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        specs::v1_5::formulation::workflow::resource_reference::ResourceReference,
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::*;

    fn example_workspace() -> Workspace {
        Workspace {
            bom_ref: "workspace-2".into(),
            uid: "workspace-1".into(),
            name: Some("My workspace".into()),
            aliases: Some(vec!["default-workspace".into()]),
            description: Some("Description here".into()),
            resource_references: Some(ResourceReferences(vec![ResourceReference::Ref {
                r#ref: "component-t".into(),
            }])),
            access_mode: Some("read-write".into()),
            mount_path: Some("/tmp/workspace".into()),
            managed_data_type: Some("ConfigMap".into()),
            volume_request: Some("requestedVolumeClaim".into()),
            volume: Some(Volume {
                uid: Some("volume-1".into()),
                name: Some("My volume".into()),
                mode: Some("filesystem".into()),
                path: Some("/".into()),
                size_allocated: Some("10GB".into()),
                persistent: Some(true),
                remote: Some(false),
                properties: None,
            }),
            properties: None,
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_output = write_element_to_string(example_workspace());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<workspace bom-ref="workspace-2">
    <uid>workspace-1</uid>
    <name>My workspace</name>
    <aliases>
        <alias>default-workspace</alias>
    </aliases>
    <description>Description here</description>
    <resourceReferences>
        <resourceReference>
            <ref>component-t</ref>
        </resourceReference>
    </resourceReferences>
    <accessMode>read-write</accessMode>
    <mountPath>/tmp/workspace</mountPath>
    <managedDataType>ConfigMap</managedDataType>
    <volumeRequest>requestedVolumeClaim</volumeRequest>
    <volume>
        <uid>volume-1</uid>
        <name>My volume</name>
        <mode>filesystem</mode>
        <path>/</path>
        <sizeAllocated>10GB</sizeAllocated>
        <persistent>true</persistent>
        <remote>false</remote>
    </volume>
</workspace>
"#;
        let actual: Workspace = read_element_from_string(input);
        let expected = example_workspace();
        assert_eq!(actual, expected);
    }
}
