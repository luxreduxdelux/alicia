use super::token::*;

//================================================================

use core::slice::Iter;

//================================================================

#[derive(Debug)]
pub enum Instruction {
    Function(Function),
}

impl Instruction {
    pub fn parse_token(token: &Token, iterator: &mut Iter<'_, Token>, list: &mut Vec<Self>) {
        match token {
            Token::Function => {
                if let Some(function) = Function::parse_token(token, iterator) {
                    list.push(Self::Function(function));
                } else {
                    // report error.
                }
            }
            _ => {}
        };
    }
}

//================================================================

#[derive(Debug, Default)]
struct Function {
    name: String,
    token_list: Vec<Token>,
}

impl Function {
    fn parse_token(token: &Token, iterator: &mut Iter<'_, Token>) -> Option<Self> {
        let mut name = None;
        let mut token_list = Vec::new();

        while let Some(token) = iterator.next() {
            match token {
                Token::String(function_name) => {
                    if name.is_none() {
                        name = Some(function_name.to_string());
                    } else {
                        // report error here.
                    }
                }
                Token::ParenthesisBegin => {
                    while let Some(token) = iterator.next() {
                        match token {
                            _ => {}
                        }
                    }
                }
                Token::BracketBegin => {
                    while let Some(token) = iterator.next() {
                        match token {
                            Token::BracketClose => {
                                println!("finish bracket");
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(name) = name {
            Some(Self { name, token_list })
        } else {
            None
        }
    }
}
