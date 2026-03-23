use thiserror::Error;

//================================================================

#[derive(Error, Debug)]
pub enum Error {
    #[error("\x1b[31merror:\x1b[0m {0}")]
    Split(crate::split::error::Error),
    #[error("\x1b[31merror:\x1b[0m {0}")]
    Parse(crate::parse::error::Error),
    #[error("\x1b[31merror:\x1b[0m {0}")]
    Machine(crate::machine::error::Error),
}

impl From<crate::split::error::Error> for Error {
    fn from(value: crate::split::error::Error) -> Self {
        Self::Split(value)
    }
}

impl From<crate::parse::error::Error> for Error {
    fn from(value: crate::parse::error::Error) -> Self {
        Self::Parse(value)
    }
}

impl From<crate::machine::error::Error> for Error {
    fn from(value: crate::machine::error::Error) -> Self {
        Self::Machine(value)
    }
}
