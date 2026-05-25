use crate::error::*;

//================================================================

use core::slice::Iter;
use std::collections::hash_map::Keys;
use std::fmt::Display;
use std::slice::IterMut;
use std::{collections::HashMap, hash::Hash};

//================================================================

#[derive(Debug, Clone)]
pub struct Source {
    pub path: String,
    pub data: String,
}

impl Source {
    pub fn new_data(path: String, data: String) -> Self {
        Self { path, data }
    }

    pub fn new_file(path: &str) -> Result<Self, Error> {
        if let Ok(data) = std::fs::read_to_string(path) {
            Ok(Self {
                path: path.to_string(),
                data,
            })
        } else {
            Error::new_kind(ErrorKind::FileNotFound(path.to_string()), None)
        }
    }
}

//================================================================

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
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

#[derive(Debug, Clone)]
pub struct OrderMap<K: Eq + Hash, V> {
    pub array: Vec<V>,
    pub order: HashMap<K, usize>,
}

impl<K: Eq + Hash, V> Default for OrderMap<K, V> {
    fn default() -> Self {
        Self {
            array: Default::default(),
            order: Default::default(),
        }
    }
}

impl<K: Eq + Hash, V> OrderMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        if let Some(index) = self.order.get(&key) {
            self.array[*index] = value;
        } else {
            self.order.insert(key, self.array.len());
            self.array.push(value);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(index) = self.order.get(key) {
            self.array.get(*index)
        } else {
            None
        }
    }

    pub fn get_index(&self, index: usize) -> Option<&V> {
        self.array.get(index)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.order.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }

    pub fn clear(&mut self) {
        self.array.clear();
        self.order.clear();
    }

    pub fn iterate(&self) -> Vec<(&K, &V)> {
        let mut result = Vec::with_capacity(self.len());

        for (v_i, value) in self.array.iter().enumerate() {
            for (key, k_i) in &self.order {
                if v_i == *k_i {
                    result.push((key, value));
                }
            }
        }

        result
    }

    pub fn keys(&self) -> Keys<'_, K, usize> {
        self.order.keys()
    }

    pub fn values(&self) -> Iter<'_, V> {
        self.array.iter()
    }

    pub fn values_mut(&mut self) -> IterMut<'_, V> {
        self.array.iter_mut()
    }
}

/*
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
*/
