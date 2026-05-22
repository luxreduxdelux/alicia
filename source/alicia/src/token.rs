use crate::buffer::*;
use crate::error::ErrorKind;
use crate::helper::*;

//================================================================

use std::fmt::Display;

//================================================================

#[derive(Debug, Clone)]
pub struct Token {
    pub point: Point,
    pub class: TokenClass,
}

impl Token {
    pub fn parse_line(
        line: &str,
        line_index: usize,
        list: &mut Vec<Token>,
        mut comment: bool,
    ) -> Result<bool, ErrorKind> {
        if line.is_empty() {
            return Ok(comment);
        }

        let mut line_buffer = LineBuffer::new(line.to_string());
        let mut inside_string = false;
        let mut inside_number = false;
        let mut inside_escape = false;

        while let Some(character) = line_buffer.next() {
            if comment {
                if character == '#'
                    && let Some(peek) = line_buffer.peek()
                    && peek == '#'
                {
                    line_buffer.next();
                    comment = false;
                } else {
                    continue;
                }
            }

            match character {
                '#' => {
                    if let Some(peek) = line_buffer.peek() {
                        if peek == '#' {
                            println!("comment enter");
                            line_buffer.next();
                            comment = true;
                            continue;
                        }
                    }

                    return Ok(comment);
                }
                '0'..='9' => {
                    inside_number = true;
                    line_buffer.push(character);
                }
                '\\' => {
                    inside_escape = true;
                    continue;
                }
                '_' => {
                    if inside_number {
                        continue;
                    } else {
                        line_buffer.push(character);
                    }
                }
                ' ' => {
                    inside_number = false;

                    if !inside_string {
                        if !line_buffer.is_empty() {
                            list.push(Self::new(
                                Point::new(line_buffer.get_cursor(), line_index),
                                &line_buffer.clear(),
                            )?);
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
                        )?);
                    }

                    inside_string = !inside_string;
                }
                '(' | ')' | '[' | ']' | '{' | '}' | '.' | ':' | ';' | ',' | '@' | '&' | '|'
                | '^' | '~' | '<' | '>' | '+' | '-' | '*' | '/' | '%' => {
                    if character == '.' && inside_number {
                        line_buffer.push(character);
                        continue;
                    }

                    inside_number = false;

                    if !inside_string {
                        if let Some(peek) = line_buffer.peek() {
                            let definition = character == ':';
                            let assign_a = character == '+';
                            let assign_s = character == '-';
                            let assign_m = character == '*';
                            let assign_d = character == '/';
                            let assign_mo = character == '%';
                            let gte = character == '>';
                            let lte = character == '<';
                            let bit_a = character == '&';
                            let bit_o = character == '|';
                            let bit_x = character == '^';
                            let bit_n = character == '~';

                            if (definition
                                || assign_a
                                || assign_s
                                || assign_m
                                || assign_d
                                || assign_mo
                                || gte
                                || lte
                                || bit_a
                                || bit_o
                                || bit_x
                                || bit_n)
                                && (peek == '='
                                    || peek == '<'
                                    || peek == '>'
                                    || peek == '?'
                                    || peek == ':')
                            {
                                line_buffer.push(character);
                                line_buffer.next();
                                line_buffer.push(peek);
                                list.push(Self::new(
                                    Point::new(line_buffer.get_cursor(), line_index),
                                    &line_buffer.clear(),
                                )?);
                                continue;
                            }
                        }

                        if !line_buffer.is_empty() {
                            list.push(Self::new(
                                Point::new(line_buffer.get_cursor(), line_index),
                                &line_buffer.clear(),
                            )?);
                        }

                        list.push(Self::new(
                            Point::new(line_buffer.get_cursor(), line_index),
                            &character.to_string(),
                        )?);
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
            )?);
        }

        Ok(comment)
    }

    fn new(point: Point, text: &str) -> Result<Self, ErrorKind> {
        let class = TokenClass::parse_text(text, point)?;
        let x = point.x - text.len();

        Ok(Self {
            point: Point::new(x, point.y),
            class,
        })
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
    Identifier(Identifier),
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Function,
    Structure,
    Enumerate,
    Let,
    Use,
    SelfLower,
    SelfUpper,
    Loop,
    Skip,
    Exit,
    Return,
    If,
    Else,
    Switch,
    Not,
    And,
    Or,
    ParenthesisBegin,
    ParenthesisClose,
    SquareBegin,
    SquareClose,
    CurlyBegin,
    CurlyClose,
    Dot,
    Colon,
    ColonSemi,
    Comma,
    At,
    Ampersand,
    AmpersandQuestion,
    Pipe,
    PipeQuestion,
    Exponent,
    Tilde,
    ShiftL,
    ShiftR,
    DefinitionVariable,
    DefinitionConstant,
    DefinitionAdd,
    DefinitionSubtract,
    DefinitionMultiply,
    DefinitionDivide,
    DefinitionModulo,
    DefinitionAnd,
    DefinitionOr,
    DefinitionExclusiveOr,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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
            Self::Identifier(value) => formatter.write_str(&value.text),
            Self::String(text)      => formatter.write_str(&format!("String(\"{text}\")")),
            Self::Integer(value)    => formatter.write_str(&format!("Integer(\"{value}\")")),
            Self::Decimal(value)    => formatter.write_str(&format!("Decimal(\"{value}\")")),
            Self::Boolean(value)    => formatter.write_str(&format!("Boolean(\"{value}\")")),
            _ => formatter.write_str(&format!("{}", self.kind())),
        }
    }
}

impl TokenClass {
    pub fn inner_string(&self) -> String {
        format!("{}", self)
    }

    #[rustfmt::skip]
    fn parse_text(text: &str, point: Point) -> Result<Self, ErrorKind> {
        Ok(match text {
            "true"      => Self::Boolean(true),
            "false"     => Self::Boolean(false),
            "function"  => Self::Function,
            "structure" => Self::Structure,
            "enumerate" => Self::Enumerate,
            "let"       => Self::Let,
            "use"       => Self::Use,
            "self"      => Self::SelfLower,
            "Self"      => Self::SelfUpper,
            "loop"      => Self::Loop,
            "skip"      => Self::Skip,
            "exit"      => Self::Exit,
            "return"    => Self::Return,
            "if"        => Self::If,
            "else"      => Self::Else,
            "switch"    => Self::Switch,
            "not"       => Self::Not,
            "and"       => Self::And,
            "or"        => Self::Or,
            "("         => Self::ParenthesisBegin,
            ")"         => Self::ParenthesisClose,
            "["         => Self::SquareBegin,
            "]"         => Self::SquareClose,
            "{"         => Self::CurlyBegin,
            "}"         => Self::CurlyClose,
            "."         => Self::Dot,
            ":"         => Self::Colon,
            ";"         => Self::ColonSemi,
            ","         => Self::Comma,
            "@"         => Self::At,
            "&"         => Self::Ampersand,
            "&?"        => Self::AmpersandQuestion,
            "|"         => Self::Pipe,
            "|?"        => Self::PipeQuestion,
            "^"         => Self::Exponent,
            "~"         => Self::Tilde,
            "<<"        => Self::ShiftL,
            ">>"        => Self::ShiftR,
            ":="        => Self::DefinitionVariable,
            "::"        => Self::DefinitionConstant,
            "+="        => Self::DefinitionAdd,
            "-="        => Self::DefinitionSubtract,
            "*="        => Self::DefinitionMultiply,
            "/="        => Self::DefinitionDivide,
            "%="        => Self::DefinitionModulo,
            "&="        => Self::DefinitionAnd,
            "|="        => Self::DefinitionOr,
            "^="        => Self::DefinitionExclusiveOr,
            "+"         => Self::Add,
            "-"         => Self::Subtract,
            "*"         => Self::Multiply,
            "/"         => Self::Divide,
            "%"         => Self::Modulo,
            ">"         => Self::GT,
            "<"         => Self::LT,
            "="         => Self::Equal,
            ">="        => Self::GTE,
            "<="        => Self::LTE,
            "!="        => Self::EqualNot,
            _ => {
                if let Ok(integer) = text.parse::<i64>() {
                    Self::Integer(integer)
                } else if let Ok(decimal) = text.parse::<f64>() {
                    Self::Decimal(decimal)
                } else if text.starts_with("\"") && text.ends_with("\"") {
                    let text = &text[1..text.len() - 1];
                    Self::String(text.to_string())
                } else if text.starts_with("0b") {
                    let text = &text[2..text.len()];

                    println!("parsing binary: {text}");

                    Self::Integer(i64::from_str_radix(text, 2).unwrap())
                } else if text.starts_with("0h") {
                    let text = &text[2..text.len()];
                    Self::Integer(i64::from_str_radix(text, 16).unwrap())
                } else if text.starts_with("0o") {
                    let text = &text[2..text.len()];
                    Self::Integer(i64::from_str_radix(text, 8).unwrap())
                } else {
                    let x = if text.len() == 1 {
                        point.x
                    } else {
                        point.x - text.len()
                    };

                    Self::Identifier(Identifier::from_string(text.to_string(), Point::new(x, point.y + 1))?)
                }
            }
        })

    }

    #[rustfmt::skip]
    pub fn kind(&self) -> TokenKind {
        match self {
            Self::Identifier(_)         => TokenKind::Identifier,
            Self::String(_)             => TokenKind::String,
            Self::Integer(_)            => TokenKind::Integer,
            Self::Decimal(_)            => TokenKind::Decimal,
            Self::Boolean(_)            => TokenKind::Boolean,
            Self::Function              => TokenKind::Function,
            Self::Structure             => TokenKind::Structure,
            Self::Enumerate             => TokenKind::Enumerate,
            Self::Let                   => TokenKind::Let,
            Self::Use                   => TokenKind::Use,
            Self::SelfLower             => TokenKind::SelfLower,
            Self::SelfUpper             => TokenKind::SelfUpper,
            Self::Loop                  => TokenKind::Loop,
            Self::Skip                  => TokenKind::Skip,
            Self::Exit                  => TokenKind::Exit,
            Self::Return                => TokenKind::Return,
            Self::If                    => TokenKind::If,
            Self::Else                  => TokenKind::Else,
            Self::Switch                => TokenKind::Switch,
            Self::Not                   => TokenKind::Not,
            Self::And                   => TokenKind::And,
            Self::Or                    => TokenKind::Or,
            Self::ParenthesisBegin      => TokenKind::ParenthesisBegin,
            Self::ParenthesisClose      => TokenKind::ParenthesisClose,
            Self::SquareBegin           => TokenKind::SquareBegin,
            Self::SquareClose           => TokenKind::SquareClose,
            Self::CurlyBegin            => TokenKind::CurlyBegin,
            Self::CurlyClose            => TokenKind::CurlyClose,
            Self::Dot                   => TokenKind::Dot,
            Self::Colon                 => TokenKind::Colon,
            Self::ColonSemi             => TokenKind::ColonSemi,
            Self::Comma                 => TokenKind::Comma,
            Self::At                    => TokenKind::At,
            Self::Ampersand             => TokenKind::Ampersand,
            Self::AmpersandQuestion     => TokenKind::AmpersandQuestion,
            Self::Pipe                  => TokenKind::Pipe,
            Self::PipeQuestion          => TokenKind::PipeQuestion,
            Self::Exponent              => TokenKind::Exponent,
            Self::Tilde                 => TokenKind::Tilde,
            Self::ShiftL                => TokenKind::ShiftL,
            Self::ShiftR                => TokenKind::ShiftR,
            Self::DefinitionVariable    => TokenKind::DefinitionVariable,
            Self::DefinitionConstant    => TokenKind::DefinitionConstant,
            Self::DefinitionAdd         => TokenKind::DefinitionAdd,
            Self::DefinitionSubtract    => TokenKind::DefinitionSubtract,
            Self::DefinitionMultiply    => TokenKind::DefinitionMultiply,
            Self::DefinitionDivide      => TokenKind::DefinitionDivide,
            Self::DefinitionModulo      => TokenKind::DefinitionModulo,
            Self::DefinitionAnd         => TokenKind::DefinitionAnd,
            Self::DefinitionOr          => TokenKind::DefinitionOr,
            Self::DefinitionExclusiveOr => TokenKind::DefinitionExclusiveOr,
            Self::Add                   => TokenKind::Add,
            Self::Subtract              => TokenKind::Subtract,
            Self::Multiply              => TokenKind::Multiply,
            Self::Divide                => TokenKind::Divide,
            Self::Modulo                => TokenKind::Modulo,
            Self::GT                    => TokenKind::GT,
            Self::LT                    => TokenKind::LT,
            Self::Equal                 => TokenKind::Equal,
            Self::GTE                   => TokenKind::GTE,
            Self::LTE                   => TokenKind::LTE,
            Self::EqualNot              => TokenKind::EqualNot,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Identifier,
    String,
    Integer,
    Decimal,
    Boolean,
    Function,
    Structure,
    Enumerate,
    Let,
    Use,
    SelfLower,
    SelfUpper,
    Loop,
    Skip,
    Exit,
    Return,
    If,
    Else,
    Switch,
    Not,
    And,
    Or,
    ParenthesisBegin,
    ParenthesisClose,
    SquareBegin,
    SquareClose,
    CurlyBegin,
    CurlyClose,
    Dot,
    Colon,
    ColonSemi,
    Comma,
    At,
    Ampersand,
    AmpersandQuestion,
    Pipe,
    PipeQuestion,
    Exponent,
    Tilde,
    ShiftL,
    ShiftR,
    DefinitionVariable,
    DefinitionConstant,
    DefinitionAdd,
    DefinitionSubtract,
    DefinitionMultiply,
    DefinitionDivide,
    DefinitionModulo,
    DefinitionAnd,
    DefinitionOr,
    DefinitionExclusiveOr,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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
            Self::Identifier            => formatter.write_str("Identifier"),
            Self::String                => formatter.write_str("String"),
            Self::Integer               => formatter.write_str("Integer"),
            Self::Decimal               => formatter.write_str("Decimal"),
            Self::Boolean               => formatter.write_str("Boolean"),
            Self::Function              => formatter.write_str("function"),
            Self::Structure             => formatter.write_str("structure"),
            Self::Enumerate             => formatter.write_str("enumerate"),
            Self::Let                   => formatter.write_str("let"),
            Self::Use                   => formatter.write_str("use"),
            Self::SelfLower             => formatter.write_str("self"),
            Self::SelfUpper             => formatter.write_str("Self"),
            Self::Loop                  => formatter.write_str("loop"),
            Self::Skip                  => formatter.write_str("skip"),
            Self::Exit                  => formatter.write_str("exit"),
            Self::Return                => formatter.write_str("return"),
            Self::If                    => formatter.write_str("if"),
            Self::Else                  => formatter.write_str("else"),
            Self::Switch                => formatter.write_str("switch"),
            Self::Not                   => formatter.write_str("not"),
            Self::And                   => formatter.write_str("and"),
            Self::Or                    => formatter.write_str("or"),
            Self::ParenthesisBegin      => formatter.write_str("("),
            Self::ParenthesisClose      => formatter.write_str(")"),
            Self::SquareBegin           => formatter.write_str("["),
            Self::SquareClose           => formatter.write_str("]"),
            Self::CurlyBegin            => formatter.write_str("{"),
            Self::CurlyClose            => formatter.write_str("}"),
            Self::Dot                   => formatter.write_str("."),
            Self::Colon                 => formatter.write_str(":"),
            Self::ColonSemi             => formatter.write_str(";"),
            Self::Comma                 => formatter.write_str(","),
            Self::At                    => formatter.write_str("@"),
            Self::Ampersand             => formatter.write_str("&"),
            Self::AmpersandQuestion     => formatter.write_str("&?"),
            Self::Pipe                  => formatter.write_str("|"),
            Self::PipeQuestion          => formatter.write_str("|?"),
            Self::Exponent              => formatter.write_str("^"),
            Self::Tilde                 => formatter.write_str("~"),
            Self::ShiftL                => formatter.write_str("&"),
            Self::ShiftR                => formatter.write_str("&"),
            Self::DefinitionVariable    => formatter.write_str(":="),
            Self::DefinitionConstant    => formatter.write_str("::"),
            Self::DefinitionAdd         => formatter.write_str("+="),
            Self::DefinitionSubtract    => formatter.write_str("-="),
            Self::DefinitionMultiply    => formatter.write_str("*="),
            Self::DefinitionDivide      => formatter.write_str("/="),
            Self::DefinitionModulo      => formatter.write_str("%="),
            Self::DefinitionAnd         => formatter.write_str("&="),
            Self::DefinitionOr          => formatter.write_str("|="),
            Self::DefinitionExclusiveOr => formatter.write_str("^="),
            Self::Add                   => formatter.write_str("+"),
            Self::Subtract              => formatter.write_str("-"),
            Self::Multiply              => formatter.write_str("*"),
            Self::Divide                => formatter.write_str("/"),
            Self::Modulo                => formatter.write_str("%"),
            Self::GT                    => formatter.write_str(">"),
            Self::LT                    => formatter.write_str("<"),
            Self::Equal                 => formatter.write_str("="),
            Self::GTE                   => formatter.write_str(">="),
            Self::LTE                   => formatter.write_str("<="),
            Self::EqualNot              => formatter.write_str("!=")
        }
    }
}
