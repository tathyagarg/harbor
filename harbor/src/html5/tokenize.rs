/// This is likely a temporary file and will be merged with some other code when I understand what
/// it is intended to integrate with. Until then, this is an independent implementation of an HTML5
/// parser.

fn preprocess_input(input: &String) -> String {
    input.replace("\r\n", "\n").replace("\r", "\n")
}

pub struct InputStream {
    input: Vec<char>,
    pos: usize,
    is_reconsume: bool,
    is_eof: bool,
}

impl InputStream {
    fn new(data: String) -> InputStream {
        InputStream {
            input: data.chars().collect::<Vec<char>>(),
            pos: 0,
            is_reconsume: false,
            is_eof: false,
        }
    }

    fn current(&self) -> char {
        self.input[self.pos]
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos + 1 > self.input.len() {
            self.is_eof = true;
            return None;
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

    fn matches(&self, text: &str, case_sensitive: Option<bool>) -> bool {
        let data_string = self.input[self.pos..self.pos + text.len() + 1]
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

#[derive(Clone)]
pub struct Tag {
    name: String,
    is_self_closing: bool,
    attributes: Vec<(String, String)>,
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

#[derive(Default, Clone)]
pub struct DOCTYPE {
    name: String,
    public_identifier: String,
    system_identifier: String,

    force_quirks: bool,
}

impl DOCTYPE {
    pub fn set_quirks(&mut self) {
        self.force_quirks = true;
    }
}

pub enum Token {
    DOCTYPE(DOCTYPE),
    StartTag(Tag),
    EndTag(Tag),
    Comment(String),
    Character(char),
    EOF,
}

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
}

#[derive(Clone, PartialEq)]
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
    CharacterReference = 72,
}

pub struct Tokenizer<'a> {
    stream: &'a mut InputStream,

    state: TokenizerState,
    prev_state: TokenizerState,

    leave_callback: Option<Box<dyn Fn(&mut Tokenizer)>>,

    return_state: Option<TokenizerState>,

    tag_token: Option<TagToken>,
    comment_token: Option<String>,
    doctype_token: Option<DOCTYPE>,

    temporary_buffer: String,

    emitted_tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new<'b>(stream: &'b mut InputStream) -> Tokenizer<'b> {
        Tokenizer {
            stream,
            state: TokenizerState::Data,
            prev_state: TokenizerState::Data,
            leave_callback: None,
            return_state: None,
            tag_token: None,
            comment_token: None,
            doctype_token: None,
            temporary_buffer: String::new(),
            emitted_tokens: vec![],
        }
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
        self.emitted_tokens.push(token);
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
        todo!()
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
            doctype.name.push(ch);
        }
    }

    pub fn tokenize(&mut self) {
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
                            self.push_to_comment('\u{FFFFD}');
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
                if self.stream.matches("--", None) {
                    _ = self.stream.consume();
                    _ = self.stream.consume();

                    self.comment_token = Some(String::new());
                    self.state = TokenizerState::CommentStart;
                } else if self.stream.matches("DOCTYPE", Some(false)) {
                    for _i in 0.."DOCTYPE".len() {
                        _ = self.stream.consume();
                    }
                    self.state = TokenizerState::DOCTYPE;
                } else if self.stream.matches("[CDATA[", Some(true)) {
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
                    let mut doc = DOCTYPE::default();
                    doc.set_quirks();

                    self.doctype_token = Some(doc);
                    self.emit_doctype();
                }
            }
            TokenizerState::BeforeDOCTYPEName => {
                // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state
                if let Some(ch) = self.stream.consume() {
                    match ch {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}' => {}
                        _ if ch.is_ascii_uppercase() => {
                            let mut token = DOCTYPE::default();
                            token.name.push(ch.to_ascii_lowercase());

                            self.doctype_token = Some(token);
                            self.state = TokenizerState::DOCTYPEName;
                        }
                        '\u{0000}' => {
                            self.error(ParseError::UnexpectedNullCharacter);

                            let mut token = DOCTYPE::default();
                            token.name.push('\u{FFFD}');

                            self.doctype_token = Some(token);
                            self.state = TokenizerState::DOCTYPEName;
                        }
                        '\u{003E}' => {
                            self.error(ParseError::MissingDOCTYPEName);

                            let mut token = DOCTYPE::default();
                            token.set_quirks();

                            self.doctype_token = Some(token);
                            self.state = TokenizerState::Data;

                            self.emit_doctype();
                        }
                        _ => {
                            let mut token = DOCTYPE::default();
                            token.name.push(ch);

                            self.doctype_token = Some(token);
                            self.state = TokenizerState::DOCTYPEName;
                        }
                    }
                } else {
                    self.error(ParseError::EOFInDOCTYPE);
                    let mut token = DOCTYPE::default();
                    token.set_quirks();

                    self.doctype_token = Some(token);

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
                    if let Some(doctype) = &mut self.doctype_token {
                        doctype.set_quirks();
                    }

                    self.emit_doctype();
                    self.emit(Token::EOF);
                }
            }
            _ => {}
        }
    }
}
