use crate::error::*;
use crate::helper::*;
use crate::token::*;

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
    pub begin: Option<Point>,
    pub close: Option<Point>,
}

impl TokenSpan {
    fn new() -> Self {
        Self {
            begin: None,
            close: None,
        }
    }

    fn add_point(&mut self, point: Point) {
        if self.begin.is_none() {
            self.begin = Some(point);
            self.close = Some(point);
        } else {
            self.close = Some(point);
        }
    }

    pub fn get_source(&self, source: &Source) -> Vec<String> {
        let begin = self.begin.unwrap();
        let close = self.close.unwrap();
        let line: Vec<String> = source.data.lines().map(|x| x.to_string()).collect();

        line[begin.y..close.y + 1].to_vec()
    }
}

pub struct TokenBuffer {
    pub source: Source,
    buffer: Vec<Token>,
    cursor: usize,
    span: Vec<TokenSpan>,
    hint: Option<ErrorHint>,
}

impl TokenBuffer {
    pub fn new(source: Source) -> Result<Self, Error> {
        let mut buffer: Vec<Token> = Vec::new();

        for (i, line) in source.data.lines().enumerate() {
            if let Err(kind) = Token::parse_line(line, i, &mut buffer) {
                return Error::new_kind(kind, None);
            }
        }

        Ok(Self {
            span: Vec::default(),
            source,
            buffer,
            cursor: usize::default(),
            hint: None,
        })
    }

    pub fn print_state(&self) {
        println!("{:#?}", &self.buffer[self.cursor..self.buffer.len()]);
    }

    pub fn push_span(&mut self) {
        self.span.push(TokenSpan::new());
    }

    pub fn get_span(&mut self) -> TokenSpan {
        self.span.pop().unwrap()
    }

    pub fn get_span_mutable(&mut self) -> &mut TokenSpan {
        self.span.last_mut().unwrap()
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
        self.push_span();
        self.hint = Some(hint);

        let result = call(self)?;

        self.hint = None;

        Ok(result)
    }

    pub fn want(&mut self, kind: TokenKind) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor).cloned() {
            self.get_span_mutable().add_point(next.point);
            self.cursor += 1;

            if next.class.kind() == kind {
                return Ok(next.clone());
            } else {
                return Error::new_info(
                    ErrorInfo::new_token(self.get_span(), Some(next.clone()), self.source.clone()),
                    ErrorKind::IncorrectTokenKind(kind, next.clone()),
                    self.hint,
                );
            }
        }

        Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous(), self.source.clone()),
            ErrorKind::ExpectingKind(kind),
            self.hint,
        )
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
        if let Some(next) = self.buffer.get(self.cursor).cloned() {
            self.get_span_mutable().add_point(next.point);
            self.cursor += 1;

            match &next.class {
                TokenClass::Identifier(identifier) => return Ok(identifier.clone()),
                _ => {
                    return Error::new_info(
                        ErrorInfo::new_token(
                            self.get_span(),
                            Some(next.clone()),
                            self.source.clone(),
                        ),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        self.hint,
                    );
                }
            }
        }

        Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous(), self.source.clone()),
            ErrorKind::ExpectingKind(TokenKind::String),
            self.hint,
        )
    }

    pub fn want_definition(&mut self) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor).cloned() {
            self.get_span_mutable().add_point(next.point);
            self.cursor += 1;

            match next.class.kind() {
                TokenKind::Definition
                | TokenKind::DefinitionAdd
                | TokenKind::DefinitionSubtract
                | TokenKind::DefinitionMultiply
                | TokenKind::DefinitionDivide => return Ok(next.clone()),
                _ => {
                    return Error::new_info(
                        ErrorInfo::new_token(
                            self.get_span(),
                            Some(next.clone()),
                            self.source.clone(),
                        ),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        self.hint,
                    );
                }
            }
        }

        Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous(), self.source.clone()),
            ErrorKind::ExpectingKind(TokenKind::String),
            self.hint,
        )
    }

    pub fn want_value(&mut self) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor).cloned() {
            self.get_span_mutable().add_point(next.point);
            self.cursor += 1;

            match next.class.kind() {
                TokenKind::Identifier
                | TokenKind::String
                | TokenKind::Integer
                | TokenKind::Decimal
                | TokenKind::Boolean
                | TokenKind::SquareBegin => return Ok(next.clone()),
                _ => {
                    return Error::new_info(
                        ErrorInfo::new_token(
                            self.get_span(),
                            Some(next.clone()),
                            self.source.clone(),
                        ),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        self.hint,
                    );
                }
            }
        }

        Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous(), self.source.clone()),
            // TO-DO expecting value
            ErrorKind::ExpectingKind(TokenKind::String),
            self.hint,
        )
    }

    pub fn peek_value(&mut self) -> Option<Token> {
        if let Some(next) = self.peek() {
            match next.class.kind() {
                TokenKind::Identifier
                | TokenKind::String
                | TokenKind::Integer
                | TokenKind::Decimal
                | TokenKind::Boolean
                | TokenKind::SquareBegin => return Some(next),
                _ => return None,
            }
        }

        None
    }

    pub fn want_operator(&mut self) -> Result<Token, Error> {
        if let Some(next) = self.buffer.get(self.cursor).cloned() {
            self.get_span_mutable().add_point(next.point);
            self.cursor += 1;

            match next.class.kind() {
                TokenKind::Add
                | TokenKind::Subtract
                | TokenKind::Multiply
                | TokenKind::Divide
                | TokenKind::Not
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::GT
                | TokenKind::LT
                | TokenKind::Equal
                | TokenKind::GTE
                | TokenKind::LTE
                | TokenKind::EqualNot
                | TokenKind::Dot
                | TokenKind::Ampersand
                | TokenKind::ParenthesisBegin
                | TokenKind::SquareBegin => {
                    return Ok(next.clone());
                }
                _ => {
                    return Error::new_info(
                        ErrorInfo::new_token(
                            self.get_span(),
                            Some(next.clone()),
                            self.source.clone(),
                        ),
                        ErrorKind::IncorrectTokenKind(TokenKind::String, next.clone()),
                        self.hint,
                    );
                }
            }
        }

        Error::new_info(
            ErrorInfo::new_token(self.get_span(), self.previous(), self.source.clone()),
            // TO-DO expecting operator
            ErrorKind::ExpectingKind(TokenKind::String),
            self.hint,
        )
    }

    pub fn peek_operator(&mut self) -> Option<Token> {
        if let Some(next) = self.peek() {
            match next.class.kind() {
                TokenKind::Add
                | TokenKind::Subtract
                | TokenKind::Multiply
                | TokenKind::Divide
                | TokenKind::Not
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::GT
                | TokenKind::LT
                | TokenKind::Equal
                | TokenKind::GTE
                | TokenKind::LTE
                | TokenKind::EqualNot
                | TokenKind::Dot
                | TokenKind::Ampersand
                | TokenKind::ParenthesisBegin
                | TokenKind::SquareBegin => return Some(next),
                _ => return None,
            }
        }

        None
    }

    pub fn get_error_info(&mut self, token: Option<Token>) -> ErrorInfo {
        ErrorInfo::new_token(self.get_span(), token, self.source.clone())
    }
}
