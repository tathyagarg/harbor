use std::ops::Deref;

use harbor::html5;
use harbor::infra;

mod common;

#[test]
fn test_css000() {
    let html_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test CSS</title>
    <style>
        body {
            background-color: lightblue;
        }
    </style>
</head>
<body>
    <h1>This is a heading</h1>
    <p>This is a paragraph.</p>
</body>
</html>"#;

    let chars = html_content.chars().collect::<Vec<char>>();
    let slice = chars.as_slice();

    let mut stream = infra::InputStream::new(slice);
    let mut tokenizer = html5::parse::Parser::new(&mut stream);

    tokenizer.tokenize();

    common::verify_element_structure(
        tokenizer.document.document().borrow().deref(),
        common::ElementStructure {
            tag_name: "html".to_string(),
            attributes: vec![],
            children: vec![
                common::ElementStructure {
                    tag_name: "head".to_string(),
                    attributes: vec![],
                    children: vec![
                        common::ElementStructure {
                            tag_name: "title".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                        common::ElementStructure {
                            tag_name: "style".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                    ],
                },
                common::ElementStructure {
                    tag_name: "body".to_string(),
                    attributes: vec![],
                    children: vec![
                        common::ElementStructure {
                            tag_name: "h1".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                        common::ElementStructure {
                            tag_name: "p".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                    ],
                },
            ],
        },
    );
}

#[test]
fn test_css001() {
    let document = common::parse_html_path_to_document("../assets/html/css001.html");

    common::verify_element_structure(
        &document,
        common::ElementStructure {
            tag_name: "html".to_string(),
            attributes: vec![],
            children: vec![
                common::ElementStructure {
                    tag_name: "head".to_string(),
                    attributes: vec![],
                    children: vec![
                        common::ElementStructure {
                            tag_name: "title".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                        common::ElementStructure {
                            tag_name: "style".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                    ],
                },
                common::ElementStructure {
                    tag_name: "body".to_string(),
                    attributes: vec![],
                    children: vec![
                        common::ElementStructure {
                            tag_name: "h1".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                        common::ElementStructure {
                            tag_name: "p".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                    ],
                },
            ],
        },
    );
}

#[test]
fn test_css002() {
    let document = common::parse_html_path_to_document("../assets/html/css002.html");

    common::verify_element_structure(
        &document,
        common::ElementStructure {
            tag_name: "html".to_string(),
            attributes: vec![],
            children: vec![
                common::ElementStructure {
                    tag_name: "head".to_string(),
                    attributes: vec![],
                    children: vec![common::ElementStructure {
                        tag_name: "title".to_string(),
                        attributes: vec![],
                        children: vec![],
                    }],
                },
                common::ElementStructure {
                    tag_name: "body".to_string(),
                    attributes: vec![],
                    children: vec![
                        common::ElementStructure {
                            tag_name: "h1".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                        common::ElementStructure {
                            tag_name: "p".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                        common::ElementStructure {
                            tag_name: "ul".to_string(),
                            attributes: vec![],
                            children: vec![
                                common::ElementStructure {
                                    tag_name: "li".to_string(),
                                    attributes: vec![],
                                    children: vec![],
                                },
                                common::ElementStructure {
                                    tag_name: "li".to_string(),
                                    attributes: vec![],
                                    children: vec![],
                                },
                                common::ElementStructure {
                                    tag_name: "li".to_string(),
                                    attributes: vec![],
                                    children: vec![],
                                },
                            ],
                        },
                    ],
                },
            ],
        },
    );
}
