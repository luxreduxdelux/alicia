use super::token::*;

//================================================================

use thiserror::Error;

//================================================================

#[derive(Error, Debug)]
pub enum AliciaError {
    #[error("Token error: {0}")]
    TokenError(TokenError),
}

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("was expecting {0:?}, got {1:?} instead.")]
    IncorrectKind(TokenKind, TokenKind),
    #[error("unknown variable kind \"{0}\".")]
    IncorrectKindVariable(String),
    #[error("was expecting {0:?}.")]
    MissingKind(TokenKind),
}
