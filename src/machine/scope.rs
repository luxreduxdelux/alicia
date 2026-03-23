use super::error::*;
use super::value::*;
use crate::parse::construct::*;
use crate::split::buffer::*;
use crate::split::token::*;

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

    pub fn parse_token(
        &mut self,
        mut token_buffer: TokenBuffer,
    ) -> Result<(), crate::helper::error::Error> {
        while let Some(token) = token_buffer.next() {
            match token.class {
                TokenClass::Function => {
                    let value = Function::parse_token(&mut token_buffer)?;
                    self.set_value(&value.name.text.clone(), Value::Function(value));
                }
                TokenClass::Structure => {
                    let value = Structure::parse_token(&mut token_buffer)?;
                    self.set_value(&value.name.text.clone(), Value::Structure(value));
                }
                TokenClass::Enumerate => {
                    let value = Enumerate::parse_token(&mut token_buffer)?;
                    self.set_value(&value.name.text.clone(), Value::Enumerate(value));
                }
                TokenClass::Use => {
                    let value = Use::parse_token(&mut token_buffer)?;
                    println!("use: {value:?}");
                    //self.set_value(&String::from(value.path.into()), Value::Use(value));
                }
                _ => {
                    return Err(crate::helper::error::Error::Machine(Error::UnknownToken(
                        token,
                    )));
                }
            };
        }

        println!("scope: {:#?}", self);

        Ok(())
    }
}
