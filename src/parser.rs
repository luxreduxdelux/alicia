use crate::error::*;
use crate::lexer::*;

//================================================================

use std::{collections::HashMap, println};

//================================================================

pub struct DefinitionBuffer {
    definition: HashMap<String, Definition>,
}

impl DefinitionBuffer {
    pub fn from_token(mut token: TokenBuffer) -> Result<(), Error> {
        while let Some(t) = token.peek(0) {
            match t.data.kind() {
                TokenKind::Function => {
                    println!("{:#?}", Function::from_token(&mut token)?);
                }
                TokenKind::Object => {
                    println!("{:#?}", Object::from_token(&mut token)?);
                }
                TokenKind::Identifier => {
                    if token.want_peek(TokenKind::Is, 1).is_some() {
                        println!("{:#?}", Is::from_token(&mut token)?);
                    } else {
                        return Err(token.error(ErrorKind::MissingToken(TokenKind::Is)));
                    }
                }
                _ => return Err(token.error_point(ErrorKind::InvalidDefinition(t.data), t.point)),
            }
        }

        Ok(())
    }
}

pub enum Definition {
    Function(Function),
    Object(Object),
    Is(Is),
}

#[derive(Clone, Debug)]
pub struct Is {
    name: String,
    kind: Kind,
}

impl Is {
    fn from_token(token: &mut TokenBuffer) -> Result<Self, Error> {
        token.begin_context(Context::Is, |token| {
            let name = token.want(TokenKind::Identifier)?.data.identifier();

            token.want(TokenKind::Is)?;

            let kind = Kind::from_token(token)?;

            Ok(Self { name, kind })
        })
    }
}

#[derive(Clone, Debug)]
pub enum Kind {
    Identifier(String),
    Integer,
    Decimal,
    Boolean,
    String,
    Array(Box<Kind>),
    Tuple(Vec<Kind>),
    Table(Box<Kind>, Box<Kind>),
    Structure(Vec<(String, Kind)>),
    Enumerate(Vec<String>),
    Set(Box<Kind>),
}

impl Kind {
    fn from_token(token: &mut TokenBuffer) -> Result<Self, Error> {
        token.begin_context(Context::Kind, |token| {
            let identifier = token.want(TokenKind::Identifier)?.data.identifier();

            Ok(match identifier.as_str() {
                "Integer" => Self::Integer,
                "Decimal" => Self::Decimal,
                "Boolean" => Self::Boolean,
                "String" => Self::String,
                "Array" => {
                    token.want(TokenKind::LT)?;
                    let k = Self::from_token(token)?;
                    token.want(TokenKind::GT)?;

                    Self::Array(Box::new(k))
                }
                "Tuple" => {
                    token.want(TokenKind::LT)?;

                    let mut t = Vec::new();

                    token.pop_until(TokenKind::GT, Some(TokenKind::Comma), |token| {
                        t.push(Self::from_token(token)?);
                        Ok(())
                    })?;

                    token.want(TokenKind::GT)?;

                    Self::Tuple(t)
                }
                "Table" => {
                    token.want(TokenKind::LT)?;

                    let k = Self::from_token(token)?;
                    token.want(TokenKind::Comma)?;
                    let v = Self::from_token(token)?;

                    token.want(TokenKind::GT)?;

                    Self::Table(Box::new(k), Box::new(v))
                }
                "Structure" => {
                    token.want(TokenKind::LT)?;

                    let mut t = Vec::new();

                    token.pop_until(TokenKind::GT, Some(TokenKind::Comma), |token| {
                        let k = token.want(TokenKind::Identifier)?.data.identifier();
                        token.want(TokenKind::Colon)?;
                        let v = Self::from_token(token)?;
                        t.push((k, v));
                        Ok(())
                    })?;

                    token.want(TokenKind::GT)?;

                    Self::Structure(t)
                }
                "Enumerate" => {
                    token.want(TokenKind::LT)?;

                    let mut t = Vec::new();

                    token.pop_until(TokenKind::GT, Some(TokenKind::Comma), |token| {
                        t.push(token.want(TokenKind::Identifier)?.data.identifier());
                        Ok(())
                    })?;

                    token.want(TokenKind::GT)?;

                    Self::Enumerate(t)
                }
                "Set" => {
                    token.want(TokenKind::LT)?;
                    let k = Self::from_token(token)?;
                    token.want(TokenKind::GT)?;

                    Self::Set(Box::new(k))
                }
                x => Self::Identifier(x.to_string()),
            })
        })
    }
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub reference: bool,
    pub name: String,
    pub kind: Kind,
}

impl Parameter {
    fn from_token(token: &mut TokenBuffer) -> Result<Self, Error> {
        token.begin_context(Context::Parameter, |token| {
            let name = token.want(TokenKind::Identifier)?.data.identifier();

            let reference = if token.want_peek(TokenKind::ArrowR, 0).is_some() {
                token.want(TokenKind::ArrowR)?;
                false
            } else {
                token.want(TokenKind::ArrowL)?;
                true
            };

            let kind = Kind::from_token(token)?;

            Ok(Self {
                reference,
                name,
                kind,
            })
        })
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub value_i: Vec<Parameter>,
    pub value_o: Option<Kind>,
}

impl Function {
    fn from_token(token: &mut TokenBuffer) -> Result<Self, Error> {
        token.begin_context(Context::Function, |token| {
            token.want(TokenKind::Function)?;

            let name = token.want(TokenKind::Identifier)?.data.identifier();

            //================================

            let mut value_i = Vec::default();

            token.want(TokenKind::RoundBegin)?;

            token.pop_until(TokenKind::RoundClose, Some(TokenKind::Comma), |token| {
                value_i.push(Parameter::from_token(token)?);
                Ok(())
            })?;

            token.want(TokenKind::RoundClose)?;

            //================================

            let value_o = if token.want_peek(TokenKind::Colon, 0).is_some() {
                token.want(TokenKind::Colon)?;

                Some(Kind::from_token(token)?)
            } else {
                None
            };

            //================================

            token.want(TokenKind::CurlyBegin)?;

            // TO-DO parse statement
            token.pop_until(TokenKind::CurlyClose, None, |token| {
                token.pop();
                Ok(())
            })?;

            token.want(TokenKind::CurlyClose)?;

            //================================

            Ok(Self {
                name,
                value_i,
                value_o,
            })
        })
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    name: String,
    observer: Vec<(String, Kind)>,
    function: Vec<Function>,
}

impl Object {
    fn from_token(token: &mut TokenBuffer) -> Result<Self, Error> {
        token.begin_context(Context::Object, |token| {
            token.want(TokenKind::Object)?;

            let name = token.want(TokenKind::Identifier)?.data.identifier();

            //================================

            let mut observer = Vec::default();
            let mut function = Vec::default();

            token.want(TokenKind::CurlyBegin)?;

            // TO-DO parse statement
            token.pop_until(TokenKind::CurlyClose, None, |token| {
                if let Some(t) = token.peek(0) {
                    match t.data.kind() {
                        TokenKind::Function => {
                            function.push(Function::from_token(token)?);
                        }
                        TokenKind::Identifier => {
                            let name = token.want(TokenKind::Identifier)?.data.identifier();
                            token.want(TokenKind::Colon)?;
                            let kind = Kind::from_token(token)?;
                            observer.push((name, kind));
                        }
                        _ => {
                            return Err(
                                token.error_point(ErrorKind::InvalidDefinition(t.data), t.point)
                            );
                        }
                    }
                }

                Ok(())
            })?;

            token.want(TokenKind::CurlyClose)?;

            //================================

            Ok(Self {
                name,
                observer,
                function,
            })
        })
    }
}

pub enum Statement {
    Assignment(),
    Expression(),
    Condition(),
    Iteration(),
    Return(),
}
