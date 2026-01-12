use std::{cell::RefCell, ops::Deref, rc::Rc};

use harbor::html5::dom::{Document, Element, IElement, NodeKind};

pub struct ElementStructure {
    pub tag_name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<ElementStructure>,
}

pub fn verify_element_structure(doc: &Document, target: ElementStructure) {
    assert!(doc.doctype().is_some(), "Document should have a doctype");

    let doc_node = doc._node.borrow();

    assert!(
        doc_node.child_nodes().len() > 0,
        "Document should have child nodes"
    );
    assert!(
        doc_node
            .child_nodes()
            .item(0)
            .is_some_and(|n| matches!(n.borrow().deref(), NodeKind::DocumentType(_))),
        "First child should be a doctype"
    );

    let html_element = doc_node
        .child_nodes()
        .item(1)
        .expect("Document should have an <html> element");

    assert!(
        matches!(html_element.borrow().deref(), NodeKind::Element(_)),
        "Second child should be an <html> element"
    );

    match html_element.borrow().deref() {
        NodeKind::Element(el) => {
            verify_html_element_structure(el, &target);
        }
        _ => unreachable!(),
    }
}

fn verify_html_element_structure(html: &Rc<RefCell<Element>>, target: &ElementStructure) {
    let html_borrow = html.borrow();

    assert_eq!(
        html_borrow.local_name, target.tag_name,
        "Tag name should match"
    );

    for (attr_name, attr_value) in &target.attributes {
        let attr = html_borrow
            .get_attribute(attr_name)
            .expect(&format!("Attribute '{}' should exist", attr_name));
        assert_eq!(
            &attr, attr_value,
            "Attribute '{}' should have value '{}'",
            attr_name, attr_value
        );
    }

    let html_node = html_borrow.node().borrow();

    assert_eq!(
        html_node
            .child_nodes()
            .filter(|child| { matches!(child.borrow().deref(), NodeKind::Element(_)) })
            .len(),
        target.children.len(),
        "Number of child nodes should match"
    );

    let mut i = 0;

    for child_target in &target.children {
        while !matches!(
            html_node
                .child_nodes()
                .item(i)
                .expect("Child node should exist")
                .borrow()
                .deref(),
            NodeKind::Element(_)
        ) {
            i += 1;
        }

        let child_node = html_node
            .child_nodes()
            .item(i)
            .expect(&format!("Child node at index {} should exist", i));

        println!(
            "Verifying child node at index {}: child node = {:?}",
            i,
            child_node.borrow().deref(),
        );

        match child_node.borrow().deref() {
            NodeKind::Element(child_el) => {
                verify_html_element_structure(child_el, &child_target);
            }
            _ => unreachable!(),
        }

        i += 1;
    }
}

pub fn parse_html_path_to_document(path: &str) -> Document {
    let html_content = std::fs::read_to_string(path).unwrap();

    let mut stream = harbor::html5::parse::InputStream::new(html_content);
    let mut parser = harbor::html5::parse::Parser::new(&mut stream);

    parser.tokenize();

    parser.document.document().clone()
}
