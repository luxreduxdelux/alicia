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

impl Identifier {
    pub fn new(text: String) -> Result<Self, Error> {
        Ok(Self { text })
    }
}
