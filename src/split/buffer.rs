use super::error::*;
use super::token::*;
use super::utility::*;

//================================================================

pub struct LineBuffer {
    buffer_line: String,
    buffer_text: String,
    cursor: usize,
}

impl LineBuffer {
    pub fn new(buffer: String) -> Self {
        Self {
            buffer_line: buffer,
            buffer_text: String::default(),
            cursor: usize::default(),
        }
    }

    pub fn next(&mut self) -> Option<char> {
        if let Some(next) = self.buffer_line.chars().nth(self.cursor) {
            self.cursor += 1;
            return Some(next);
        }

        None
    }

    fn want_peek(&mut self, character: char) -> bool {
        if let Some(next) = self.buffer_line.chars().nth(self.cursor) {
            if next == character {
                return true;
            }
        }

        false
    }

    pub fn peek(&self) -> Option<char> {
        if let Some(next) = self.buffer_line.chars().nth(self.cursor) {
            return Some(next);
        }

        None
    }

    pub fn push(&mut self, character: char) {
        self.buffer_text.push(character);
    }

    pub fn clear(&mut self) -> String {
        let clone = self.buffer_text.clone();
        self.buffer_text.clear();

        clone
    }

    pub fn is_empty(&self) -> bool {
        self.buffer_text.is_empty()
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }
}

//================================================================

pub struct TokenBuffer {
    source: Source,
    buffer: Vec<Token>,
    cursor: usize,
}

impl TokenBuffer {
    pub fn new(source: Source) -> Self {
        let mut buffer: Vec<Token> = Vec::new();

        for (i, line) in source.data.lines().enumerate() {
            Token::parse_line(line, i, &mut buffer);
        }

        println!("token list: {buffer:#?}");

        Self {
            source,
            buffer,
            cursor: usize::default(),
        }
    }

    pub fn print_state(&self) {
        println!("{:?}", &self.buffer[self.cursor..self.buffer.len()]);
    }

    pub fn next(&mut self) -> Option<Token> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;
            return Some(next.clone());
        }

        None
    }

    pub fn want(&mut self, kind: TokenKind) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;

            if next.data.kind() == kind {
                return Ok(next.clone());
            } else {
                return Err(Error::IncorrectKind(kind, next.clone()));
            }
        }

        Err(Error::ExpectingKind(kind))
    }

    pub fn want_peek(&mut self, kind: TokenKind) -> bool {
        if let Some(next) = self.buffer.get(self.cursor)
            && next.data.kind() == kind
        {
            return true;
        }

        false
    }

    pub fn peek(&self) -> Option<Token> {
        if let Some(next) = self.buffer.get(self.cursor) {
            return Some(next.clone());
        }

        None
    }
}
