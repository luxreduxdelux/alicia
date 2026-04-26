use crate::helper::error::*;
use crate::stage_1::buffer::*;
use crate::stage_1::helper::*;
use crate::stage_1::token::*;
use crate::stage_2::construct::*;
use crate::stage_4::machine::Argument;
use crate::stage_4::machine::Value;
use std::fmt::Debug;

//================================================================

use std::collections::HashMap;

//================================================================

#[derive(Debug, Clone)]
pub enum NativeArgument {
    Variable,
    Constant(Vec<ExpressionKind>),
}

#[derive(Debug, Clone)]
pub struct FunctionNative {
    pub name: String,
    pub call: fn(Argument) -> Option<Value>,
    pub enter: NativeArgument,
    pub leave: ExpressionKind,
}

impl FunctionNative {
    pub fn new(
        name: String,
        call: fn(Argument) -> Option<Value>,
        enter: NativeArgument,
        leave: ExpressionKind,
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

    pub fn parse_buffer(&mut self, mut token_buffer: TokenBuffer) -> Result<(), Error> {
        self.source.push(token_buffer.source.clone());

        while let Some(token) = token_buffer.peek() {
            match token.class {
                TokenClass::Function => {
                    let function = Function::parse_token(&mut token_buffer)?;
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
                    return Err(Error::new_info(
                        token_buffer.get_error_info(Some(token.clone())),
                        ErrorKind::UnknownTokenGlobal(token),
                        Some(ErrorHint::Global),
                    ));
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
