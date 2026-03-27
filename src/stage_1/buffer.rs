use super::helper::*;
use super::token::*;
use crate::helper::error::*;

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

        Self {
            source,
            buffer,
            cursor: usize::default(),
        }
    }

    pub fn print_state(&self) {
        println!("{:?}", &self.buffer[self.cursor..self.buffer.len()]);
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor
    }

    pub fn next(&mut self) -> Option<Token> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;
            return Some(next.clone());
        }

        None
    }

    pub fn previous(&self) -> Option<Token> {
        if let Some(previous) = self.buffer.get(self.cursor.saturating_sub(1)) {
            return Some(previous.clone());
        }

        None
    }

    pub fn want(&mut self, kind: TokenKind, hint: ErrorHint) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;

            if next.class.kind() == kind {
                return Ok(next.clone());
            } else {
                return Err(Error::new_info(
                    ErrorInfo::new(self.source.clone(), Some(next.clone())),
                    ErrorKind::IncorrectTokenKind(kind, next.clone()),
                    Some(hint),
                ));
            }
        }

        return Err(Error::new_info(
            ErrorInfo::new(self.source.clone(), self.previous()),
            ErrorKind::ExpectingKind(kind),
            Some(hint),
        ));
    }

    pub fn want_peek(&mut self, kind: TokenKind) -> bool {
        if let Some(next) = self.buffer.get(self.cursor)
            && next.class.kind() == kind
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

    pub fn peek_ahead(&self, ahead: usize) -> Option<Token> {
        if let Some(next) = self.buffer.get(self.cursor + ahead) {
            return Some(next.clone());
        }

        None
    }

    pub fn want_identifier(&mut self, hint: ErrorHint) -> Result<Identifier, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;

            if next.class.kind() == TokenKind::String {
                match next.class.inner_string().try_into() {
                    Ok(identifier) => return Ok(identifier),
                    Err(error) => {
                        return Err(Error::new_info(
                            ErrorInfo::new(self.source.clone(), Some(next.clone())),
                            error,
                            Some(hint),
                        ));
                    }
                }
            } else {
                return Err(Error::new_info(
                    ErrorInfo::new(self.source.clone(), Some(next.clone())),
                    ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                    Some(hint),
                ));
            };
        }

        return Err(Error::new_info(
            ErrorInfo::new(self.source.clone(), self.previous()),
            ErrorKind::ExpectingKind(TokenKind::String),
            Some(hint),
        ));
    }

    pub fn want_definition(&mut self, hint: ErrorHint) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;

            match next.class.kind() {
                TokenKind::Definition
                | TokenKind::DefinitionAdd
                | TokenKind::DefinitionSubtract
                | TokenKind::DefinitionMultiply
                | TokenKind::DefinitionDivide => return Ok(next.clone()),
                _ => {
                    return Err(Error::new_info(
                        ErrorInfo::new(self.source.clone(), Some(next.clone())),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        Some(hint),
                    ));
                }
            }
        }

        return Err(Error::new_info(
            ErrorInfo::new(self.source.clone(), self.previous()),
            ErrorKind::ExpectingKind(TokenKind::String),
            Some(hint),
        ));
    }

    pub fn get_error_info(&self, token: Option<Token>) -> ErrorInfo {
        ErrorInfo::new(self.source.clone(), token)
    }
}
