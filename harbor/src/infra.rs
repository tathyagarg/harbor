use std::fmt::Debug;

pub fn is_leading_surrogate(code: u32) -> bool {
    (0xD800..=0xDBFF).contains(&code)
}

pub fn is_trailing_surrogate(code: u32) -> bool {
    (0xDC00..=0xDFFF).contains(&code)
}

pub fn is_surrogate(code: u32) -> bool {
    is_leading_surrogate(code) || is_trailing_surrogate(code)
}

pub fn char_is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || !ch.is_ascii() || ch == '\u{005F}'
}

pub fn char_is_ident(ch: char) -> bool {
    char_is_ident_start(ch) || ch.is_ascii_digit() || ch == '\u{002D}'
}

pub fn is_valid_escape(first: char, second: Option<char>) -> bool {
    if first != '\u{005C}' {
        return false;
    }

    match second {
        Some(second_ch) if second_ch != '\u{000A}' => true,
        _ => false,
    }
}

pub fn would_start_ident(chars: &[char]) -> bool {
    if chars.is_empty() {
        return false;
    }

    match chars[0] {
        '\u{002D}' => {
            if chars.len() < 2 {
                return false;
            }
            match chars[1] {
                second if char_is_ident_start(second) || second == '\u{002D}' => true,
                second
                    if is_valid_escape(
                        second,
                        if chars.len() < 3 {
                            None
                        } else {
                            Some(chars[2])
                        },
                    ) =>
                {
                    true
                }
                _ => false,
            }
        }
        first if char_is_ident_start(first) => true,
        '\u{005C}' => {
            return is_valid_escape(
                '\u{005C}',
                if chars.len() < 2 {
                    None
                } else {
                    Some(chars[1])
                },
            );
        }
        _ => false,
    }
}

pub fn char_is_whitespace(ch: char) -> bool {
    matches!(ch, '\u{0009}' | '\u{000A}' | '\u{0020}')
}

pub fn char_is_non_printable(ch: char) -> bool {
    let code = ch as u32;
    // implicit code > 0x0000
    (code <= 0x0008)
        || code == 0x000B
        || (code >= 0x000E && code <= 0x001F)
        || code == 0x007F
        || (code >= 0x0080 && code <= 0x009F)
        || (code >= 0xFDD0 && code <= 0xFDEF)
        || (code & 0xFFFE) == 0xFFFE && code >= 0xFFFE && code <= 0x10FFFF
}

#[derive(Clone)]
pub struct InputStream<T> {
    input: Vec<T>,
    pos: usize,
    is_reconsume: bool,

    pub is_eof: bool,

    is_started: bool,
}

impl<T> InputStream<T>
where
    T: Clone,
{
    pub fn new(data: &[T]) -> InputStream<T> {
        InputStream {
            input: data.to_vec(),
            pos: 0,
            is_reconsume: false,
            is_eof: false,
            is_started: false,
        }
    }

    pub fn current(&self) -> T {
        self.input[self.pos].clone()
    }

    pub fn peek(&self) -> Option<T> {
        if self.is_reconsume {
            return Some(self.current());
        }

        if !self.is_started {
            return Some(self.current());
        }

        if self.pos + 1 >= self.input.len() {
            return None;
        }

        Some(self.input[self.pos + 1].clone())
    }

    pub fn peek_nth(&self, n: usize) -> Option<T> {
        let diff = if self.is_reconsume { 0 } else { 1 };

        if !self.is_started {
            if self.pos + diff + n - 1 >= self.input.len() {
                return None;
            }

            return Some(self.input[self.pos + diff + n - 1].clone());
        }

        if self.pos + diff + n >= self.input.len() {
            return None;
        }

        Some(self.input[self.pos + diff + n].clone())
    }

    pub fn peek_range(&self, start: usize, n: usize) -> Option<&[T]> {
        if self.pos + start + n >= self.input.len() {
            return None;
        }
        let data_string = &self.input[self.pos + start..self.pos + start + n];
        Some(data_string)
    }

    fn advance(&mut self) -> Option<T> {
        if !self.is_started {
            self.is_started = true;
            return Some(self.current());
        }

        if self.pos + 1 >= self.input.len() {
            self.is_eof = true;
            return None;
        }

        self.pos += 1;
        Some(self.current())
    }

    pub fn consume(&mut self) -> Option<T> {
        if self.is_eof {
            return None;
        }

        if self.is_reconsume {
            self.is_reconsume = false;
            Some(self.current())
        } else {
            self.advance()
        }
    }

    // pub fn push(&mut self, item: T) {
    //     self.input.push(item);
    // }

    pub fn reconsume(&mut self) {
        self.is_reconsume = true;
    }

    pub fn finish(&mut self) -> Vec<T> {
        self.is_eof = true;
        self.input.drain(self.pos..).collect()
    }
}

impl InputStream<char> {
    pub fn matches(
        &self,
        text: &str,
        case_sensitive: Option<bool>,
        start_from_next: Option<bool>,
    ) -> bool {
        let add = if start_from_next.unwrap_or(false) {
            if self.is_started { 1 } else { 0 }
        } else {
            0
        };

        if self.pos + text.len() + add > self.input.len() {
            return false;
        }

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

impl<T> Debug for InputStream<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.input.iter()).finish()
    }
}

pub trait Serializable {
    fn serialize(&self) -> String;
}
