use super::error::*;

//================================================================

#[derive(Debug, Clone)]
pub enum Token {
    String(String),
    Function,
    Let,
    ParenthesisBegin,
    ParenthesisClose,
    BracketBegin,
    BracketClose,
    Colon,
    Comma,
    Assignment,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    String,
    Function,
    Let,
    ParenthesisBegin,
    ParenthesisClose,
    BracketBegin,
    BracketClose,
    Colon,
    Comma,
    Assignment,
}

impl Token {
    pub fn parse_line(line: &str, list: &mut Vec<Token>) {
        if line.is_empty() {
            return;
        }

        let mut line_buffer = LineBuffer::new(line.to_string());
        let mut inside_string = false;

        while let Some(character) = line_buffer.next() {
            match character {
                ' ' => {
                    if !inside_string {
                        if !line_buffer.is_empty() {
                            list.push(Self::parse_text(&line_buffer.clear()));
                        }
                    } else {
                        line_buffer.push(character);
                    }
                }
                '"' => {
                    if inside_string {
                        if !line_buffer.is_empty() {
                            list.push(Self::parse_text(&line_buffer.clear()));
                        }
                    }

                    inside_string = !inside_string;
                }
                '(' | ')' | ',' | ':' | '<' | '>' | '!' => {
                    //line_buffer.push(character);

                    if let Some(peek) = line_buffer.peek() {
                        let assignment = character == ':' && peek == '=';
                        let ge_than = character == '>' && peek == '=';
                        let le_than = character == '<' && peek == '=';

                        if assignment || ge_than || le_than {
                            line_buffer.push(character);
                            line_buffer.next();
                            line_buffer.push(peek);
                            list.push(Self::parse_text(&line_buffer.clear()));
                            continue;
                        }
                    }

                    if !line_buffer.is_empty() {
                        list.push(Self::parse_text(&line_buffer.clear()));
                    }

                    list.push(Self::parse_text(&character.to_string()));
                }
                _ => line_buffer.push(character),
            }
        }

        if !line_buffer.is_empty() {
            list.push(Self::parse_text(&line_buffer.clear()));
        }
    }

    pub fn inner_string(&self) -> String {
        match self {
            Token::String(text) => text.clone(),
            _ => panic!(
                "Internal Alicia error: inner_string() on a token that was thought to be a Token::String token."
            ),
        }
    }

    fn parse_text(text: &str) -> Self {
        match text {
            "function" => Self::Function,
            "let" => Self::Let,
            "(" => Self::ParenthesisBegin,
            ")" => Self::ParenthesisClose,
            "{" => Self::BracketBegin,
            "}" => Self::BracketClose,
            ":" => Self::Colon,
            "," => Self::Comma,
            ":=" => Self::Assignment,
            _ => {
                Self::String(text.to_string())
                //if let Some(immediate) = Immediate::parse(text) {
                //    Self::Immediate(immediate)
                //} else {
                //    Self::String(text.to_string())
                //}
            }
        }
    }

    pub fn kind(&self) -> TokenKind {
        match self {
            Token::String(_) => TokenKind::String,
            Token::Function => TokenKind::Function,
            Token::Let => TokenKind::Let,
            Token::ParenthesisBegin => TokenKind::ParenthesisBegin,
            Token::ParenthesisClose => TokenKind::ParenthesisClose,
            Token::BracketBegin => TokenKind::BracketBegin,
            Token::BracketClose => TokenKind::BracketClose,
            Token::Colon => TokenKind::Colon,
            Token::Comma => TokenKind::Comma,
            Token::Assignment => TokenKind::Assignment,
        }
    }
}

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

    pub fn want_peek(&mut self, character: char) -> bool {
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
}

//================================================================

pub struct TokenBuffer {
    buffer: Vec<Token>,
    cursor: usize,
}

impl TokenBuffer {
    pub fn new(source: &str) -> Self {
        let mut buffer: Vec<Token> = Vec::new();

        for line in source.lines() {
            Token::parse_line(line, &mut buffer);
        }

        //println!("token list: {buffer:#?}");

        Self {
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

    pub fn want(&mut self, kind: TokenKind) -> Result<Token, AliciaError> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;

            if next.kind() == kind {
                return Ok(next.clone());
            } else {
                return Err(AliciaError::TokenError(TokenError::IncorrectKind(
                    kind,
                    next.kind(),
                )));
            }
        }

        Err(AliciaError::TokenError(TokenError::MissingKind(kind)))
    }

    pub fn want_peek(&mut self, kind: TokenKind) -> bool {
        if let Some(next) = self.buffer.get(self.cursor) {
            if next.kind() == kind {
                return true;
            }
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
