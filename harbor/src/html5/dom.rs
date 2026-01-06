use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Weak;
use std::{cell::RefCell, rc::Rc};

use crate::{
    html5::{HTML_NAMESPACE, parse::Token, tag_groups::*},
    http::{self, url::Serializable},
};

type DOMString = String;
type USVString = String;

pub struct InsertLocation {
    parent: Rc<RefCell<NodeKind>>,
    pos: usize,
}

impl InsertLocation {
    pub fn new(parent: Rc<RefCell<NodeKind>>, pos: usize) -> Self {
        Self { parent, pos }
    }

    pub fn parent(&self) -> &Rc<RefCell<NodeKind>> {
        &self.parent
    }

    pub fn preceding(&self) -> Option<Rc<RefCell<NodeKind>>> {
        if self.pos == 0 {
            None
        } else {
            self.parent
                .borrow()
                .node()
                .borrow()
                .child_nodes()
                .item(self.pos - 1)
                .cloned()
        }
    }

    pub fn insert(&mut self, node: &mut NodeKind) {
        self.parent
            .borrow_mut()
            .node()
            .borrow_mut()
            .child_nodes_mut()
            .insert(self.pos, node);

        node.set_parent(Some(Rc::clone(&self.parent.borrow().node())));

        self.pos += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Node(Node),
    Element(Rc<RefCell<Element>>),
    Text(Text),
    Comment(Comment),
    DocumentType(DocumentType),
    Document(Document),
}

impl NodeKind {
    pub fn node(&self) -> Rc<RefCell<Node>> {
        match self {
            NodeKind::Node(n) => panic!("NodeKind::Node does not contain a Rc<RefCell<Node>>"),
            NodeKind::Element(e) => Rc::clone(&e.borrow()._node),
            NodeKind::Text(t) => Rc::clone(&t._character_data._node),
            NodeKind::Comment(c) => Rc::clone(&c._character_data._node),
            NodeKind::DocumentType(dt) => Rc::clone(&dt._node),
            NodeKind::Document(d) => Rc::clone(&d._node),
        }
    }

    // pub fn node_mut(&mut self) -> &mut Node {
    //     match self {
    //         NodeKind::Node(n) => n,
    //         NodeKind::Element(e) => &mut e._node,
    //         NodeKind::Text(t) => &mut t._character_data._node,
    //         NodeKind::Comment(c) => &mut c._character_data._node,
    //         NodeKind::DocumentType(dt) => &mut dt._node,
    //         NodeKind::Document(_) => panic!("MUTABLE FUCK"),
    //     }
    // }

    pub fn set_parent(&mut self, parent: Option<Rc<RefCell<Node>>>) {
        let self_node = self.node();
        let mut node = self_node.borrow_mut();

        if parent.is_none() {
            node._parent_node = None;
            return;
        }

        node._parent_node = Some(Rc::downgrade(&parent.unwrap()));
    }

    pub fn custom_element_registry(&self) -> Option<CustomElementRegistry> {
        match self {
            NodeKind::Document(d) => d._custom_element_registry.as_ref().cloned(),
            NodeKind::Element(e) => e.borrow().custom_element_registry.as_ref().cloned(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeList {
    _nodes: Vec<Rc<RefCell<NodeKind>>>,
}

impl NodeList {
    pub fn new() -> Self {
        Self { _nodes: vec![] }
    }

    pub fn length(&self) -> usize {
        self._nodes.len()
    }

    pub fn item(&self, index: usize) -> Option<&Rc<RefCell<NodeKind>>> {
        self._nodes.get(index)
    }

    pub fn item_mut(&mut self, index: usize) -> Option<&mut Rc<RefCell<NodeKind>>> {
        self._nodes.get_mut(index)
    }

    pub fn insert(&mut self, index: usize, node: &NodeKind) {
        self._nodes
            .insert(index, Rc::new(RefCell::new(node.clone())));
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Rc<RefCell<NodeKind>>> {
        self._nodes.iter()
    }

    pub fn push(&mut self, node: &Rc<RefCell<NodeKind>>) {
        self._nodes.push(Rc::clone(node));
    }

    pub fn pop(&mut self) -> Option<Rc<RefCell<NodeKind>>> {
        self._nodes.pop()
    }

    pub fn map<F, T>(&self, mut f: F) -> Vec<T>
    where
        F: FnMut(&Rc<RefCell<NodeKind>>) -> T,
    {
        self._nodes.iter().map(|n| f(n)).collect()
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
    pub node_document: Option<Weak<RefCell<Document>>>,

    _parent_node: Option<Weak<RefCell<Node>>>,
    _child_nodes: NodeList,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self._node_type == other._node_type
            && self._node_name == other._node_name
            && self._base_uri == other._base_uri
            && self._child_nodes == other._child_nodes
    }
}

impl Eq for Node {}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("_node_type", &self._node_type)
            .field("node_document", &self.node_document)
            // .field("_node_name", &self._node_name)
            // .field("_base_uri", &self._base_uri)
            .field("_parent_node", &self._parent_node)
            .field("_child_nodes", &self._child_nodes)
            .finish()?;

        Ok(())
    }
}

trait INode {
    fn new() -> Self
    where
        Self: Sized;

    fn node_type(&self) -> u16;
    fn node_name(&self) -> DOMString;
}

impl Node {
    pub fn base_uri(&self) -> String {
        if let Some(document) = &self.node_document
            && let Some(url) = document.upgrade()
        {
            return url.borrow().document_base_url().serialize();
        }
        self._base_uri.clone()
    }

    pub fn parent_node(&self) -> Option<&Weak<RefCell<Node>>> {
        self._parent_node.as_ref()
    }

    pub fn parent_node_mut(&mut self) -> Option<&mut Weak<RefCell<Node>>> {
        self._parent_node.as_mut()
    }

    pub fn has_child_nodes(&self) -> bool {
        !self._child_nodes._nodes.is_empty()
    }

    pub fn child_nodes(&self) -> &NodeList {
        &self._child_nodes
    }

    pub fn child_nodes_mut(&mut self) -> &mut NodeList {
        &mut self._child_nodes
    }

    pub fn first_child(&self) -> Option<&Rc<RefCell<NodeKind>>> {
        self._child_nodes.item(0)
    }

    pub fn last_child(&self) -> Option<&Rc<RefCell<NodeKind>>> {
        let len = self._child_nodes.length();
        if len == 0 {
            None
        } else {
            self._child_nodes.item(len - 1)
        }
    }

    pub fn append_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<NodeKind>>) {
        {
            let mut child_borrow = child.borrow_mut();
            child_borrow.set_parent(Some(Rc::clone(parent)));
        }

        let mut parent_borrow = parent.borrow_mut();
        parent_borrow._child_nodes._nodes.push(child);
    }

    pub fn position_of_child(&self, child: &NodeKind) -> Option<usize> {
        self._child_nodes
            ._nodes
            .iter()
            .position(|n| n.borrow().deref() == child)
    }

    pub fn remove_child(&mut self, child: &NodeKind) {
        if let Some(pos) = self
            ._child_nodes
            ._nodes
            .iter()
            .position(|n| n.borrow().deref() == child)
        {
            {
                let mut child = self._child_nodes._nodes[pos].borrow_mut();
                child.set_parent(None);
            }

            self._child_nodes._nodes.remove(pos);
        }
    }

    pub fn pop_child(&mut self, nth: Option<usize>) -> Option<Rc<RefCell<NodeKind>>> {
        if let Some(index) = nth {
            if index < self._child_nodes.length() {
                {
                    let mut child = self._child_nodes._nodes[index].borrow_mut();
                    child.set_parent(None);
                }

                Some(self._child_nodes._nodes.remove(index))
            } else {
                None
            }
        } else {
            self._child_nodes._nodes.pop()
        }
    }

    pub fn node_document(&self) -> Option<Document> {
        if let Some(weak_doc) = &self.node_document {
            if let Some(strong_doc) = weak_doc.upgrade() {
                return Some(strong_doc.borrow().clone());
            }
        }
        None
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CharacterData {
    _node: Rc<RefCell<Node>>,

    pub data: DOMString,
}

impl Debug for CharacterData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharacterData")
            .field("data", &self.data)
            .finish()
    }
}

impl CharacterData {
    pub fn length(&self) -> usize {
        self.data.len()
    }

    pub fn substring_data(&self, offset: usize, count: usize) -> DOMString {
        let end = std::cmp::min(offset + count, self.data.len());
        self.data[offset..end].to_string()
    }

    pub fn append_ch(&mut self, ch: char) {
        self.data.push(ch);
    }

    pub fn append_data(&mut self, data: &str) {
        self.data.push_str(data);
    }

    pub fn insert_data(&mut self, offset: usize, data: &str) {
        if offset <= self.data.len() {
            self.data.insert_str(offset, data);
        }
    }

    pub fn delete_data(&mut self, offset: usize, count: usize) {
        let end = std::cmp::min(offset + count, self.data.len());
        if offset < end {
            self.data.replace_range(offset..end, "");
        }
    }

    pub fn replace_data(&mut self, offset: usize, count: usize, data: &str) {
        let end = std::cmp::min(offset + count, self.data.len());
        if offset < end {
            self.data.replace_range(offset..end, data);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    _character_data: CharacterData,
}

impl Text {
    pub fn new(data: &str, _document: Rc<RefCell<Document>>) -> Self {
        let document = _document.borrow();

        Self {
            _character_data: CharacterData {
                _node: Rc::new(RefCell::new(Node {
                    _node_type: NodeType::Text,
                    _node_name: "#text".to_string(),
                    _base_uri: document.document_base_url().serialize(),
                    node_document: Some(Rc::downgrade(&_document)),
                    _parent_node: None,
                    _child_nodes: NodeList::new(),
                })),
                data: data.to_string(),
            },
        }
    }

    pub fn split_text(&mut self, offset: usize) -> Text {
        let original_data = &self._character_data.data;
        let split_data = original_data[offset..].to_string();
        self._character_data.data = original_data[..offset].to_string();

        Text::new(
            &split_data,
            self._character_data
                ._node
                .borrow()
                .node_document
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap(),
        )
    }

    pub fn push(&mut self, ch: char) {
        self._character_data.append_ch(ch);
    }
}

impl INode for Text {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new("", Rc::new(RefCell::new(Document::default())))
    }

    fn node_type(&self) -> u16 {
        NodeType::Text as u16
    }

    fn node_name(&self) -> DOMString {
        DOMString::from("#text")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    _character_data: CharacterData,
}

impl Comment {
    pub fn new(data: &str, _document: Rc<RefCell<Document>>) -> Self {
        let document = _document.borrow();

        Self {
            _character_data: CharacterData {
                _node: Rc::new(RefCell::new(Node {
                    _node_type: NodeType::Comment,
                    _node_name: "#comment".to_string(),
                    _base_uri: document.document_base_url().serialize(),
                    node_document: Some(Rc::downgrade(&_document)),
                    _parent_node: None,
                    _child_nodes: NodeList::new(),
                })),
                data: data.to_string(),
            },
        }
    }
}

impl INode for Comment {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new("", Rc::new(RefCell::new(Document::default())))
    }

    fn node_type(&self) -> u16 {
        NodeType::Comment as u16
    }

    fn node_name(&self) -> DOMString {
        DOMString::from("#comment")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentType {
    _node: Rc<RefCell<Node>>,

    _name: DOMString,
    _public_id: DOMString,
    _system_id: DOMString,
}

impl INode for DocumentType {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new("", Some(Rc::new(RefCell::new(Document::default()))))
    }

    fn node_type(&self) -> u16 {
        NodeType::DocumentType as u16
    }

    fn node_name(&self) -> DOMString {
        self._name.clone()
    }
}

impl DocumentType {
    pub fn new(name: &str, _document: Option<Rc<RefCell<Document>>>) -> Self {
        if _document.is_none() {
            Self {
                _node: Rc::new(RefCell::new(Node {
                    _node_type: NodeType::DocumentType,
                    _node_name: name.to_string(),
                    _base_uri: "".to_string(),
                    node_document: None,
                    _parent_node: None,
                    _child_nodes: NodeList::new(),
                })),
                _name: name.to_string(),
                _public_id: String::new(),
                _system_id: String::new(),
            }
        } else {
            let base_uri = _document
                .as_ref()
                .unwrap()
                .borrow()
                .document_base_url()
                .serialize();

            Self {
                _node: Rc::new(RefCell::new(Node {
                    _node_type: NodeType::DocumentType,
                    _node_name: name.to_string(),
                    _base_uri: base_uri,
                    node_document: Some(Rc::downgrade(&_document.unwrap())),
                    _parent_node: None,
                    _child_nodes: NodeList::new(),
                })),
                _name: name.to_string(),
                _public_id: String::new(),
                _system_id: String::new(),
            }
        }
    }

    pub fn with_public_id(mut self, public_id: &str) -> Self {
        self._public_id = public_id.to_string();
        self
    }

    pub fn with_system_id(mut self, system_id: &str) -> Self {
        self._system_id = system_id.to_string();
        self
    }

    pub fn name(&self) -> &str {
        &self._name
    }

    pub fn public_id(&self) -> &str {
        &self._public_id
    }

    pub fn system_id(&self) -> &str {
        &self._system_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomElementDefinition {
    // Placeholder for custom element definition properties
    name: String,
    local_name: String,
    // TODO: Add more fields as per spec:
    // https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element-definition
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomElementRegistry {
    pub is_scoped: bool,

    pub scoped_document_set: Vec<Document>,
    pub definitions: Vec<CustomElementDefinition>,
}

impl CustomElementRegistry {
    pub fn new(is_scoped: bool) -> Self {
        Self {
            is_scoped,
            scoped_document_set: vec![],
            definitions: vec![],
        }
    }
}

fn lookup_definition(
    _registry: &Option<CustomElementRegistry>,
    _local_name: &str,
    _namespace: &Option<String>,
    _is: &Option<String>,
) -> Option<CustomElementDefinition> {
    if _registry.is_none() {
        return None;
    }

    if _namespace.is_none()
        || _namespace
            .as_ref()
            .is_some_and(|n| n != "http://www.w3.org/1999/xhtml")
    {
        return None;
    }

    let registry = _registry.as_ref().unwrap();
    for _definition in &registry.definitions {
        // Placeholder for matching logic
        // if definition.matches(local_name) {
        //     return Some(definition);
        // }
    }

    for _definition in &registry.definitions {
        // Placeholder for matching logic with 'is' attribute
        // if let Some(is_value) = is {
        //     if definition.matches_is(is_value, local_name) {
        //         return Some(definition);
        //     }
        // }
    }

    return None;
}

pub enum CustomElementRegistryOrDefault {
    Default,
    Registry(CustomElementRegistry),
}

#[derive(Debug, Clone)]
pub struct Attr {
    _node: Box<Node>,

    _namespace: Option<DOMString>,
    _namespace_prefix: Option<DOMString>,
    _local_name: DOMString,
    _value: DOMString,

    _element: Option<Weak<RefCell<Element>>>,
}

impl PartialEq for Attr {
    fn eq(&self, other: &Self) -> bool {
        self._namespace == other._namespace
            && self._namespace_prefix == other._namespace_prefix
            && self._local_name == other._local_name
            && self._value == other._value
    }
}

impl Eq for Attr {}

impl Attr {
    pub fn new(
        namespace: Option<String>,
        prefix: Option<String>,
        local_name: String,
        value: String,
        element: Option<Weak<RefCell<Element>>>,
        _document: Rc<RefCell<Document>>,
    ) -> Self {
        let document = _document.borrow();

        Self {
            _node: Box::new(Node {
                _node_type: NodeType::Attribute,
                _node_name: local_name.clone(),
                _base_uri: document.document_base_url().serialize(),
                node_document: Some(Rc::downgrade(&_document)),
                _parent_node: None,
                _child_nodes: NodeList::new(),
            }),
            _namespace: namespace,
            _namespace_prefix: prefix,
            _local_name: local_name,
            _value: value,
            _element: element,
        }
    }

    pub fn namespace_uri(&self) -> Option<&str> {
        self._namespace.as_deref()
    }

    pub fn prefix(&self) -> Option<&str> {
        self._namespace_prefix.as_deref()
    }

    pub fn local_name(&self) -> &str {
        &self._local_name
    }

    pub fn value(&self) -> &str {
        &self._value
    }

    /// Sets the value of the attribute.
    /// The specification regarding this is SHIT, so I'm ignoring it
    pub fn set_value(&mut self, value: &str) {
        self._value = value.to_string();
    }

    pub fn owner_element(&self) -> &Option<Weak<RefCell<Element>>> {
        &self._element
    }

    pub fn specified(&self) -> bool {
        true
    }
}

pub type ElementID = String;

pub enum ElementKind {
    Element,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Element {
    _node: Rc<RefCell<Node>>,

    pub id: ElementID,

    pub namespace: Option<DOMString>,
    pub namespace_prefix: Option<DOMString>,
    pub local_name: DOMString,

    pub custom_element_registry: Option<CustomElementRegistry>,
    pub custom_element_state: DOMString,
    // custom element definition
    pub is_value: Option<DOMString>,

    attribute_list: Vec<Attr>,

    _token: Option<Token>,
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("local_name", &self.local_name)
            .field("attribute_list", &self.attribute_list)
            .field("_node", &self._node)
            .field("_token", &self._token)
            .finish()
    }
}

impl Element {
    pub fn new(
        _document: Rc<RefCell<Document>>,
        local_name: String,
        namespace: Option<String>,
        prefix: Option<String>,
        is: Option<String>,
        _synchronous_custom_elements: Option<bool>,
        _registry: Option<CustomElementRegistryOrDefault>,
    ) -> Element {
        let document = _document.borrow();

        let registry = match _registry {
            Some(CustomElementRegistryOrDefault::Registry(reg)) => Some(reg),
            Some(CustomElementRegistryOrDefault::Default) => {
                document._custom_element_registry.clone()
            }
            None => None,
        };

        let definition = lookup_definition(&registry, &local_name, &namespace, &is);

        if definition
            .as_ref()
            .is_some_and(|def| def.name != local_name)
        {
            let result = Element::create_element_internal::<Element>(
                Rc::clone(&_document),
                namespace,
                prefix,
                &local_name,
                "custom".to_string(),
                &is,
                registry.as_ref(),
            );

            // if synchronous_custom_elements.unwrap_or(false) {
            //     todo!("upgrade the element to a custom element immediately");
            // } else {
            //     todo!("enqueue the element for upgrade");
            // }

            return result;
        } else if definition.is_some() {
            todo!("handle the case where the definition exists");
        } else {
            return Element::create_element_internal::<Element>(
                Rc::clone(&_document),
                namespace,
                prefix,
                &local_name,
                "uncustomized".to_string(),
                &is,
                registry.as_ref(),
            );
        }
    }

    pub fn token(&self) -> Option<&Token> {
        self._token.as_ref()
    }

    pub fn with_token(mut self, token: Token) -> Self {
        self._token = Some(token);
        self
    }

    pub fn is_special(&self) -> bool {
        SPECIAL_CATEGORY_NAMES.contains(&self.local_name.as_str())
    }

    pub fn is_special_excluding(&self, excluding: &[&str]) -> bool {
        SPECIAL_CATEGORY_NAMES
            .iter()
            .filter(|name| !excluding.contains(name))
            .any(|name| *name == self.local_name.as_str())
    }

    pub fn from_token(
        token: &Token,
        namespace: &str,
        intended_parent: &NodeKind,
    ) -> Rc<RefCell<Element>> {
        let parent_node = intended_parent.node();
        let intended_parent_node = parent_node.borrow();

        let tag = match &token {
            Token::StartTag(t) => t,
            Token::EndTag(t) => t,
            _ => panic!("Token must be a StartTag or EndTag, got: {:?}", token),
        };

        let document = intended_parent_node
            .node_document
            .as_ref()
            .expect("Intended parent must have a document")
            .upgrade()
            .expect("Document must be valid");

        let local_name = tag.name.clone();

        let registry = intended_parent.custom_element_registry();
        let defintion =
            lookup_definition(&registry, &local_name, &Some(namespace.to_string()), &None);

        let will_execute_script = defintion.is_some();

        if will_execute_script {
            todo!("handle custom element creation with script execution");
        }

        let element = Rc::new(RefCell::new(
            Element::new(
                Rc::clone(&document),
                local_name,
                Some(namespace.to_string()),
                None,
                None,
                Some(will_execute_script),
                registry.map(|r| CustomElementRegistryOrDefault::Registry(r.clone())),
            )
            .with_token(token.clone()),
        ));

        let element_weak = Rc::downgrade(&element);

        {
            let mut element_mut = element.borrow_mut();

            for (name, value) in tag.attributes.iter() {
                element_mut.attribute_list.push(Attr::new(
                    None,
                    None,
                    name.clone(),
                    value.clone(),
                    Some(Weak::clone(&element_weak)),
                    Rc::clone(&document),
                ));
            }
        }

        // TODO: other random bullshit

        element
    }

    fn create_element_internal<T: IElement>(
        _document: Rc<RefCell<Document>>,
        namespace: Option<String>,
        prefix: Option<String>,
        local_name: &String,
        state: String,
        is: &Option<String>,
        _registry: Option<&CustomElementRegistry>,
    ) -> T {
        let element = T::empty()
            .with_namespace(&namespace)
            .with_prefix(&prefix)
            .with_local_name(local_name)
            .with_custom_element_state(&state)
            .with_is_value(is)
            .with_custom_element_registry(&_registry.cloned());

        element.node().borrow_mut().node_document = Some(Rc::downgrade(&_document));

        // assert!(element attribute list is empty)

        element
    }

    pub fn qualified_name(&self) -> DOMString {
        match &self.namespace_prefix {
            Some(prefix) => format!("{}:{}", prefix, self.local_name),
            None => self.local_name.clone(),
        }
    }

    fn uppercase_qualified_name(&self) -> DOMString {
        let qualified_name = self.qualified_name();

        if self
            .namespace
            .as_ref()
            .is_some_and(|n| n.as_str() == HTML_NAMESPACE)
            && self
                .node()
                .borrow()
                .node_document()
                .as_ref()
                .unwrap()
                .is_html()
        {
            qualified_name.to_uppercase()
        } else {
            qualified_name
        }
    }

    pub fn attributes(&self) -> &Vec<Attr> {
        &self.attribute_list
    }

    pub fn push_attr_raw_rc(element: &Rc<RefCell<Element>>, name: &str, value: &str) {
        let attr = Attr::new(
            None,
            None,
            name.to_string(),
            value.to_string(),
            Some(Rc::downgrade(element)),
            Rc::clone(
                &element
                    .borrow()
                    .node()
                    .borrow()
                    .node_document
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap(),
            ),
        );

        element.borrow_mut().attribute_list.push(attr);
    }

    // pub fn push_attr_raw(&mut self, name: &str, value: &str) {
    //     let attr = Attr::new(
    //         None,
    //         None,
    //         name.to_string(),
    //         value.to_string(),
    //         Some(self),
    //         Rc::clone(
    //             &self
    //                 .node()
    //                 .borrow()
    //                 .node_document
    //                 .as_ref()
    //                 .unwrap()
    //                 .upgrade()
    //                 .unwrap(),
    //         ),
    //     );

    //     self.attribute_list.push(attr);
    // }

    pub fn push_attribute(&mut self, attr: Attr) {
        self.attribute_list.push(attr);
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        for attr in &self.attribute_list {
            if attr.local_name() == name {
                return Some(attr.value());
            }
        }
        None
    }

    pub fn namespace_uri(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
}

impl INode for Element {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            _node: Rc::new(RefCell::new(Node {
                _node_type: NodeType::Element,
                _node_name: "".to_string(),
                _base_uri: "".to_string(),
                node_document: None,
                _parent_node: None,
                _child_nodes: NodeList::new(),
            })),
            id: "".to_string(),
            namespace: None,
            namespace_prefix: None,
            local_name: "".to_string(),
            custom_element_state: "".to_string(),
            custom_element_registry: None,
            is_value: None,
            attribute_list: vec![],
            _token: None,
        }
    }

    fn node_type(&self) -> u16 {
        NodeType::Element as u16
    }

    fn node_name(&self) -> DOMString {
        // Placeholder implementation
        unimplemented!()
    }
}

pub trait IElement {
    fn empty() -> Self
    where
        Self: Sized;

    fn with_namespace(self, namespace: &Option<String>) -> Self
    where
        Self: Sized;

    fn with_prefix(self, prefix: &Option<String>) -> Self
    where
        Self: Sized;

    fn with_local_name(self, local_name: &str) -> Self
    where
        Self: Sized;

    fn with_custom_element_state(self, state: &str) -> Self
    where
        Self: Sized;

    fn with_custom_element_registry(self, registry: &Option<CustomElementRegistry>) -> Self
    where
        Self: Sized;

    fn with_is_value(self, is: &Option<String>) -> Self
    where
        Self: Sized;

    fn with_id(self, id: &str) -> Self
    where
        Self: Sized;

    fn node(&self) -> &Rc<RefCell<Node>>;
}

impl IElement for Element {
    fn empty() -> Self
    where
        Self: Sized,
    {
        Self {
            _node: Rc::new(RefCell::new(Node {
                _node_type: NodeType::Element,
                _node_name: "".to_string(),
                _base_uri: "".to_string(),
                node_document: None,
                _parent_node: None,
                _child_nodes: NodeList::new(),
            })),
            id: "".to_string(),
            namespace: None,
            namespace_prefix: None,
            local_name: "".to_string(),
            custom_element_state: "".to_string(),
            custom_element_registry: None,
            is_value: None,
            attribute_list: vec![],
            _token: None,
        }
    }

    fn with_namespace(mut self, namespace: &Option<String>) -> Self
    where
        Self: Sized,
    {
        self.namespace = namespace.clone();
        self
    }

    fn with_prefix(mut self, prefix: &Option<String>) -> Self
    where
        Self: Sized,
    {
        self.namespace_prefix = prefix.clone();
        self
    }

    fn with_local_name(mut self, local_name: &str) -> Self
    where
        Self: Sized,
    {
        self.local_name = local_name.to_string();
        self
    }

    fn with_custom_element_state(mut self, state: &str) -> Self
    where
        Self: Sized,
    {
        self.custom_element_state = state.to_string();
        self
    }

    fn with_custom_element_registry(mut self, registry: &Option<CustomElementRegistry>) -> Self
    where
        Self: Sized,
    {
        self.custom_element_registry = registry.clone();
        self
    }

    fn with_is_value(mut self, is: &Option<String>) -> Self
    where
        Self: Sized,
    {
        self.is_value = is.clone();
        self
    }

    fn with_id(mut self, id: &str) -> Self
    where
        Self: Sized,
    {
        self.id = id.to_string();
        self
    }

    // fn node_mut(&mut self) -> &mut Node {
    //     &mut self._node
    // }

    fn node(&self) -> &Rc<RefCell<Node>> {
        &self._node
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Opaque,
    Tuple(
        String,
        http::url::Host,
        Option<u16>,
        Option<http::url::Domain>,
    ),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DOMImplementation {
    // Placeholder for DOMImplementation properties and methods
}

#[derive(Clone, PartialEq, Eq)]
pub struct Document {
    pub _node: Rc<RefCell<Node>>,

    _encoding: &'static encoding_rs::Encoding,

    _content_type: &'static str,
    _url: http::url::URL,
    _origin: Origin,

    _type: &'static str,
    _mode: &'static str,

    _allow_declarative_shadow_roots: bool,

    _custom_element_registry: Option<CustomElementRegistry>,

    _implementation: DOMImplementation,

    parser_cannot_change_mode: bool,
}

impl Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document")
            .field("_node", &self._node)
            .finish()
    }
}

impl Default for Document {
    /// Creates a new Document with default values.
    /// According to the HTML5 specification:
    /// > Unless stated otherwise, a document’s encoding is the utf-8 encoding, content type is "application/xml", URL is "about:blank", origin is an opaque origin, type is "xml", mode is "no-quirks", allow declarative shadow roots is false, and custom element registry is null.
    fn default() -> Self {
        let document = Self {
            _node: Rc::new(RefCell::new(Node {
                _node_type: NodeType::Document,
                _node_name: "#document".to_string(),
                _base_uri: "".to_string(),
                node_document: None,
                _parent_node: None,
                _child_nodes: NodeList::new(),
            })),
            _encoding: encoding_rs::Encoding::for_label(b"utf-8").unwrap(),

            _content_type: "application/xml",
            _url: http::url::URL::pure_parse(String::from("about:blank")).unwrap(),
            _origin: Origin::Opaque,

            _type: "xml",
            _mode: "no-quirks",

            _allow_declarative_shadow_roots: false,

            _custom_element_registry: None,

            _implementation: DOMImplementation {},

            parser_cannot_change_mode: false,
        };

        document._node.borrow_mut().node_document =
            Some(Rc::downgrade(&Rc::new(RefCell::new(document.clone()))));
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
    pub fn new(origin: Origin) -> Rc<RefCell<Self>> {
        let doc = Rc::new(RefCell::new(Self {
            _origin: origin,
            ..Self::default()
        }));

        doc.borrow_mut()._node.borrow_mut().node_document = Some(Rc::downgrade(&doc));
        doc
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

    pub fn doctype(&self) -> Option<Rc<RefCell<NodeKind>>> {
        for child in self._node.borrow().child_nodes().iter() {
            if let NodeKind::DocumentType(_) = child.borrow().deref() {
                return Some(Rc::clone(child));
            }
        }
        None
    }
}
