use crate::helper::error::*;

//================================================================

use std::fmt::Display;

//================================================================

#[derive(Debug, Clone)]
pub struct Source {
    pub path: String,
    pub data: String,
}

impl Source {
    /*
    pub fn new_data(path: String, data: String) -> Self {
        Self { path, data }
    }
    */

    pub fn new_file(path: &str) -> Result<Self, Error> {
        if let Ok(data) = std::fs::read_to_string(path) {
            Ok(Self {
                path: path.to_string(),
                data,
            })
        } else {
            Err(Error::new_kind(
                ErrorKind::FileNotFound(path.to_string()),
                None,
            ))
        }
    }
}

//================================================================

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub text: String,
    pub point: Point,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

impl Identifier {
    pub fn from_string(text: String, point: Point) -> Result<Self, ErrorKind> {
        for (i, character) in text.chars().enumerate() {
            if i == 0 && character.is_numeric() {
                return Err(ErrorKind::IncorrectIdentifierNumber(text, character));
            } else if !(character.is_alphanumeric() || character == '_') {
                return Err(ErrorKind::IncorrectIdentifierSymbol(text, character));
            }
        }

        Ok(Self { text, point })
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.text
    }
}

//================================================================

#[derive(Debug, Clone, Default)]
pub struct Path {
    pub list: Vec<Identifier>,
}

impl From<Path> for String {
    fn from(value: Path) -> Self {
        let mut string = String::new();

        for identifier in value.list {
            string.push_str(&identifier.text);
        }

        string
    }
}

impl Path {
    pub fn push(&mut self, text: Identifier) -> () {
        self.list.push(text);
    }
}
