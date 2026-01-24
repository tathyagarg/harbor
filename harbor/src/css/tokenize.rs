use std::{cell::RefCell, rc::Weak};

use crate::{
    css::r#box::Box,
    infra::{
        InputStream, char_is_ident, char_is_non_printable, char_is_whitespace, is_valid_escape,
        would_start_ident,
    },
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HashType {
    ID,
    Unrestricted,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NumberType {
    Integer,
    Number,
}

#[derive(Debug, Clone)]
pub struct HashToken {
    pub value: String,
    pub hash_type: HashType,
}

#[derive(Debug, Clone)]
pub struct Dimension {
    pub value: f64,
    pub number_type: NumberType,
    pub unit: String,
}

impl Dimension {
    pub fn resolve(&self, parents: &Vec<Weak<RefCell<Box>>>) -> f64 {
        match self.unit.as_str() {
            "px" => self.value,
            "%" => {
                if let Some(parent) = parents.last() {
                    if let Some(parent_box) = parent.upgrade() {
                        let parent_borrowed = parent_box.borrow();
                        return parent_borrowed._content_width * (self.value / 100.0);
                    }
                }

                self.value
            }
            "em" => {
                if let Some(parent) = parents.last() {
                    if let Some(parent_box) = parent.upgrade() {
                        let parent_borrowed = parent_box.borrow();
                        return parent_borrowed.get_font_size() * self.value;
                    }
                }

                self.value * 16.0
            }
            "rem" => {
                if let Some(root) = parents.first() {
                    if let Some(root_box) = root.upgrade() {
                        let root_borrowed = root_box.borrow();
                        return root_borrowed.get_font_size() * self.value;
                    }
                }

                self.value * 16.0
            }
            _ => self.value,
        }
    }
}

pub type Percentage = f64;

/// https://www.w3.org/TR/css-syntax-3/#tokenization
#[derive(Debug, Clone)]
pub enum CSSToken {
    Ident(String),
    Function(String),
    AtKeyword(String),
    Hash(HashToken),
    String(String),
    BadString,
    URL(String),
    BadURL,
    Delim(char),
    Number { value: f64, number_type: NumberType },
    Percentage(Percentage),
    Dimension(Dimension),
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

impl CSSToken {
    pub fn string_value(&self) -> String {
        match self {
            CSSToken::Ident(value)
            | CSSToken::Function(value)
            | CSSToken::AtKeyword(value)
            | CSSToken::Hash(HashToken { value, .. })
            | CSSToken::String(value)
            | CSSToken::URL(value) => value.clone(),
            _ => String::new(),
        }
    }
}

impl PartialEq for CSSToken {
    fn eq(&self, other: &Self) -> bool {
        if matches!(
            self,
            CSSToken::Number { .. } | CSSToken::Percentage(_) | CSSToken::Dimension { .. }
        ) && matches!(
            other,
            CSSToken::Number { .. } | CSSToken::Percentage(_) | CSSToken::Dimension { .. }
        ) {
            false
        } else if let CSSToken::Hash(HashToken {
            value: value_a,
            hash_type: hash_type_a,
        }) = self
            && let CSSToken::Hash(HashToken {
                value: value_b,
                hash_type: hash_type_b,
            }) = other
        {
            value_a == value_b && hash_type_a == hash_type_b
        } else if let CSSToken::String(value_a) = self
            && let CSSToken::String(value_b) = other
        {
            value_a == value_b
        } else if let CSSToken::Ident(value_a) = self
            && let CSSToken::Ident(value_b) = other
        {
            value_a == value_b
        } else if let CSSToken::Function(value_a) = self
            && let CSSToken::Function(value_b) = other
        {
            value_a == value_b
        } else if let CSSToken::URL(value_a) = self
            && let CSSToken::URL(value_b) = other
        {
            value_a == value_b
        } else if let CSSToken::AtKeyword(value_a) = self
            && let CSSToken::AtKeyword(value_b) = other
        {
            value_a == value_b
        } else if let CSSToken::Delim(value_a) = self
            && let CSSToken::Delim(value_b) = other
        {
            value_a == value_b
        } else {
            std::mem::discriminant(self) == std::mem::discriminant(other)
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

/// Assumes neither the first nor the second code point has been consumed.
fn stream_is_valid_escape(stream: &mut InputStream<char>) -> bool {
    let first = stream.peek();
    let second = stream.peek_nth(1);

    is_valid_escape(first.unwrap_or('\0'), second)
}

fn consume_comments(stream: &mut InputStream<char>) {
    if stream.matches("/*", None, Some(true)) {
        stream.consume();
        stream.consume();

        while !stream.matches("*/", None, None) && !stream.is_eof {
            stream.consume();
        }

        stream.consume();
    }
}

/// It assumes that the U+005C REVERSE SOLIDUS (\) has already been consumed and that the next input
/// code point has already been verified to be part of a valid escape. It will return a code point.
fn consume_escape(stream: &mut InputStream<char>) -> char {
    match stream.consume() {
        Some(ch) if ch.is_ascii_hexdigit() => {
            let mut hex_digits = String::new();
            hex_digits.push(ch);

            for _ in 0..5 {
                if let Some(next_ch) = stream.peek() {
                    if next_ch.is_ascii_hexdigit() {
                        hex_digits.push(stream.consume().unwrap());
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if let Some(next_ch) = stream.peek() {
                if next_ch.is_whitespace() {
                    stream.consume();
                }
            }

            let code_point = u32::from_str_radix(&hex_digits, 16).unwrap();
            std::char::from_u32(code_point).unwrap_or('\u{FFFD}')
        }
        Some(ch) => ch,
        None => '\u{FFFD}',
    }
}

fn consume_string(stream: &mut InputStream<char>, ending: Option<char>) -> CSSToken {
    let ending = ending.unwrap_or_else(|| stream.current());
    let mut token = String::new();

    loop {
        match stream.consume() {
            Some(ch) if ch == ending => return CSSToken::String(token),
            Some(ch) if ch == '\u{000A}' => {
                stream.reconsume();
                return CSSToken::BadString;
            }
            Some(ch) if ch == '\u{005C}' => match stream.peek() {
                Some('\u{000A}') => {
                    stream.consume();
                }
                Some(_) => {
                    let escaped = consume_escape(stream);
                    token.push(escaped);
                }
                None => return CSSToken::BadString,
            },
            Some(ch) => token.push(ch),
            None => return CSSToken::BadString,
        }
    }
}

fn consume_ident_seq(stream: &mut InputStream<char>) -> String {
    let mut result = String::new();

    loop {
        match stream.consume() {
            Some(ch) if char_is_ident(ch) => result.push(ch),
            Some('\u{005C}') if is_valid_escape('\u{005C}', stream.peek()) => {
                let escaped = consume_escape(stream);
                result.push(escaped);
            }
            _ => {
                stream.reconsume();
                return result;
            }
        }
    }
}

fn consume_number(stream: &mut InputStream<char>) -> (f64, NumberType) {
    let mut repr = String::new();
    let mut number_type = NumberType::Integer;

    if stream.peek().is_some_and(|ch| ch == '+' || ch == '-') {
        repr.push(stream.consume().unwrap());
    }

    while stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
        repr.push(stream.consume().unwrap());
    }

    if stream.peek() == Some('.') {
        if stream.peek_nth(1).is_some_and(|s| s.is_ascii_digit()) {
            repr.push(stream.consume().unwrap()); // consume '.'
            repr.push(stream.consume().unwrap()); // consume digit

            number_type = NumberType::Number;

            while stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                repr.push(stream.consume().unwrap());
            }
        }

        if stream.peek().is_some_and(|ch| ch == 'e' || ch == 'E') {
            if stream.peek_nth(1).is_some_and(|c2| {
                ((c2 == '+' || c2 == '-')
                    && stream.peek_nth(2).is_some_and(|c3| c3.is_ascii_digit()))
                    || c2.is_ascii_digit()
            }) {
                repr.push(stream.consume().unwrap()); // consume 'e' or 'E'

                if stream.peek().is_some_and(|ch| ch == '+' || ch == '-') {
                    repr.push(stream.consume().unwrap()); // consume '+' or '-'
                }

                while stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                    repr.push(stream.consume().unwrap());
                }

                number_type = NumberType::Number;
            }
        }
    }

    let number_value = repr.parse::<f64>().unwrap();
    (number_value, number_type)
}

fn consume_numeric(stream: &mut InputStream<char>) -> CSSToken {
    let (number_value, number_type) = consume_number(stream);

    if stream
        .peek_range(1, 3)
        .is_some_and(|s| would_start_ident(s))
    {
        let mut token = CSSToken::Dimension(Dimension {
            value: number_value,
            number_type,
            unit: String::new(),
        });

        if let CSSToken::Dimension(Dimension { unit, .. }) = &mut token {
            *unit = consume_ident_seq(stream);
        }

        return token;
    }

    if stream.peek() == Some('\u{0025}') {
        stream.consume();
        return CSSToken::Percentage(number_value);
    }

    return CSSToken::Number {
        value: number_value,
        number_type,
    };
}

fn consume_remnants_of_bad_url(stream: &mut InputStream<char>) {
    loop {
        match stream.consume() {
            Some('\u{0029}') | None => return,
            Some('\u{005C}') if is_valid_escape('\u{005C}', stream.peek()) => {
                consume_escape(stream);
            }
            _ => {}
        }
    }
}

fn consume_url(stream: &mut InputStream<char>) -> CSSToken {
    let mut result = String::new();

    while stream.peek().is_some_and(|ch| char_is_whitespace(ch)) {
        stream.consume();
    }

    loop {
        match stream.consume() {
            Some('\u{0029}') => return CSSToken::URL(result),
            None => return CSSToken::URL(result),
            Some(ch) if char_is_whitespace(ch) => {
                while stream.peek().is_some_and(|ch| char_is_whitespace(ch)) {
                    stream.consume();
                }

                let peeked = stream.peek();

                if peeked.is_some_and(|ch| ch == '\u{0029}') || peeked.is_none() {
                    stream.consume();
                    return CSSToken::URL(result);
                } else {
                    consume_remnants_of_bad_url(stream);
                    return CSSToken::BadURL;
                }
            }
            Some('\u{0022}' | '\u{0027}' | '\u{0028}') => {
                consume_remnants_of_bad_url(stream);
                return CSSToken::BadURL;
            }
            Some(ch) if char_is_non_printable(ch) => {
                consume_remnants_of_bad_url(stream);
                return CSSToken::BadURL;
            }
            Some('\u{005C}') => {
                if is_valid_escape('\u{005C}', stream.peek()) {
                    let escaped = consume_escape(stream);
                    result.push(escaped);
                } else {
                    consume_remnants_of_bad_url(stream);
                    return CSSToken::BadURL;
                }
            }
            Some(_) => {
                result.push(stream.current());
            }
        }
    }
}

fn consume_ident_like(stream: &mut InputStream<char>) -> CSSToken {
    let result = consume_ident_seq(stream);

    if result.eq_ignore_ascii_case("url") && stream.peek().is_some_and(|c| c == '\u{0028}') {
        stream.consume();

        while stream.peek().is_some_and(|ch| char_is_whitespace(ch))
            && stream.peek_nth(2).is_some_and(|c| char_is_whitespace(c))
        {
            stream.consume();
        }

        if stream.peek().is_some_and(|ch| {
            ch == '\u{0022}'
                || ch == '\u{0027}'
                || (char_is_whitespace(ch)
                    && stream
                        .peek_nth(2)
                        .is_some_and(|c| ch == '\u{0022}' || c == '\u{0027}'))
        }) {
            return CSSToken::Function(result);
        } else {
            return consume_url(stream);
        }
    } else if stream.peek().is_some_and(|ch| ch == '\u{0028}') {
        stream.consume();
        return CSSToken::Function(result);
    }

    CSSToken::Ident(result)
}

fn consume(stream: &mut InputStream<char>) -> CSSToken {
    consume_comments(stream);

    match stream.consume() {
        Some(ch) => {
            match ch {
                '\u{0020}' | '\u{0009}' | '\u{000A}' => {
                    // consume as many whitespace as possible
                    while stream.peek().is_some_and(|ch| char_is_whitespace(ch)) {
                        stream.consume();
                    }

                    return CSSToken::Whitespace;
                }
                '\u{0022}' => return consume_string(stream, None),
                '\u{0023}' => {
                    if stream.peek().is_some_and(|ch| char_is_ident(ch))
                        || stream_is_valid_escape(stream)
                    {
                        let mut hash = HashToken {
                            value: String::new(),
                            hash_type: HashType::Unrestricted,
                        };

                        if stream
                            .peek_range(1, 3)
                            .is_some_and(|s| would_start_ident(s))
                        {
                            hash.hash_type = HashType::ID;
                        }

                        let value = consume_ident_seq(stream);
                        return CSSToken::Hash(HashToken {
                            value,
                            hash_type: hash.hash_type,
                        });
                    }

                    return CSSToken::Delim(ch);
                }
                '\u{0027}' => return consume_string(stream, None),
                '\u{0028}' => return CSSToken::LeftParenthesis,
                '\u{0029}' => return CSSToken::RightParenthesis,
                '\u{002B}' => {
                    if stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                        stream.reconsume();
                        return consume_numeric(stream);
                    } else {
                        return CSSToken::Delim(ch);
                    }
                }
                '\u{002C}' => return CSSToken::Comma,
                '\u{002D}' => {
                    if stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                        stream.reconsume();
                        return consume_numeric(stream);
                    } else if stream
                        .peek_range(1, 2)
                        .is_some_and(|s| s == &['\u{002D}', '\u{003E}'])
                    {
                        stream.consume();
                        stream.consume();

                        return CSSToken::CDC;
                    } else if stream
                        .peek_range(0, 3)
                        .is_some_and(|s| would_start_ident(s))
                    {
                        stream.reconsume();
                        return consume_ident_like(stream);
                    } else {
                        return CSSToken::Delim(ch);
                    }
                }
                '\u{002E}' => {
                    if stream.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                        stream.reconsume();
                        return consume_numeric(stream);
                    } else {
                        return CSSToken::Delim(ch);
                    }
                }
                '\u{003A}' => return CSSToken::Colon,
                '\u{003B}' => return CSSToken::Semicolon,
                '\u{003C}' => {
                    if stream
                        .peek_range(1, 3)
                        .is_some_and(|s| s == &['\u{0021}', '\u{002D}', '\u{002D}'])
                    {
                        stream.consume();
                        stream.consume();
                        stream.consume();

                        return CSSToken::CDO;
                    } else {
                        return CSSToken::Delim(ch);
                    }
                }
                '\u{0040}' => {
                    if stream
                        .peek_range(0, 3)
                        .is_some_and(|s| would_start_ident(s))
                    {
                        let at_keyword = consume_ident_seq(stream);
                        return CSSToken::AtKeyword(at_keyword);
                    } else {
                        return CSSToken::Delim(ch);
                    }
                }
                '\u{005B}' => return CSSToken::LeftSquareBracket,
                '\u{005C}' => {
                    if is_valid_escape('\u{005C}', stream.peek()) {
                        stream.reconsume();
                        return consume_ident_like(stream);
                    } else {
                        return CSSToken::Delim(ch);
                    }
                }
                '\u{005D}' => return CSSToken::RightSquareBracket,
                '\u{007B}' => return CSSToken::LeftCurlyBracket,
                '\u{007D}' => return CSSToken::RightCurlyBracket,
                ch if ch.is_ascii_digit() => {
                    stream.reconsume();
                    return consume_numeric(stream);
                }
                ch if char_is_ident(ch) => {
                    stream.reconsume();
                    return consume_ident_like(stream);
                }
                _ => {
                    return CSSToken::Delim(ch);
                }
            }
        }
        None => {
            println!("End of file reached.");
            return CSSToken::EOF;
        }
    }
}

pub fn tokenize(stream: &mut InputStream<char>) -> Vec<CSSToken> {
    println!("Starting tokenization...");
    let mut tokens = Vec::new();

    loop {
        let token = consume(stream);
        tokens.push(token.clone());

        if let CSSToken::EOF = token {
            break;
        }
    }

    tokens
}

pub fn tokenize_from_string(input: String) -> Vec<CSSToken> {
    let char_slice = input.chars().collect::<Vec<char>>();
    let slice = &char_slice[..];

    let mut stream = InputStream::new(slice);
    tokenize(&mut stream)
}
