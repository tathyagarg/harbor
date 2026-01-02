#![allow(dead_code)]

pub mod dom;
/// Custom implementation of the HTML5 spec:
/// https://html.spec.whatwg.org/
pub mod parse;

pub const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
