use super::token::*;

//================================================================

use thiserror::Error;

//================================================================

#[derive(Error, Debug)]
pub enum Error {
    #[error("file \"{0}\" not found.")]
    FileNotFound(String),
    #[error("was expecting \"{0}\", got \"{1}\" instead.")]
    IncorrectKind(TokenKind, Token),
    #[error("was expecting \"{0}\".")]
    ExpectingKind(TokenKind),
    #[error("invalid identifier \"{0}\", cannot start with a number \"{1}\".")]
    IncorrectIdentifierNumber(String, char),
    #[error("invalid identifier \"{0}\", cannot use symbol \"{1}\".")]
    IncorrectIdentifierSymbol(String, char),
}
