use super::token::*;
use crate::runtime::machine::*;

//================================================================

use std::fmt::Display;
use thiserror::Error;

//================================================================

#[derive(Error, Debug)]
pub enum AliciaError {
    #[error("Source error: {0}")]
    SourceError(SourceError),
    #[error("Parse error: {0}")]
    ParseError(ParseError),
    #[error("Type error: {0}")]
    TypeError(TypeError),
}

#[derive(Error, Debug)]
pub struct ParseError {
    line: String,
}

impl ParseError {
    pub fn new_token(source: &Source, token: &Token, token_error: &TokenError) -> Self {
        let mut line = String::new();

        line.push_str(&format!(
            "Error in file \"{}\" (line: {}, column: {}): {token_error}",
            source.path,
            token.point.y + 1,
            token.point.x + 1
        ));

        Self { line }
    }

    pub fn new(source: &Source, token_error: TokenError) -> Self {
        let mut line = String::new();

        line.push_str(&format!("Error in file \"{}\":\n", source.path));

        line.push_str(&format!("{token_error}"));

        Self { line }
    }
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.line)
    }
}

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("file \"{0}\" not found.")]
    FileNotFound(String),
}

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("was expecting \"{0}\", got \"{1}\" instead.")]
    IncorrectKind(TokenKind, Token),
    #[error("was expecting \"{0}\".")]
    ExpectingKind(TokenKind),
}

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("was expecting \"{0:?}\", got \"{1:?}\" instead.")]
    IncorrectKind(ValueKind, ValueKind),
    #[error("unknown kind \"{0}\".")]
    UnknownKind(String),
    #[error("could not parse \"{0}\" as a valid Integer value.")]
    IntegerParseFail(String),
    #[error("could not parse \"{0}\" as a valid Decimal value.")]
    DecimalParseFail(String),
    #[error("could not parse \"{0}\" as a valid Boolean value.")]
    BooleanParseFail(String),
}
