use roxmltree::Node;

use crate::parser::constants::attribute;
use crate::parser::node_parser::parse_node;
use crate::parser::types::{Alias, RsEntity, Struct, StructField, StructFieldSource, TypeModifier};
use crate::parser::utils::get_documentation;
use crate::parser::xsd_elements::{ElementType, UseType, XsdNode};

const SUPPORTED_CONTENT_TYPES: [ElementType; 1] =
    [ElementType::SimpleType];

pub fn parse_attribute(node: &Node, parent: &Node) -> RsEntity {
    if parent.xsd_type() == ElementType::Schema {
        return parse_global_attribute(node);
    }

    let name = node
        .attr_name()
        .or_else(|| node.attr_ref())
        .expect("All attributes have name or ref")
        .to_string();
    
    let type_modifier = match node.attr_use() {
        UseType::Optional => TypeModifier::Option,
        UseType::Prohibited => TypeModifier::Empty,
        UseType::Required => TypeModifier::None,
    };

    if node.has_attribute(attribute::TYPE) || node.has_attribute(attribute::REF) {
        let type_name = node
            .attr_type()
            .unwrap_or_else(|| node.attr_ref().unwrap_or("String"))
            .to_string();

        return RsEntity::StructField(StructField {
            type_name,
            comment: get_documentation(node),
            subtypes: vec![],
            name,
            source: StructFieldSource::Attribute,
            type_modifiers: vec![type_modifier],
        });
    }

    let content_node = node
        .children()
        .filter(|n| SUPPORTED_CONTENT_TYPES.contains(&n.xsd_type()))
        .last()
        .unwrap_or_else(|| {
        panic!(
            "Must have content if no 'type' or 'ref' attribute: {:?}",
            node
        )
    });

    let mut field_type = parse_node(&content_node, node);
    field_type.set_name(format!("{}Type", name).as_str());


    RsEntity::StructField(StructField {
        name,
        type_name: field_type.name().to_string(),
        comment: get_documentation(node),
        subtypes: vec![field_type],
        source: StructFieldSource::Attribute,
        type_modifiers: vec![type_modifier],
    })
}

fn parse_global_attribute(node: &Node) -> RsEntity {
    if let Some(reference) = node.attr_ref() {
        return RsEntity::Alias(Alias {
            name: reference.to_string(),
            original: reference.to_string(),
            comment: get_documentation(node),
            ..Default::default()
        });
    }

    let name = node
        .attr_name()
        .unwrap_or_else(|| panic!("Name attribute required. {:?}", node));

    if let Some(ty) = node.attr_type() {
        return RsEntity::Alias(Alias {
            name: name.to_string(),
            original: ty.to_string(),
            comment: get_documentation(node),
            ..Default::default()
        });
    }

    if let Some(content) = node
        .children()
        .filter(|n| n.is_element() && n.xsd_type() == ElementType::SimpleType)
        .last()
    {
        let mut entity = parse_node(&content, node);
        entity.set_name(name);
        return entity;
    }

    RsEntity::Struct(Struct {
        name: name.to_string(),
        ..Default::default()
    })
}

#[cfg(test)]
mod test {
    use crate::parser::attribute::parse_global_attribute;
    use crate::parser::types::RsEntity;
    use crate::parser::utils::find_child;

    #[test]
    fn test_global_attribute_with_nested_type() {
        let doc = roxmltree::Document::parse(
            r#"
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:xmime="http://www.w3.org/2005/05/xmlmime"
           targetNamespace="http://www.w3.org/2005/05/xmlmime" >

            <xs:attribute name="contentType">
                <xs:simpleType>
                    <xs:restriction base="xs:string" >
                        <xs:minLength value="3" />
                    </xs:restriction>
                </xs:simpleType>
            </xs:attribute>
        </xs:schema>
        "#,
        )
        .unwrap();

        let schema = doc.root_element();
        let attribute = find_child(&schema, "attribute").unwrap();
        match parse_global_attribute(&attribute) {
            RsEntity::TupleStruct(ts) => {
                assert_eq!(ts.name, "contentType");
                assert_eq!(ts.type_name, "xs:string");
                assert_eq!(ts.facets.len(), 1);
            }
            _ => unreachable!("Test Failed!"),
        }
    }

    #[test]
    fn test_global_attribute_with_type() {
        let doc = roxmltree::Document::parse(
            r#"
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:xmime="http://www.w3.org/2005/05/xmlmime"
           targetNamespace="http://www.w3.org/2005/05/xmlmime" >
            <xs:attribute name="expectedContentTypes" type="xs:string" />
        </xs:schema>
        "#,
        )
        .unwrap();

        let schema = doc.root_element();
        let attribute = find_child(&schema, "attribute").unwrap();
        match parse_global_attribute(&attribute) {
            RsEntity::Alias(ts) => {
                assert_eq!(ts.name, "expectedContentTypes");
                assert_eq!(ts.original, "xs:string");
                assert_eq!(ts.subtypes.len(), 0);
            }
            _ => unreachable!("Test Failed!"),
        }
    }
}
