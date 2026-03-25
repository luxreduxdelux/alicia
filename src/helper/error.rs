use crate::stage_1::helper::*;
use crate::stage_1::token::*;

//================================================================

use std::fmt::Display;
use thiserror::Error;

//================================================================

pub struct Error {
    info: Option<Box<ErrorInfo>>,
    hint: Option<Box<ErrorHint>>,
    kind: ErrorKind,
}

impl Error {
    pub fn new_info(info: ErrorInfo, kind: ErrorKind, hint: Option<ErrorHint>) -> Self {
        Self {
            info: Some(Box::new(info)),
            kind,
            hint: hint.map(Box::new),
        }
    }

    pub fn new_kind(kind: ErrorKind, hint: Option<ErrorHint>) -> Self {
        Self {
            info: None,
            kind,
            hint: hint.map(Box::new),
        }
    }

    fn text_box(file: &str, text: &str, line: usize, character: usize) -> String {
        let mut text_box = String::default();
        let line_size = line.to_string().len() + 2;

        text_box.push('\n');

        text_box.push('╭');
        text_box.push_str(&'─'.to_string().repeat(line_size));
        text_box.push('🭬');
        text_box.push_str(file);
        text_box.push('\n');

        text_box.push('│');
        text_box.push_str(&' '.to_string().repeat(line_size));
        text_box.push('│');
        text_box.push('\n');

        text_box.push('│');
        text_box.push(' ');
        text_box.push_str(&line.to_string());
        text_box.push(' ');
        text_box.push('│');
        text_box.push(' ');
        text_box.push_str(text);
        text_box.push('\n');

        text_box.push('│');
        text_box.push_str(&' '.to_string().repeat(line_size));
        text_box.push('│');
        text_box.push_str(&' '.to_string().repeat(character));
        text_box.push('─');
        text_box.push('\n');

        text_box.push('╰');
        text_box.push_str(&'─'.to_string().repeat(line_size + 1));

        text_box
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = if let Some(info) = &self.info {
            if let Some(token) = &info.token {
                let code = info.source.data.lines().nth(token.point.y).unwrap();

                Self::text_box(
                    &format!(
                        "{}:{}:{}",
                        info.source.path,
                        token.point.y + 1,
                        token.point.x
                    ),
                    code,
                    token.point.y + 1,
                    token.point.x,
                )
            } else {
                format!("\n{}", info.source.path)
            }
        } else {
            "".to_string()
        };

        let (context, hint) = if let Some(hint) = &self.hint {
            hint.help()
        } else {
            (String::default(), String::default())
        };

        f.write_str(&format!("error{context}: {}{text}{hint}", self.kind))
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
    Global,
    Definition,
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
            ErrorHint::Global => (" parsing global".to_string(), String::default()),
            ErrorHint::Definition => (
                " parsing definition".to_string(),
                "\nexample definition: let foo : String := \"hello\"".to_string(),
            ),
            ErrorHint::Invocation => (
                " parsing invocation".to_string(),
                "\nexample invocation: foo()".to_string(),
            ),
            ErrorHint::Variable => (
                " parsing variable".to_string(),
                "\nexample variable: foo : String".to_string(),
            ),
            ErrorHint::Function => (
                " parsing function".to_string(),
                "\nexample function: function foo(a: String) { }".to_string(),
            ),
            ErrorHint::Structure => (
                " parsing structure".to_string(),
                "\nexample strcuture: structure foo { a: String }".to_string(),
            ),
            ErrorHint::Enumerate => (
                " parsing enumerate".to_string(),
                "\nexample enumerate: enumerate foo { a, b, c }".to_string(),
            ),
            ErrorHint::Use => (
                " parsing use".to_string(),
                "\nexample use: use module_name".to_string(),
            ),
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
    UnknownTokenGlobal(Token),
    #[error("unknown token \"{0}\".")]
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
