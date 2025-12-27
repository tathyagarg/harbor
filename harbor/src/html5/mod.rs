#![allow(dead_code)]
/// Custom implementation of the HTML5 spec:
/// https://html.spec.whatwg.org/

/// Although USVString and String aren't identical, I am using this alias for the time being and
/// may change it later.
type USVString = String;

type DOMString = String;

/// Placeholder
type ValueOfType = i32;

enum DocumentReadyState {
    Loading,
    Interactive,
    Complete,
}

pub struct Document {
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

/// [Exposed=Window]
#[derive(Default)]
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
#[derive(Default)]
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
