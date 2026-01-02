#![allow(dead_code)]

use crate::http::{self, url::Serializable};
/// Custom implementation of the HTML5 spec:
/// https://html.spec.whatwg.org/
pub mod parse;

type DOMString = String;
type USVString = String;

#[derive(Clone, Copy, PartialEq)]
pub enum NodeType {
    Element = 1,
    Attribute = 2,
    Text = 3,
    CDataSection = 4,
    EntityReference = 5,
    Entity = 6,
    ProcessingInstruction = 7,
    Comment = 8,
    Document = 9,
    DocumentType = 10,
    DocumentFragment = 11,
    Notation = 12,
}

#[derive(Clone)]
pub struct NodeList {
    _nodes: Vec<Box<Node>>,
}

impl NodeList {
    pub fn new() -> Self {
        Self { _nodes: vec![] }
    }

    pub fn length(&self) -> usize {
        self._nodes.len()
    }

    pub fn item(&self, index: usize) -> Option<&Box<Node>> {
        self._nodes.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Box<Node>> {
        self._nodes.iter()
    }
}

#[derive(Clone)]
pub struct Node {
    pub _node_type: NodeType,
    pub _node_name: DOMString,

    pub _base_uri: DOMString,

    /// The Document this Node belongs to
    /// Although ideally this would be a weak reference to avoid circular references,
    /// for simplicity we use an Option<Document> here.
    pub node_document: Option<Document>,

    _parent_node: Option<Box<Node>>,
    _child_nodes: NodeList,
}

trait INode {
    fn node_type(&self) -> u16;
    fn node_name(&self) -> &DOMString;
}

impl Node {
    pub fn base_uri(&self) -> String {
        let url = self.node_document.as_ref().unwrap().document_base_url();
        url.serialize()
    }

    pub fn parent_node(&self) -> Option<&Box<Node>> {
        self._parent_node.as_ref()
    }

    pub fn has_child_nodes(&self) -> bool {
        !self._child_nodes._nodes.is_empty()
    }

    pub fn child_nodes(&self) -> &NodeList {
        &self._child_nodes
    }

    pub fn first_child(&self) -> Option<&Box<Node>> {
        self._child_nodes.item(0)
    }

    pub fn last_child(&self) -> Option<&Box<Node>> {
        let len = self._child_nodes.length();
        if len == 0 {
            None
        } else {
            self._child_nodes.item(len - 1)
        }
    }
}

#[derive(Clone)]
pub struct CustomElementRegistry {
    pub is_scoped: bool,

    pub document_set: Vec<Document>,
}

pub struct Element {
    _node: Box<Node>,

    pub namespace: Option<DOMString>,
    pub namespace_prefix: Option<DOMString>,
    pub local_name: DOMString,

    // custom element registry
    pub custom_element_state: DOMString,
    // custom element definition
    // is value
}

impl Element {
    pub fn new(
        document: &Document,
        local_name: String,
        namespace: Option<String>,
        prefix: Option<String>,
        // is: Option<String>,
        synchronous_custom_elements: Option<bool>,
    ) -> Element {
        todo!()
    }
}

impl INode for Element {
    fn node_type(&self) -> u16 {
        NodeType::Element as u16
    }

    fn node_name(&self) -> &DOMString {
        // Placeholder implementation
        unimplemented!()
    }
}

#[derive(Clone)]
pub enum Origin {
    Opaque,
    Tuple(
        String,
        http::url::Host,
        Option<u16>,
        Option<http::url::Domain>,
    ),
}

#[derive(Clone)]
pub struct DOMImplementation {
    // Placeholder for DOMImplementation properties and methods
}

#[derive(Clone)]
pub struct Document {
    _node: Box<Node>,

    _encoding: &'static encoding_rs::Encoding,

    _content_type: &'static str,
    _url: http::url::URL,
    _origin: Origin,

    _type: &'static str,
    _mode: &'static str,

    _allow_declarative_shadow_roots: bool,

    _custom_element_registry: Option<CustomElementRegistry>,

    _implementation: DOMImplementation,
}

impl Default for Document {
    /// Creates a new Document with default values.
    /// According to the HTML5 specification:
    /// > Unless stated otherwise, a document’s encoding is the utf-8 encoding, content type is "application/xml", URL is "about:blank", origin is an opaque origin, type is "xml", mode is "no-quirks", allow declarative shadow roots is false, and custom element registry is null.
    fn default() -> Self {
        let mut document = Self {
            _node: Box::new(Node {
                _node_type: NodeType::Document,
                _node_name: "#document".to_string(),
                _base_uri: "".to_string(),
                node_document: None,
                _parent_node: None,
                _child_nodes: NodeList::new(),
            }),
            _encoding: encoding_rs::Encoding::for_label(b"utf-8").unwrap(),
            _content_type: "application/xml",
            _url: http::url::URL::pure_parse(String::from("about:blank")).unwrap(),
            _origin: Origin::Opaque,
            _type: "xml",
            _mode: "no-quirks",
            _allow_declarative_shadow_roots: false,
            _custom_element_registry: None,
            _implementation: DOMImplementation {},
        };

        document._node.node_document = Some(document.clone());
        document.ensure_maintains_integrity();

        document
    }
}

impl Document {
    /// Creates a new Document with the specified origin.
    ///
    /// NOTE: Ideally, the origin would be derived according to the spec
    /// However, for simplicity, we accept an Origin parameter.
    ///
    /// TODO: Implement according to spec:
    /// > ... set this’s origin to the origin of current global object’s associated Document.
    pub fn new(origin: Origin) -> Self {
        Self {
            _origin: origin,
            ..Self::default()
        }
    }

    fn ensure_maintains_integrity(&self) {
        assert!(matches!(self._type, "html" | "xml"));
        assert!(matches!(
            self._mode,
            "no-quirks" | "quirks" | "limited-quirks"
        ));
    }

    pub fn is_xml(&self) -> bool {
        self._type == "xml"
    }

    pub fn is_html(&self) -> bool {
        self._type == "html"
    }

    pub fn is_quirks_mode(&self) -> bool {
        self._mode == "quirks"
    }

    pub fn is_no_quirks_mode(&self) -> bool {
        self._mode == "no-quirks"
    }

    pub fn is_limited_quirks_mode(&self) -> bool {
        self._mode == "limited-quirks"
    }

    pub fn implementation(&self) -> &DOMImplementation {
        &self._implementation
    }

    pub fn url(&self) -> &http::url::URL {
        &self._url
    }

    pub fn document_uri(&self) -> &http::url::URL {
        &self._url
    }

    pub fn compat_mode(&self) -> &str {
        if self.is_quirks_mode() {
            "BackCompat"
        } else {
            "CSS1Compat"
        }
    }

    pub fn character_set(&self) -> &'static encoding_rs::Encoding {
        self._encoding
    }

    pub fn charset(&self) -> &'static encoding_rs::Encoding {
        self._encoding
    }

    pub fn input_encoding(&self) -> &'static encoding_rs::Encoding {
        self._encoding
    }

    pub fn content_type(&self) -> &str {
        self._content_type
    }

    /// TODO: Implement according to spec:
    /// The document base URL of a Document document is the URL record obtained by running these steps:
    /// 1. If document has no descendant base element that has an href attribute, then return document's fallback base URL.
    /// 2. Otherwise, return the frozen base URL of the first base element in document that has an href attribute, in tree order.
    ///
    /// NOTE: For simplicity, this implementation always returns the document's URL.
    pub fn document_base_url(&self) -> &http::url::URL {
        &self._url
    }

    pub fn doctype(&self) -> Option<&Box<Node>> {
        for child in self._node.child_nodes().iter() {
            if child._node_type == NodeType::DocumentType {
                return Some(child);
            }
        }
        None
    }
}
