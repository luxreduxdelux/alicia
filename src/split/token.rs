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
        let mut inside_escape = false;

        while let Some(character) = line_buffer.next() {
            match character {
                '\\' => {
                    inside_escape = true;
                    continue;
                }
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
                    if inside_escape {
                        line_buffer.push(character);
                        continue;
                    }

                    line_buffer.push(character);

                    if inside_string && !line_buffer.is_empty() {
                        list.push(Self::new(
                            Point::new(line_buffer.get_cursor(), line_index),
                            &line_buffer.clear(),
                        ));
                    }

                    inside_string = !inside_string;
                }
                '#' => return,
                '(' | ')' | '{' | '}' | '.' | ':' | ',' | '&' | '<' | '>' | '+' | '-' | '*'
                | '/' => {
                    if !inside_string {
                        if let Some(peek) = line_buffer.peek() {
                            let definition = character == ':';
                            let assign_a = character == '+';
                            let assign_s = character == '-';
                            let assign_m = character == '*';
                            let assign_d = character == '/';
                            let gte = character == '>';
                            let lte = character == '<';

                            if (definition
                                || assign_a
                                || assign_s
                                || assign_m
                                || assign_d
                                || gte
                                || lte)
                                && peek == '='
                            {
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

            inside_escape = false;
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
    Skip,
    Exit,
    If,
    Else,
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
    Definition,
    DefinitionAdd,
    DefinitionSubtract,
    DefinitionMultiply,
    DefinitionDivide,
    Add,
    Subtract,
    Multiply,
    Divide,
    GT,
    LT,
    Equal,
    GTE,
    LTE,
    EqualNot,
}

impl Display for TokenClass {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::String(text)       => formatter.write_str(text),
            Self::Integer(text)      => formatter.write_str(&text.to_string()),
            Self::Decimal(text)      => formatter.write_str(&text.to_string()),
            Self::Boolean(text)      => formatter.write_str(&text.to_string()),
            Self::Function           => formatter.write_str("function"),
            Self::Structure          => formatter.write_str("structure"),
            Self::Enumerate          => formatter.write_str("enumerate"),
            Self::Let                => formatter.write_str("let"),
            Self::Use                => formatter.write_str("use"),
            Self::For                => formatter.write_str("for"),
            Self::While              => formatter.write_str("while"),
            Self::Skip               => formatter.write_str("skip"),
            Self::Exit               => formatter.write_str("exit"),
            Self::If                 => formatter.write_str("if"),
            Self::Else               => formatter.write_str("else"),
            Self::Not                => formatter.write_str("not"),
            Self::And                => formatter.write_str("and"),
            Self::Or                 => formatter.write_str("or"),
            Self::ParenthesisBegin   => formatter.write_str("("),
            Self::ParenthesisClose   => formatter.write_str(")"),
            Self::CurlyBegin         => formatter.write_str("{"),
            Self::CurlyClose         => formatter.write_str("}"),
            Self::Dot                => formatter.write_str("."),
            Self::Colon              => formatter.write_str(":"),
            Self::Comma              => formatter.write_str(","),
            Self::Ampersand          => formatter.write_str("&"),
            Self::Definition         => formatter.write_str(":="),
            Self::DefinitionAdd      => formatter.write_str("+="),
            Self::DefinitionSubtract => formatter.write_str("-="),
            Self::DefinitionMultiply => formatter.write_str("*="),
            Self::DefinitionDivide   => formatter.write_str("/="),
            Self::Add                => formatter.write_str("+"),
            Self::Subtract           => formatter.write_str("-"),
            Self::Multiply           => formatter.write_str("*"),
            Self::Divide             => formatter.write_str("/"),
            Self::GT                 => formatter.write_str(">"),
            Self::LT                 => formatter.write_str("<"),
            Self::Equal              => formatter.write_str("="),
            Self::GTE                => formatter.write_str(">="),
            Self::LTE                => formatter.write_str("<="),
            Self::EqualNot           => formatter.write_str("!="),
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
            "skip"      => Self::Skip,
            "exit"      => Self::Exit,
            "if"        => Self::If,
            "else"      => Self::Else,
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
            ":="        => Self::Definition,
            "+="        => Self::DefinitionAdd,
            "-="        => Self::DefinitionSubtract,
            "*="        => Self::DefinitionMultiply,
            "/="        => Self::DefinitionDivide,
            "+"         => Self::Add,
            "-"         => Self::Subtract,
            "*"         => Self::Multiply,
            "/"         => Self::Divide,
            ">"         => Self::GT,
            "<"         => Self::LT,
            "="         => Self::Equal,
            ">="        => Self::GTE,
            "<="        => Self::LTE,
            "!="        => Self::EqualNot,
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
            Self::String(_)          => TokenKind::String,
            Self::Integer(_)         => TokenKind::Integer,
            Self::Decimal(_)         => TokenKind::Decimal,
            Self::Boolean(_)         => TokenKind::Boolean,
            Self::Function           => TokenKind::Function,
            Self::Structure          => TokenKind::Structure,
            Self::Enumerate          => TokenKind::Enumerate,
            Self::Let                => TokenKind::Let,
            Self::Use                => TokenKind::Use,
            Self::For                => TokenKind::For,
            Self::While              => TokenKind::While,
            Self::Skip               => TokenKind::Skip,
            Self::Exit               => TokenKind::Exit,
            Self::If                 => TokenKind::If,
            Self::Else               => TokenKind::Else,
            Self::Not                => TokenKind::Not,
            Self::And                => TokenKind::And,
            Self::Or                 => TokenKind::Or,
            Self::ParenthesisBegin   => TokenKind::ParenthesisBegin,
            Self::ParenthesisClose   => TokenKind::ParenthesisClose,
            Self::CurlyBegin         => TokenKind::CurlyBegin,
            Self::CurlyClose         => TokenKind::CurlyClose,
            Self::Dot                => TokenKind::Dot,
            Self::Colon              => TokenKind::Colon,
            Self::Comma              => TokenKind::Comma,
            Self::Ampersand          => TokenKind::Ampersand,
            Self::Definition         => TokenKind::Definition,
            Self::DefinitionAdd      => TokenKind::DefinitionAdd,
            Self::DefinitionSubtract => TokenKind::DefinitionSubtract,
            Self::DefinitionMultiply => TokenKind::DefinitionMultiply,
            Self::DefinitionDivide   => TokenKind::DefinitionDivide,
            Self::Add                => TokenKind::Add,
            Self::Subtract           => TokenKind::Subtract,
            Self::Multiply           => TokenKind::Multiply,
            Self::Divide             => TokenKind::Divide,
            Self::GT                 => TokenKind::GT,
            Self::LT                 => TokenKind::LT,
            Self::Equal              => TokenKind::Equal,
            Self::GTE                => TokenKind::GTE,
            Self::LTE                => TokenKind::LTE,
            Self::EqualNot           => TokenKind::EqualNot,
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
    Skip,
    Exit,
    If,
    Else,
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
    Definition,
    DefinitionAdd,
    DefinitionSubtract,
    DefinitionMultiply,
    DefinitionDivide,
    Add,
    Subtract,
    Multiply,
    Divide,
    GT,
    LT,
    Equal,
    GTE,
    LTE,
    EqualNot,
}

impl Display for TokenKind {
    #[rustfmt::skip]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::String             => formatter.write_str("String"),
            Self::Integer            => formatter.write_str("Integer"),
            Self::Decimal            => formatter.write_str("Decimal"),
            Self::Boolean            => formatter.write_str("Boolean"),
            Self::Function           => formatter.write_str("function"),
            Self::Structure          => formatter.write_str("structure"),
            Self::Enumerate          => formatter.write_str("enumerate"),
            Self::Let                => formatter.write_str("let"),
            Self::Use                => formatter.write_str("use"),
            Self::For                => formatter.write_str("for"),
            Self::While              => formatter.write_str("while"),
            Self::Skip               => formatter.write_str("skip"),
            Self::Exit               => formatter.write_str("exit"),
            Self::If                 => formatter.write_str("if"),
            Self::Else               => formatter.write_str("else"),
            Self::Not                => formatter.write_str("not"),
            Self::And                => formatter.write_str("and"),
            Self::Or                 => formatter.write_str("or"),
            Self::ParenthesisBegin   => formatter.write_str("("),
            Self::ParenthesisClose   => formatter.write_str(")"),
            Self::CurlyBegin         => formatter.write_str("{"),
            Self::CurlyClose         => formatter.write_str("}"),
            Self::Dot                => formatter.write_str("."),
            Self::Colon              => formatter.write_str(":"),
            Self::Comma              => formatter.write_str(","),
            Self::Ampersand          => formatter.write_str("&"),
            Self::Definition         => formatter.write_str(":="),
            Self::DefinitionAdd      => formatter.write_str("+="),
            Self::DefinitionSubtract => formatter.write_str("-="),
            Self::DefinitionMultiply => formatter.write_str("*="),
            Self::DefinitionDivide   => formatter.write_str("/="),
            Self::Add                => formatter.write_str("+"),
            Self::Subtract           => formatter.write_str("-"),
            Self::Multiply           => formatter.write_str("*"),
            Self::Divide             => formatter.write_str("/"),
            Self::GT                 => formatter.write_str(">"),
            Self::LT                 => formatter.write_str("<"),
            Self::Equal              => formatter.write_str("="),
            Self::GTE                => formatter.write_str(">="),
            Self::LTE                => formatter.write_str("<="),
            Self::EqualNot           => formatter.write_str("!=")
        }
    }
}
