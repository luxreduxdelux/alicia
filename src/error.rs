use crate::lexer::*;

//================================================================

use std::fmt::Display;
use thiserror::Error;

//================================================================

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub name: Option<String>,
    pub hint: Option<String>,
    pub point: Option<Point>,
    pub context: Vec<Context>,
}

impl Error {
    pub fn kind(kind: ErrorKind) -> Self {
        Self {
            kind,
            name: None,
            hint: None,
            point: None,
            context: Default::default(),
        }
    }

    pub fn with_name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub fn with_hint(self, hint: String) -> Self {
        Self {
            hint: Some(hint),
            ..self
        }
    }

    pub fn with_point(self, point: Point) -> Self {
        Self {
            point: Some(point),
            ..self
        }
    }

    pub fn with_context(self, context: Vec<Context>) -> Self {
        Self { context, ..self }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = if let Some(n) = &self.name
            && let Some(p) = &self.point
        {
            format!("({}:{}:{}): ", n, p.y + 1, p.x + 1,)
        } else {
            String::default()
        };

        let context = if self.context.is_empty() {
            String::default()
        } else {
            format!("{:?} ", self.context)
        };

        f.write_str(&format!("error: {location}{context}{}", self.kind))
    }
}

impl std::error::Error for Error {}

//================================================================

#[derive(Clone, Debug)]
pub enum Context {
    Function,
    Object,
    Is,
    Parameter,
    Kind,
}

#[derive(Clone, Debug, Error)]
pub enum ErrorKind {
    #[error("\"{0}\" is not a valid identifier.")]
    InvalidIdentifier(String),
    #[error("was expecting: \"{0}\", got: \"{1}\"")]
    InvalidToken(TokenKind, TokenData),
    #[error("was expecting: \"{0}\"")]
    MissingToken(TokenKind),
    #[error("was expecting one of \"function\", \"object\", \"{{identifier}} is\", got: \"{0}\"")]
    InvalidDefinition(TokenData),
}
