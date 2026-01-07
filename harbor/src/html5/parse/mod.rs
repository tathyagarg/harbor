use core::panic;
use std::cell::Ref;
use std::{cell::RefCell, rc::Rc};

mod afe;
mod open_elems;
mod stream;
mod tokenize;
mod tree;

pub use afe::{ActiveFormattingElements, ElementOrMarker};
pub use open_elems::OpenElementsStack;
pub use stream::InputStream;
pub use tokenize::{ParseError, ParserState};
pub use tree::{DOCTYPE, InsertMode, Tag, TagToken, Token};

pub use crate::html5::{dom::*, tag_groups::*};

#[derive(Debug)]
pub struct _Document {
    pub document: Rc<RefCell<Document>>,
}

impl _Document {
    pub fn document(&self) -> Ref<Document> {
        self.document.borrow()
    }

    pub fn document_mut(&self) -> std::cell::RefMut<Document> {
        self.document.borrow_mut()
    }
}

fn preprocess_input(input: &String) -> String {
    input.replace("\r\n", "\n").replace("\r", "\n")
}

fn is_leading_surrogate(code: u32) -> bool {
    (0xD800..=0xDBFF).contains(&code)
}

fn is_trailing_surrogate(code: u32) -> bool {
    (0xDC00..=0xDFFF).contains(&code)
}

fn is_surrogate(code: u32) -> bool {
    is_leading_surrogate(code) || is_trailing_surrogate(code)
}

fn is_noncharacter(code: u32) -> bool {
    matches!(
        code,
        0xFDD0
            ..=0xFDEF
                | 0xFFFE
                | 0xFFFF
                | 0x1FFFE
                | 0x1FFFF
                | 0x2FFFE
                | 0x2FFFF
                | 0x3FFFE
                | 0x3FFFF
                | 0x4FFFE
                | 0x4FFFF
                | 0x5FFFE
                | 0x5FFFF
                | 0x6FFFE
                | 0x6FFFF
                | 0x7FFFE
                | 0x7FFFF
                | 0x8FFFE
                | 0x8FFFF
                | 0x9FFFE
                | 0x9FFFF
                | 0xAFFFE
                | 0xAFFFF
                | 0xBFFFE
                | 0xBFFFF
                | 0xCFFFE
                | 0xCFFFF
                | 0xDFFFE
                | 0xDFFFF
                | 0xEFFFE
                | 0xEFFFF
                | 0xFFFFE
                | 0xFFFFF
                | 0x10FFFE
                | 0x10FFFF
    )
}

fn is_c0_control(code: u32) -> bool {
    (0x0000..=0x001F).contains(&code)
}

fn is_control(code: u32) -> bool {
    is_c0_control(code) || (0x007F..=0x009F).contains(&code)
}

fn is_ascii_whitespace(ch: char) -> bool {
    matches!(ch, '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}')
}

fn map_character_reference(code: u32) -> u32 {
    match code {
        0x80 => 0x20AC,
        0x82 => 0x201A,
        0x83 => 0x0192,
        0x84 => 0x201E,
        0x85 => 0x2026,
        0x86 => 0x2020,
        0x87 => 0x2021,
        0x88 => 0x02C6,
        0x89 => 0x2030,
        0x8A => 0x0160,
        0x8B => 0x2039,
        0x8C => 0x0152,
        0x8E => 0x017D,
        0x91 => 0x2018,
        0x92 => 0x2019,
        0x93 => 0x201C,
        0x94 => 0x201D,
        0x95 => 0x2022,
        0x96 => 0x2013,
        0x97 => 0x2014,
        0x98 => 0x02DC,
        0x99 => 0x2122,
        0x9A => 0x0161,
        0x9B => 0x203A,
        0x9C => 0x0153,
        0x9E => 0x017E,
        0x9F => 0x0178,
        _ => code,
    }
}

pub struct Parser<'a> {
    stream: &'a mut InputStream,

    state: ParserState,
    prev_state: ParserState,

    insertion_mode: InsertMode,
    original_insertion_mode: Option<InsertMode>,

    leave_callback: Option<Box<dyn Fn(&mut Parser)>>,

    return_state: Option<ParserState>,

    tag_token: Option<TagToken>,
    comment_token: Option<String>,
    doctype_token: Option<DOCTYPE>,

    temporary_buffer: String,
    character_reference_code: u32,

    pub document: _Document,

    active_formatting_elements: ActiveFormattingElements,
    open_elements_stack: OpenElementsStack,

    head_element_id: Option<ElementID>,

    pub emitted_tokens: Vec<Token>,

    flag_scripting: bool,
    flag_frameset_ok: bool,
}
