use crate::buffer::*;
use crate::construct::*;
use crate::error::*;
use crate::helper::*;
use crate::machine::Argument;
use crate::machine::Machine;
use crate::machine::Value;
use crate::machine::ValueType;
use crate::token::*;
use std::fmt::Debug;

//================================================================

use std::collections::HashMap;

//================================================================

#[derive(Debug, Clone)]
pub enum NativeArgument {
    Variable,
    Constant(&'static [ValueType]),
}

#[derive(Debug, Clone)]
pub struct FunctionNative {
    pub name: String,
    pub call: fn(&mut Machine, Argument) -> Option<Value>,
    pub enter: NativeArgument,
    pub leave: ValueType,
}

impl FunctionNative {
    pub fn new(
        name: String,
        call: fn(&mut Machine, Argument) -> Option<Value>,
        enter: NativeArgument,
        leave: ValueType,
    ) -> Self {
        Self {
            name,
            call,
            enter,
            leave,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Function(Function),
    FunctionNative(FunctionNative),
    Structure(Structure),
    Enumerate(Enumerate),
    Definition(Definition),
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub source: Vec<Source>,
    pub symbol: HashMap<String, Declaration>,
    pub parent: Option<Box<Self>>,
    pub slot: usize,
}

impl Scope {
    pub fn new(parent: Option<Box<Self>>) -> Self {
        let slot = if let Some(parent) = &parent {
            parent.slot
        } else {
            usize::default()
        };

        Self {
            source: Vec::default(),
            symbol: HashMap::default(),
            parent,
            slot,
        }
    }

    pub fn get_active_source(&self) -> Source {
        self.source.last().unwrap().clone()
    }

    pub fn parse_buffer(&mut self, mut token_buffer: TokenBuffer) -> Result<(), Error> {
        self.source.push(token_buffer.source.clone());

        while let Some(token) = token_buffer.peek() {
            match token.class {
                TokenClass::Function => {
                    let function = Function::parse_token(&mut token_buffer, None)?;
                    self.set_declaration(function.name.clone(), Declaration::Function(function));
                }
                TokenClass::Structure => {
                    let structure = Structure::parse_token(&mut token_buffer)?;
                    self.set_declaration(structure.name.clone(), Declaration::Structure(structure));
                }
                TokenClass::Enumerate => {
                    let enumerate = Enumerate::parse_token(&mut token_buffer)?;
                    self.set_declaration(enumerate.name.clone(), Declaration::Enumerate(enumerate));
                }
                TokenClass::Let => {
                    let definition = Definition::parse_token(&mut token_buffer)?;
                    self.set_declaration(
                        definition.name.clone(),
                        Declaration::Definition(definition),
                    );
                }
                //TokenClass::Use => {
                //    let value = Use::parse_token(&mut token_buffer)?;
                //    println!("use: {value:?}");
                //}
                _ => {
                    return Error::new_info(
                        token_buffer.get_error_info(Some(token.clone())),
                        ErrorKind::UnknownTokenGlobal(token),
                        Some(ErrorHint::Global),
                    );
                }
            };
        }

        Ok(())
    }

    pub fn print(&self) {
        println!("scope: {:#?}", self.symbol);

        if let Some(parent) = &self.parent {
            parent.print();
        }
    }

    pub fn get_declaration(&self, name: Identifier) -> Option<&Declaration> {
        if let Some(declaration) = self.symbol.get(&name.text) {
            Some(declaration)
        } else if let Some(parent) = &self.parent {
            parent.get_declaration(name)
        } else {
            None
        }
    }

    pub fn get_function(&self, name: Identifier) -> Option<&Function> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::Function(f) = d
        {
            Some(f)
        } else {
            None
        }
    }

    pub fn get_function_native(&self, name: Identifier) -> Option<&FunctionNative> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::FunctionNative(f) = d
        {
            Some(f)
        } else {
            None
        }
    }

    pub fn get_structure(&self, name: Identifier) -> Option<&Structure> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::Structure(s) = d
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_enumerate(&self, name: Identifier) -> Option<&Enumerate> {
        if let Some(d) = self.get_declaration(name)
            && let Declaration::Enumerate(e) = d
        {
            Some(e)
        } else {
            None
        }
    }

    pub fn set_declaration(&mut self, name: Identifier, value: Declaration) {
        self.symbol.insert(name.to_string(), value);
    }

    pub fn get_slot(&self) -> usize {
        self.slot
    }

    pub fn get_and_add_slot(&mut self) -> usize {
        self.slot += 1;
        self.slot - 1
    }
}
