use crate::helper::error::*;
use crate::stage_1::buffer::*;
use crate::stage_1::helper::*;
use crate::stage_1::token::*;
use crate::stage_2::construct::*;

//================================================================

use std::collections::HashMap;

//================================================================

#[derive(Debug)]
enum Declaration {
    Function(Function),
    Structure(Structure),
    Enumerate(Enumerate),
    Definition(Definition),
}

pub struct Scope {
    symbol: HashMap<Identifier, Declaration>,
}

impl Scope {
    pub fn new(mut token_buffer: TokenBuffer) -> Result<Self, Error> {
        let mut symbol = HashMap::default();

        while let Some(token) = token_buffer.peek() {
            match token.class {
                TokenClass::Function => {
                    let function = Function::parse_token(&mut token_buffer)?;
                    symbol.insert(function.name.clone(), Declaration::Function(function));
                }
                TokenClass::Structure => {
                    let structure = Structure::parse_token(&mut token_buffer)?;
                    symbol.insert(structure.name.clone(), Declaration::Structure(structure));
                }
                TokenClass::Enumerate => {
                    let enumerate = Enumerate::parse_token(&mut token_buffer)?;
                    symbol.insert(enumerate.name.clone(), Declaration::Enumerate(enumerate));
                }
                TokenClass::Let => {
                    let definition = Definition::parse_token(&mut token_buffer)?;
                    symbol.insert(definition.name.clone(), Declaration::Definition(definition));
                }
                TokenClass::Use => {
                    let value = Use::parse_token(&mut token_buffer)?;
                    println!("use: {value:?}");
                }
                _ => {
                    return Err(Error::new_info(
                        token_buffer.get_error_info(Some(token.clone())),
                        ErrorKind::UnknownTokenGlobal(token),
                        Some(ErrorHint::Global),
                    ));
                }
            };
        }

        println!("{symbol:#?}");

        Ok(Self { symbol })
    }
}
