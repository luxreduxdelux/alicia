use super::error::*;
use super::instruction::*;
use super::token::*;
use crate::runtime::machine::*;

//================================================================

pub struct Source {}

impl Source {
    pub fn parse(path: &str) -> Result<Self, AliciaError> {
        let file = std::fs::read_to_string(path).unwrap();
        let mut token_buffer = TokenBuffer::new(&file);

        let mut list: Vec<Declaration> = Vec::new();

        while let Some(token) = token_buffer.next() {
            match token {
                Token::Function => {
                    /*
                    list.push(Declaration::Function(Function::parse_token(
                        &token,
                        &mut token_buffer,
                    )?));
                    */

                    let function = Function::parse_token(&token, &mut token_buffer)?;

                    Machine::execute_function(function);

                    break;
                }
                _ => {}
            };
        }

        //Instruction::parse_token(&mut token_buffer, &mut list)?;
        //println!("AST: {list:#?}");

        Ok(Self {})
    }
}
