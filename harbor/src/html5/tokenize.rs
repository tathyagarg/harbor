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
}

pub enum TagToken {
    Start(Tag),
    End(Tag),
}

pub enum Token {
    DOCTYPE,
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
}

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
    BeforeAttributeName = 32,
    SelfClosingStartTag = 40,
    BogusComment = 41,
    MarkupDeclarationOpen = 42,
    CharacterReference = 72,
}

pub struct Tokenizer<'a> {
    stream: &'a mut InputStream,
    state: TokenizerState,
    return_state: Option<TokenizerState>,

    tag_token: Option<TagToken>,
    comment_token: Option<String>,
    temporary_buffer: String,

    emitted_tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new<'b>(stream: &'b mut InputStream) -> Tokenizer<'b> {
        Tokenizer {
            stream,
            state: TokenizerState::Data,
            return_state: None,
            tag_token: None,
            comment_token: None,
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

    fn emit_current_tag(&mut self) {
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
        if let Some(tag_tok) = &mut self.tag_token {
            match tag_tok {
                TagToken::Start(tag) => tag.name.push(ch),
                TagToken::End(tag) => tag.name.push(ch),
            }
        }
    }

    pub fn tokenize(&mut self) {
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
                            self.emit_current_tag();
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
                            self.emit_current_tag();
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
                            self.emit_current_tag();
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
                            self.emit_current_tag();
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
            _ => {}
        }
    }
}
