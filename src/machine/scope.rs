use super::value::*;

//================================================================

use std::collections::HashMap;

//================================================================

#[derive(Debug, Default)]
pub struct Scope<'a> {
    symbol: HashMap<String, Value>,
    parent: Option<Box<&'a Scope<'a>>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope>) -> Self {
        Self {
            symbol: HashMap::default(),
            parent: parent.map(Box::new),
        }
    }

    pub fn set_value(&mut self, name: &str, value: Value) {
        self.symbol.insert(name.to_string(), value);
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        if let Some(symbol) = self.symbol.get(name) {
            Some(symbol)
        } else if let Some(parent) = &self.parent {
            Some(parent.get_value(name)?)
        } else {
            None
        }
    }
}
