use core::panic;
// use std::borrow::Borrow;
use std::cell::Ref;
use std::{cell::RefCell, rc::Rc};

use crate::html5::{self, dom::*};

/// This is likely a temporary file and will be merged with some other code when I understand what
/// it is intended to integrate with. Until then, this is an independent implementation of an HTML5
/// parser.
///
/// Future tattu: I doubt this will be temporary

macro_rules! concat_arrays {
    ( $ty:ty, $default:expr => $($arr:expr),* $(,)? ) => {{
        const __CONCAT_ARRAYS_LEN: usize = 0 $( + $arr.len() )*;
        const __CONCAT_ARRAYS_RESULT: [$ty; __CONCAT_ARRAYS_LEN] = {
            let mut result = [$default; __CONCAT_ARRAYS_LEN];
            let mut result_idx = 0;
            $(
                let arr = $arr;
                let mut src_idx = 0;
                while src_idx < arr.len() {
                    result[result_idx] = arr[src_idx];
                    src_idx += 1;
                    result_idx += 1;
                }
            )*
            result
        };
        __CONCAT_ARRAYS_RESULT
    }};
}

const DEFAULT_SCOPE_NAMES: [&str; 14] = [
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template", "mi", "mo",
    "mn", "ms", "mtext",
];

const BUTTON_SCOPE_NAMES: [&str; 15] =
    concat_arrays!(&str, "" => &DEFAULT_SCOPE_NAMES, &["button"]);

const IMPLIED_END_TAGS: [&str; 10] = [
    "dd", "dt", "li", "option", "optgroup", "p", "rb", "rp", "rt", "rtc",
];

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

pub struct InputStream {
    input: Vec<char>,
    pos: usize,
    is_reconsume: bool,
    is_eof: bool,

    is_started: bool,
}

impl InputStream {
    pub fn new(data: String) -> InputStream {
        InputStream {
            input: data.chars().collect::<Vec<char>>(),
            pos: 0,
            is_reconsume: false,
            is_eof: false,
            is_started: false,
        }
    }

    fn current(&self) -> char {
        self.input[self.pos]
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos + 1 >= self.input.len() {
            self.is_eof = true;
            return None;
        }

        if !self.is_started {
            self.is_started = true;
            return Some(self.current());
        }
        self.pos += 1;
        Some(self.current())
    }

    fn consume(&mut self) -> Option<char> {
        if self.is_reconsume {
            self.is_reconsume = false;
            Some(self.current())
        } else {
            self.advance()
        }
    }

    fn reconsume(&mut self) {
        self.is_reconsume = true;
    }

    fn matches(
        &self,
        text: &str,
        case_sensitive: Option<bool>,
        start_from_next: Option<bool>,
    ) -> bool {
        let add = if start_from_next.unwrap_or(false) {
            1
        } else {
            0
        };

        let data_string = self.input[self.pos + add..self.pos + text.len() + add]
            .iter()
            .collect::<String>();

        let data = data_string.as_str();

        if case_sensitive.unwrap_or(true) {
            data.eq(text)
        } else {
            data.eq_ignore_ascii_case(text)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub is_self_closing: bool,
    pub attributes: Vec<(String, String)>,
}

impl Tag {
    fn empty() -> Tag {
        Tag::new(&String::new())
    }

    fn new(name: &String) -> Tag {
        Tag {
            name: name.clone(),
            is_self_closing: false,
            attributes: vec![],
        }
    }

    fn attribute_names_iter(&self) -> impl Iterator<Item = String> {
        self.attributes.iter().map(|x| x.0.clone())
    }

    fn attribute_names(&self) -> Vec<String> {
        self.attribute_names_iter().collect()
    }
}

pub enum TagToken {
    Start(Tag),
    End(Tag),
}

impl TagToken {
    fn apply(&mut self, f: impl Fn(&mut Tag) -> Tag) -> Tag {
        match self {
            TagToken::Start(start) => f(start),
            TagToken::End(end) => f(end),
        }
    }

    fn apply_no_ret(&mut self, f: impl Fn(&mut Tag)) {
        self.apply(|t: &mut Tag| {
            f(t);
            t.clone()
        });
    }

    fn new_tag_attr(&mut self, data: Option<(String, String)>) {
        self.apply_no_ret(|t: &mut Tag| {
            t.attributes
                .push(data.clone().unwrap_or((String::new(), String::new())))
        });
    }

    fn push_to_attr_name(&mut self, ch: char) {
        self.apply_no_ret(|t: &mut Tag| t.attributes.last_mut().unwrap().0.push(ch));
    }

    fn push_to_attr_val(&mut self, ch: char) {
        self.apply_no_ret(|t: &mut Tag| t.attributes.last_mut().unwrap().1.push(ch));
    }

    fn set_self_closing(&mut self) {
        self.apply_no_ret(|t: &mut Tag| t.is_self_closing = true);
    }

    fn current_attr(&mut self) -> (String, String) {
        self.apply(|t| t.clone()).attributes.last().unwrap().clone()
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct DOCTYPE {
    name: Option<String>,
    public_identifier: Option<String>,
    system_identifier: Option<String>,

    force_quirks: bool,
}

impl DOCTYPE {
    pub fn set_quirks(&mut self) {
        self.force_quirks = true;
    }

    pub fn with_name(&self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self.clone()
        }
    }

    pub fn with_empty_name(&self) -> Self {
        self.with_name(String::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    DOCTYPE(DOCTYPE),
    StartTag(Tag),
    EndTag(Tag),
    Comment(String),
    Character(char),
    EOF,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedNullCharacter,
    UnexpectedQuestionMarkInsteadOfTagName,
    EOFBeforeTagName,
    InvalidFirstCharacterOfTagName,
    MissingEndTagName,
    EOFInTag,
    EOFInScriptHTMLCommentLikeText,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedCharacterInAttributeName,
    DuplicateAttribute,
    MissingAttributeValue,
    UnexpectedCharacterInUnquotedAttributeValue,
    UnexpectedSolidusInTag,
    IncorrectlyOpenedComment,
    AbruptClosingOfEmptyComment,
    EOFInComment,
    NestedComment,
    IncorrectlyClosedComment,
    EOFInDOCTYPE,
    MissingWhitespaceBeforeDOCTYPEName,
    MissingDOCTYPEName,
    InvalidCharacterSequenceAfterDOCTYPEName,
    MissingWhitespaceAfterDOCTYPEPublicKeyword,
    MissingDOCTYPEPublicIdentifier,
    MissingQuoteBeforeDOCTYPEPublicIdentifier,
    AbruptDOCTYPEPublicIdentifier,
    MissingWhitespaceBetweenDOCTYPEPublicAndSystemIdentifiers,
    MissingQuoteBeforeDOCTYPESystemIdentifier,
    MissingWhitespaceAfterDOCTYPESystemKeyword,
    MissingDOCTYPESystemIdentifier,
    AbruptDOCTYPESystemIdentifier,
    UnexpectedCharacterAfterDOCTYPESystemIdentifier,
    EOFInCDATA,
    AbsenceOfDigitsInNumericCharacterReference,
    MissingSemicolonAfterCharacterReference,
    MissingWhitespaceBetweenAttributes,
    NullCharacterReference,
    CharacterReferenceOutsideUnicodeRange,
    SurrogateCharacterReference,
    NoncharacterCharacterReference,
    ControlCharacterReference,
    Custom(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenizerState {
    Data = 1,
    RCDATA = 2,
    RAWTEXT = 3,
    ScriptData = 4,
    PLAINTEXT = 5,
    TagOpen = 6,
    EndTagOpen = 7,
    TagName = 8,
    RCDATALessThanSign = 9,
    RCDATAEndTagOpen = 10,
    RCDATAEndTagName = 11,
    RAWTEXTLessThanSign = 12,
    RAWTEXTEndTagOpen = 13,
    RAWTEXTEndTagName = 14,
    ScriptDataLessThanSign = 15,
    ScriptDataEndTagOpen = 16,
    ScriptDataEndTagName = 17,
    ScriptDataEscapeStart = 18,
    ScriptDataEscapeStartDash = 19,
    ScriptDataEscaped = 20,
    ScriptDataEscapedDash = 21,
    ScriptDataEscapedDashDash = 22,
    ScriptDataEscapedLessThanSign = 23,
    ScriptDataEscapedEndTagOpen = 24,
    ScriptDataEscapedEndTagName = 25,
    ScriptDataDoubleEscapeStart = 26,
    ScriptDataDoubleEscaped = 27,
    ScriptDataDoubleEscapedDash = 28,
    ScriptDataDoubleEscapedDashDash = 29,
    ScriptDataDoubleEscapedLessThanSign = 30,
    ScriptDataDoubleEscapeEnd = 31,
    BeforeAttributeName = 32,
    AttributeName = 33,
    AfterAttributeName = 34,
    BeforeAttributeValue = 35,
    AttributeValueDoubleQuoted = 36,
    AttributeValueSingleQuoted = 37,
    AttributeValueUnquoted = 38,
    AfterAttributeValueQuoted = 39,
    SelfClosingStartTag = 40,
    BogusComment = 41,
    MarkupDeclarationOpen = 42,
    CommentStart = 43,
    CommentStartDash = 44,
    Comment = 45,
    CommentLessThanSign = 46,
    CommentLessThanSignBang = 47,
    CommentLessThanSignBangDash = 48,
    CommentLessThanSignBangDashDash = 49,
    CommentEndDash = 50,
    CommentEnd = 51,
    CommentEndBang = 52,
    DOCTYPE = 53,
    BeforeDOCTYPEName = 54,
    DOCTYPEName = 55,
    AfterDOCTYPEName = 56,
    AfterDOCTYPEPublicKeyword = 57,
    BeforeDOCTYPEPublicIdentifier = 58,
    DOCTYPEPublicIdentifierDoubleQuoted = 59,
    DOCTYPEPublicIdentifierSingleQuoted = 60,
    AfterDOCTYPEPublicIdentifier = 61,
    BetweenDOCTYPEPublicAndSystemIdentifiers = 62,
    AfterDOCTYPESystemKeyword = 63,
    BeforeDOCTYPESystemIdentifier = 64,
    DOCTYPESystemIdentifierDoubleQuoted = 65,
    DOCTYPESystemIdentifierSingleQuoted = 66,
    AfterDOCTYPESystemIdentifier = 67,
    BogusDOCTYPE = 68,
    CDATASection = 69,
    CDATASectionBracket = 70,
    CDATASectionEnd = 71,
    CharacterReference = 72,
    NamedCharacterReference = 73,
    AmbiguousAmpersand = 74,
    NumericCharacterReference = 75,
    HexadecimalCharacterReferenceStart = 76,
    DecimalCharacterReferenceStart = 77,
    HexadecimalCharacterReference = 78,
    DecimalCharacterReference = 79,
    NumericCharacterReferenceEnd = 80,
}

#[derive(Debug, Clone)]
pub enum InsertMode {
    Initial,
    BeforeHTML,
    BeforeHead,
    InHead,
    InHeadNoScript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

impl InsertMode {
    pub fn handle_initial(tokenizer: &mut Tokenizer, token: Token) -> bool {
        match token {
            Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}') => {}
            Token::Comment(data) => {
                tokenizer._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(doctype) => {
                if doctype.name.as_ref().unwrap().to_ascii_lowercase() != "html"
                    || doctype.public_identifier.is_some()
                    || doctype
                        .system_identifier
                        .as_ref()
                        .is_some_and(|idtfr| idtfr.as_str() != "about:legacy-compat")
                {
                    tokenizer.error(ParseError::UnexpectedCharacterAfterDOCTYPESystemIdentifier);
                }

                let doctype = DocumentType::new(
                    doctype.name.unwrap_or(String::new()).as_str(),
                    // NOTE: this will be overwritten by append child anyway, so clone is
                    // fine
                    Some(Rc::clone(&tokenizer.document.document)),
                )
                .with_public_id(doctype.public_identifier.unwrap_or(String::new()).as_str())
                .with_system_id(doctype.system_identifier.unwrap_or(String::new()).as_str());

                Node::append_child(
                    &Rc::clone(&tokenizer.document.document_mut()._node),
                    Rc::new(RefCell::new(NodeKind::DocumentType(doctype))),
                );

                // TODO: Set quirks mode if needed:
                // Reference: https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode

                tokenizer.insertion_mode = InsertMode::BeforeHTML;
            }
            _ => {
                // TODO: Set quirks mode:

                tokenizer.insertion_mode = InsertMode::BeforeHTML;
                return false;
            }
        }

        return true;
    }

    fn handle_before_html(tokenizer: &mut Tokenizer, token: Token) -> bool {
        match token {
            Token::DOCTYPE(_) => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in before html insertion mode",
                ));
            }
            Token::Comment(data) => {
                tokenizer._insert_comment(data.as_str(), None);
            }
            Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}') => {}
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                let element = Element::from_token(
                    &token,
                    html5::HTML_NAMESPACE,
                    &NodeKind::Document(tokenizer.document.document().clone()),
                );
                let document = tokenizer.document.document_mut();

                Node::append_child(
                    &Rc::clone(&document._node),
                    Rc::new(RefCell::new(NodeKind::Element(element.clone()))),
                );

                tokenizer.open_elements_stack.push(element);

                tokenizer.insertion_mode = InsertMode::BeforeHead;
            }
            Token::EndTag(Tag { name, .. })
                if !matches!(name.as_str(), "head" | "body" | "html" | "br") =>
            {
                tokenizer.error(ParseError::Custom(
                    "Unexpected end tag token in before html insertion mode",
                ));
            }
            _ => {
                let element = Element::from_token(
                    &Token::StartTag(Tag::new(&String::from("html"))),
                    "html",
                    &NodeKind::Document(tokenizer.document.document().clone()),
                );

                let document = tokenizer.document.document_mut();

                Node::append_child(
                    &Rc::clone(&document._node),
                    Rc::new(RefCell::new(NodeKind::Element(element.clone()))),
                );

                tokenizer.open_elements_stack.push(element);

                tokenizer.insertion_mode = InsertMode::BeforeHead;

                return false;
            }
        }

        return true;
    }

    fn handle_before_head(tokenizer: &mut Tokenizer, token: Token) -> bool {
        match token {
            Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}') => {}
            Token::Comment(data) => {
                tokenizer._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(_) => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in before head insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                InsertMode::handle_in_body(tokenizer, token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "head" => {
                let head = tokenizer
                    .open_elements_stack
                    .insert_html_element(&token)
                    .with_id("head");
                tokenizer.head_element_id = Some(head.id.clone());
                tokenizer.insertion_mode = InsertMode::InHead;
            }
            Token::EndTag(Tag { name, .. })
                if !matches!(name.as_str(), "head" | "body" | "html" | "br") =>
            {
                tokenizer.error(ParseError::Custom(
                    "Unexpected end tag token in before head insertion mode",
                ));
            }
            _ => {
                let head = tokenizer
                    .open_elements_stack
                    .insert_html_element(&Token::StartTag(Tag::new(&String::from("head"))))
                    .with_id("head");
                tokenizer.head_element_id = Some(head.id.clone());
                tokenizer.insertion_mode = InsertMode::InHead;

                return false;
            }
        }

        return true;
    }

    fn handle_in_head(tokenizer: &mut Tokenizer, token: Token) -> bool {
        match token {
            Token::Character(ch)
                if matches!(
                    ch,
                    '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
                ) =>
            {
                tokenizer._insert_character(ch);
            }
            Token::Comment(data) => {
                tokenizer._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(_) => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in in head insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                InsertMode::handle_in_body(tokenizer, token);
            }
            Token::StartTag(ref tag)
                if matches!(tag.name.as_str(), "base" | "basefont" | "bgsound" | "link") =>
            {
                tokenizer.open_elements_stack.insert_html_element(&token);
                tokenizer.open_elements_stack.pop();
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "meta" => {
                tokenizer.open_elements_stack.insert_html_element(&token);
                tokenizer.open_elements_stack.pop();
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "title" => {
                tokenizer._generic_rcdata_parsing_algorithm(&token);
            }
            Token::StartTag(ref tag)
                if (tag.name.as_str() == "noscript" && tokenizer.flag_scripting)
                    || (matches!(tag.name.as_str(), "noframes" | "style")) =>
            {
                tokenizer._generic_text_parsing_algorithm(&token);
            }
            Token::StartTag(ref tag)
                if (tag.name.as_str() == "noscript" && !tokenizer.flag_scripting) =>
            {
                tokenizer.open_elements_stack.insert_html_element(&token);
                tokenizer.insertion_mode = InsertMode::InHeadNoScript;
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "script" => {
                let element = Element::from_token(
                    &token,
                    html5::HTML_NAMESPACE,
                    &NodeKind::Element(
                        tokenizer
                            .open_elements_stack
                            .adjusted_current_node()
                            .unwrap()
                            .clone(),
                    ),
                );

                // TODO: Handle script element parsing correctly

                tokenizer
                    .open_elements_stack
                    .appropriate_insertion_place(None)
                    .insert(&mut NodeKind::Element(element.clone()));

                tokenizer.open_elements_stack.push(element);
                tokenizer.state = TokenizerState::ScriptData;

                tokenizer.original_insertion_mode = Some(tokenizer.insertion_mode.clone());
                tokenizer.insertion_mode = InsertMode::Text;
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "head" => {
                tokenizer.open_elements_stack.pop();
                tokenizer.insertion_mode = InsertMode::AfterHead;
            }
            // TODO: Start Tag "template", End Tag "template"
            Token::StartTag(ref start) if start.name.as_str() == "head" => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected start tag token in in head insertion mode",
                ));
            }
            Token::EndTag(ref tag) if !matches!(tag.name.as_str(), "body" | "html" | "br") => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected end tag token in in head insertion mode",
                ));
            }
            // }
            _ => {
                tokenizer.open_elements_stack.pop();
                tokenizer.insertion_mode = InsertMode::AfterHead;
                return false;
            }
        }

        return true;
    }

    fn handle_in_head_noscript(tokenizer: &mut Tokenizer, token: Token) -> bool {
        match token {
            Token::DOCTYPE(_) => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in in head noscript insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                InsertMode::handle_in_body(tokenizer, token);
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "noscript" => {
                tokenizer.open_elements_stack.pop();
                tokenizer.insertion_mode = InsertMode::InHead;
            }
            Token::Character('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}')
            | Token::Comment(_) => {
                InsertMode::handle_in_head(tokenizer, token);
            }
            Token::StartTag(ref tag)
                if matches!(tag.name.as_str(), "base" | "basefont" | "bgsound" | "link") =>
            {
                InsertMode::handle_in_head(tokenizer, token);
            }
            Token::StartTag(ref tag) if matches!(tag.name.as_str(), "head" | "noscript") => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected start tag token in in head noscript insertion mode",
                ));
            }
            Token::EndTag(ref tag) if tag.name.as_str() != "br" => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected end tag token in in head noscript insertion mode",
                ));
            }
            _ => {
                tokenizer.error(ParseError::Custom(
                    "Anything else token in in head noscript insertion mode",
                ));

                tokenizer.open_elements_stack.pop();
                tokenizer.insertion_mode = InsertMode::InHead;
                return false;
            }
        }

        return true;
    }

    fn handle_after_head(tokenizer: &mut Tokenizer, token: Token) -> bool {
        match token {
            Token::Character(ch)
                if matches!(
                    ch,
                    '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
                ) =>
            {
                tokenizer._insert_character(ch);
            }
            Token::Comment(data) => {
                tokenizer._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(_) => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in after head insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                InsertMode::handle_in_body(tokenizer, token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "body" => {
                tokenizer.open_elements_stack.insert_html_element(&token);
                tokenizer.flag_frameset_ok = false;

                tokenizer.insertion_mode = InsertMode::InBody;
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "frameset" => {
                tokenizer.open_elements_stack.insert_html_element(&token);
                tokenizer.insertion_mode = InsertMode::InFrameset;
            }
            Token::StartTag(ref tag)
                if matches!(
                    tag.name.as_str(),
                    "base"
                        | "basefont"
                        | "bgsound"
                        | "link"
                        | "meta"
                        | "noframes"
                        | "script"
                        | "style"
                        | "template"
                        | "title"
                ) =>
            {
                tokenizer.error(ParseError::Custom(
                    "Unexpected start tag token in after head insertion mode",
                ));

                // TODO: Process the token using the rules for the "in head" insertion mode
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "template" => {
                InsertMode::handle_in_head(tokenizer, token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "head" => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected start tag token in after head insertion mode",
                ));
            }
            Token::EndTag(ref tag) if !matches!(tag.name.as_str(), "body" | "html" | "br") => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected end tag token in after head insertion mode",
                ));
            }
            _ => {
                tokenizer
                    .open_elements_stack
                    .insert_html_element(&Token::StartTag(Tag::new(&String::from("body"))));
                tokenizer.insertion_mode = InsertMode::InBody;

                return false;
            }
        }

        return true;
    }

    fn handle_in_body(tokenizer: &mut Tokenizer, token: Token) -> bool {
        // NOTE: Oh boy - this is a chonker
        match token {
            Token::Character('\u{0000}') => {
                tokenizer.error(ParseError::UnexpectedNullCharacter);
            }
            Token::Character(ch)
                if matches!(
                    ch,
                    '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
                ) =>
            {
                tokenizer._reconstruct_active_formatting_elements();
                tokenizer._insert_character(ch);
            }
            Token::Character(ch) => {
                tokenizer._reconstruct_active_formatting_elements();
                tokenizer._insert_character(ch);
                tokenizer.flag_frameset_ok = false;
            }
            Token::Comment(data) => {
                tokenizer._insert_comment(data.as_str(), None);
            }
            Token::DOCTYPE(_) => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected DOCTYPE token in in body insertion mode",
                ));
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "html" => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected html start tag token in in body insertion mode",
                ));

                // TODO:
                // If there is a template element on the stack of open elements, then ignore the token.
                // Otherwise, for each attribute on the token, check to see if the attribute is already
                // present on the top element of the stack of open elements. If it is not, add the attribute
                // and its corresponding value to that element.
            }
            Token::StartTag(ref tag)
                if matches!(
                    tag.name.as_str(),
                    "base"
                        | "basefont"
                        | "bgsound"
                        | "link"
                        | "meta"
                        | "noframes"
                        | "script"
                        | "style"
                        | "template"
                        | "title"
                ) =>
            {
                InsertMode::handle_in_head(tokenizer, token);
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "template" => {
                InsertMode::handle_in_head(tokenizer, token);
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "body" => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected body start tag token in in body insertion mode",
                ));

                if tokenizer.open_elements_stack.elements.len() == 1
                    || tokenizer.open_elements_stack.elements[1].qualified_name() != "body"
                    || tokenizer._is_element_on_open_elements("template")
                {
                    // ignore
                    return true;
                } else {
                    tokenizer.flag_frameset_ok = false;

                    let body_element = &mut tokenizer.open_elements_stack.elements[1];

                    for (attr_name, attr_value) in &tag.attributes {
                        if !body_element
                            .attributes()
                            .iter()
                            .any(|attr| attr.local_name() == attr_name.as_str())
                        {
                            body_element.push_attr_raw(attr_name.as_str(), attr_value.as_str());
                        }
                    }
                }
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "frameset" => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected frameset start tag token in in body insertion mode",
                ));

                if tokenizer.open_elements_stack.elements.len() == 1
                    || tokenizer.open_elements_stack.elements[1].qualified_name() != "body"
                    || !tokenizer.flag_frameset_ok
                {
                    return true;
                } else {
                    let second = &mut tokenizer.open_elements_stack.elements[1];

                    let position = second
                        .node()
                        .borrow()
                        .parent_node()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .position_of_child(&NodeKind::Element(second.clone()));

                    second
                        .node()
                        .borrow_mut()
                        .parent_node_mut()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .pop_child(position);

                    tokenizer.open_elements_stack.elements.drain(1..);
                    tokenizer.open_elements_stack.insert_html_element(&token);

                    tokenizer.insertion_mode = InsertMode::InFrameset;
                }
            }
            Token::EOF => {
                if !tokenizer.open_elements_stack.elements.is_empty() {
                    InsertMode::handle_in_template(tokenizer, token);
                } else {
                    if tokenizer
                        .open_elements_stack
                        .has_non_special_element_in_scope()
                    {
                        tokenizer.error(ParseError::Custom(
                            "Unexpected EOF token in in body insertion mode",
                        ));
                    }

                    return true;
                }
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "body" => {
                if tokenizer
                    .open_elements_stack
                    .has_non_special_element_in_scope()
                {
                    tokenizer.error(ParseError::Custom(
                        "Unexpected end tag token in in body insertion mode",
                    ));
                }

                tokenizer.insertion_mode = InsertMode::AfterBody;
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "html" => {
                if tokenizer
                    .open_elements_stack
                    .has_non_special_element_in_scope()
                {
                    tokenizer.error(ParseError::Custom(
                        "Unexpected end tag token in in body insertion mode",
                    ));
                }

                tokenizer.insertion_mode = InsertMode::AfterBody;

                return false;
            }
            Token::StartTag(ref tag)
                if matches!(
                    tag.name.as_str(),
                    "address"
                        | "article"
                        | "aside"
                        | "blockquote"
                        | "center"
                        | "details"
                        | "dialog"
                        | "dir"
                        | "div"
                        | "dl"
                        | "fieldset"
                        | "figcaption"
                        | "figure"
                        | "footer"
                        | "header"
                        | "hgroup"
                        | "main"
                        | "nav"
                        | "ol"
                        | "p"
                        | "search"
                        | "section"
                        | "summary"
                        | "ul"
                ) =>
            {
                if tokenizer
                    .open_elements_stack
                    .has_element_in_button_scope("p")
                {
                    tokenizer.open_elements_stack.close_p_tag();
                }

                tokenizer.open_elements_stack.insert_html_element(&token);
            }
            Token::StartTag(ref tag)
                if matches!(tag.name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") =>
            {
                if tokenizer
                    .open_elements_stack
                    .has_element_in_button_scope("p")
                {
                    tokenizer.open_elements_stack.close_p_tag();
                }

                if tokenizer
                    .open_elements_stack
                    .adjusted_current_node()
                    .is_some_and(|el| {
                        matches!(
                            el.qualified_name().as_str(),
                            "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                        )
                    })
                {
                    tokenizer.error(ParseError::Custom(
                        "Unexpected heading start tag token in in body insertion mode",
                    ));

                    tokenizer.open_elements_stack.pop();
                }

                tokenizer.open_elements_stack.insert_html_element(&token);
            }
            Token::StartTag(ref tag) if matches!(tag.name.as_str(), "pre" | "listing") => {
                if tokenizer
                    .open_elements_stack
                    .has_element_in_button_scope("p")
                {
                    tokenizer.open_elements_stack.close_p_tag();
                }

                tokenizer.open_elements_stack.insert_html_element(&token);

                // TODO:
                // If the next token is a U+000A LINE FEED (LF) character token,
                // then ignore that token and move on to the next one.
                // (Newlines at the start of `pre` blocks are ignored as an authoring convenience.)

                tokenizer.flag_frameset_ok = false;
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "form" => {
                todo!("Handle form start tag in in body insertion mode");
            }
            Token::StartTag(ref tag) if tag.name.as_str() == "li" => {
                tokenizer.flag_frameset_ok = false;

                let mut node = tokenizer
                    .open_elements_stack
                    .adjusted_current_node()
                    .cloned();
            }
            _ => {}
        }

        return true;
    }

    fn handle_text(tokenizer: &mut Tokenizer, token: Token) -> bool {
        match token {
            Token::Character(ch) => {
                tokenizer._insert_character(ch);
            }
            Token::EOF => {
                tokenizer.error(ParseError::Custom(
                    "Unexpected EOF token in text insertion mode",
                ));

                tokenizer.open_elements_stack.pop();
                tokenizer.insertion_mode = tokenizer.original_insertion_mode.clone().unwrap();
                return false;
            }
            Token::EndTag(ref tag) if tag.name.as_str() == "script" => {
                todo!("Handle script end tag correctly");
            }
            Token::EndTag(_) => {
                tokenizer.open_elements_stack.pop();
                tokenizer.insertion_mode = tokenizer.original_insertion_mode.clone().unwrap();
            }
            _ => {
                unreachable!("Unexpected token in text insertion mode");
            }
        }

        return true;
    }

    fn handle_in_template(tokenizer: &mut Tokenizer, token: Token) -> bool {
        match token {
            _ => {}
        }

        return true;
    }

    pub fn handle(&self, tokenizer: &mut Tokenizer, token: &Token) -> bool {
        let token = token.clone();

        match self {
            InsertMode::Initial => InsertMode::handle_initial(tokenizer, token),
            InsertMode::BeforeHTML => InsertMode::handle_before_html(tokenizer, token),
            InsertMode::BeforeHead => InsertMode::handle_before_head(tokenizer, token),
            InsertMode::InHead => InsertMode::handle_in_head(tokenizer, token),
            InsertMode::InHeadNoScript => InsertMode::handle_in_head_noscript(tokenizer, token),
            InsertMode::AfterHead => InsertMode::handle_after_head(tokenizer, token),
            InsertMode::InBody => InsertMode::handle_in_body(tokenizer, token),
            InsertMode::Text => InsertMode::handle_text(tokenizer, token),
            _ => true,
        }
    }
}

#[derive(Clone)]
pub enum ElementOrMarker {
    Element(Element),
    Marker,
}

pub struct ActiveFormattingElements {
    elements: Vec<ElementOrMarker>,
}

impl ActiveFormattingElements {
    pub fn new() -> ActiveFormattingElements {
        ActiveFormattingElements { elements: vec![] }
    }

    pub fn last_marker(&self) -> Option<usize> {
        for (i, element) in self.elements.iter().enumerate().rev() {
            if matches!(element, ElementOrMarker::Marker) {
                return Some(i);
            }
        }
        None
    }

    pub fn push(&mut self, element: Element) {
        let last_marker = self.last_marker().unwrap_or(0);

        if self.elements[last_marker + 1..]
            .iter()
            .map(|el| match el {
                ElementOrMarker::Element(e) => e.clone(),
                ElementOrMarker::Marker => panic!("Should not encounter marker here"),
            })
            .filter(|el| {
                el.qualified_name() == element.qualified_name()
                    && element.attributes().len() == el.attributes().len()
                    && element.attributes().iter().all(|attr| {
                        el.get_attribute(attr.local_name())
                            .is_some_and(|v| v == attr.value())
                    })
                    && el.namespace_uri() == element.namespace_uri()
            })
            .count()
            >= 3
        {
            let first_matching_index = self
                .elements
                .iter()
                .position(|el| match el {
                    ElementOrMarker::Element(e) => {
                        e.qualified_name() == element.qualified_name()
                            && element.attributes().len() == e.attributes().len()
                            && element.attributes().iter().all(|attr| {
                                e.get_attribute(attr.local_name())
                                    .is_some_and(|v| v == attr.value())
                            })
                            && e.namespace_uri() == element.namespace_uri()
                    }
                    ElementOrMarker::Marker => false,
                })
                .unwrap();

            self.elements.remove(first_matching_index);
        }

        self.elements.push(ElementOrMarker::Element(element));
    }

    pub fn reconstruct(&mut self, tokenizer: &mut Tokenizer) {
        tokenizer._reconstruct_active_formatting_elements();
    }
}

struct OpenElementsStack {
    elements: Vec<Element>,
}

impl OpenElementsStack {
    pub fn new() -> OpenElementsStack {
        OpenElementsStack { elements: vec![] }
    }

    pub fn push(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn pop(&mut self) -> Option<Element> {
        self.elements.pop()
    }

    pub fn pop_until(&mut self, target_name: &str) {
        while let Some(element) = self.pop() {
            if element.qualified_name() == target_name {
                break;
            }
        }
    }

    pub fn contains(&self, element: &Element) -> bool {
        self.elements.iter().any(|el| el.id == element.id)
    }

    pub fn contains_tag(&self, tag_name: &str) -> bool {
        self.elements
            .iter()
            .any(|el| el.qualified_name() == tag_name)
    }

    fn adjusted_current_node(&self) -> Option<&Element> {
        self.elements.last()
    }

    fn adjusted_current_node_mut(&mut self) -> Option<&mut Element> {
        self.elements.last_mut()
    }

    fn appropriate_insertion_place(&mut self, override_target: Option<&Element>) -> InsertLocation {
        let target = override_target
            .unwrap_or_else(|| {
                self.adjusted_current_node()
                    .expect("No current node for appropriate insertion place")
            })
            .to_owned();

        let adjusted_insertion_position = target.node().borrow().child_nodes().length();

        InsertLocation::new(
            Rc::new(RefCell::new(NodeKind::Element(target))),
            adjusted_insertion_position,
        )
    }

    fn insert_foreign_element(
        &mut self,
        token: &Token,
        namespace: &str,
        only_add_to_element_stack: bool,
    ) -> Element {
        let mut adjusted_insertion_location = self.appropriate_insertion_place(None);

        let element = Element::from_token(
            token,
            namespace,
            &*adjusted_insertion_location.parent().borrow(),
        );

        if !only_add_to_element_stack {
            adjusted_insertion_location.insert(&mut NodeKind::Element(element.clone()));
        }

        self.push(element.clone());
        element
    }

    fn insert_html_element(&mut self, token: &Token) -> Element {
        self.insert_foreign_element(token, html5::HTML_NAMESPACE, false)
    }

    pub fn has_element_in_specific_scope(&self, target_name: &str, scope_names: &[&str]) -> bool {
        for element in self.elements.iter().rev() {
            if element.qualified_name() == target_name {
                return true;
            }

            if scope_names.contains(&element.qualified_name().as_str()) {
                return false;
            }
        }

        false
    }

    pub fn has_element_in_default_scope(&self, target_name: &str) -> bool {
        // TODO: Fact check list
        self.has_element_in_specific_scope(target_name, &DEFAULT_SCOPE_NAMES)
    }

    pub fn has_non_special_element_in_scope(&self) -> bool {
        let special_elements = [
            "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc", "tbody", "td",
            "tfoot", "th", "thead", "tr", "body", "html",
        ];

        for element in self.elements.iter().rev() {
            if !special_elements.contains(&element.qualified_name().as_str()) {
                return true;
            }
        }

        false
    }

    pub fn has_element_in_button_scope(&self, target_name: &str) -> bool {
        self.has_element_in_specific_scope(target_name, &BUTTON_SCOPE_NAMES)
    }

    pub fn generate_implied_end_tags(&mut self, exclude: Option<&str>) {
        loop {
            let current_node = match self.adjusted_current_node() {
                Some(node) => node,
                None => break,
            };

            if IMPLIED_END_TAGS.contains(&current_node.qualified_name().as_str())
                && Some(current_node.qualified_name().as_str()) != exclude
            {
                self.pop();
            } else {
                break;
            }
        }
    }

    pub fn close_p_tag(&mut self) {
        self.generate_implied_end_tags(Some("p"));
        self.pop_until("p")
    }
}

pub struct Tokenizer<'a> {
    stream: &'a mut InputStream,

    state: TokenizerState,
    prev_state: TokenizerState,

    insertion_mode: InsertMode,
    original_insertion_mode: Option<InsertMode>,

    leave_callback: Option<Box<dyn Fn(&mut Tokenizer)>>,

    return_state: Option<TokenizerState>,

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

impl<'a> Tokenizer<'a> {
    pub fn new<'b>(stream: &'b mut InputStream) -> Tokenizer<'b> {
        Tokenizer {
            stream,

            state: TokenizerState::Data,
            prev_state: TokenizerState::Data,

            insertion_mode: InsertMode::Initial,
            original_insertion_mode: None,

            leave_callback: None,

            return_state: None,

            tag_token: None,
            comment_token: None,
            doctype_token: None,

            temporary_buffer: String::new(),
            character_reference_code: 0,

            // Initialize an empty document
            document: _Document {
                document: Document::new(Origin::Opaque),
            },

            active_formatting_elements: ActiveFormattingElements::new(),
            open_elements_stack: OpenElementsStack::new(),

            head_element_id: None,

            emitted_tokens: vec![],

            flag_scripting: false,
            flag_frameset_ok: true,
        }
    }

    fn _is_element_on_open_elements(&self, name: &str) -> bool {
        self.open_elements_stack
            .elements
            .iter()
            .any(|el| el.qualified_name() == name)
    }

    fn _reconstruct_active_formatting_elements(&mut self) {
        if self.active_formatting_elements.elements.is_empty() {
            return;
        }

        if let Some(ElementOrMarker::Element(element)) =
            self.active_formatting_elements.elements.last()
            && self.open_elements_stack.contains(element)
        {
            return;
        }

        if let Some(ElementOrMarker::Marker) = self.active_formatting_elements.elements.last() {
            return;
        }

        let mut entry_pos = self.active_formatting_elements.elements.len() - 1;
        let mut entry = self.active_formatting_elements.elements[entry_pos].clone();

        let mut jump_to_create = false;

        loop {
            // Rewind:
            if entry_pos == 0 {
                jump_to_create = true;
                break;
            }

            entry_pos -= 1;
            entry = self.active_formatting_elements.elements[entry_pos].clone();

            if matches!(entry, ElementOrMarker::Marker)
                || self.open_elements_stack.contains(match &entry {
                    ElementOrMarker::Element(e) => e,
                    ElementOrMarker::Marker => panic!("Should not encounter marker here"),
                })
            {
                break;
            }
        }

        loop {
            // Advance:
            if !jump_to_create {
                entry_pos += 1;
                entry = self.active_formatting_elements.elements[entry_pos].clone();
            }

            // Create:
            let element = match &entry {
                ElementOrMarker::Element(e) => e,
                ElementOrMarker::Marker => panic!("Should not encounter marker here"),
            };

            let new_element = self.open_elements_stack.insert_html_element(
                &element
                    .token()
                    .expect("Element has no token - how the hell was it created?"),
            );

            self.active_formatting_elements.elements[entry_pos] =
                ElementOrMarker::Element(new_element);

            jump_to_create = false;

            if entry_pos + 1 >= self.active_formatting_elements.elements.len() {
                break;
            }
        }
    }

    fn _insert_character(&mut self, ch: char) {
        let mut location = self.open_elements_stack.appropriate_insertion_place(None);
        if matches!(
            location.parent().borrow().node().borrow()._node_type,
            NodeType::Document
        ) {
            return;
        }

        if let Some(text) = location.preceding()
            && let NodeKind::Text(text_node) = &mut *text.borrow_mut()
        {
            text_node.push(ch);
            return;
        } else {
            let node_doc = Rc::clone(
                &location
                    .parent()
                    .borrow()
                    .node()
                    .borrow()
                    .node_document
                    .as_ref()
                    .and_then(|w| w.upgrade())
                    .unwrap(),
            );

            location.insert(&mut NodeKind::Text(Text::new(
                ch.to_string().as_str(),
                node_doc,
            )));
        }
    }

    pub fn _insert_comment(&mut self, data: &str, position: Option<InsertLocation>) {
        let mut location =
            position.unwrap_or_else(|| self.open_elements_stack.appropriate_insertion_place(None));

        let comment = Comment::new(
            data,
            Rc::clone(
                &location
                    .parent()
                    .borrow()
                    .node()
                    .borrow()
                    .node_document
                    .as_ref()
                    .and_then(|w| w.upgrade())
                    .unwrap(),
            ),
        );

        location.insert(&mut NodeKind::Comment(comment));
    }

    fn _generic_text_parsing_algorithm(&mut self, token: &Token) {
        self.open_elements_stack.insert_html_element(token);
        self.state = TokenizerState::RAWTEXT;

        self.original_insertion_mode = Some(self.insertion_mode.clone());
        self.insertion_mode = InsertMode::Text;
    }

    fn _generic_rcdata_parsing_algorithm(&mut self, token: &Token) {
        self.open_elements_stack.insert_html_element(token);
        self.state = TokenizerState::RCDATA;

        self.original_insertion_mode = Some(self.insertion_mode.clone());
        self.insertion_mode = InsertMode::Text;
    }

    fn is_current_appropriate_end_tag(&self) -> bool {
        match &self.tag_token {
            Some(TagToken::End(tag)) => self.is_appropriate_end_tag(tag),
            _ => false,
        }
    }

    fn is_appropriate_end_tag(&self, tag: &Tag) -> bool {
        let last_start = self
            .emitted_tokens
            .iter()
            .rev()
            .find(|tok| matches!(tok, Token::StartTag(_)));

        match last_start {
            Some(tok) => {
                if let Token::StartTag(start_tag) = tok {
                    tag.name == start_tag.name
                } else {
                    unreachable!()
                }
            }
            None => false,
        }
    }

    fn emit(&mut self, token: Token) {
        self.emitted_tokens.push(token.clone());
        let mut mode = self.insertion_mode.clone();

        // println!("Ttok state: {:?}", self.state);
        // println!("Handling: {:?} token: {:?}", mode, token);
        while !mode.handle(self, &token) {
            mode = self.insertion_mode.clone();
        }
    }

    fn emit_doctype(&mut self) {
        self.emit(Token::DOCTYPE(self.doctype_token.clone().unwrap()));
    }

    fn emit_comment(&mut self) {
        self.emit(Token::Comment(
            self.comment_token.clone().unwrap_or(String::new()),
        ));
    }

    fn emit_tag(&mut self) {
        if let Some(tag) = &self.tag_token {
            match tag {
                TagToken::Start(tag) => self.emit(Token::StartTag(tag.clone())),
                TagToken::End(tag) => self.emit(Token::EndTag(tag.clone())),
            }
        }
    }

    fn error(&self, err: ParseError) {
        // For now, just print the error to the console.
        println!("Parse error: {:?}", err);
    }

    fn reconsume(&mut self, state: TokenizerState) {
        self.state = state;
        self.stream.reconsume();
    }

    fn push_to_tag(&mut self, ch: char) {
        self.tag_token
            .as_mut()
            .unwrap()
            .apply_no_ret(|t: &mut Tag| t.name.push(ch))
    }

    fn new_tag_attr(&mut self, data: Option<(String, String)>) {
        self.tag_token.as_mut().unwrap().new_tag_attr(data);
    }

    fn push_to_attr_name(&mut self, ch: char) {
        self.tag_token.as_mut().unwrap().push_to_attr_name(ch);
    }

    fn push_to_attr_val(&mut self, ch: char) {
        self.tag_token.as_mut().unwrap().push_to_attr_val(ch);
    }

    fn tag_set_self_closing(&mut self) {
        self.tag_token.as_mut().unwrap().set_self_closing();
    }

    fn tag_attribute_names_iter(&self) -> impl Iterator<Item = String> {
        if let Some(tag_tok) = &self.tag_token {
            match tag_tok {
                TagToken::Start(start) => start.attribute_names_iter(),
                TagToken::End(end) => end.attribute_names_iter(),
            }
        } else {
            unreachable!()
        }
    }

    fn tag_attribute_names(&self) -> Vec<String> {
        self.tag_attribute_names_iter().collect()
    }

    fn curr_tag_attr_name(&mut self) -> String {
        self.tag_token.as_mut().unwrap().current_attr().0
    }

    fn push_to_comment(&mut self, ch: char) {
        if let Some(comment) = &mut self.comment_token {
            comment.push(ch);
        }
    }

    fn push_to_doctype_name(&mut self, ch: char) {
        if let Some(doctype) = &mut self.doctype_token {
            if let Some(name) = &mut doctype.name {
                name.push(ch);
            }
        }
    }

    fn set_doctype_quirks(&mut self) {
        if let Some(doctype) = &mut self.doctype_token {
            doctype.set_quirks();
        }
    }

    fn char_ref_as_part_of_attr(&self) -> bool {
        self.return_state.as_ref().is_some_and(|s| {
            matches!(
                s,
                TokenizerState::AttributeValueDoubleQuoted
                    | TokenizerState::AttributeValueSingleQuoted
                    | TokenizerState::AttributeValueUnquoted
            )
        })
    }

    fn flush_consumed_as_char_ref(&mut self) {
        let part_of_attr = self.char_ref_as_part_of_attr();

        for ch in self.temporary_buffer.clone().chars() {
            if part_of_attr {
                self.push_to_attr_val(ch);
            } else {
                self.emit(Token::Character(ch));
            }
        }
    }

    pub fn tokenize(&mut self) {
        while !self.stream.is_eof {
            self.step();
        }
    }

    pub fn step(&mut self) {
        if self.prev_state != self.state {
            if let Some(callback) = self.leave_callback.take() {
                callback(self);
            }
        }

        self.prev_state = self.state.clone();

        match self.state {
            TokenizerState::Data => {
                // https://html.spec.whatwg.org/multipage/parsing.html#data-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0026}' => {
                            self.return_state = Some(TokenizerState::Data);
                            self.state = TokenizerState::CharacterReference
                        }
                        '\u{003C}' => {
                            self.state = TokenizerState::TagOpen;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.emit(Token::Character(self.stream.current()));
                        }
                        _ => {
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::RCDATA => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0026}' => {
                            self.return_state = Some(TokenizerState::RCDATA);
                            self.state = TokenizerState::CharacterReference
                        }
                        '\u{003C}' => {
                            self.state = TokenizerState::RCDATALessThanSign;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::RAWTEXT => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003C}' => {
                            self.state = TokenizerState::RAWTEXTLessThanSign;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::ScriptData => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003C}' => {
                            self.state = TokenizerState::ScriptDataLessThanSign;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::PLAINTEXT => {
                // https://html.spec.whatwg.org/multipage/parsing.html#plaintext-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::TagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0021}' => {
                            self.state = TokenizerState::MarkupDeclarationOpen;
                        }
                        '\u{002F}' => {
                            self.state = TokenizerState::EndTagOpen;
                        }
                        _ if ch.is_ascii_alphabetic() => {
                            self.tag_token = Some(TagToken::Start(Tag::empty()));
                            self.reconsume(TokenizerState::TagName);
                        }
                        '\u{003F}' => {
                            self.error(ParseError::UnexpectedQuestionMarkInsteadOfTagName);
                            self.comment_token = Some(String::new());
                            self.reconsume(TokenizerState::BogusComment);
                        }
                        _ => {
                            self.error(ParseError::InvalidFirstCharacterOfTagName);
                            self.emit(Token::Character('\u{003C}'));
                            self.reconsume(TokenizerState::Data);
                        }
                    }
                } else {
                    self.error(ParseError::EOFBeforeTagName);
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::EndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        _ if ch.is_ascii_alphabetic() => {
                            self.tag_token = Some(TagToken::End(Tag::empty()));
                            self.reconsume(TokenizerState::TagName);
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingEndTagName);
                            self.state = TokenizerState::Data;
                        }
                        _ => {
                            self.error(ParseError::InvalidFirstCharacterOfTagName);
                            self.comment_token = Some(String::new());
                            self.reconsume(TokenizerState::BogusComment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFBeforeTagName);
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::TagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        '\u{002F}' => {
                            self.state = TokenizerState::SelfClosingStartTag;
                        }
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        _ if ch.is_ascii_uppercase() => {
                            self.push_to_tag(ch.to_ascii_lowercase());
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_tag('\u{FFFD}');
                        }
                        _ => {
                            self.push_to_tag(ch);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::RCDATALessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state
                if let Some(ch) = self.stream.consume()
                    && matches!(ch, '\u{002F}')
                {
                    self.temporary_buffer = String::new();
                    self.state = TokenizerState::RCDATAEndTagOpen;
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.reconsume(TokenizerState::RCDATA);
                }
            }
            TokenizerState::RCDATAEndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-open-state
                if let Some(ch) = self.stream.consume()
                    && ch.is_ascii_alphabetic()
                {
                    self.tag_token = Some(TagToken::End(Tag::empty()));
                    self.reconsume(TokenizerState::RCDATAEndTagName);
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.reconsume(TokenizerState::RCDATA);
                }
            }
            TokenizerState::RCDATAEndTagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-name-state
                let anything_else = |_self: &mut Tokenizer<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.emit(Token::Character('\u{002F}'));
                    for ch in _self.temporary_buffer.clone().chars() {
                        _self.emit(Token::Character(ch));
                    }

                    _self.reconsume(TokenizerState::RCDATA);
                };

                if let Some(ch) = self.stream.consume() {
                    let is_appr = self.is_current_appropriate_end_tag();

                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' if is_appr => {
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        '\u{002F}' if is_appr => {
                            self.state = TokenizerState::SelfClosingStartTag;
                        }
                        '\u{003E}' if is_appr => {
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        _ if ch.is_ascii_uppercase() => {
                            self.push_to_tag(ch.to_ascii_lowercase());
                            self.temporary_buffer.push(ch);
                        }
                        _ if ch.is_ascii_lowercase() => {
                            self.push_to_tag(ch);
                            self.temporary_buffer.push(ch);
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self)
                }
            }
            TokenizerState::RAWTEXTLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-less-than-sign-state
                if let Some(ch) = self.stream.consume()
                    && matches!(ch, '\u{002F}')
                {
                    self.temporary_buffer = String::new();
                    self.state = TokenizerState::RAWTEXTEndTagOpen;
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.reconsume(TokenizerState::RAWTEXT);
                }
            }
            TokenizerState::RAWTEXTEndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-open-state
                if let Some(ch) = self.stream.consume()
                    && ch.is_ascii_alphabetic()
                {
                    self.tag_token = Some(TagToken::End(Tag::empty()));
                    self.reconsume(TokenizerState::RAWTEXTEndTagOpen);
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.reconsume(TokenizerState::RAWTEXT);
                }
            }
            TokenizerState::RAWTEXTEndTagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-name-state
                let anything_else = |_self: &mut Tokenizer<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.emit(Token::Character('\u{002F}'));
                    for ch in _self.temporary_buffer.clone().chars() {
                        _self.emit(Token::Character(ch));
                    }

                    _self.reconsume(TokenizerState::RAWTEXT);
                };

                if let Some(ch) = self.stream.consume() {
                    let is_appr = self.is_current_appropriate_end_tag();

                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' if is_appr => {
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        '\u{002F}' if is_appr => {
                            self.state = TokenizerState::SelfClosingStartTag;
                        }
                        '\u{003E}' if is_appr => {
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        _ if ch.is_ascii_uppercase() => {
                            self.push_to_tag(ch.to_ascii_lowercase());
                            self.temporary_buffer.push(ch);
                        }
                        _ if ch.is_ascii_lowercase() => {
                            self.push_to_tag(ch);
                            self.temporary_buffer.push(ch);
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::ScriptDataLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002F}' => {
                            self.temporary_buffer = String::new();
                            self.state = TokenizerState::ScriptDataEndTagOpen;
                        }
                        '\u{0021}' => {
                            self.state = TokenizerState::ScriptDataEscapeStart;

                            self.emit(Token::Character('\u{003C}'));
                            self.emit(Token::Character('\u{0021}'));
                        }
                        _ => {
                            self.emit(Token::Character('\u{003C}'));
                            self.reconsume(TokenizerState::ScriptData);
                        }
                    }
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.reconsume(TokenizerState::ScriptData);
                }
            }
            TokenizerState::ScriptDataEndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
                if let Some(ch) = self.stream.consume()
                    && ch.is_ascii_alphabetic()
                {
                    self.tag_token = Some(TagToken::End(Tag::empty()));
                    self.reconsume(TokenizerState::ScriptDataEndTagName);
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.reconsume(TokenizerState::ScriptData);
                }
            }
            TokenizerState::ScriptDataEndTagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
                let anything_else = |_self: &mut Tokenizer<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.emit(Token::Character('\u{002F}'));
                    for ch in _self.temporary_buffer.clone().chars() {
                        _self.emit(Token::Character(ch));
                    }

                    _self.reconsume(TokenizerState::ScriptData);
                };

                if let Some(ch) = self.stream.consume() {
                    let is_appr = self.is_current_appropriate_end_tag();

                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' if is_appr => {
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        '\u{002F}' if is_appr => {
                            self.state = TokenizerState::SelfClosingStartTag;
                        }
                        '\u{003E}' if is_appr => {
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        _ if ch.is_ascii_uppercase() => {
                            self.push_to_tag(ch.to_ascii_lowercase());
                            self.temporary_buffer.push(ch);
                        }
                        _ if ch.is_ascii_lowercase() => {
                            self.push_to_tag(ch);
                            self.temporary_buffer.push(ch);
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::ScriptDataEscapeStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-state
                if let Some(ch) = self.stream.consume()
                    && matches!(ch, '\u{002D}')
                {
                    self.state = TokenizerState::ScriptDataEscapeStartDash;
                    self.emit(Token::Character('\u{002D}'));
                } else {
                    self.reconsume(TokenizerState::ScriptData);
                }
            }
            TokenizerState::ScriptDataEscapeStartDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-dash-state
                if let Some(ch) = self.stream.consume()
                    && matches!(ch, '\u{002D}')
                {
                    self.state = TokenizerState::ScriptDataEscapedDashDash;
                    self.emit(Token::Character('\u{002D}'));
                } else {
                    self.reconsume(TokenizerState::ScriptData);
                }
            }
            TokenizerState::ScriptDataEscaped => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.state = TokenizerState::ScriptDataEscapedDash;
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = TokenizerState::ScriptDataEscapedLessThanSign;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::ScriptDataEscapedDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.state = TokenizerState::ScriptDataEscapedDashDash;
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = TokenizerState::ScriptDataEscapedLessThanSign;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.state = TokenizerState::ScriptDataEscaped;
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.state = TokenizerState::ScriptDataEscaped;
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                }
            }
            TokenizerState::ScriptDataEscapedDashDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = TokenizerState::ScriptDataEscapedLessThanSign;
                        }
                        '\u{003E}' => {
                            self.state = TokenizerState::ScriptData;
                            self.emit(Token::Character('\u{003E}'));
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.state = TokenizerState::ScriptDataEscaped;
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.state = TokenizerState::ScriptData;
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::ScriptDataEscapedLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-less-than-sign-state
                let anything_else = |_self: &mut Tokenizer<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.reconsume(TokenizerState::ScriptData);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002F}' => {
                            self.temporary_buffer = String::new();
                            self.state = TokenizerState::ScriptDataEscapedEndTagOpen;
                        }
                        _ if ch.is_ascii_alphabetic() => {
                            self.temporary_buffer = String::new();
                            self.emit(Token::Character('\u{003C}'));
                            self.reconsume(TokenizerState::ScriptDataDoubleEscapeStart);
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self)
                }
            }
            TokenizerState::ScriptDataEscapedEndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-open-state
                if let Some(ch) = self.stream.consume()
                    && ch.is_ascii_alphabetic()
                {
                    self.tag_token = Some(TagToken::End(Tag::empty()));
                    self.reconsume(TokenizerState::ScriptDataEscapedEndTagName);
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.reconsume(TokenizerState::ScriptDataEscaped);
                }
            }
            TokenizerState::ScriptDataEscapedEndTagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-name-state
                let is_appr = self.is_current_appropriate_end_tag();

                let anything_else = |_self: &mut Tokenizer<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.emit(Token::Character('\u{002F}'));

                    for ch in _self.temporary_buffer.clone().chars() {
                        _self.emit(Token::Character(ch));
                    }

                    _self.reconsume(TokenizerState::ScriptDataEscaped);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' if is_appr => {
                            self.state = TokenizerState::BeforeAttributeName;
                        }
                        '\u{002F}' if is_appr => {
                            self.state = TokenizerState::SelfClosingStartTag;
                        }
                        '\u{003E}' if is_appr => {
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        _ if ch.is_ascii_uppercase() => {
                            self.push_to_tag(ch.to_ascii_lowercase());
                            self.temporary_buffer.push(ch);
                        }
                        _ if ch.is_ascii_lowercase() => {
                            self.push_to_tag(ch);
                            self.temporary_buffer.push(ch);
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self)
                }
            }
            TokenizerState::ScriptDataDoubleEscapeStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-start-state
                let anything_else = |_self: &mut Tokenizer<'a>| {
                    _self.reconsume(TokenizerState::ScriptDataEscaped);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{00A}' | '\u{000C}' | '\u{0020}' | '\u{002F}'
                        | '\u{003E}' => {
                            self.state = if self.temporary_buffer.as_str() == "script" {
                                TokenizerState::ScriptDataDoubleEscaped
                            } else {
                                TokenizerState::ScriptDataEscaped
                            };

                            self.emit(Token::Character(ch));
                        }
                        _ if ch.is_ascii_uppercase() => {
                            self.temporary_buffer.push(ch.to_ascii_lowercase());
                            self.emit(Token::Character(ch));
                        }
                        _ if ch.is_ascii_lowercase() => {
                            self.temporary_buffer.push(ch);
                            self.emit(Token::Character(ch));
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self)
                }
            }
            TokenizerState::ScriptDataDoubleEscaped => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.state = TokenizerState::ScriptDataDoubleEscapedDash;
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = TokenizerState::ScriptDataEscapedLessThanSign;
                            self.emit(Token::Character('\u{003C}'));
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::ScriptDataDoubleEscapedDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.state = TokenizerState::ScriptDataDoubleEscapedDashDash;
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = TokenizerState::ScriptDataDoubleEscapedLessThanSign;
                            self.emit(Token::Character('\u{003C}'));
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.state = TokenizerState::ScriptDataDoubleEscaped;
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.state = TokenizerState::ScriptDataDoubleEscaped;
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::ScriptDataDoubleEscapedDashDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => self.emit(Token::Character('\u{002D}')),
                        '\u{003C}' => {
                            self.state = TokenizerState::ScriptDataDoubleEscapedLessThanSign;
                            self.emit(Token::Character('\u{003C}'));
                        }
                        '\u{003E}' => {
                            self.state = TokenizerState::ScriptData;
                            self.emit(Token::Character('\u{003E}'));
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.state = TokenizerState::ScriptDataDoubleEscaped;
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.state = TokenizerState::ScriptDataDoubleEscaped;
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::ScriptDataDoubleEscapedLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-less-than-sign-state
                if let Some(ch) = self.stream.consume()
                    && ch == '\u{002F}'
                {
                    self.temporary_buffer = String::new();
                    self.state = TokenizerState::ScriptDataDoubleEscapeEnd;
                    self.emit(Token::Character('\u{002F}'));
                } else {
                    self.reconsume(TokenizerState::ScriptDataDoubleEscaped);
                }
            }
            TokenizerState::ScriptDataDoubleEscapeEnd => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-end-state
                let anything_else = |_self: &mut Tokenizer<'a>| {
                    _self.reconsume(TokenizerState::ScriptDataDoubleEscaped);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '\u{002F}'
                        | '\u{003E}' => {
                            self.state = if self.temporary_buffer.as_str() == "script" {
                                TokenizerState::ScriptDataEscaped
                            } else {
                                TokenizerState::ScriptDataDoubleEscaped
                            };

                            self.emit(Token::Character(ch));
                        }
                        _ if ch.is_ascii_uppercase() => {
                            self.temporary_buffer.push(ch.to_ascii_lowercase());
                            self.emit(Token::Character(ch));
                        }
                        _ if ch.is_ascii_lowercase() => {
                            self.temporary_buffer.push(ch);
                            self.emit(Token::Character(ch));
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::BeforeAttributeName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
                let eof_or_002f_003e = |_self: &mut Tokenizer<'a>| {
                    _self.reconsume(TokenizerState::AfterAttributeName);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            // ignore
                        }
                        '\u{002F}' | '\u{003E}' => eof_or_002f_003e(self),
                        '\u{003D}' => {
                            self.error(ParseError::UnexpectedEqualsSignBeforeAttributeName);
                            self.new_tag_attr(Some((String::from(ch), String::new())));
                            self.state = TokenizerState::AttributeName;
                        }
                        _ => {
                            self.new_tag_attr(None);
                            self.reconsume(TokenizerState::AttributeName);
                        }
                    }
                } else {
                    eof_or_002f_003e(self);
                }
            }
            TokenizerState::AttributeName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state

                self.leave_callback = Some(Box::new(|_self: &mut Tokenizer| {
                    let curr_tag_name = _self.curr_tag_attr_name();

                    for name in _self.tag_attribute_names() {
                        if curr_tag_name == name {
                            _self.error(ParseError::DuplicateAttribute);
                            _self
                                .tag_token
                                .as_mut()
                                .unwrap()
                                .apply_no_ret(|t| _ = t.attributes.pop());
                        }
                    }
                }));

                // there's a lot of chars other than U+0009 that this runs for but I'm not making a
                // var name that long
                let eof_or_0009 = |_self: &mut Tokenizer<'a>| {
                    _self.reconsume(TokenizerState::AfterAttributeName);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '\u{002F}'
                        | '\u{0003E}' => eof_or_0009(self),
                        '\u{003D}' => self.state = TokenizerState::BeforeAttributeValue,
                        _ if ch.is_ascii_uppercase() => {
                            self.push_to_attr_name(ch.to_ascii_lowercase())
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_attr_name('\u{FFFD}');
                        }
                        _ => {
                            if matches!(ch, '\u{0022}' | '\u{0027}' | '\u{003C}') {
                                self.error(ParseError::UnexpectedCharacterInAttributeName);
                            }

                            self.push_to_attr_name(ch);
                        }
                    }
                } else {
                    eof_or_0009(self);
                }
            }
            TokenizerState::AfterAttributeName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{002F}' => self.state = TokenizerState::SelfClosingStartTag,
                        '\u{003D}' => self.state = TokenizerState::BeforeAttributeValue,
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        _ => {
                            self.new_tag_attr(None);
                            self.reconsume(TokenizerState::AttributeName);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::BeforeAttributeValue => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
                let anything_else = |_self: &mut Tokenizer| {
                    _self.reconsume(TokenizerState::AttributeValueUnquoted);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{0022}' => self.state = TokenizerState::AttributeValueDoubleQuoted,
                        '\u{0027}' => self.state = TokenizerState::AttributeValueSingleQuoted,
                        '\u{003E}' => {
                            self.error(ParseError::MissingAttributeValue);
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::AttributeValueDoubleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0022}' => self.state = TokenizerState::AfterAttributeValueQuoted,
                        '\u{0026}' => {
                            self.return_state = Some(TokenizerState::AttributeValueDoubleQuoted);
                            self.state = TokenizerState::CharacterReference;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_attr_name('\u{FFFD}');
                        }
                        _ => self.push_to_attr_val(ch),
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::AttributeValueSingleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0027}' => self.state = TokenizerState::AfterAttributeValueQuoted,
                        '\u{0026}' => {
                            self.return_state = Some(TokenizerState::AttributeValueSingleQuoted);
                            self.state = TokenizerState::CharacterReference;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_attr_val('\u{FFFD}');
                        }
                        _ => self.push_to_attr_val(ch),
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::AttributeValueUnquoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = TokenizerState::BeforeAttributeName
                        }
                        '\u{0026}' => {
                            self.return_state = Some(TokenizerState::AttributeValueUnquoted);
                            self.state = TokenizerState::CharacterReference;
                        }
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_attr_val('\u{FFFD}');
                        }
                        _ => {
                            if matches!(
                                ch,
                                '\u{0022}' | '\u{0027}' | '\u{003C}' | '\u{003D}' | '\u{0060}'
                            ) {
                                self.error(ParseError::UnexpectedCharacterInUnquotedAttributeValue);
                            }

                            self.push_to_attr_val(ch);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF)
                }
            }
            TokenizerState::AfterAttributeValueQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-quoted-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = TokenizerState::BeforeAttributeName
                        }
                        '\u{002F}' => self.state = TokenizerState::SelfClosingStartTag,
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_tag();
                        }
                        _ => {
                            self.error(ParseError::MissingWhitespaceBetweenAttributes);
                            self.reconsume(TokenizerState::BeforeAttributeName);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::SelfClosingStartTag => {
                // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
                if let Some(ch) = self.stream.consume() {
                    if ch == '\u{003E}' {
                        self.tag_set_self_closing();
                        self.state = TokenizerState::Data;
                        self.emit_tag();
                    } else {
                        self.error(ParseError::UnexpectedSolidusInTag);
                        self.reconsume(TokenizerState::BeforeAttributeName);
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::BogusComment => {
                // https://html.spec.whatwg.org/multipage/parsing.html#bogus-comment-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_comment();
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_comment('\u{FFFD}');
                        }
                        _ => self.push_to_comment(ch),
                    }
                } else {
                    self.emit_comment();
                }
            }
            TokenizerState::MarkupDeclarationOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
                // what the actual FUCK is this state
                if self.stream.matches("--", None, Some(true)) {
                    _ = self.stream.consume();
                    _ = self.stream.consume();

                    self.comment_token = Some(String::new());
                    self.state = TokenizerState::CommentStart;
                } else if self.stream.matches("DOCTYPE", Some(false), Some(true)) {
                    for _i in 0.."DOCTYPE".len() {
                        _ = self.stream.consume();
                    }
                    self.state = TokenizerState::DOCTYPE;
                } else if self.stream.matches("[CDATA[", Some(true), Some(true)) {
                    for _i in 0.."[CDATA[".len() {
                        _ = self.stream.consume();
                    }

                    // TODO: If there is an adjusted current node and it is not an element in the
                    // HTML namespace, then switch to the CDATA section state.
                    // Otherwise, this is a cdata-in-html-content parse error.

                    self.comment_token = Some(String::from("[CDATA["));
                    self.state = TokenizerState::BogusComment;
                } else {
                    // NOTE: Nothing is consumed via this state - this is intended!

                    self.error(ParseError::IncorrectlyOpenedComment);
                    self.comment_token = Some(String::new());
                    self.state = TokenizerState::BogusComment;
                }
            }
            TokenizerState::CommentStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-state
                let anything_else =
                    |_self: &mut Tokenizer| _self.reconsume(TokenizerState::Comment);

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => self.state = TokenizerState::CommentStartDash,
                        '\u{003E}' => {
                            self.error(ParseError::AbruptClosingOfEmptyComment);
                            self.state = TokenizerState::Data;
                            self.emit_comment();
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::CommentStartDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => self.state = TokenizerState::CommentEnd,
                        '\u{003E}' => {
                            self.error(ParseError::AbruptClosingOfEmptyComment);
                            self.state = TokenizerState::Data;
                            self.emit_comment();
                        }
                        _ => {
                            self.push_to_comment('\u{002D}');
                            self.reconsume(TokenizerState::Comment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::Comment => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003C}' => {
                            self.push_to_comment(ch);
                            self.state = TokenizerState::CommentLessThanSign;
                        }
                        '\u{002D}' => self.state = TokenizerState::CommentEndDash,
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_comment('\u{FFFD}');
                        }
                        _ => self.push_to_comment(ch),
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::CommentLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-state
                let anything_else =
                    |_self: &mut Tokenizer| _self.reconsume(TokenizerState::Comment);

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0021}' => {
                            self.push_to_comment(ch);
                            self.state = TokenizerState::CommentLessThanSignBang;
                        }
                        '\u{003C}' => self.push_to_comment(ch),
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::CommentLessThanSignBang => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-state
                if self.stream.consume().is_some_and(|c| c == '\u{002D}') {
                    self.state = TokenizerState::CommentLessThanSignBangDash;
                } else {
                    self.reconsume(TokenizerState::Comment);
                }
            }
            TokenizerState::CommentLessThanSignBangDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-state
                if self.stream.consume().is_some_and(|c| c == '\u{002D}') {
                    self.state = TokenizerState::CommentLessThanSignBangDashDash;
                } else {
                    self.reconsume(TokenizerState::CommentEndDash);
                }
            }
            TokenizerState::CommentLessThanSignBangDashDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-dash-state
                let consumed = self.stream.consume();

                if consumed.is_none() || consumed.is_some_and(|c| c == '\u{003E}') {
                    self.reconsume(TokenizerState::CommentEnd);
                } else {
                    self.error(ParseError::NestedComment);
                    self.reconsume(TokenizerState::CommentEnd);
                }
            }
            TokenizerState::CommentEndDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => self.state = TokenizerState::CommentEnd,
                        _ => {
                            self.push_to_comment(ch);
                            self.reconsume(TokenizerState::Comment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::CommentEnd => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_comment();
                        }
                        '\u{0021}' => self.state = TokenizerState::CommentEndBang,
                        '\u{002D}' => self.push_to_comment('\u{002D}'),
                        _ => {
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{002D}');
                            self.reconsume(TokenizerState::Comment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::CommentEndBang => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-bang-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{0021}');

                            self.state = TokenizerState::CommentEndDash;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::IncorrectlyClosedComment);
                            self.state = TokenizerState::Data;
                            self.emit_comment();
                        }
                        _ => {
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{0021}');

                            self.reconsume(TokenizerState::Comment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::DOCTYPE => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = TokenizerState::BeforeDOCTYPEName
                        }
                        '\u{003E}' => self.reconsume(TokenizerState::BeforeDOCTYPEName),
                        _ => {
                            self.error(ParseError::MissingWhitespaceBeforeDOCTYPEName);
                            self.reconsume(TokenizerState::BeforeDOCTYPEName);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.doctype_token = Some(DOCTYPE::default());

                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::BeforeDOCTYPEName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        _ if ch.is_ascii_uppercase() => {
                            self.doctype_token = Some(
                                DOCTYPE::default().with_name(ch.to_ascii_lowercase().to_string()),
                            );
                            self.state = TokenizerState::DOCTYPEName;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);

                            self.doctype_token =
                                Some(DOCTYPE::default().with_name('\u{FFFD}'.to_string()));
                            self.state = TokenizerState::DOCTYPEName;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPEName);

                            self.doctype_token = Some(DOCTYPE::default());
                            self.state = TokenizerState::Data;

                            self.set_doctype_quirks();
                            self.emit_doctype();
                        }
                        _ => {
                            self.doctype_token = Some(DOCTYPE::default().with_name(ch.to_string()));
                            self.state = TokenizerState::DOCTYPEName;
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.doctype_token = Some(DOCTYPE::default());

                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::DOCTYPEName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = TokenizerState::AfterDOCTYPEName
                        }
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ if ch.is_ascii_uppercase() => {
                            self.push_to_doctype_name(ch.to_ascii_lowercase());
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_doctype_name('\u{FFFD}');
                        }
                        _ => self.push_to_doctype_name(ch),
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::AfterDOCTYPEName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            if self.stream.matches("public", Some(false), None) {
                                for _i in 0.."public".len() {
                                    _ = self.stream.consume();
                                }

                                self.state = TokenizerState::AfterDOCTYPEPublicKeyword;
                            } else if self.stream.matches("system", Some(false), None) {
                                for _i in 0.."system".len() {
                                    _ = self.stream.consume();
                                }

                                self.state = TokenizerState::AfterDOCTYPESystemKeyword;
                            } else {
                                self.error(ParseError::InvalidCharacterSequenceAfterDOCTYPEName);
                                self.set_doctype_quirks();

                                self.reconsume(TokenizerState::BogusDOCTYPE);
                            }
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);

                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::AfterDOCTYPEPublicKeyword => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-keyword-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = TokenizerState::BeforeDOCTYPEPublicIdentifier;
                        }
                        '\u{0022}' => {
                            self.error(ParseError::MissingWhitespaceAfterDOCTYPEPublicKeyword);
                            self.doctype_token.as_mut().unwrap().public_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPEPublicIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.error(ParseError::MissingWhitespaceAfterDOCTYPEPublicKeyword);
                            self.doctype_token.as_mut().unwrap().public_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPEPublicIdentifierSingleQuoted;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(TokenizerState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::BeforeDOCTYPEPublicIdentifier => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-public-identifier-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{0022}' => {
                            self.doctype_token.as_mut().unwrap().public_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPEPublicIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.doctype_token.as_mut().unwrap().public_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPEPublicIdentifierSingleQuoted;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(TokenizerState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::DOCTYPEPublicIdentifierDoubleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(double-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0022}' => self.state = TokenizerState::AfterDOCTYPEPublicIdentifier,
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.doctype_token
                                .as_mut()
                                .unwrap()
                                .public_identifier
                                .as_mut()
                                .unwrap()
                                .push('\u{FFFD}');
                        }
                        '\u{003E}' => {
                            self.error(ParseError::AbruptDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.doctype_token
                                .as_mut()
                                .unwrap()
                                .public_identifier
                                .as_mut()
                                .unwrap()
                                .push(ch);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::DOCTYPEPublicIdentifierSingleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(single-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0027}' => self.state = TokenizerState::AfterDOCTYPEPublicIdentifier,
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.doctype_token
                                .as_mut()
                                .unwrap()
                                .public_identifier
                                .as_mut()
                                .unwrap()
                                .push('\u{FFFD}');
                        }
                        '\u{003E}' => {
                            self.error(ParseError::AbruptDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.doctype_token
                                .as_mut()
                                .unwrap()
                                .public_identifier
                                .as_mut()
                                .unwrap()
                                .push(ch);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::AfterDOCTYPEPublicIdentifier => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-identifier-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiers;
                        }
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        '\u{0022}' => {
                            self.error(
                                ParseError::MissingWhitespaceBetweenDOCTYPEPublicAndSystemIdentifiers,
                            );
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPESystemIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.error(
                                ParseError::MissingWhitespaceBetweenDOCTYPEPublicAndSystemIdentifiers,
                            );
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPESystemIdentifierSingleQuoted;
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(TokenizerState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::BetweenDOCTYPEPublicAndSystemIdentifiers => {
                // https://html.spec.whatwg.org/multipage/parsing.html#between-doctype-public-and-system-identifiers-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        '\u{0022}' => {
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPESystemIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPESystemIdentifierSingleQuoted;
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(TokenizerState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::AfterDOCTYPESystemKeyword => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-keyword-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = TokenizerState::BeforeDOCTYPESystemIdentifier;
                        }
                        '\u{0022}' => {
                            self.error(ParseError::MissingWhitespaceAfterDOCTYPESystemKeyword);
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPESystemIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.error(ParseError::MissingWhitespaceAfterDOCTYPESystemKeyword);
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPESystemIdentifierSingleQuoted;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(TokenizerState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::BeforeDOCTYPESystemIdentifier => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-system-identifier-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{0022}' => {
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPESystemIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = TokenizerState::DOCTYPESystemIdentifierSingleQuoted;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(TokenizerState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::DOCTYPESystemIdentifierDoubleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(double-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0022}' => self.state = TokenizerState::AfterDOCTYPESystemIdentifier,
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.doctype_token
                                .as_mut()
                                .unwrap()
                                .system_identifier
                                .as_mut()
                                .unwrap()
                                .push('\u{FFFD}');
                        }
                        '\u{003E}' => {
                            self.error(ParseError::AbruptDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.doctype_token
                                .as_mut()
                                .unwrap()
                                .system_identifier
                                .as_mut()
                                .unwrap()
                                .push(ch);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::DOCTYPESystemIdentifierSingleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(single-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0027}' => self.state = TokenizerState::AfterDOCTYPESystemIdentifier,
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.doctype_token
                                .as_mut()
                                .unwrap()
                                .system_identifier
                                .as_mut()
                                .unwrap()
                                .push('\u{FFFD}');
                        }
                        '\u{003E}' => {
                            self.error(ParseError::AbruptDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.doctype_token
                                .as_mut()
                                .unwrap()
                                .system_identifier
                                .as_mut()
                                .unwrap()
                                .push(ch);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::AfterDOCTYPESystemIdentifier => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-identifier-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            // NOTE: Does not set the doctype to quirks mode

                            self.error(ParseError::UnexpectedCharacterAfterDOCTYPESystemIdentifier);
                            self.reconsume(TokenizerState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::BogusDOCTYPE => {
                // https://html.spec.whatwg.org/multipage/parsing.html#bogus-doctype-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003E}' => {
                            self.state = TokenizerState::Data;
                            self.emit_doctype();
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                        }
                        _ => {}
                    }
                } else {
                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::CDATASection => {
                // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{005D}' => {
                            self.state = TokenizerState::CDATASectionBracket;
                        }
                        _ => {
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInCDATA);
                    self.emit(Token::EOF);
                }
            }
            TokenizerState::CDATASectionBracket => {
                // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-bracket-state
                if self.stream.consume().is_some_and(|c| c == '\u{005D}') {
                    self.state = TokenizerState::CDATASectionEnd;
                } else {
                    self.emit(Token::Character('\u{005D}'));
                    self.reconsume(TokenizerState::CDATASection);
                }
            }
            TokenizerState::CDATASectionEnd => {
                // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-end-state
                let anything_else = |_self: &mut Tokenizer| {
                    _self.emit(Token::Character('\u{005D}'));
                    _self.emit(Token::Character('\u{005D}'));
                    _self.reconsume(TokenizerState::CDATASection);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{005D}' => self.emit(Token::Character('\u{005D}')),
                        '\u{003E}' => self.state = TokenizerState::Data,
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::CharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state
                self.temporary_buffer = String::new();
                self.temporary_buffer.push('\u{0026}');

                let anything_else = |_self: &mut Tokenizer| {
                    _self.flush_consumed_as_char_ref();
                    let return_state = _self.return_state.clone().unwrap();
                    _self.reconsume(return_state);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0023}' => {
                            self.temporary_buffer.push(ch);
                            self.state = TokenizerState::NumericCharacterReference;
                        }
                        _ if ch.is_ascii_alphanumeric() => {
                            self.reconsume(TokenizerState::NamedCharacterReference);
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::NamedCharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#named-character-reference-state
                // TODO: FUCK THIS
                for _ in 0..100 {
                    println!("TATTU YOU NEED TO DO TS ITS RELEVANT");
                }
            }
            TokenizerState::AmbiguousAmpersand => {
                // https://html.spec.whatwg.org/multipage/parsing.html#ambiguous-ampersand-state
                // TODO: This is only reachable through NamedCharacterReference
            }
            TokenizerState::NumericCharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-state
                self.character_reference_code = 0;

                if self
                    .stream
                    .consume()
                    .is_some_and(|ch| matches!(ch, '\u{0078}' | '\u{0058}'))
                {
                    self.temporary_buffer.push(self.stream.current());
                    self.state = TokenizerState::HexadecimalCharacterReferenceStart;
                } else {
                    self.reconsume(TokenizerState::DecimalCharacterReferenceStart);
                }
            }
            TokenizerState::HexadecimalCharacterReferenceStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-start-state
                if self
                    .stream
                    .consume()
                    .is_some_and(|ch| ch.is_ascii_hexdigit())
                {
                    self.reconsume(TokenizerState::HexadecimalCharacterReference);
                } else {
                    self.error(ParseError::AbsenceOfDigitsInNumericCharacterReference);
                    self.flush_consumed_as_char_ref();
                    self.reconsume(self.return_state.clone().unwrap());
                }
            }
            TokenizerState::DecimalCharacterReferenceStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-start-state
                if self.stream.consume().is_some_and(|ch| ch.is_ascii_digit()) {
                    self.reconsume(TokenizerState::DecimalCharacterReference);
                } else {
                    self.error(ParseError::AbsenceOfDigitsInNumericCharacterReference);
                    self.flush_consumed_as_char_ref();
                    self.reconsume(self.return_state.clone().unwrap());
                }
            }
            TokenizerState::HexadecimalCharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-state
                let anything_else = |_self: &mut Tokenizer| {
                    _self.error(ParseError::MissingSemicolonAfterCharacterReference);
                    _self.reconsume(TokenizerState::NumericCharacterReferenceEnd);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        _ if ch.is_ascii_digit() => {
                            self.character_reference_code *= 16;
                            self.character_reference_code += ch as u32 - 0x30;
                        }
                        '\u{0041}'..='\u{0046}' => {
                            self.character_reference_code *= 16;
                            self.character_reference_code += ch as u32 - 0x37;
                        }
                        '\u{0061}'..='\u{0066}' => {
                            self.character_reference_code *= 16;
                            self.character_reference_code += ch as u32 - 0x57;
                        }
                        '\u{003B}' => {
                            self.state = TokenizerState::NumericCharacterReferenceEnd;
                        }
                        _ => {
                            anything_else(self);
                        }
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::DecimalCharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-state
                let anything_else = |_self: &mut Tokenizer| {
                    _self.error(ParseError::MissingSemicolonAfterCharacterReference);
                    _self.reconsume(TokenizerState::NumericCharacterReferenceEnd);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        _ if ch.is_ascii_digit() => {
                            self.character_reference_code *= 10;
                            self.character_reference_code += ch as u32 - 0x30;
                        }
                        '\u{003B}' => {
                            self.state = TokenizerState::NumericCharacterReferenceEnd;
                        }
                        _ => {
                            anything_else(self);
                        }
                    }
                } else {
                    anything_else(self);
                }
            }
            TokenizerState::NumericCharacterReferenceEnd => {
                // https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state
                match self.character_reference_code {
                    0x00 => {
                        self.error(ParseError::NullCharacterReference);
                        self.character_reference_code = 0xFFFD;
                    }
                    0x10FFFF.. => {
                        self.error(ParseError::CharacterReferenceOutsideUnicodeRange);
                        self.character_reference_code = 0xFFFD;
                    }
                    _ if is_surrogate(self.character_reference_code) => {
                        self.error(ParseError::SurrogateCharacterReference);
                        self.character_reference_code = 0xFFFD;
                    }
                    _ if is_noncharacter(self.character_reference_code) => {
                        self.error(ParseError::NoncharacterCharacterReference);
                    }
                    _ => {
                        if self.character_reference_code == 0x0D
                            || (is_control(self.character_reference_code)
                                && !is_ascii_whitespace(
                                    char::from_u32(self.character_reference_code).unwrap(),
                                ))
                        {
                            self.error(ParseError::ControlCharacterReference);
                        }

                        self.character_reference_code =
                            map_character_reference(self.character_reference_code);
                    }
                }

                self.temporary_buffer = String::new();
                self.temporary_buffer
                    .push(char::from_u32(self.character_reference_code).unwrap_or('\u{FFFD}'));
                self.flush_consumed_as_char_ref();

                let return_state = self.return_state.clone().unwrap();
                self.state = return_state;
            }
        }
    }
}
