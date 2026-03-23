use super::buffer::*;
use super::helper::*;

//================================================================

use std::fmt::Display;

//================================================================

#[derive(Debug, Clone)]
pub struct Token {
    pub point: Point,
    pub class: TokenClass,
}

impl Token {
    pub fn parse_line(line: &str, line_index: usize, list: &mut Vec<Token>) {
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
                            list.push(Self::new(
                                Point::new(line_buffer.get_cursor(), line_index),
                                &line_buffer.clear(),
                            ));
                        }
                    } else {
                        line_buffer.push(character);
                    }
                }
                '"' => {
                    line_buffer.push(character);

                    if inside_string {
                        if !line_buffer.is_empty() {
                            list.push(Self::new(
                                Point::new(line_buffer.get_cursor(), line_index),
                                &line_buffer.clear(),
                            ));
                        }
                    }

                    inside_string = !inside_string;
                }
                '(' | ')' | '{' | '}' | '.' | ':' | ',' | '&' | '<' | '>' => {
                    if !inside_string {
                        if let Some(peek) = line_buffer.peek() {
                            let assignment = character == ':' && peek == '=';
                            let ge_than = character == '>' && peek == '=';
                            let le_than = character == '<' && peek == '=';

                            if assignment || ge_than || le_than {
                                line_buffer.push(character);
                                line_buffer.next();
                                line_buffer.push(peek);
                                list.push(Self::new(
                                    Point::new(line_buffer.get_cursor(), line_index),
                                    &line_buffer.clear(),
                                ));
                                continue;
                            }
                        }

                        if !line_buffer.is_empty() {
                            list.push(Self::new(
                                Point::new(line_buffer.get_cursor(), line_index),
                                &line_buffer.clear(),
                            ));
                        }

                        list.push(Self::new(
                            Point::new(line_buffer.get_cursor(), line_index),
                            &character.to_string(),
                        ));
                    } else {
                        line_buffer.push(character);
                    }
                }
                _ => line_buffer.push(character),
            }
        }

        if !line_buffer.is_empty() {
            list.push(Self::new(
                Point::new(line_buffer.get_cursor(), line_index),
                &line_buffer.clear(),
            ));
        }
    }

    fn new(point: Point, text: &str) -> Self {
        Self {
            point,
            class: TokenClass::parse_text(text),
        }
    }
}

impl Display for Token {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        formatter.write_str(&format!("{}", self.class))
    }
}

#[derive(Debug, Clone)]
pub enum TokenClass {
    String(String),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
    Function,
    Structure,
    Enumerate,
    Let,
    Use,
    For,
    While,
    If,
    Else,
    ElseIf,
    In,
    Not,
    And,
    Or,
    ParenthesisBegin,
    ParenthesisClose,
    CurlyBegin,
    CurlyClose,
    Dot,
    Colon,
    Comma,
    Ampersand,
    Assignment,
    //AssignmentAdd,
    //AssignmentSubtract,
    //AssignmentMultiply,
    //AssignmentDivide,
    //Add,
    //Subtract,
    //Multiply,
    //Divide
    //GT,
    //LT,
    //GTE,
    //LTE,
}

impl Display for TokenClass {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::String(text)     => formatter.write_str(text),
            Self::Integer(text)    => formatter.write_str(&text.to_string()),
            Self::Decimal(text)    => formatter.write_str(&text.to_string()),
            Self::Boolean(text)    => formatter.write_str(&text.to_string()),
            Self::Function         => formatter.write_str("function"),
            Self::Structure        => formatter.write_str("structure"),
            Self::Enumerate        => formatter.write_str("enumerate"),
            Self::Let              => formatter.write_str("let"),
            Self::Use              => formatter.write_str("use"),
            Self::For              => formatter.write_str("for"),
            Self::While            => formatter.write_str("while"),
            Self::If               => formatter.write_str("if"),
            Self::Else             => formatter.write_str("else if"),
            Self::ElseIf           => formatter.write_str("else"),
            Self::In               => formatter.write_str("in"),
            Self::Not              => formatter.write_str("not"),
            Self::And              => formatter.write_str("and"),
            Self::Or               => formatter.write_str("or"),
            Self::ParenthesisBegin => formatter.write_str("("),
            Self::ParenthesisClose => formatter.write_str(")"),
            Self::CurlyBegin       => formatter.write_str("{"),
            Self::CurlyClose       => formatter.write_str("}"),
            Self::Dot              => formatter.write_str("."),
            Self::Colon            => formatter.write_str(":"),
            Self::Comma            => formatter.write_str(","),
            Self::Ampersand        => formatter.write_str("&"),
            Self::Assignment       => formatter.write_str(":=")
        }
    }
}

impl TokenClass {
    pub fn inner_string(&self) -> String {
        match self {
            Self::String(text) => text.clone(),
            _ => panic!(
                "Internal Alicia error: inner_string() on a token that was thought to be a Token::String token."
            ),
        }
    }

    #[rustfmt::skip]
    fn parse_text(text: &str) -> Self {
        match text {
            "true"      => Self::Boolean(true),
            "false"     => Self::Boolean(false),
            "function"  => Self::Function,
            "structure" => Self::Structure,
            "enumerate" => Self::Enumerate,
            "let"       => Self::Let,
            "use"       => Self::Use,
            "for"       => Self::For,
            "while"     => Self::While,
            "if"        => Self::If,
            "in"        => Self::In,
            "not"       => Self::Not,
            "and"       => Self::And,
            "or"        => Self::Or,
            "("         => Self::ParenthesisBegin,
            ")"         => Self::ParenthesisClose,
            "{"         => Self::CurlyBegin,
            "}"         => Self::CurlyClose,
            "."         => Self::Dot,
            ":"         => Self::Colon,
            ","         => Self::Comma,
            "&"         => Self::Ampersand,
            ":="        => Self::Assignment,
            _ => {
                if let Ok(decimal) = text.parse::<f32>() {
                    Self::Decimal(decimal)
                } else if let Ok(integer) = text.parse::<i32>() {
                    Self::Integer(integer)
                } else {
                    Self::String(text.to_string())
                }
            }
        }
    }

    #[rustfmt::skip]
    pub fn kind(&self) -> TokenKind {
        match self {
            Self::String(_)        => TokenKind::String,
            Self::Integer(_)       => TokenKind::Integer,
            Self::Decimal(_)       => TokenKind::Decimal,
            Self::Boolean(_)       => TokenKind::Boolean,
            Self::Function         => TokenKind::Function,
            Self::Structure        => TokenKind::Structure,
            Self::Enumerate        => TokenKind::Enumerate,
            Self::Let              => TokenKind::Let,
            Self::Use              => TokenKind::Use,
            Self::For              => TokenKind::For,
            Self::While            => TokenKind::While,
            Self::If               => TokenKind::If,
            Self::Else             => TokenKind::Else,
            Self::ElseIf           => TokenKind::ElseIf,
            Self::In               => TokenKind::In,
            Self::Not              => TokenKind::Not,
            Self::And              => TokenKind::And,
            Self::Or               => TokenKind::Or,
            Self::ParenthesisBegin => TokenKind::ParenthesisBegin,
            Self::ParenthesisClose => TokenKind::ParenthesisClose,
            Self::CurlyBegin       => TokenKind::CurlyBegin,
            Self::CurlyClose       => TokenKind::CurlyClose,
            Self::Dot              => TokenKind::Dot,
            Self::Colon            => TokenKind::Colon,
            Self::Comma            => TokenKind::Comma,
            Self::Ampersand        => TokenKind::Ampersand,
            Self::Assignment       => TokenKind::Assignment,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    String,
    Integer,
    Decimal,
    Boolean,
    Function,
    Structure,
    Enumerate,
    Let,
    Use,
    For,
    While,
    If,
    Else,
    ElseIf,
    In,
    Not,
    And,
    Or,
    ParenthesisBegin,
    ParenthesisClose,
    CurlyBegin,
    CurlyClose,
    Dot,
    Colon,
    Comma,
    Ampersand,
    Assignment,
}

impl Display for TokenKind {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::String           => formatter.write_str("String"),
            Self::Integer          => formatter.write_str("Integer"),
            Self::Decimal          => formatter.write_str("Decimal"),
            Self::Boolean          => formatter.write_str("Boolean"),
            Self::Function         => formatter.write_str("function"),
            Self::Structure        => formatter.write_str("structure"),
            Self::Enumerate        => formatter.write_str("enumerate"),
            Self::Let              => formatter.write_str("let"),
            Self::Use              => formatter.write_str("use"),
            Self::For              => formatter.write_str("for"),
            Self::While            => formatter.write_str("while"),
            Self::If               => formatter.write_str("if"),
            Self::Else             => formatter.write_str("else if"),
            Self::ElseIf           => formatter.write_str("else"),
            Self::In               => formatter.write_str("in"),
            Self::Not              => formatter.write_str("not"),
            Self::And              => formatter.write_str("and"),
            Self::Or               => formatter.write_str("or"),
            Self::ParenthesisBegin => formatter.write_str("("),
            Self::ParenthesisClose => formatter.write_str(")"),
            Self::CurlyBegin       => formatter.write_str("{"),
            Self::CurlyClose       => formatter.write_str("}"),
            Self::Dot              => formatter.write_str("."),
            Self::Colon            => formatter.write_str(":"),
            Self::Comma            => formatter.write_str(","),
            Self::Ampersand        => formatter.write_str("&"),
            Self::Assignment       => formatter.write_str(":="),
        }
    }
}
