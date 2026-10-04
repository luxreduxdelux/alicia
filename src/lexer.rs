use std::matches;

//================================================================

use crate::error::*;

//================================================================

#[derive(Clone, Debug)]
pub struct TokenBuffer {
    name: String,
    pub token: Vec<Token>,
    index: usize,
    point: Point,
    context: Vec<Context>,
}

impl TokenBuffer {
    fn new(name: String) -> Self {
        Self {
            name,
            token: Default::default(),
            index: Default::default(),
            point: Default::default(),
            context: Default::default(),
        }
    }

    pub fn begin_context<T, F: FnMut(&mut Self) -> Result<T, Error>>(
        &mut self,
        context: Context,
        mut call: F,
    ) -> Result<T, Error> {
        self.context.push(context);
        let v = call(self)?;
        self.context.pop();

        Ok(v)
    }

    /// Analyze text to construct a token buffer from it.
    pub fn from_text(name: &str, text: &str) -> Result<Self, Error> {
        let mut token = TokenBuffer::new(name.to_string());

        for (row, line) in text.lines().enumerate() {
            let mut line = LineBuffer::new(line.to_string(), row);
            let mut text = String::default();
            let mut in_number = false;
            //let mut in_string = false;
            let mut in_space = false;

            while let Some(c) = line.pop() {
                if TokenData::is_symbol(c) {
                    if in_number && c == '.' {
                        text.push(c);
                    } else {
                        if !text.is_empty() {
                            token.push(&line, &text)?;
                            text.clear();
                        }

                        line.set_anchor();

                        text.push(c);

                        if let Some(p) = line.peek(0)
                            && (p == '=' || p == '-' || p == '>')
                        {
                            line.pop();
                            text.push(p);
                        }

                        token.push(&line, &text)?;
                        text.clear();

                        in_space = true;
                    }
                } else if c == ' ' {
                    if !text.is_empty() {
                        token.push(&line, &text)?;
                        text.clear();
                    }

                    in_number = false;
                    in_space = true;
                } else if c == '#' {
                    line.clear();
                } else {
                    if in_space {
                        line.set_anchor();
                        in_space = false;
                    }

                    if c.is_numeric() {
                        in_number = true;
                    }

                    text.push(c);
                }
            }

            if !text.is_empty() {
                token.push(&line, &text)?;
                text.clear();
            }
        }

        Ok(token)
    }

    /// Append a new token, from text.
    fn push(&mut self, line: &LineBuffer, text: &str) -> Result<(), Error> {
        let t = Token::new_text(
            line.point_anchor,
            line.point.x - line.point_anchor.x - 1,
            text,
        );

        self.point = line.point_anchor;

        if let Ok(t) = t {
            self.token.push(t);
        } else if let Err(e) = t {
            return Err(Error::kind(e)
                .with_name(self.name.clone())
                .with_point(line.point_anchor));
        }

        Ok(())
    }

    /// Pop the next token.
    pub fn pop(&mut self) -> Option<Token> {
        if let Some(t) = self.token.get(self.index) {
            self.index += 1;
            Some(t.clone())
        } else {
            None
        }
    }

    pub fn pop_until<F: FnMut(&mut Self) -> Result<(), Error>>(
        &mut self,
        delimiter: TokenKind,
        follow_up: Option<TokenKind>,
        mut call: F,
    ) -> Result<(), Error> {
        while self.want_peek(delimiter, 0).is_none() {
            call(self)?;
            //value_i.push(Parameter::from_token(token));

            if let Some(follow_up) = follow_up
                && self.want_peek(follow_up, 0).is_some()
            {
                self.want(follow_up)?;
            }
        }

        Ok(())
    }

    pub fn error(&self, kind: ErrorKind) -> Error {
        Error::kind(kind)
            .with_name(self.name.to_string())
            .with_point(self.point)
            .with_context(self.context.clone())
    }

    pub fn error_point(&self, kind: ErrorKind, point: Point) -> Error {
        Error::kind(kind)
            .with_name(self.name.to_string())
            .with_point(point)
            .with_context(self.context.clone())
    }

    /// Request the next token to be of a given kind.
    pub fn want(&mut self, kind: TokenKind) -> Result<Token, Error> {
        if let Some(t) = self.token.get(self.index) {
            if t.data.kind() == kind {
                self.index += 1;
                return Ok(t.clone());
            } else {
                return Err(
                    self.error_point(ErrorKind::InvalidToken(kind, t.data.clone()), t.point)
                );
            }
        }

        Err(self.error(ErrorKind::MissingToken(kind)))
    }

    /// Query if the next token is of a given kind.
    pub fn want_peek(&mut self, kind: TokenKind, index: usize) -> Option<Token> {
        if let Some(t) = self.token.get(self.index + index)
            && t.data.kind() == kind
        {
            return Some(t.clone());
        }

        None
    }

    /// Query if the next token.
    pub fn peek(&mut self, index: usize) -> Option<Token> {
        self.token.get(self.index + index).cloned()
    }
}

struct LineBuffer {
    line: String,
    point: Point,
    point_anchor: Point,
}

impl LineBuffer {
    fn new(line: String, row: usize) -> Self {
        Self {
            line,
            point: Point::new(0, row),
            point_anchor: Point::new(0, row),
        }
    }

    /// Pop the next character.
    fn pop(&mut self) -> Option<char> {
        self.point.x += 1;
        self.line.chars().nth(self.point.x - 1)
    }

    /// Query the next character, with some look-ahead.
    fn peek(&self, index: usize) -> Option<char> {
        self.line.chars().nth(self.point.x + index)
    }

    /// Clear the line.
    fn clear(&mut self) {
        self.point.x = self.line.len();
    }

    /// Set an anchor point, using the current point.
    fn set_anchor(&mut self) {
        self.point_anchor = Point::new(self.point.x - 1, self.point.y);
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Point {
    /// X-position, or "column".
    pub x: usize,
    /// Y-position, or "row"/"line".
    pub y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub point: Point,
    pub scale: usize,
    pub data: TokenData,
}

impl Token {
    fn new_text(point: Point, scale: usize, text: &str) -> Result<Self, ErrorKind> {
        Ok(Self {
            point,
            scale: scale.max(1),
            data: TokenData::from_text(text)?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Integer,
    Decimal,
    Boolean,
    String,
    RoundBegin,
    RoundClose,
    CurlyBegin,
    CurlyClose,
    SquareBegin,
    SquareClose,
    Assignment,
    Add,
    Subtract,
    Multiply,
    Divide,
    GT,
    GTE,
    LT,
    LTE,
    Equal,
    EqualNot,
    Not,
    And,
    Or,
    Dot,
    Colon,
    ColonSemi,
    Comma,
    At,
    ArrowR,
    ArrowL,
    Program,
    Function,
    Object,
    Is,
    If,
    Else,
    While,
    Return,
}

impl std::fmt::Display for TokenKind {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Identifier    => "{identifier}",
            Self::Integer       => "{integer}",
            Self::Decimal       => "{decimal}",
            Self::Boolean       => "{boolean}",
            Self::String        => "{string}",
            Self::RoundBegin    => "(",
            Self::RoundClose    => ")",
            Self::CurlyBegin    => "{",
            Self::CurlyClose    => "}",
            Self::SquareBegin   => "[",
            Self::SquareClose   => "]",
            Self::Assignment    => ":=",
            Self::Add           => "+",
            Self::Subtract      => "-",
            Self::Multiply      => "*",
            Self::Divide        => "/",
            Self::GT            => ">",
            Self::GTE           => ">=",
            Self::LT            => "<",
            Self::LTE           => "<=",
            Self::Equal         => "=",
            Self::EqualNot      => "!=",
            Self::Not           => "not",
            Self::And           => "and",
            Self::Or            => "or",
            Self::Dot           => ".",
            Self::Colon         => ":",
            Self::ColonSemi     => ";",
            Self::Comma         => ",",
            Self::At            => "@",
            Self::ArrowR        => "->",
            Self::ArrowL        => "<-",
            Self::Program       => "<>",
            Self::Function      => "function",
            Self::Object        => "object",
            Self::Is            => "is",
            Self::If            => "if",
            Self::Else          => "else",
            Self::While         => "while",
            Self::Return        => "return",
        };

        f.write_str(s)
    }
}

#[derive(Clone, Debug)]
pub enum TokenData {
    Identifier(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    String(String),
    RoundBegin,
    RoundClose,
    CurlyBegin,
    CurlyClose,
    SquareBegin,
    SquareClose,
    Assignment,
    Add,
    Subtract,
    Multiply,
    Divide,
    GT,
    GTE,
    LT,
    LTE,
    Equal,
    EqualNot,
    Not,
    And,
    Or,
    Dot,
    Colon,
    ColonSemi,
    Comma,
    At,
    ArrowR,
    ArrowL,
    Program,
    Function,
    Object,
    Is,
    If,
    Else,
    While,
    Return,
}

impl std::fmt::Display for TokenData {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Identifier(v) => &v.to_string(),
            Self::Integer(v)    => &v.to_string(),
            Self::Decimal(v)    => &v.to_string(),
            Self::Boolean(v)    => &v.to_string(),
            Self::String(v)     => &v.to_string(),
            Self::RoundBegin    => "(",
            Self::RoundClose    => ")",
            Self::CurlyBegin    => "{",
            Self::CurlyClose    => "}",
            Self::SquareBegin   => "[",
            Self::SquareClose   => "]",
            Self::Assignment    => ":=",
            Self::Add           => "+",
            Self::Subtract      => "-",
            Self::Multiply      => "*",
            Self::Divide        => "/",
            Self::GT            => ">",
            Self::GTE           => ">=",
            Self::LT            => "<",
            Self::LTE           => "<=",
            Self::Equal         => "=",
            Self::EqualNot      => "!=",
            Self::Not           => "not",
            Self::And           => "and",
            Self::Or            => "or",
            Self::Dot           => ".",
            Self::Colon         => ":",
            Self::ColonSemi     => ";",
            Self::Comma         => ",",
            Self::At            => "@",
            Self::ArrowR        => "->",
            Self::ArrowL        => "<-",
            Self::Program       => "<>",
            Self::Function      => "function",
            Self::Object        => "object",
            Self::Is            => "is",
            Self::If            => "if",
            Self::Else          => "else",
            Self::While         => "while",
            Self::Return        => "return",
        };

        f.write_str(s)
    }
}

impl TokenData {
    pub fn identifier(self) -> String {
        if let Self::Identifier(v) = self {
            v
        } else {
            panic!("TokenData::identifier(): token is not an identifier")
        }
    }

    #[rustfmt::skip]
    /// Query the kind of this token.
    pub fn kind(&self) -> TokenKind {
        match self {
            Self::Identifier(_) => TokenKind::Identifier,
            Self::Integer(_)    => TokenKind::Integer,
            Self::Decimal(_)    => TokenKind::Decimal,
            Self::Boolean(_)    => TokenKind::Boolean,
            Self::String(_)     => TokenKind::String,
            Self::RoundBegin    => TokenKind::RoundBegin,
            Self::RoundClose    => TokenKind::RoundClose,
            Self::CurlyBegin    => TokenKind::CurlyBegin,
            Self::CurlyClose    => TokenKind::CurlyClose,
            Self::SquareBegin   => TokenKind::SquareBegin,
            Self::SquareClose   => TokenKind::SquareClose,
            Self::Assignment    => TokenKind::Assignment,
            Self::Add           => TokenKind::Add,
            Self::Subtract      => TokenKind::Subtract,
            Self::Multiply      => TokenKind::Multiply,
            Self::Divide        => TokenKind::Divide,
            Self::GT            => TokenKind::GT,
            Self::GTE           => TokenKind::GTE,
            Self::LT            => TokenKind::LT,
            Self::LTE           => TokenKind::LTE,
            Self::Equal         => TokenKind::Equal,
            Self::EqualNot      => TokenKind::EqualNot,
            Self::Not           => TokenKind::Not,
            Self::And           => TokenKind::And,
            Self::Or            => TokenKind::Or,
            Self::Dot           => TokenKind::Dot,
            Self::Colon         => TokenKind::Colon,
            Self::ColonSemi     => TokenKind::ColonSemi,
            Self::Comma         => TokenKind::Comma,
            Self::At            => TokenKind::At,
            Self::ArrowR        => TokenKind::ArrowR,
            Self::ArrowL        => TokenKind::ArrowL,
            Self::Program       => TokenKind::Program,
            Self::Function      => TokenKind::Function,
            Self::Object        => TokenKind::Object,
            Self::Is            => TokenKind::Is,
            Self::If            => TokenKind::If,
            Self::Else          => TokenKind::Else,
            Self::While         => TokenKind::While,
            Self::Return        => TokenKind::Return,
        }
    }

    #[rustfmt::skip]
    /// Test if a given character is a symbol/delimiter.
    fn is_symbol(character: char) -> bool {
        matches!(character,
            '(' |
            ')' |
            '{' |
            '}' |
            '[' |
            ']' |
            '+' |
            '-' |
            '*' |
            '/' |
            '>' |
            '<' |
            '=' |
            '!' |
            '.' |
            ':' |
            ';' |
            ',' |
            '@'
        )
    }

    fn parse_identifier(text: &str) -> Result<String, ErrorKind> {
        for (i, character) in text.chars().enumerate() {
            if i == 0 {
                if !character.is_alphabetic() {
                    return Err(ErrorKind::InvalidIdentifier(text.to_string()).into());
                }
            } else {
                if !(character.is_alphanumeric() || character == '_') {
                    return Err(ErrorKind::InvalidIdentifier(text.to_string()).into());
                }
            }
        }

        Ok(text.to_string())
    }

    #[rustfmt::skip]
    /// Convert a given text into a token.
    fn from_text(text: &str) -> Result<Self, ErrorKind> {
        Ok(match text {
            "("         => Self::RoundBegin,
            ")"         => Self::RoundClose,
            "{"         => Self::CurlyBegin,
            "}"         => Self::CurlyClose,
            "["         => Self::SquareBegin,
            "]"         => Self::SquareClose,
            ":="        => Self::Assignment,
            "+"         => Self::Add,
            "-"         => Self::Subtract,
            "*"         => Self::Multiply,
            "/"         => Self::Divide,
            ">"         => Self::GT,
            ">="        => Self::GTE,
            "<"         => Self::LT,
            "<="        => Self::LTE,
            "="         => Self::Equal,
            "!="        => Self::EqualNot,
            "not"       => Self::Not,
            "and"       => Self::And,
            "or"        => Self::Or,
            "."         => Self::Dot,
            ":"         => Self::Colon,
            ";"         => Self::ColonSemi,
            ","         => Self::Comma,
            "@"         => Self::At,
            "->"        => Self::ArrowR,
            "<-"        => Self::ArrowL,
            "<>"        => Self::Program,
            "function"  => Self::Function,
            "object"    => Self::Object,
            "is"        => Self::Is,
            "if"        => Self::If,
            "else"      => Self::Else,
            "while"     => Self::While,
            "return"    => Self::Return,
            x => {
                if let Ok(i) = x.parse::<i64>() {
                    Self::Integer(i)
                } else if let Ok(f) = x.parse::<f64>() {
                    Self::Decimal(f)
                } else if x == "true" {
                    Self::Boolean(true)
                } else if x == "false" {
                    Self::Boolean(false)
                } else if x.starts_with('"') && x.ends_with('"') {
                    Self::String(x[1..x.len() - 1].to_string())
                } else {
                    Self::Identifier(Self::parse_identifier(x)?)
                }
            },
        })
    }
}
