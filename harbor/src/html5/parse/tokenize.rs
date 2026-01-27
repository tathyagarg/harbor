use crate::html5::{
    dom::*,
    parse::{
        _Document, ActiveFormattingElements, DOCTYPE, ElementOrMarker, InputStream, InsertMode,
        NAMED_CHARACTER_REFERENCES, OpenElementsStack, Parser, Tag, TagToken, Token,
        is_ascii_whitespace, is_control, is_noncharacter, map_character_reference,
    },
};
use crate::infra::is_surrogate;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    UnknownNamedCharacterReference,
    Custom(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserState {
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

impl<'a> Parser<'a> {
    pub fn new(stream: &mut InputStream<char>) -> Parser {
        Parser {
            stream,

            state: ParserState::Data,
            prev_state: ParserState::Data,

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

    pub fn _is_element_on_open_elements(&self, name: &str) -> bool {
        self.open_elements_stack
            .elements
            .iter()
            .any(|el| el.borrow().qualified_name() == name)
    }

    pub fn _reconstruct_active_formatting_elements(&mut self) {
        if self.active_formatting_elements.elements.is_empty() {
            return;
        }

        if let Some(ElementOrMarker::Element(element)) =
            self.active_formatting_elements.elements.last()
            && self.open_elements_stack.contains_rc(element)
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
                || self.open_elements_stack.contains_rc(match &entry {
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
                    .borrow()
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

    pub fn _insert_character(&mut self, ch: char) {
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
            text_node.borrow_mut().push(ch);
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

            location.insert(&mut NodeKind::Text(Rc::new(RefCell::new(Text::new(
                ch.to_string().as_str(),
                node_doc,
            )))));
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

    pub fn _generic_text_parsing_algorithm(&mut self, token: &Token) {
        self.open_elements_stack.insert_html_element(token);
        self.state = ParserState::RAWTEXT;

        self.original_insertion_mode = Some(self.insertion_mode.clone());
        self.insertion_mode = InsertMode::Text;
    }

    pub fn _generic_rcdata_parsing_algorithm(&mut self, token: &Token) {
        self.open_elements_stack.insert_html_element(token);
        self.state = ParserState::RCDATA;

        self.original_insertion_mode = Some(self.insertion_mode.clone());
        self.insertion_mode = InsertMode::Text;
    }

    pub fn is_current_appropriate_end_tag(&self) -> bool {
        match &self.tag_token {
            Some(TagToken::End(tag)) => self.is_appropriate_end_tag(tag),
            _ => false,
        }
    }

    pub fn is_appropriate_end_tag(&self, tag: &Tag) -> bool {
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

    pub fn emit(&mut self, token: Token) {
        self.emitted_tokens.push(token.clone());
        let mut mode = self.insertion_mode.clone();

        while !mode.handle(self, &token) {
            mode = self.insertion_mode.clone();
        }
    }

    pub fn emit_doctype(&mut self) {
        self.emit(Token::DOCTYPE(self.doctype_token.clone().unwrap()));
    }

    pub fn emit_comment(&mut self) {
        self.emit(Token::Comment(
            self.comment_token.clone().unwrap_or(String::new()),
        ));
    }

    pub fn emit_tag(&mut self) {
        if let Some(tag) = &self.tag_token {
            match tag {
                TagToken::Start(tag) => self.emit(Token::StartTag(tag.clone())),
                TagToken::End(tag) => self.emit(Token::EndTag(tag.clone())),
            }
        }
    }

    pub fn error(&self, err: ParseError) {
        // For now, just print the error to the console.
        eprintln!("Parse error: {:?}", err);
    }

    pub fn reconsume(&mut self, state: ParserState) {
        self.state = state;
        self.stream.reconsume();
    }

    pub fn push_to_tag(&mut self, ch: char) {
        self.tag_token
            .as_mut()
            .unwrap()
            .apply_no_ret(|t: &mut Tag| t.name.push(ch))
    }

    pub fn new_tag_attr(&mut self, data: Option<(String, String)>) {
        self.tag_token.as_mut().unwrap().new_tag_attr(data);
    }

    pub fn push_to_attr_name(&mut self, ch: char) {
        self.tag_token.as_mut().unwrap().push_to_attr_name(ch);
    }

    pub fn push_to_attr_val(&mut self, ch: char) {
        self.tag_token.as_mut().unwrap().push_to_attr_val(ch);
    }

    pub fn tag_set_self_closing(&mut self) {
        self.tag_token.as_mut().unwrap().set_self_closing();
    }

    pub fn tag_attribute_names_iter(&self) -> impl Iterator<Item = String> {
        if let Some(tag_tok) = &self.tag_token {
            match tag_tok {
                TagToken::Start(start) => start.attribute_names_iter(),
                TagToken::End(end) => end.attribute_names_iter(),
            }
        } else {
            unreachable!()
        }
    }

    pub fn tag_attribute_names(&self) -> Vec<String> {
        self.tag_attribute_names_iter().collect()
    }

    pub fn curr_tag_attr_name(&mut self) -> String {
        self.tag_token.as_mut().unwrap().current_attr().0
    }

    pub fn push_to_comment(&mut self, ch: char) {
        if let Some(comment) = &mut self.comment_token {
            comment.push(ch);
        }
    }

    pub fn push_to_doctype_name(&mut self, ch: char) {
        if let Some(doctype) = &mut self.doctype_token {
            if let Some(name) = &mut doctype.name {
                name.push(ch);
            }
        }
    }

    pub fn set_doctype_quirks(&mut self) {
        if let Some(doctype) = &mut self.doctype_token {
            doctype.set_quirks();
        }
    }

    pub fn char_ref_as_part_of_attr(&self) -> bool {
        self.return_state.as_ref().is_some_and(|s| {
            matches!(
                s,
                ParserState::AttributeValueDoubleQuoted
                    | ParserState::AttributeValueSingleQuoted
                    | ParserState::AttributeValueUnquoted
            )
        })
    }

    pub fn flush_consumed_as_char_ref(&mut self) {
        let part_of_attr = self.char_ref_as_part_of_attr();

        for ch in self.temporary_buffer.clone().chars() {
            if part_of_attr {
                self.push_to_attr_val(ch);
            } else {
                self.emit(Token::Character(ch));
            }
        }
    }

    pub fn parse(&mut self) {
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
            ParserState::Data => {
                // https://html.spec.whatwg.org/multipage/parsing.html#data-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0026}' => {
                            self.return_state = Some(ParserState::Data);
                            self.state = ParserState::CharacterReference
                        }
                        '\u{003C}' => {
                            self.state = ParserState::TagOpen;
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
            ParserState::RCDATA => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0026}' => {
                            self.return_state = Some(ParserState::RCDATA);
                            self.state = ParserState::CharacterReference
                        }
                        '\u{003C}' => {
                            self.state = ParserState::RCDATALessThanSign;
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
            ParserState::RAWTEXT => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003C}' => {
                            self.state = ParserState::RAWTEXTLessThanSign;
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
            ParserState::ScriptData => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003C}' => {
                            self.state = ParserState::ScriptDataLessThanSign;
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
            ParserState::PLAINTEXT => {
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
            ParserState::TagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0021}' => {
                            self.state = ParserState::MarkupDeclarationOpen;
                        }
                        '\u{002F}' => {
                            self.state = ParserState::EndTagOpen;
                        }
                        _ if ch.is_ascii_alphabetic() => {
                            self.tag_token = Some(TagToken::Start(Tag::empty()));
                            self.reconsume(ParserState::TagName);
                        }
                        '\u{003F}' => {
                            self.error(ParseError::UnexpectedQuestionMarkInsteadOfTagName);
                            self.comment_token = Some(String::new());
                            self.reconsume(ParserState::BogusComment);
                        }
                        _ => {
                            self.error(ParseError::InvalidFirstCharacterOfTagName);
                            self.emit(Token::Character('\u{003C}'));
                            self.reconsume(ParserState::Data);
                        }
                    }
                } else {
                    self.error(ParseError::EOFBeforeTagName);
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::EOF);
                }
            }
            ParserState::EndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        _ if ch.is_ascii_alphabetic() => {
                            self.tag_token = Some(TagToken::End(Tag::empty()));
                            self.reconsume(ParserState::TagName);
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingEndTagName);
                            self.state = ParserState::Data;
                        }
                        _ => {
                            self.error(ParseError::InvalidFirstCharacterOfTagName);
                            self.comment_token = Some(String::new());
                            self.reconsume(ParserState::BogusComment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFBeforeTagName);
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.emit(Token::EOF);
                }
            }
            ParserState::TagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = ParserState::BeforeAttributeName;
                        }
                        '\u{002F}' => {
                            self.state = ParserState::SelfClosingStartTag;
                        }
                        '\u{003E}' => {
                            self.state = ParserState::Data;
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
            ParserState::RCDATALessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state
                if let Some(ch) = self.stream.consume()
                    && matches!(ch, '\u{002F}')
                {
                    self.temporary_buffer = String::new();
                    self.state = ParserState::RCDATAEndTagOpen;
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.reconsume(ParserState::RCDATA);
                }
            }
            ParserState::RCDATAEndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-open-state
                if let Some(ch) = self.stream.consume()
                    && ch.is_ascii_alphabetic()
                {
                    self.tag_token = Some(TagToken::End(Tag::empty()));
                    self.reconsume(ParserState::RCDATAEndTagName);
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.reconsume(ParserState::RCDATA);
                }
            }
            ParserState::RCDATAEndTagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-name-state
                let anything_else = |_self: &mut Parser<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.emit(Token::Character('\u{002F}'));
                    for ch in _self.temporary_buffer.clone().chars() {
                        _self.emit(Token::Character(ch));
                    }

                    _self.reconsume(ParserState::RCDATA);
                };

                if let Some(ch) = self.stream.consume() {
                    let is_appr = self.is_current_appropriate_end_tag();

                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' if is_appr => {
                            self.state = ParserState::BeforeAttributeName;
                        }
                        '\u{002F}' if is_appr => {
                            self.state = ParserState::SelfClosingStartTag;
                        }
                        '\u{003E}' if is_appr => {
                            self.state = ParserState::Data;
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
            ParserState::RAWTEXTLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-less-than-sign-state
                if let Some(ch) = self.stream.consume()
                    && matches!(ch, '\u{002F}')
                {
                    self.temporary_buffer = String::new();
                    self.state = ParserState::RAWTEXTEndTagOpen;
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.reconsume(ParserState::RAWTEXT);
                }
            }
            ParserState::RAWTEXTEndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-open-state
                if let Some(ch) = self.stream.consume()
                    && ch.is_ascii_alphabetic()
                {
                    self.tag_token = Some(TagToken::End(Tag::empty()));
                    self.reconsume(ParserState::RAWTEXTEndTagName);
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.reconsume(ParserState::RAWTEXT);
                }
            }
            ParserState::RAWTEXTEndTagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-name-state
                let anything_else = |_self: &mut Parser<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.emit(Token::Character('\u{002F}'));
                    for ch in _self.temporary_buffer.clone().chars() {
                        _self.emit(Token::Character(ch));
                    }

                    _self.reconsume(ParserState::RAWTEXT);
                };

                if let Some(ch) = self.stream.consume() {
                    let is_appr = self.is_current_appropriate_end_tag();

                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' if is_appr => {
                            self.state = ParserState::BeforeAttributeName;
                        }
                        '\u{002F}' if is_appr => {
                            self.state = ParserState::SelfClosingStartTag;
                        }
                        '\u{003E}' if is_appr => {
                            self.state = ParserState::Data;
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
            ParserState::ScriptDataLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002F}' => {
                            self.temporary_buffer = String::new();
                            self.state = ParserState::ScriptDataEndTagOpen;
                        }
                        '\u{0021}' => {
                            self.state = ParserState::ScriptDataEscapeStart;

                            self.emit(Token::Character('\u{003C}'));
                            self.emit(Token::Character('\u{0021}'));
                        }
                        _ => {
                            self.emit(Token::Character('\u{003C}'));
                            self.reconsume(ParserState::ScriptData);
                        }
                    }
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.reconsume(ParserState::ScriptData);
                }
            }
            ParserState::ScriptDataEndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
                if let Some(ch) = self.stream.consume()
                    && ch.is_ascii_alphabetic()
                {
                    self.tag_token = Some(TagToken::End(Tag::empty()));
                    self.reconsume(ParserState::ScriptDataEndTagName);
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.reconsume(ParserState::ScriptData);
                }
            }
            ParserState::ScriptDataEndTagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
                let anything_else = |_self: &mut Parser<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.emit(Token::Character('\u{002F}'));
                    for ch in _self.temporary_buffer.clone().chars() {
                        _self.emit(Token::Character(ch));
                    }

                    _self.reconsume(ParserState::ScriptData);
                };

                if let Some(ch) = self.stream.consume() {
                    let is_appr = self.is_current_appropriate_end_tag();

                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' if is_appr => {
                            self.state = ParserState::BeforeAttributeName;
                        }
                        '\u{002F}' if is_appr => {
                            self.state = ParserState::SelfClosingStartTag;
                        }
                        '\u{003E}' if is_appr => {
                            self.state = ParserState::Data;
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
            ParserState::ScriptDataEscapeStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-state
                if let Some(ch) = self.stream.consume()
                    && matches!(ch, '\u{002D}')
                {
                    self.state = ParserState::ScriptDataEscapeStartDash;
                    self.emit(Token::Character('\u{002D}'));
                } else {
                    self.reconsume(ParserState::ScriptData);
                }
            }
            ParserState::ScriptDataEscapeStartDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-dash-state
                if let Some(ch) = self.stream.consume()
                    && matches!(ch, '\u{002D}')
                {
                    self.state = ParserState::ScriptDataEscapedDashDash;
                    self.emit(Token::Character('\u{002D}'));
                } else {
                    self.reconsume(ParserState::ScriptData);
                }
            }
            ParserState::ScriptDataEscaped => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.state = ParserState::ScriptDataEscapedDash;
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = ParserState::ScriptDataEscapedLessThanSign;
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
            ParserState::ScriptDataEscapedDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.state = ParserState::ScriptDataEscapedDashDash;
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = ParserState::ScriptDataEscapedLessThanSign;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.state = ParserState::ScriptDataEscaped;
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.state = ParserState::ScriptDataEscaped;
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                }
            }
            ParserState::ScriptDataEscapedDashDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = ParserState::ScriptDataEscapedLessThanSign;
                        }
                        '\u{003E}' => {
                            self.state = ParserState::ScriptData;
                            self.emit(Token::Character('\u{003E}'));
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.state = ParserState::ScriptDataEscaped;
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.state = ParserState::ScriptData;
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                    self.emit(Token::EOF);
                }
            }
            ParserState::ScriptDataEscapedLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-less-than-sign-state
                let anything_else = |_self: &mut Parser<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.reconsume(ParserState::ScriptData);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002F}' => {
                            self.temporary_buffer = String::new();
                            self.state = ParserState::ScriptDataEscapedEndTagOpen;
                        }
                        _ if ch.is_ascii_alphabetic() => {
                            self.temporary_buffer = String::new();
                            self.emit(Token::Character('\u{003C}'));
                            self.reconsume(ParserState::ScriptDataDoubleEscapeStart);
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self)
                }
            }
            ParserState::ScriptDataEscapedEndTagOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-open-state
                if let Some(ch) = self.stream.consume()
                    && ch.is_ascii_alphabetic()
                {
                    self.tag_token = Some(TagToken::End(Tag::empty()));
                    self.reconsume(ParserState::ScriptDataEscapedEndTagName);
                } else {
                    self.emit(Token::Character('\u{003C}'));
                    self.emit(Token::Character('\u{002F}'));
                    self.reconsume(ParserState::ScriptDataEscaped);
                }
            }
            ParserState::ScriptDataEscapedEndTagName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-name-state
                let is_appr = self.is_current_appropriate_end_tag();

                let anything_else = |_self: &mut Parser<'a>| {
                    _self.emit(Token::Character('\u{003C}'));
                    _self.emit(Token::Character('\u{002F}'));

                    for ch in _self.temporary_buffer.clone().chars() {
                        _self.emit(Token::Character(ch));
                    }

                    _self.reconsume(ParserState::ScriptDataEscaped);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' if is_appr => {
                            self.state = ParserState::BeforeAttributeName;
                        }
                        '\u{002F}' if is_appr => {
                            self.state = ParserState::SelfClosingStartTag;
                        }
                        '\u{003E}' if is_appr => {
                            self.state = ParserState::Data;
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
            ParserState::ScriptDataDoubleEscapeStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-start-state
                let anything_else = |_self: &mut Parser<'a>| {
                    _self.reconsume(ParserState::ScriptDataEscaped);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{00A}' | '\u{000C}' | '\u{0020}' | '\u{002F}'
                        | '\u{003E}' => {
                            self.state = if self.temporary_buffer.as_str() == "script" {
                                ParserState::ScriptDataDoubleEscaped
                            } else {
                                ParserState::ScriptDataEscaped
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
            ParserState::ScriptDataDoubleEscaped => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.state = ParserState::ScriptDataDoubleEscapedDash;
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = ParserState::ScriptDataEscapedLessThanSign;
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
            ParserState::ScriptDataDoubleEscapedDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.state = ParserState::ScriptDataDoubleEscapedDashDash;
                            self.emit(Token::Character('\u{002D}'));
                        }
                        '\u{003C}' => {
                            self.state = ParserState::ScriptDataDoubleEscapedLessThanSign;
                            self.emit(Token::Character('\u{003C}'));
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.state = ParserState::ScriptDataDoubleEscaped;
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.state = ParserState::ScriptDataDoubleEscaped;
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                    self.emit(Token::EOF);
                }
            }
            ParserState::ScriptDataDoubleEscapedDashDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-dash-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => self.emit(Token::Character('\u{002D}')),
                        '\u{003C}' => {
                            self.state = ParserState::ScriptDataDoubleEscapedLessThanSign;
                            self.emit(Token::Character('\u{003C}'));
                        }
                        '\u{003E}' => {
                            self.state = ParserState::ScriptData;
                            self.emit(Token::Character('\u{003E}'));
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.state = ParserState::ScriptDataDoubleEscaped;
                            self.emit(Token::Character('\u{FFFD}'));
                        }
                        _ => {
                            self.state = ParserState::ScriptDataDoubleEscaped;
                            self.emit(Token::Character(ch));
                        }
                    }
                } else {
                    self.error(ParseError::EOFInScriptHTMLCommentLikeText);
                    self.emit(Token::EOF);
                }
            }
            ParserState::ScriptDataDoubleEscapedLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escaped-less-than-sign-state
                if let Some(ch) = self.stream.consume()
                    && ch == '\u{002F}'
                {
                    self.temporary_buffer = String::new();
                    self.state = ParserState::ScriptDataDoubleEscapeEnd;
                    self.emit(Token::Character('\u{002F}'));
                } else {
                    self.reconsume(ParserState::ScriptDataDoubleEscaped);
                }
            }
            ParserState::ScriptDataDoubleEscapeEnd => {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-end-state
                let anything_else = |_self: &mut Parser<'a>| {
                    _self.reconsume(ParserState::ScriptDataDoubleEscaped);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '\u{002F}'
                        | '\u{003E}' => {
                            self.state = if self.temporary_buffer.as_str() == "script" {
                                ParserState::ScriptDataEscaped
                            } else {
                                ParserState::ScriptDataDoubleEscaped
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
            ParserState::BeforeAttributeName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
                let eof_or_002f_003e = |_self: &mut Parser<'a>| {
                    _self.reconsume(ParserState::AfterAttributeName);
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
                            self.state = ParserState::AttributeName;
                        }
                        _ => {
                            self.new_tag_attr(None);
                            self.reconsume(ParserState::AttributeName);
                        }
                    }
                } else {
                    eof_or_002f_003e(self);
                }
            }
            ParserState::AttributeName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state

                self.leave_callback = Some(Box::new(|_self: &mut Parser| {
                    let curr_tag_name = _self.curr_tag_attr_name();
                    let attr_names = _self.tag_attribute_names();

                    for name in attr_names.iter().take(attr_names.len() - 1) {
                        if curr_tag_name == name.clone() {
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
                let eof_or_0009 = |_self: &mut Parser<'a>| {
                    _self.reconsume(ParserState::AfterAttributeName);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' | '\u{002F}'
                        | '\u{0003E}' => eof_or_0009(self),
                        '\u{003D}' => self.state = ParserState::BeforeAttributeValue,
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
            ParserState::AfterAttributeName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{002F}' => self.state = ParserState::SelfClosingStartTag,
                        '\u{003D}' => self.state = ParserState::BeforeAttributeValue,
                        '\u{003E}' => {
                            self.state = ParserState::Data;
                            self.emit_tag();
                        }
                        _ => {
                            self.new_tag_attr(None);
                            self.reconsume(ParserState::AttributeName);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            ParserState::BeforeAttributeValue => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
                let anything_else = |_self: &mut Parser| {
                    _self.reconsume(ParserState::AttributeValueUnquoted);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{0022}' => self.state = ParserState::AttributeValueDoubleQuoted,
                        '\u{0027}' => self.state = ParserState::AttributeValueSingleQuoted,
                        '\u{003E}' => {
                            self.error(ParseError::MissingAttributeValue);
                            self.state = ParserState::Data;
                            self.emit_tag();
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            ParserState::AttributeValueDoubleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0022}' => self.state = ParserState::AfterAttributeValueQuoted,
                        '\u{0026}' => {
                            self.return_state = Some(ParserState::AttributeValueDoubleQuoted);
                            self.state = ParserState::CharacterReference;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);
                            self.push_to_attr_name('\u{FFFD}');
                        }
                        _ => {
                            self.push_to_attr_val(ch);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            ParserState::AttributeValueSingleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0027}' => self.state = ParserState::AfterAttributeValueQuoted,
                        '\u{0026}' => {
                            self.return_state = Some(ParserState::AttributeValueSingleQuoted);
                            self.state = ParserState::CharacterReference;
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
            ParserState::AttributeValueUnquoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = ParserState::BeforeAttributeName
                        }
                        '\u{0026}' => {
                            self.return_state = Some(ParserState::AttributeValueUnquoted);
                            self.state = ParserState::CharacterReference;
                        }
                        '\u{003E}' => {
                            self.state = ParserState::Data;
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
            ParserState::AfterAttributeValueQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-quoted-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = ParserState::BeforeAttributeName
                        }
                        '\u{002F}' => self.state = ParserState::SelfClosingStartTag,
                        '\u{003E}' => {
                            self.state = ParserState::Data;
                            self.emit_tag();
                        }
                        _ => {
                            self.error(ParseError::MissingWhitespaceBetweenAttributes);
                            self.reconsume(ParserState::BeforeAttributeName);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            ParserState::SelfClosingStartTag => {
                // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
                if let Some(ch) = self.stream.consume() {
                    if ch == '\u{003E}' {
                        self.tag_set_self_closing();
                        self.state = ParserState::Data;
                        self.emit_tag();
                    } else {
                        self.error(ParseError::UnexpectedSolidusInTag);
                        self.reconsume(ParserState::BeforeAttributeName);
                    }
                } else {
                    self.error(ParseError::EOFInTag);
                    self.emit(Token::EOF);
                }
            }
            ParserState::BogusComment => {
                // https://html.spec.whatwg.org/multipage/parsing.html#bogus-comment-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003E}' => {
                            self.state = ParserState::Data;
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
            ParserState::MarkupDeclarationOpen => {
                // https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
                // what the actual FUCK is this state
                if self.stream.matches("--", None, Some(true)) {
                    _ = self.stream.consume();
                    _ = self.stream.consume();

                    self.comment_token = Some(String::new());
                    self.state = ParserState::CommentStart;
                } else if self.stream.matches("DOCTYPE", Some(false), Some(true)) {
                    for _i in 0.."DOCTYPE".len() {
                        _ = self.stream.consume();
                    }
                    self.state = ParserState::DOCTYPE;
                } else if self.stream.matches("[CDATA[", Some(true), Some(true)) {
                    for _i in 0.."[CDATA[".len() {
                        _ = self.stream.consume();
                    }

                    // TODO: If there is an adjusted current node and it is not an element in the
                    // HTML namespace, then switch to the CDATA section state.
                    // Otherwise, this is a cdata-in-html-content parse error.

                    self.comment_token = Some(String::from("[CDATA["));
                    self.state = ParserState::BogusComment;
                } else {
                    // NOTE: Nothing is consumed via this state - this is intended!

                    self.error(ParseError::IncorrectlyOpenedComment);
                    self.comment_token = Some(String::new());
                    self.state = ParserState::BogusComment;
                }
            }
            ParserState::CommentStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-state
                let anything_else = |_self: &mut Parser| _self.reconsume(ParserState::Comment);

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => self.state = ParserState::CommentStartDash,
                        '\u{003E}' => {
                            self.error(ParseError::AbruptClosingOfEmptyComment);
                            self.state = ParserState::Data;
                            self.emit_comment();
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            ParserState::CommentStartDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => self.state = ParserState::CommentEnd,
                        '\u{003E}' => {
                            self.error(ParseError::AbruptClosingOfEmptyComment);
                            self.state = ParserState::Data;
                            self.emit_comment();
                        }
                        _ => {
                            self.push_to_comment('\u{002D}');
                            self.reconsume(ParserState::Comment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            ParserState::Comment => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003C}' => {
                            self.push_to_comment(ch);
                            self.state = ParserState::CommentLessThanSign;
                        }
                        '\u{002D}' => self.state = ParserState::CommentEndDash,
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
            ParserState::CommentLessThanSign => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-state
                let anything_else = |_self: &mut Parser| _self.reconsume(ParserState::Comment);

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0021}' => {
                            self.push_to_comment(ch);
                            self.state = ParserState::CommentLessThanSignBang;
                        }
                        '\u{003C}' => self.push_to_comment(ch),
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            ParserState::CommentLessThanSignBang => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-state
                if self.stream.consume().is_some_and(|c| c == '\u{002D}') {
                    self.state = ParserState::CommentLessThanSignBangDash;
                } else {
                    self.reconsume(ParserState::Comment);
                }
            }
            ParserState::CommentLessThanSignBangDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-state
                if self.stream.consume().is_some_and(|c| c == '\u{002D}') {
                    self.state = ParserState::CommentLessThanSignBangDashDash;
                } else {
                    self.reconsume(ParserState::CommentEndDash);
                }
            }
            ParserState::CommentLessThanSignBangDashDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-dash-state
                let consumed = self.stream.consume();

                if consumed.is_none() || consumed.is_some_and(|c| c == '\u{003E}') {
                    self.reconsume(ParserState::CommentEnd);
                } else {
                    self.error(ParseError::NestedComment);
                    self.reconsume(ParserState::CommentEnd);
                }
            }
            ParserState::CommentEndDash => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-dash-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => self.state = ParserState::CommentEnd,
                        _ => {
                            self.push_to_comment(ch);
                            self.reconsume(ParserState::Comment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            ParserState::CommentEnd => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003E}' => {
                            self.state = ParserState::Data;
                            self.emit_comment();
                        }
                        '\u{0021}' => self.state = ParserState::CommentEndBang,
                        '\u{002D}' => self.push_to_comment('\u{002D}'),
                        _ => {
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{002D}');
                            self.reconsume(ParserState::Comment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            ParserState::CommentEndBang => {
                // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-bang-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{002D}' => {
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{0021}');

                            self.state = ParserState::CommentEndDash;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::IncorrectlyClosedComment);
                            self.state = ParserState::Data;
                            self.emit_comment();
                        }
                        _ => {
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{002D}');
                            self.push_to_comment('\u{0021}');

                            self.reconsume(ParserState::Comment);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInComment);
                    self.emit_comment();
                    self.emit(Token::EOF);
                }
            }
            ParserState::DOCTYPE => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = ParserState::BeforeDOCTYPEName
                        }
                        '\u{003E}' => self.reconsume(ParserState::BeforeDOCTYPEName),
                        _ => {
                            self.error(ParseError::MissingWhitespaceBeforeDOCTYPEName);
                            self.reconsume(ParserState::BeforeDOCTYPEName);
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
            ParserState::BeforeDOCTYPEName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        _ if ch.is_ascii_uppercase() => {
                            self.doctype_token = Some(
                                DOCTYPE::default().with_name(ch.to_ascii_lowercase().to_string()),
                            );
                            self.state = ParserState::DOCTYPEName;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);

                            self.doctype_token =
                                Some(DOCTYPE::default().with_name('\u{FFFD}'.to_string()));
                            self.state = ParserState::DOCTYPEName;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPEName);

                            self.doctype_token = Some(DOCTYPE::default());
                            self.state = ParserState::Data;

                            self.set_doctype_quirks();
                            self.emit_doctype();
                        }
                        _ => {
                            self.doctype_token = Some(DOCTYPE::default().with_name(ch.to_string()));
                            self.state = ParserState::DOCTYPEName;
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
            ParserState::DOCTYPEName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = ParserState::AfterDOCTYPEName
                        }
                        '\u{003E}' => {
                            self.state = ParserState::Data;
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
            ParserState::AfterDOCTYPEName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{003E}' => {
                            self.state = ParserState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            if self.stream.matches("public", Some(false), None) {
                                for _i in 1.."public".len() {
                                    _ = self.stream.consume();
                                }

                                self.state = ParserState::AfterDOCTYPEPublicKeyword;
                            } else if self.stream.matches("system", Some(false), None) {
                                for _i in 1.."system".len() {
                                    _ = self.stream.consume();
                                }

                                self.state = ParserState::AfterDOCTYPESystemKeyword;
                            } else {
                                self.error(ParseError::InvalidCharacterSequenceAfterDOCTYPEName);
                                self.set_doctype_quirks();

                                self.reconsume(ParserState::BogusDOCTYPE);
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
            ParserState::AfterDOCTYPEPublicKeyword => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-keyword-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = ParserState::BeforeDOCTYPEPublicIdentifier;
                        }
                        '\u{0022}' => {
                            self.error(ParseError::MissingWhitespaceAfterDOCTYPEPublicKeyword);
                            self.doctype_token.as_mut().unwrap().public_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPEPublicIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.error(ParseError::MissingWhitespaceAfterDOCTYPEPublicKeyword);
                            self.doctype_token.as_mut().unwrap().public_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPEPublicIdentifierSingleQuoted;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.state = ParserState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(ParserState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            ParserState::BeforeDOCTYPEPublicIdentifier => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-public-identifier-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{0022}' => {
                            self.doctype_token.as_mut().unwrap().public_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPEPublicIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.doctype_token.as_mut().unwrap().public_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPEPublicIdentifierSingleQuoted;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.state = ParserState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPEPublicIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(ParserState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            ParserState::DOCTYPEPublicIdentifierDoubleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(double-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0022}' => self.state = ParserState::AfterDOCTYPEPublicIdentifier,
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

                            self.state = ParserState::Data;
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
            ParserState::DOCTYPEPublicIdentifierSingleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(single-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0027}' => self.state = ParserState::AfterDOCTYPEPublicIdentifier,
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

                            self.state = ParserState::Data;
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
            ParserState::AfterDOCTYPEPublicIdentifier => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-identifier-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = ParserState::BetweenDOCTYPEPublicAndSystemIdentifiers;
                        }
                        '\u{003E}' => {
                            self.state = ParserState::Data;
                            self.emit_doctype();
                        }
                        '\u{0022}' => {
                            self.error(
                                ParseError::MissingWhitespaceBetweenDOCTYPEPublicAndSystemIdentifiers,
                            );
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPESystemIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.error(
                                ParseError::MissingWhitespaceBetweenDOCTYPEPublicAndSystemIdentifiers,
                            );
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPESystemIdentifierSingleQuoted;
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(ParserState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            ParserState::BetweenDOCTYPEPublicAndSystemIdentifiers => {
                // https://html.spec.whatwg.org/multipage/parsing.html#between-doctype-public-and-system-identifiers-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{003E}' => {
                            self.state = ParserState::Data;
                            self.emit_doctype();
                        }
                        '\u{0022}' => {
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPESystemIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPESystemIdentifierSingleQuoted;
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(ParserState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            ParserState::AfterDOCTYPESystemKeyword => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-keyword-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {
                            self.state = ParserState::BeforeDOCTYPESystemIdentifier;
                        }
                        '\u{0022}' => {
                            self.error(ParseError::MissingWhitespaceAfterDOCTYPESystemKeyword);
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPESystemIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.error(ParseError::MissingWhitespaceAfterDOCTYPESystemKeyword);
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPESystemIdentifierSingleQuoted;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.state = ParserState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(ParserState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            ParserState::BeforeDOCTYPESystemIdentifier => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-system-identifier-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{0022}' => {
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPESystemIdentifierDoubleQuoted;
                        }
                        '\u{0027}' => {
                            self.doctype_token.as_mut().unwrap().system_identifier =
                                Some(String::new());
                            self.state = ParserState::DOCTYPESystemIdentifierSingleQuoted;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.state = ParserState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            self.error(ParseError::MissingQuoteBeforeDOCTYPESystemIdentifier);
                            self.set_doctype_quirks();

                            self.reconsume(ParserState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            ParserState::DOCTYPESystemIdentifierDoubleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(double-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0022}' => self.state = ParserState::AfterDOCTYPESystemIdentifier,
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

                            self.state = ParserState::Data;
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
            ParserState::DOCTYPESystemIdentifierSingleQuoted => {
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-system-identifier-(single-quoted)-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0027}' => self.state = ParserState::AfterDOCTYPESystemIdentifier,
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

                            self.state = ParserState::Data;
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
            ParserState::AfterDOCTYPESystemIdentifier => {
                // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-system-identifier-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        '\u{003E}' => {
                            self.state = ParserState::Data;
                            self.emit_doctype();
                        }
                        _ => {
                            // NOTE: Does not set the doctype to quirks mode

                            self.error(ParseError::UnexpectedCharacterAfterDOCTYPESystemIdentifier);
                            self.reconsume(ParserState::BogusDOCTYPE);
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    self.set_doctype_quirks();

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            ParserState::BogusDOCTYPE => {
                // https://html.spec.whatwg.org/multipage/parsing.html#bogus-doctype-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{003E}' => {
                            self.state = ParserState::Data;
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
            ParserState::CDATASection => {
                // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{005D}' => {
                            self.state = ParserState::CDATASectionBracket;
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
            ParserState::CDATASectionBracket => {
                // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-bracket-state
                if self.stream.consume().is_some_and(|c| c == '\u{005D}') {
                    self.state = ParserState::CDATASectionEnd;
                } else {
                    self.emit(Token::Character('\u{005D}'));
                    self.reconsume(ParserState::CDATASection);
                }
            }
            ParserState::CDATASectionEnd => {
                // https://html.spec.whatwg.org/multipage/parsing.html#cdata-section-end-state
                let anything_else = |_self: &mut Parser| {
                    _self.emit(Token::Character('\u{005D}'));
                    _self.emit(Token::Character('\u{005D}'));
                    _self.reconsume(ParserState::CDATASection);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{005D}' => self.emit(Token::Character('\u{005D}')),
                        '\u{003E}' => self.state = ParserState::Data,
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            ParserState::CharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state
                self.temporary_buffer = String::new();
                self.temporary_buffer.push('\u{0026}');

                let anything_else = |_self: &mut Parser| {
                    _self.flush_consumed_as_char_ref();
                    let return_state = _self.return_state.clone().unwrap();
                    _self.reconsume(return_state);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0023}' => {
                            self.temporary_buffer.push(ch);
                            self.state = ParserState::NumericCharacterReference;
                        }
                        _ if ch.is_ascii_alphanumeric() => {
                            self.reconsume(ParserState::NamedCharacterReference);
                        }
                        _ => anything_else(self),
                    }
                } else {
                    anything_else(self);
                }
            }
            ParserState::NamedCharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#named-character-reference-state
                let mut last_match: Option<(&'static str, &'static str)> = None;
                let mut consumed = String::new();

                loop {
                    let c = match self.stream.peek() {
                        Some(ch) => ch,
                        None => break,
                    };

                    self.temporary_buffer.push(c);

                    let mut lo = 0;
                    let mut hi = NAMED_CHARACTER_REFERENCES.len();

                    while lo < hi {
                        let mid = (lo + hi) / 2;
                        if NAMED_CHARACTER_REFERENCES[mid].0 < &self.temporary_buffer[1..] {
                            lo = mid + 1;
                        } else {
                            hi = mid;
                        }
                    }

                    if lo >= NAMED_CHARACTER_REFERENCES.len()
                        || !NAMED_CHARACTER_REFERENCES[lo]
                            .0
                            .starts_with(&self.temporary_buffer[1..])
                    {
                        self.temporary_buffer.pop();
                        break;
                    }

                    self.stream.consume();
                    consumed.push(c);

                    if NAMED_CHARACTER_REFERENCES[lo].0 == &self.temporary_buffer[1..] {
                        last_match = Some(NAMED_CHARACTER_REFERENCES[lo]);
                    }
                }

                println!("Exited with: {:?}", last_match);

                if let Some((name, value)) = last_match {
                    let ends_with_semicolon = name.ends_with(';');

                    if self.char_ref_as_part_of_attr()
                        && !ends_with_semicolon
                        && self
                            .stream
                            .peek()
                            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '=')
                    {
                        self.flush_consumed_as_char_ref();
                        self.state = self.return_state.clone().unwrap();
                    } else {
                        if !ends_with_semicolon {
                            self.error(ParseError::MissingSemicolonAfterCharacterReference);
                        }

                        self.temporary_buffer = String::from(value);
                        self.flush_consumed_as_char_ref();
                        self.state = self.return_state.clone().unwrap();
                    }
                } else {
                    self.flush_consumed_as_char_ref();
                    self.state = ParserState::AmbiguousAmpersand;
                }
            }
            ParserState::AmbiguousAmpersand => {
                // https://html.spec.whatwg.org/multipage/parsing.html#ambiguous-ampersand-state
                // TODO: This is only reachable through NamedCharacterReference
                match self.stream.consume() {
                    Some(ch) => match ch {
                        _ if ch.is_ascii_alphanumeric() => {
                            if self.char_ref_as_part_of_attr() {
                                self.push_to_attr_val(ch);
                            } else {
                                self.emit(Token::Character(ch));
                            }
                        }
                        '\u{003B}' => {
                            self.error(ParseError::UnknownNamedCharacterReference);
                            self.reconsume(self.return_state.clone().unwrap());
                        }
                        _ => self.reconsume(self.return_state.clone().unwrap()),
                    },
                    None => {
                        self.reconsume(self.return_state.clone().unwrap());
                    }
                }
            }
            ParserState::NumericCharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-state
                self.character_reference_code = 0;

                if self
                    .stream
                    .consume()
                    .is_some_and(|ch| matches!(ch, '\u{0078}' | '\u{0058}'))
                {
                    self.temporary_buffer.push(self.stream.current());
                    self.state = ParserState::HexadecimalCharacterReferenceStart;
                } else {
                    self.reconsume(ParserState::DecimalCharacterReferenceStart);
                }
            }
            ParserState::HexadecimalCharacterReferenceStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-start-state
                if self
                    .stream
                    .consume()
                    .is_some_and(|ch| ch.is_ascii_hexdigit())
                {
                    self.reconsume(ParserState::HexadecimalCharacterReference);
                } else {
                    self.error(ParseError::AbsenceOfDigitsInNumericCharacterReference);
                    self.flush_consumed_as_char_ref();
                    self.reconsume(self.return_state.clone().unwrap());
                }
            }
            ParserState::DecimalCharacterReferenceStart => {
                // https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-start-state
                if self.stream.consume().is_some_and(|ch| ch.is_ascii_digit()) {
                    self.reconsume(ParserState::DecimalCharacterReference);
                } else {
                    self.error(ParseError::AbsenceOfDigitsInNumericCharacterReference);
                    self.flush_consumed_as_char_ref();
                    self.reconsume(self.return_state.clone().unwrap());
                }
            }
            ParserState::HexadecimalCharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-state
                let anything_else = |_self: &mut Parser| {
                    _self.error(ParseError::MissingSemicolonAfterCharacterReference);
                    _self.reconsume(ParserState::NumericCharacterReferenceEnd);
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
                            self.state = ParserState::NumericCharacterReferenceEnd;
                        }
                        _ => {
                            anything_else(self);
                        }
                    }
                } else {
                    anything_else(self);
                }
            }
            ParserState::DecimalCharacterReference => {
                // https://html.spec.whatwg.org/multipage/parsing.html#decimal-character-reference-state
                let anything_else = |_self: &mut Parser| {
                    _self.error(ParseError::MissingSemicolonAfterCharacterReference);
                    _self.reconsume(ParserState::NumericCharacterReferenceEnd);
                };

                if let Some(ch) = self.stream.consume() {
                    match ch {
                        _ if ch.is_ascii_digit() => {
                            self.character_reference_code *= 10;
                            self.character_reference_code += ch as u32 - 0x30;
                        }
                        '\u{003B}' => {
                            self.state = ParserState::NumericCharacterReferenceEnd;
                        }
                        _ => {
                            anything_else(self);
                        }
                    }
                } else {
                    anything_else(self);
                }
            }
            ParserState::NumericCharacterReferenceEnd => {
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
