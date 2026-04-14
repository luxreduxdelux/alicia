use crate::stage_1::buffer::*;
use crate::stage_1::helper::*;
use crate::stage_1::token::*;

//================================================================

use std::fmt::Display;
use thiserror::Error;

//================================================================

#[derive(Debug)]
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

    fn slice_list(list: &Vec<(String, usize)>) -> Vec<(String, usize)> {
        if list.len() >= 6 {
            let mut slice = Vec::new();

            for x in 0..3 {
                if let Some(line) = list.get(x) {
                    slice.push(line.clone())
                }
            }

            for x in list.len() - 3..list.len() {
                if let Some(line) = list.get(x) {
                    slice.push(line.clone())
                }
            }

            slice
        } else {
            list.to_vec()
        }
    }

    fn text_box(file: &str, token_span: &TokenSpan, point: Point) -> String {
        let mut text_box = String::default();
        let line_size = token_span
            .list
            .iter()
            .max_by(|x, y| (x.1 + 1).cmp(&(y.1 + 1)));
        let line_size = (line_size.unwrap().1 + 1).to_string();
        let line_size = line_size.len() + 2;

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

        let slice = Self::slice_list(&token_span.list);

        for (text, line) in &slice {
            let line = line + 1;
            let line_text = line.to_string();

            text_box.push('│');
            text_box.push(' ');
            text_box.push_str(&line_text);
            text_box.push_str(&' '.to_string().repeat(line_size - (line_text.len() + 2)));
            text_box.push(' ');
            text_box.push('│');
            text_box.push(' ');
            text_box.push_str(text);
            text_box.push('\n');

            if line == point.y + 1 {
                break;
            }
        }

        text_box.push('│');
        text_box.push_str(&' '.to_string().repeat(line_size));
        text_box.push('│');

        text_box.push_str(&' '.to_string().repeat(point.x));
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
            if let Some(point) = &info.point {
                Self::text_box(
                    &format!("{}:{}:{}", info.token_span.path, point.y + 1, point.x),
                    &info.token_span,
                    *point,
                )
            } else {
                info.token_span.path.to_string()
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

#[derive(Debug)]
pub struct ErrorInfo {
    token_span: TokenSpan,
    point: Option<Point>,
}

impl ErrorInfo {
    pub fn new_token(token_span: TokenSpan, token: Option<Token>) -> Self {
        let point = if let Some(token) = token {
            Some(token.point)
        } else {
            None
        };

        Self { token_span, point }
    }

    pub fn new_point(token_span: TokenSpan, point: Option<Point>) -> Self {
        Self { token_span, point }
    }
}

//================================================================

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorHint {
    Global,
    Definition,
    Assignment,
    Invocation,
    Indexation,
    Function,
    Variable,
    Structure,
    Enumerate,
    Expression,
    Use,
    Return,
    Condition,
    Iteration,
    Block,
}

impl ErrorHint {
    fn help(&self) -> (String, String) {
        match self {
            ErrorHint::Global => (" parsing global".to_string(), String::default()),
            ErrorHint::Definition => (
                " parsing definition".to_string(),
                "\nexample definition: let foo : String := \"hello\"".to_string(),
            ),
            ErrorHint::Assignment => (
                " parsing assignment".to_string(),
                "\nexample assignment: foo := \"hello\"".to_string(),
            ),
            ErrorHint::Invocation => (
                " parsing invocation".to_string(),
                "\nexample invocation: foo()".to_string(),
            ),
            ErrorHint::Indexation => (
                " parsing indexation".to_string(),
                "\nexample indexation: foo[0]".to_string(),
            ),
            ErrorHint::Variable => (
                " parsing variable".to_string(),
                "\nexample variable: foo : String".to_string(),
            ),
            ErrorHint::Function => (
                " parsing function".to_string(),
                "\nexample function: function foo(a: String) { ... }".to_string(),
            ),
            ErrorHint::Structure => (
                " parsing structure".to_string(),
                "\nexample strcuture: structure foo { a: String }".to_string(),
            ),
            ErrorHint::Enumerate => (
                " parsing enumerate".to_string(),
                "\nexample enumerate: enumerate foo { a, b, c }".to_string(),
            ),
            ErrorHint::Expression => (
                " parsing expression".to_string(),
                "\nexample expression: (2 + 2) * 4".to_string(),
            ),
            ErrorHint::Use => (
                " parsing use".to_string(),
                "\nexample use: use module_name".to_string(),
            ),
            ErrorHint::Return => (
                " parsing return".to_string(),
                "\nexample return: return 1".to_string(),
            ),
            ErrorHint::Condition => (
                " parsing condition".to_string(),
                "\nexample condition: if a { ... } else if b { ... } else { }".to_string(),
            ),
            ErrorHint::Iteration => (
                " parsing iteration".to_string(),
                "\nexample iteration: loop { ... }".to_string(),
            ),
            ErrorHint::Block => (
                " parsing block".to_string(),
                "\nexample block: { ... }".to_string(),
            ),
        }
    }
}

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("unknown kind \"{0}\".")]
    UnknownKind(String),
    #[error(
        "was expecting one of \"function\", \"structure\", \"enumerate\", \"import\", found \"{0}\"."
    )]
    UnknownTokenGlobal(Token),
    #[error("unknown token \"{0}\".")]
    UnknownToken(Token),
    #[error("unknown symbol \"{0}\".")]
    UnknownSymbol(Identifier),
    #[error("\"{0}\" is not a valid function to invoke.")]
    InvalidInvocation(Identifier),
    #[error("\"{0}\" is not a valid variable to assign.")]
    InvalidAssignment(Identifier),
    #[error("cannot \"skip\" outside of an iteration block.")]
    InvalidSkip,
    #[error("cannot \"exit\" outside of an iteration block.")]
    InvalidExit,
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
