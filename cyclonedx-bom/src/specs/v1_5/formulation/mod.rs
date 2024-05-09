mod workflow;

use serde::{Deserialize, Serialize};
use xml::writer;

use crate::{
    elem_tag,
    errors::BomError,
    get_elements,
    models::formulation as models,
    specs::common::property::Properties,
    utilities::{convert_optional, convert_optional_vec, try_convert_optional},
    xml::{
        attribute_or_error, to_xml_write_error, write_close_tag, write_list_tag, FromXml, ToXml,
        VecXmlReader,
    },
};

use self::workflow::Workflow;

use super::{component::Components, service::Services};

// #definitions/formula
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Formula {
    #[serde(rename = "bom-ref", skip_serializing_if = "Option::is_none")]
    bom_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    services: Option<Services>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflows: Option<Vec<Workflow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
}

impl TryFrom<models::Formula> for Formula {
    type Error = BomError;

    fn try_from(formula: models::Formula) -> Result<Self, Self::Error> {
        Ok(Self {
            bom_ref: formula.bom_ref.map(|br| br.0),
            components: try_convert_optional(formula.components)?,
            services: convert_optional(formula.services),
            workflows: convert_optional_vec(formula.workflows),
            properties: convert_optional(formula.properties),
        })
    }
}

impl From<Formula> for models::Formula {
    fn from(formula: Formula) -> Self {
        Self {
            bom_ref: formula.bom_ref.map(crate::models::bom::BomReference),
            components: convert_optional(formula.components),
            services: convert_optional(formula.services),
            workflows: convert_optional_vec(formula.workflows),
            properties: convert_optional(formula.properties),
        }
    }
}

const FORMULA_TAG: &str = "formula";
const BOM_REF_ATTR: &str = "bom-ref";
const COMPONENTS_TAG: &str = "components";
const SERVICES_TAG: &str = "services";
const WORKFLOWS_TAG: &str = "workflows";
const PROPERTIES_TAG: &str = "properties";

elem_tag!(WorkflowTag = "workflow");

impl ToXml for Formula {
    fn write_xml_element<W: std::io::prelude::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let mut start_element = writer::XmlEvent::start_element(FORMULA_TAG);

        if let Some(bom_ref) = &self.bom_ref {
            start_element = start_element.attr(BOM_REF_ATTR, bom_ref.as_ref())
        }

        writer
            .write(start_element)
            .map_err(to_xml_write_error(FORMULA_TAG))?;

        self.components.write_xml_element(writer)?;
        self.services.write_xml_element(writer)?;
        if let Some(workflows) = &self.workflows {
            write_list_tag(writer, WORKFLOWS_TAG, workflows)?;
        }
        self.properties.write_xml_element(writer)?;

        write_close_tag(writer, FORMULA_TAG)
    }
}

impl FromXml for Formula {
    fn read_xml_element<R: std::io::prelude::Read>(
        event_reader: &mut xml::EventReader<R>,
        element_name: &xml::name::OwnedName,
        attributes: &[xml::attribute::OwnedAttribute],
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        let bom_ref = attribute_or_error(element_name, attributes, BOM_REF_ATTR).ok();

        get_elements! {
            event_reader, element_name,
            COMPONENTS_TAG => components: Components,
            SERVICES_TAG => services: Services,
            WORKFLOWS_TAG => workflows: VecXmlReader<Workflow, WorkflowTag>,
            PROPERTIES_TAG => properties: Properties,
        };

        Ok(Self {
            bom_ref,
            components,
            services,
            workflows: workflows.map(Vec::from),
            properties,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        specs::common::component::v1_5::Component,
        xml::test::{read_element_from_string, write_element_to_string},
    };

    use super::*;

    pub(crate) fn example_formula() -> Formula {
        Formula {
            bom_ref: Some("formula-1".into()),
            components: Some(Components(vec![Component {
                component_type: "platform".into(),
                mime_type: None,
                bom_ref: Some("component-1".into()),
                supplier: None,
                author: None,
                publisher: None,
                group: None,
                name: "Pipeline controller image".into(),
                version: Some("v0.47.0".into()),
                description: None,
                scope: None,
                hashes: None,
                licenses: None,
                copyright: None,
                cpe: None,
                purl: None,
                swid: None,
                modified: None,
                pedigree: None,
                external_references: None,
                properties: None,
                components: None,
                evidence: None,
                signature: None,
                model_card: None,
                data: None,
            }])),
            services: None,
            workflows: None,
            properties: None,
        }
    }

    pub(crate) fn corresponding_formula() -> models::Formula {
        models::Formula {
            bom_ref: Some(crate::models::bom::BomReference::new("formula-1")),
            components: Some(crate::models::component::Components(vec![
                crate::models::component::Component {
                    component_type: crate::models::component::Classification::Platform,
                    mime_type: None,
                    bom_ref: Some("component-1".into()),
                    supplier: None,
                    author: None,
                    publisher: None,
                    group: None,
                    name: "Pipeline controller image".into(),
                    version: Some("v0.47.0".into()),
                    description: None,
                    scope: None,
                    hashes: None,
                    licenses: None,
                    copyright: None,
                    cpe: None,
                    purl: None,
                    swid: None,
                    modified: None,
                    pedigree: None,
                    external_references: None,
                    properties: None,
                    components: None,
                    evidence: None,
                    signature: None,
                    model_card: None,
                    data: None,
                },
            ])),
            services: None,
            workflows: None,
            properties: None,
        }
    }

    #[test]
    fn it_should_write_xml_full() {
        let xml_input = write_element_to_string(example_formula());
        insta::assert_snapshot!(xml_input);
    }

    #[test]
    fn it_should_read_xml_full() {
        let input = r#"
<formula bom-ref="formula-1">
    <components>
        <component type="platform" bom-ref="component-1">
            <name>Pipeline controller image</name>
            <version>v0.47.0</version>
        </component>
    </components>
</formula>
"#;
        let actual: Formula = read_element_from_string(input);
        let expected = example_formula();
        assert_eq!(actual, expected);
    }
}
