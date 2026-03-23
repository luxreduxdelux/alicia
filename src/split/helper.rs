use super::error::*;

//================================================================

pub struct Source {
    pub path: String,
    pub data: String,
}

impl Source {
    pub fn new(path: String, data: String) -> Self {
        Self { path, data }
    }

    pub fn new_file(path: &str) -> Result<Self, Error> {
        if let Ok(data) = std::fs::read_to_string(path) {
            Ok(Self {
                path: path.to_string(),
                data,
            })
        } else {
            Err(Error::FileNotFound(path.to_string()))
        }
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Identifier {
    pub text: String,
}

impl TryFrom<String> for Identifier {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        for (i, character) in value.chars().enumerate() {
            if i == 0 && character.is_numeric() {
                return Err(Error::IncorrectIdentifierNumber(value, character));
            } else if !(character.is_alphanumeric() || character == '_') {
                return Err(Error::IncorrectIdentifierSymbol(value, character));
            }
        }

        Ok(Self { text: value })
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.text
    }
}
