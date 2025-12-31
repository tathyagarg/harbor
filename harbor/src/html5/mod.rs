#![allow(dead_code)]
/// Custom implementation of the HTML5 spec:
/// https://html.spec.whatwg.org/
pub mod tokenize;

/// Although USVString and String aren't identical, I am using this alias for the time being and
/// may change it later.
type USVString = String;

type DOMString = String;

/// Placeholder
type ValueOfType = i32;

#[derive(Clone)]
enum DocumentReadyState {
    Loading,
    Interactive,
    Complete,
}

#[derive(Clone)]
pub struct Document {
    // : Node
    node: Box<Node>,

    /// [PutForwards=href, LegacyUnforgeable] readonly attribute Location? location
    location: Option<Location>,

    /// attribute USVString domain
    domain: USVString,

    /// readonly attribute USVString referrer;
    referrer_ro: USVString,

    /// attribute USVString cookie;
    cookie: USVString,

    /// readonly attribute DOMString lastModified;
    last_modified_ro: DOMString,

    /// readonly attribute DocumentReadyState readyState;
    ready_state_ro: DocumentReadyState,
    // TODO: Rest of the goddamn attributes
}

trait IDocument {}

/// [Exposed=Window]
#[derive(Default, Clone)]
pub struct Location {
    /// [LegacyUnforgeable] stringifier attribute USVString href;
    href: USVString,

    /// [LegacyUnforgeable] readonly attribute USVString origin;
    origin_ro: USVString,

    /// [LegacyUnforgeable] attribute USVString protocol;
    protocol: USVString,

    /// [LegacyUnforgeable] attribute USVString host;
    /// Returns the Location object's URL's host and port (if different from the default port for the scheme).
    host: USVString,

    /// [LegacyUnforgeable] attribute USVString hostname;
    hostname: USVString,

    /// [LegacyUnforgeable] attribute USVString port;
    port: USVString,

    /// [LegacyUnforgeable] attribute USVString pathname;
    pathname: USVString,

    /// [LegacyUnforgeable] attribute USVString search;
    search: USVString,

    /// [LegacyUnforgeable] attribute USVString hash;
    hash: USVString,

    /// [LegacyUnforgeable, SameObject] readonly attribute DOMStringList ancestorOrigins;
    ancestor_origins: DOMStringList,

    /// [[DefineOwnProperty]]("valueOf", { [[Value]]: valueOf, [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false })
    value_of: ValueOfType,
}

// TODO: Implement Location functions:
// https://html.spec.whatwg.org/#the-location-interface
impl Location {
    pub fn new() -> Self {
        Self::default()
    }

    /// [LegacyUnforgeable] undefined assign(USVString url);
    pub fn assign(&mut self, url: USVString) -> () {}

    /// [LegacyUnforgeable] undefined replace(USVString url);
    pub fn replace(&mut self, url: USVString) -> () {}

    /// [LegacyUnforgeable] undefined reload();
    pub fn reload(&mut self) -> () {}
}

/// [Exposed=(Window,Worker)]
#[derive(Default, Clone)]
pub struct DOMStringList {
    list: Vec<DOMString>,
}

impl DOMStringList {
    /// readonly attribute unsigned long length;
    pub fn length(&self) -> u32 {
        self.list.len() as u32
    }

    /// getter DOMString? item(unsigned long index);
    fn item(&self, index: u32) -> Option<DOMString> {
        if index + 1 > self.length() {
            return None;
        }

        let elem = self.list.iter().nth(index as usize).unwrap().to_owned();
        Some(elem)
    }

    /// boolean contains(DOMString string);
    fn contains(&self, string: DOMString) -> bool {
        self.list.contains(&string)
    }
}

pub struct EventTarget {
    listeners: Vec<i32>,
}

pub trait IEventTarget {
    /// constructor();
    fn new() -> Self;

    /// undefined addEventListener(DOMString type, EventListener? callback, optional (AddEventListenerOptions or boolean) options = {});
    fn add_event_listener(
        &mut self,
        type_: DOMString,
        callback: Option<EventListener>,
        options: Option<AddEventListenerOptions>,
    ) -> ();

    /// undefined removeEventListener(DOMString type, EventListener? callback, optional (EventListenerOptions or boolean) options = {});
    fn remove_event_listener(
        &mut self,
        type_: DOMString,
        callback: Option<EventListener>,
        options: Option<EventListenerOptions>,
    ) -> ();

    /// boolean dispatchEvent(Event event);
    fn dispatch_event(&self, event: Event) -> bool;
}

/// TODO
impl IEventTarget for EventTarget {
    fn new() -> Self {
        todo!()
    }

    fn add_event_listener(
        &mut self,
        type_: DOMString,
        callback: Option<EventListener>,
        options: Option<AddEventListenerOptions>,
    ) -> () {
        todo!()
    }

    fn remove_event_listener(
        &mut self,
        type_: DOMString,
        callback: Option<EventListener>,
        options: Option<EventListenerOptions>,
    ) -> () {
        todo!()
    }

    fn dispatch_event(&self, event: Event) -> bool {
        todo!()
    }
}

pub struct EventListenerOptions {
    /// boolean capture = false;
    capture: bool,
}

pub struct AddEventListenerOptions {
    /// : EventListenerOptions
    event_listener_options: EventListenerOptions,

    /// boolean passive;
    passive: bool,

    /// boolean once = false;
    once: bool,

    /// AbortSignal signal;
    signal: AbortSignal,
}

/// TODO: Implement Abort signal
pub struct AbortSignal {}

pub struct EventListener {}

pub trait IEventListener {
    /// undefined handleEvent(Event event);
    fn handle_event(&self, event: Event) -> ();
}

/// TODO
pub struct Event {}

pub trait IEvent {}

#[derive(Clone)]
pub enum NodeType {
    ElementNode = 1,
    AttributeNode = 2,
    TextNode = 3,
    CDataSectionNode = 4,
    EntityReferenceNode = 5, // legacy
    EntityNode = 6,          // legacy
    ProcessingInstructionNode = 7,
    CommentNode = 8,
    DocumentNode = 9,
    DocumentTypeNode = 10,
    DocumentFragmentNode = 11,
    NotationNode = 12, // legacy
}

pub enum DocumentPosition {
    Disconnected = 0x01,
    Preceding = 0x02,
    Following = 0x04,
    Contains = 0x08,
    ContainedBy = 0x10,
    ImplementationSpecific = 0x20,
}

#[derive(Clone)]
pub struct Node {
    /// readonly attribute unsigned short nodeType;
    node_type_ro: NodeType,

    /// readonly attribute DOMString nodeName;
    node_name_ro: DOMString,

    /// readonly attribute boolean isConnected;
    is_connected_ro: bool,

    /// readonly attribute Document? ownerDocument;
    owner_document_ro: Option<Document>,

    /// readonly attribute Node? parentNode;
    parent_node_ro: Box<Option<Node>>,

    /// readonly attribute Element? parentElement;
    /// TODO

    /// [SameObject] readonly attribute NodeList childNodes;
    child_nodes_ro: NodeList,

    /// readonly attribute Node? firstChild;
    first_child_ro: Box<Option<Node>>,

    /// readonly attribute Node? lastChild;
    last_child_ro: Box<Option<Node>>,

    /// readonly attribute Node? previousSibling;
    previous_sibling_ro: Box<Option<Node>>,

    /// readonly attribute Node? nextSibling;
    next_sibling_ro: Box<Option<Node>>,

    /// [CEReactions] attribute DOMString? nodeValue;
    node_value: Option<DOMString>,

    /// [CEReactions] attribute DOMString? textContent;
    text_content: Option<DOMString>,
}

struct GetRootNodeOptions {
    composed: bool,
}

trait INode {
    /// Node getRootNode(optional GetRootNodeOptions options = {});
    fn get_root_node(&self, options: Option<GetRootNodeOptions>) -> Node;

    /// boolean hasChildNodes();
    fn has_child_nodes(&self) -> bool;

    /// [CEReactions] undefined normalize();
    fn normalize(&mut self) -> ();

    /// [CEReactions, NewObject] Node cloneNode(optional boolean subtree = false);
    fn clone_node(&self, subtree: Option<bool>) -> Node;

    /// boolean isEqualNode(Node? otherNode);
    fn is_equal_node(&self, other_node: Option<&Node>) -> bool;

    /// boolean isSameNode(Node? otherNode);
    fn is_same_node(&self, other_node: Option<&Node>) -> bool;

    /// unsigned short compareDocumentPosition(Node other);
    fn compare_document_position(&self, other: &Node) -> DocumentPosition;

    /// boolean contains(Node? other);
    fn contains(&self, other: Option<&Node>) -> bool;

    /// DOMString? lookupPrefix(DOMString? namespace);
    fn lookup_prefix(&self, namespace: Option<DOMString>) -> Option<DOMString>;

    /// DOMString? lookupNamespaceURI(DOMString? prefix);
    fn lookup_namespace_uri(&self, prefix: Option<DOMString>) -> Option<DOMString>;

    /// boolean isDefaultNamespace(DOMString? namespace);
    fn is_default_namespace(&self, namespace: Option<DOMString>) -> bool;

    /// [CEReactions] Node insertBefore(Node node, Node? child);
    fn insert_before(&mut self, node: &Node, child: Option<&Node>) -> Node;

    /// [CEReactions] Node appendChild(Node node);
    fn append_child(&mut self, node: &Node) -> Node;

    /// [CEReactions] Node replaceChild(Node node, Node child);
    fn replace_child(&mut self, node: &Node, child: &Node) -> Node;

    /// [CEReactions] Node removeChild(Node child);
    fn remove_child(&mut self, child: &Node) -> Node;

    /// readonly attribute USVString baseURI;
    fn base_uri(&self) -> USVString;
}

// impl INode for Node {
//     fn base_uri(&self) -> USVString {
//         self.owner_document_ro.unwrap()
//     }
// }

/// [Exposed=Window]
#[derive(Clone)]
struct NodeList {
    iterable: Vec<Node>,
}

trait INodeList {
    fn item(&self, index: u32) -> Option<Node>;
    fn length(&self) -> u32;
}

impl INodeList for NodeList {
    fn item(&self, index: u32) -> Option<Node> {
        if index + 1 > self.length() {
            return None;
        }

        let elem = self.iterable.iter().nth(index as usize).unwrap().to_owned();
        Some(elem)
    }

    fn length(&self) -> u32 {
        self.iterable.len() as u32
    }
}
