use crate::infra::{self, *};

#[derive(Debug, Clone)]
pub enum HashType {
    ID,
    Unrestricted,
}

#[derive(Debug, Clone)]
pub enum NumberType {
    Integer,
    Number,
}

/// https://www.w3.org/TR/css-syntax-3/#tokenization
#[derive(Debug, Clone)]
pub enum CSSToken {
    Ident(String),
    Function(String),
    AtKeyword(String),
    Hash {
        value: String,
        hash_type: HashType,
    },
    String(String),
    BadString,
    URL(String),
    BadURL,
    Delim(char),
    Number {
        value: f64,
        number_type: NumberType,
    },
    Percentage(f64),
    Dimension {
        value: f64,
        number_type: NumberType,
        unit: String,
    },
    Whitespace,
    CDO,
    CDC,
    Colon,
    Semicolon,
    Comma,
    LeftSquareBracket,
    RightSquareBracket,
    LeftParenthesis,
    RightParenthesis,
    LeftCurlyBracket,
    RightCurlyBracket,
    EOF,
}

pub fn preprocess(input: &String) -> String {
    let step_1 = input
        .replace("\r\n", "\n")
        .replace("\r", "\n")
        .replace("\u{000C}", "\n")
        .replace("\u{0000}", "\u{FFFD}");

    let mut result = String::new();

    for ch in step_1.chars() {
        if infra::is_surrogate(ch as u32) {
            result.push('\u{FFFD}');
        } else {
            result.push(ch);
        }
    }

    result
}

pub struct CSSParser {
    stream: InputStream,

    _tokens: Vec<CSSToken>,
}

impl CSSParser {
    pub fn new(data: String) -> CSSParser {
        let preprocessed_data = preprocess(&data);
        let stream = InputStream::new(preprocessed_data);

        CSSParser {
            stream,
            _tokens: Vec::new(),
        }
    }

    pub fn tokens(&self) -> &Vec<CSSToken> {
        &self._tokens
    }

    /// Assumes neither the first nor the second code point has been consumed.
    fn is_valid_escape(&mut self) -> bool {
        let first = self.stream.peek();
        let second = self.stream.peek_nth(1);

        is_valid_escape(first.unwrap_or('\0'), second)
    }

    fn consume_comments(&mut self) {
        if self.stream.matches("/*", None, Some(true)) {
            self.stream.consume();
            self.stream.consume();

            while !self.stream.matches("*/", None, None) && !self.stream.is_eof {
                self.stream.consume();
            }

            self.stream.consume();
        }
    }

    /// It assumes that the U+005C REVERSE SOLIDUS (\) has already been consumed and that the next input
    /// code point has already been verified to be part of a valid escape. It will return a code point.
    fn consume_escape(&mut self) -> char {
        match self.stream.consume() {
            Some(ch) if ch.is_ascii_hexdigit() => {
                let mut hex_digits = String::new();
                hex_digits.push(ch);

                for _ in 0..5 {
                    if let Some(next_ch) = self.stream.peek() {
                        if next_ch.is_ascii_hexdigit() {
                            hex_digits.push(self.stream.consume().unwrap());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if let Some(next_ch) = self.stream.peek() {
                    if next_ch.is_whitespace() {
                        self.stream.consume();
                    }
                }

                let code_point = u32::from_str_radix(&hex_digits, 16).unwrap();
                std::char::from_u32(code_point).unwrap_or('\u{FFFD}')
            }
            Some(ch) => ch,
            None => '\u{FFFD}',
        }
    }

    fn consume_string(&mut self, ending: Option<char>) -> CSSToken {
        let ending = ending.unwrap_or_else(|| self.stream.current());
        let mut token = String::new();

        loop {
            match self.stream.consume() {
                Some(ch) if ch == ending => return CSSToken::String(token),
                Some(ch) if ch == '\u{000A}' => {
                    self.stream.reconsume();
                    return CSSToken::BadString;
                }
                Some(ch) if ch == '\u{005C}' => match self.stream.peek() {
                    Some('\u{000A}') => {
                        self.stream.consume();
                    }
                    Some(_) => {
                        let escaped = self.consume_escape();
                        token.push(escaped);
                    }
                    None => return CSSToken::BadString,
                },
                Some(ch) => token.push(ch),
                None => return CSSToken::BadString,
            }
        }
    }

    fn consume_ident_seq(&mut self) -> String {
        let mut result = String::new();

        loop {
            match self.stream.consume() {
                Some(ch) if char_is_ident(ch) => result.push(ch),
                Some('\u{005C}') if is_valid_escape('\u{005C}', self.stream.peek()) => {
                    let escaped = self.consume_escape();
                    result.push(escaped);
                }
                _ => {
                    self.stream.reconsume();
                    return result;
                }
            }
        }
    }

    fn consume_number(&mut self) -> (f64, NumberType) {
        let mut repr = String::new();
        let mut number_type = NumberType::Integer;

        if self.stream.peek().is_some_and(|ch| ch == '+' || ch == '-') {
            repr.push(self.stream.consume().unwrap());
        }

        while self.stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            repr.push(self.stream.consume().unwrap());
        }

        if self.stream.peek() == Some('.') {
            if self.stream.peek_nth(2).is_some_and(|s| s.is_ascii_digit()) {
                repr.push(self.stream.consume().unwrap()); // consume '.'
                repr.push(self.stream.consume().unwrap()); // consume digit

                number_type = NumberType::Number;

                while self.stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                    repr.push(self.stream.consume().unwrap());
                }
            }

            if self.stream.peek().is_some_and(|ch| ch == 'e' || ch == 'E') {
                if self.stream.peek_nth(2).is_some_and(|c2| {
                    ((c2 == '+' || c2 == '-')
                        && self
                            .stream
                            .peek_nth(3)
                            .is_some_and(|c3| c3.is_ascii_digit()))
                        || c2.is_ascii_digit()
                }) {
                    repr.push(self.stream.consume().unwrap()); // consume 'e' or 'E'

                    if self.stream.peek().is_some_and(|ch| ch == '+' || ch == '-') {
                        repr.push(self.stream.consume().unwrap()); // consume '+' or '-'
                    }

                    while self.stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                        repr.push(self.stream.consume().unwrap());
                    }

                    number_type = NumberType::Number;
                }
            }
        }

        let number_value = repr.parse::<f64>().unwrap();
        (number_value, number_type)
    }

    fn consume_numeric(&mut self) -> CSSToken {
        let (number_value, number_type) = self.consume_number();

        if self
            .stream
            .peek_range(1, 3)
            .is_some_and(|s| would_start_ident(s))
        {
            let mut token = CSSToken::Dimension {
                value: number_value,
                number_type,
                unit: String::new(),
            };

            if let CSSToken::Dimension { unit, .. } = &mut token {
                *unit = self.consume_ident_seq();
            }

            return token;
        }

        if self.stream.peek() == Some('\u{0025}') {
            self.stream.consume();
            return CSSToken::Percentage(number_value);
        }

        return CSSToken::Number {
            value: number_value,
            number_type,
        };
    }

    fn consume_remnants_of_bad_url(&mut self) {
        loop {
            match self.stream.consume() {
                Some('\u{0029}') | None => return,
                Some('\u{005C}') if is_valid_escape('\u{005C}', self.stream.peek()) => {
                    self.consume_escape();
                }
                _ => {}
            }
        }
    }

    fn consume_url(&mut self) -> CSSToken {
        let mut result = String::new();

        while self.stream.peek().is_some_and(|ch| char_is_whitespace(ch)) {
            self.stream.consume();
        }

        loop {
            match self.stream.consume() {
                Some('\u{0029}') => return CSSToken::URL(result),
                None => return CSSToken::URL(result),
                Some(ch) if char_is_whitespace(ch) => {
                    while self.stream.peek().is_some_and(|ch| char_is_whitespace(ch)) {
                        self.stream.consume();
                    }

                    let peeked = self.stream.peek();

                    if peeked.is_some_and(|ch| ch == '\u{0029}') || peeked.is_none() {
                        self.stream.consume();
                        return CSSToken::URL(result);
                    } else {
                        self.consume_remnants_of_bad_url();
                        return CSSToken::BadURL;
                    }
                }
                Some('\u{0022}' | '\u{0027}' | '\u{0028}') => {
                    self.consume_remnants_of_bad_url();
                    return CSSToken::BadURL;
                }
                Some(ch) if char_is_non_printable(ch) => {
                    self.consume_remnants_of_bad_url();
                    return CSSToken::BadURL;
                }
                Some('\u{005C}') => {
                    if is_valid_escape('\u{005C}', self.stream.peek()) {
                        let escaped = self.consume_escape();
                        result.push(escaped);
                    } else {
                        self.consume_remnants_of_bad_url();
                        return CSSToken::BadURL;
                    }
                }
                Some(_) => {
                    result.push(self.stream.current());
                }
            }
        }
    }

    fn consume_ident_like(&mut self) -> CSSToken {
        let result = self.consume_ident_seq();

        if result.eq_ignore_ascii_case("url") && self.stream.peek().is_some_and(|c| c == '\u{0028}')
        {
            self.stream.consume();

            while self.stream.peek().is_some_and(|ch| char_is_whitespace(ch))
                && self
                    .stream
                    .peek_nth(2)
                    .is_some_and(|c| char_is_whitespace(c))
            {
                self.stream.consume();
            }

            if self.stream.peek().is_some_and(|ch| {
                ch == '\u{0022}'
                    || ch == '\u{0027}'
                    || (char_is_whitespace(ch)
                        && self
                            .stream
                            .peek_nth(2)
                            .is_some_and(|c| ch == '\u{0022}' || c == '\u{0027}'))
            }) {
                return CSSToken::Function(result);
            } else {
                return self.consume_url();
            }
        } else if self.stream.peek().is_some_and(|ch| ch == '\u{0028}') {
            self.stream.consume();
            return CSSToken::Function(result);
        }

        CSSToken::Ident(result)
    }

    fn consume(&mut self) -> CSSToken {
        self.consume_comments();

        match self.stream.consume() {
            Some(ch) => {
                match ch {
                    '\u{0020}' | '\u{0009}' | '\u{000A}' => {
                        // consume as many whitespace as possible
                        while self.stream.peek().is_some_and(|ch| char_is_whitespace(ch)) {
                            self.stream.consume();
                        }

                        return CSSToken::Whitespace;
                    }
                    '\u{0022}' => return self.consume_string(None),
                    '\u{0023}' => {
                        if self.stream.peek().is_some_and(|ch| char_is_ident(ch))
                            || self.is_valid_escape()
                        {
                            let mut hash = CSSToken::Hash {
                                value: String::new(),
                                hash_type: HashType::Unrestricted,
                            };

                            if self
                                .stream
                                .peek_range(1, 3)
                                .is_some_and(|s| would_start_ident(s))
                            {
                                if let CSSToken::Hash { hash_type, .. } = &mut hash {
                                    *hash_type = HashType::ID;
                                }
                            }

                            let value = self.consume_ident_seq();
                            return CSSToken::Hash {
                                value,
                                hash_type: match hash {
                                    CSSToken::Hash { hash_type, .. } => hash_type,
                                    _ => HashType::Unrestricted,
                                },
                            };
                        }

                        return CSSToken::Delim(ch);
                    }
                    '\u{0027}' => return self.consume_string(None),
                    '\u{0028}' => return CSSToken::LeftParenthesis,
                    '\u{0029}' => return CSSToken::RightParenthesis,
                    '\u{002B}' => {
                        if self.stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                            self.stream.reconsume();
                            return self.consume_numeric();
                        } else {
                            return CSSToken::Delim(ch);
                        }
                    }
                    '\u{002C}' => return CSSToken::Comma,
                    '\u{002D}' => {
                        if self.stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                            self.stream.reconsume();
                            return self.consume_numeric();
                        } else if self.stream.peek_range(1, 2).is_some_and(|s| {
                            s.chars().nth(0).unwrap_or('\0') == '\u{002D}'
                                && s.chars().nth(1).unwrap_or('\0') == '\u{003E}'
                        }) {
                            self.stream.consume();
                            self.stream.consume();

                            return CSSToken::CDC;
                        } else if self
                            .stream
                            .peek_range(0, 3)
                            .is_some_and(|s| would_start_ident(s))
                        {
                            self.stream.reconsume();
                            return self.consume_ident_like();
                        } else {
                            return CSSToken::Delim(ch);
                        }
                    }
                    '\u{002E}' => {
                        if self.stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                            self.stream.reconsume();
                            return self.consume_numeric();
                        } else {
                            return CSSToken::Delim(ch);
                        }
                    }
                    '\u{003A}' => return CSSToken::Colon,
                    '\u{003B}' => return CSSToken::Semicolon,
                    '\u{003C}' => {
                        if self
                            .stream
                            .peek_range(1, 3)
                            .is_some_and(|s| s == "\u{0021}\u{002D}\u{002D}")
                        {
                            self.stream.consume();
                            self.stream.consume();
                            self.stream.consume();

                            return CSSToken::CDO;
                        } else {
                            return CSSToken::Delim(ch);
                        }
                    }
                    '\u{0040}' => {
                        if self
                            .stream
                            .peek_range(0, 3)
                            .is_some_and(|s| would_start_ident(s))
                        {
                            let at_keyword = self.consume_ident_seq();
                            return CSSToken::AtKeyword(at_keyword);
                        } else {
                            return CSSToken::Delim(ch);
                        }
                    }
                    '\u{005B}' => return CSSToken::LeftSquareBracket,
                    '\u{005C}' => {
                        if is_valid_escape('\u{005C}', self.stream.peek()) {
                            self.stream.reconsume();
                            return self.consume_ident_like();
                        } else {
                            return CSSToken::Delim(ch);
                        }
                    }
                    '\u{005D}' => return CSSToken::RightSquareBracket,
                    '\u{007B}' => return CSSToken::LeftCurlyBracket,
                    '\u{007D}' => return CSSToken::RightCurlyBracket,
                    ch if ch.is_ascii_digit() => {
                        self.stream.reconsume();
                        return self.consume_numeric();
                    }
                    ch if char_is_ident_start(ch) => {
                        self.stream.reconsume();
                        return self.consume_ident_like();
                    }
                    _ => {
                        return CSSToken::Delim(ch);
                    }
                }
            }
            None => {
                return CSSToken::EOF;
            }
        }
    }

    pub fn tokenize(&mut self) {
        loop {
            let token = self.consume();
            self._tokens.push(token.clone());

            if let CSSToken::EOF = token {
                break;
            }
        }
    }

    pub fn parse(&mut self) {
        // Parsing logic goes here
    }
}
