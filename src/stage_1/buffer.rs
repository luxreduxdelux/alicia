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

#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub list: Vec<(String, usize)>,
    pub path: String,
    pub line: Option<Point>,
}

impl TokenSpan {
    fn new(path: String) -> Self {
        Self {
            list: Vec::default(),
            path,
            line: None,
        }
    }

    fn push(&mut self, source: &Source, token: &Token) {
        if let Some(line) = &mut self.line {
            if line.y != token.point.y {
                let line = source.data.lines().nth(token.point.y).unwrap();

                self.list.push((line.to_string(), token.point.y));
                self.line = Some(token.point);
            }
        } else {
            let line = source.data.lines().nth(token.point.y).unwrap();

            self.list.push((line.to_string(), token.point.y));
            self.line = Some(token.point);
        }
    }
}

pub struct TokenBuffer {
    source: Source,
    buffer: Vec<Token>,
    cursor: usize,
    span: TokenSpan,
    hint: Option<ErrorHint>,
}

impl TokenBuffer {
    pub fn new(source: Source) -> Result<Self, Error> {
        let mut buffer: Vec<Token> = Vec::new();

        for (i, line) in source.data.lines().enumerate() {
            if let Err(kind) = Token::parse_line(line, i, &mut buffer) {
                return Err(Error::new_kind(kind, None));
            }
        }

        Ok(Self {
            span: TokenSpan::new(source.path.clone()),
            source,
            buffer,
            cursor: usize::default(),
            hint: None,
        })
    }

    pub fn print_state(&self) {
        println!("{:#?}", &self.buffer[self.cursor..self.buffer.len()]);
    }

    pub fn get_span(&self) -> TokenSpan {
        self.span.clone()
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

    pub fn parse<T, F: FnMut(&mut Self) -> Result<T, Error>>(
        &mut self,
        hint: ErrorHint,
        mut call: F,
    ) -> Result<T, Error> {
        if hint == ErrorHint::Function
            || hint == ErrorHint::Structure
            || hint == ErrorHint::Enumerate
            || hint == ErrorHint::Definition
            || hint == ErrorHint::Assignment
        {
            self.span = TokenSpan::new(self.source.path.clone());
        }

        self.hint = Some(hint);

        let result = call(self)?;

        self.hint = None;

        Ok(result)
    }

    pub fn want(&mut self, kind: TokenKind) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.span.push(&self.source, next);
            self.cursor += 1;

            if next.class.kind() == kind {
                return Ok(next.clone());
            } else {
                return Err(Error::new_info(
                    ErrorInfo::new_token(self.get_span(), Some(next.clone())),
                    ErrorKind::IncorrectTokenKind(kind, next.clone()),
                    self.hint,
                ));
            }
        }

        Err(Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous()),
            ErrorKind::ExpectingKind(kind),
            self.hint,
        ))
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

    pub fn want_identifier(&mut self) -> Result<Identifier, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.span.push(&self.source, next);
            self.cursor += 1;

            match &next.class {
                TokenClass::Identifier(identifier) => return Ok(identifier.clone()),
                _ => {
                    return Err(Error::new_info(
                        ErrorInfo::new_token(self.get_span(), Some(next.clone())),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        self.hint,
                    ));
                }
            }
        }

        Err(Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous()),
            ErrorKind::ExpectingKind(TokenKind::String),
            self.hint,
        ))
    }

    pub fn want_definition(&mut self) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.span.push(&self.source, next);
            self.cursor += 1;

            match next.class.kind() {
                TokenKind::Definition
                | TokenKind::DefinitionAdd
                | TokenKind::DefinitionSubtract
                | TokenKind::DefinitionMultiply
                | TokenKind::DefinitionDivide => return Ok(next.clone()),
                _ => {
                    return Err(Error::new_info(
                        ErrorInfo::new_token(self.get_span(), Some(next.clone())),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        self.hint,
                    ));
                }
            }
        }

        Err(Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous()),
            ErrorKind::ExpectingKind(TokenKind::String),
            self.hint,
        ))
    }

    pub fn want_value(&mut self) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.span.push(&self.source, &next);
            self.cursor += 1;

            match next.class.kind() {
                TokenKind::Identifier
                | TokenKind::String
                | TokenKind::Integer
                | TokenKind::Decimal
                | TokenKind::Boolean => return Ok(next.clone()),
                _ => {
                    return Err(Error::new_info(
                        ErrorInfo::new_token(self.get_span(), Some(next.clone())),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        self.hint,
                    ));
                }
            }
        }

        Err(Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous()),
            // TO-DO expecting value
            ErrorKind::ExpectingKind(TokenKind::String),
            self.hint,
        ))
    }

    pub fn peek_value(&mut self) -> Option<Token> {
        if let Some(next) = self.peek() {
            match next.class.kind() {
                TokenKind::Identifier
                | TokenKind::String
                | TokenKind::Integer
                | TokenKind::Decimal
                | TokenKind::Boolean => return Some(next),
                _ => return None,
            }
        }

        None
    }

    pub fn want_operator(&mut self) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.span.push(&self.source, &next);
            self.cursor += 1;

            match next.class.kind() {
                TokenKind::Add | TokenKind::Subtract | TokenKind::Multiply | TokenKind::Divide => {
                    return Ok(next.clone());
                }
                _ => {
                    return Err(Error::new_info(
                        ErrorInfo::new_token(self.get_span(), Some(next.clone())),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        self.hint,
                    ));
                }
            }
        }

        Err(Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous()),
            // TO-DO expecting operator
            ErrorKind::ExpectingKind(TokenKind::String),
            self.hint,
        ))
    }

    #[rustfmt::skip]
    pub fn peek_operator(&mut self) -> Option<Token> {
        if let Some(next) = self.peek() {
            match next.class.kind() {
                TokenKind::Add      |
                TokenKind::Subtract |
                TokenKind::Multiply |
                TokenKind::Divide   => return Some(next),
                _ => return None,
            }
        }

        None
    }

    pub fn get_error_info(&self, token: Option<Token>) -> ErrorInfo {
        ErrorInfo::new_token(self.get_span(), token)
    }
}
