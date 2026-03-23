use super::value::*;
use crate::split::token::*;

//================================================================

use thiserror::Error;

//================================================================

#[derive(Error, Debug)]
pub enum Error {
    #[error("was expecting \"{0:?}\", got \"{1:?}\" instead.")]
    IncorrectKind(ValueKind, ValueKind),
    #[error("unknown kind \"{0}\".")]
    UnknownKind(String),
    #[error("was expecting one of \"function\", \"structure\", \"enumerate\", found \"{0}\".")]
    UnknownToken(Token),
    #[error("could not parse \"{0}\" as a valid Integer value.")]
    IntegerParseFail(String),
    #[error("could not parse \"{0}\" as a valid Decimal value.")]
    DecimalParseFail(String),
    #[error("could not parse \"{0}\" as a valid Boolean value.")]
    BooleanParseFail(String),
}
