pub struct InputStream {
    input: Vec<char>,
    pos: usize,
    is_reconsume: bool,

    pub is_eof: bool,

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

    pub fn current(&self) -> char {
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

    pub fn consume(&mut self) -> Option<char> {
        if self.is_reconsume {
            self.is_reconsume = false;
            Some(self.current())
        } else {
            self.advance()
        }
    }

    pub fn reconsume(&mut self) {
        self.is_reconsume = true;
    }

    pub fn matches(
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
