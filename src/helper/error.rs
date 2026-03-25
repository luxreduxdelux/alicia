use crate::split::helper::*;
use crate::split::token::*;

//================================================================

use std::fmt::Display;
use thiserror::Error;

//================================================================

pub struct Error {
    info: Option<ErrorInfo>,
    hint: Option<ErrorHint>,
    kind: ErrorKind,
}

impl Error {
    pub fn new_info(info: ErrorInfo, kind: ErrorKind, hint: Option<ErrorHint>) -> Self {
        Self {
            info: Some(info),
            kind,
            hint,
        }
    }

    pub fn new_kind(kind: ErrorKind, hint: Option<ErrorHint>) -> Self {
        Self {
            info: None,
            kind,
            hint,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = if let Some(info) = &self.info {
            if let Some(token) = &info.token {
                format!(
                    "\"{}\":{}:{}, ",
                    info.source.path,
                    token.point.y + 1,
                    token.point.x + 1
                )
            } else {
                format!(" \"{}\", ", info.source.path)
            }
        } else {
            "".to_string()
        };

        let text = if let Some(info) = &self.info
            && let Some(token) = &info.token
        {
            let code = info.source.data.lines().nth(token.point.y).unwrap();
            let mut line = String::new();

            for i in 0..token.point.x {
                if i == token.point.x - 1 {
                    line.push('^');
                } else {
                    line.push('.');
                }
            }

            format!("\n{code}\n{line}")
        } else {
            "".to_string()
        };

        let (context, hint) = if let Some(hint) = &self.hint {
            hint.help()
        } else {
            (String::default(), String::default())
        };

        f.write_str(&format!(
            "error{context}: {info}{}\n{text}\n\n{hint}",
            self.kind
        ))
    }
}

//================================================================

pub struct ErrorInfo {
    source: Source,
    token: Option<Token>,
}

impl ErrorInfo {
    pub fn new(source: Source, token: Option<Token>) -> Self {
        Self { source, token }
    }
}

//================================================================

pub enum ErrorHint {
    Assignment,
    Invocation,
    Function,
    Variable,
    Structure,
    Enumerate,
    Use,
}

impl ErrorHint {
    fn help(&self) -> (String, String) {
        match self {
            ErrorHint::Assignment => (
                " parsing assignment".to_string(),
                "help: let foo : String := \"hello\"".to_string(),
            ),
            ErrorHint::Invocation => (" parsing invocation".to_string(), "help: foo()".to_string()),
            ErrorHint::Variable => (
                " parsing variable".to_string(),
                "help: foo : String".to_string(),
            ),
            ErrorHint::Function => (
                " parsing function".to_string(),
                "help: function foo(a: String) { }".to_string(),
            ),
            ErrorHint::Structure => (
                " parsing structure".to_string(),
                "help: structure foo { a: String }".to_string(),
            ),
            ErrorHint::Enumerate => (
                " parsing enumerate".to_string(),
                "help: enumerate foo { a, b, c }".to_string(),
            ),
            _ => (String::default(), String::default()),
        }
    }
}

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("unknown kind \"{0}\".")]
    UnknownKind(String),
    #[error(
        "was expecting one of \"function\", \"structure\", \"enumerate\", \"use\", found \"{0}\"."
    )]
    UnknownToken(Token),
    #[error("could not parse \"{0}\" as a valid Integer value.")]
    IntegerParseFail(String),
    #[error("could not parse \"{0}\" as a valid Decimal value.")]
    DecimalParseFail(String),
    #[error("could not parse \"{0}\" as a valid Boolean value.")]
    BooleanParseFail(String),
    #[error("file \"{0}\" not found.")]
    FileNotFound(String),
    #[error("was expecting \"{0}\", got \"{1}\" instead.")]
    IncorrectTokenKind(TokenKind, Token),
    #[error("was expecting \"{0}\".")]
    ExpectingKind(TokenKind),
    #[error("invalid identifier \"{0}\", cannot start with a number \"{1}\".")]
    IncorrectIdentifierNumber(String, char),
    #[error("invalid identifier \"{0}\", cannot use symbol \"{1}\".")]
    IncorrectIdentifierSymbol(String, char),
}
