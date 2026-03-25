use crate::helper::error::*;
use crate::stage_1::buffer::*;
use crate::stage_1::helper::*;
use crate::stage_1::token::*;

//================================================================

#[derive(Debug, Clone)]
pub enum Instruction {
    Function(Function),
    Structure(Structure),
    Enumerate(Enumerate),
    Definition(Definition),
    Invocation(Invocation),
}

impl Instruction {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        // TO-DO for, while, if

        while let Some(token) = token_buffer.peek() {
            match token.class {
                TokenClass::Function => Ok(Self::Function(Function::parse_token(token_buffer)?)),
                TokenClass::Structure => Ok(Self::Structure(Structure::parse_token(token_buffer)?)),
                TokenClass::Enumerate => Ok(Self::Enumerate(Enumerate::parse_token(token_buffer)?)),
                TokenClass::String(_) => {
                    let cursor = token_buffer.get_cursor();

                    if let Ok(definition) = Definition::parse_token(token_buffer) {
                        return Ok(Self::Definition(definition));
                    }

                    token_buffer.set_cursor(cursor);
                    return Ok(Self::Invocation(Invocation::parse_token(token_buffer)?));
                }
                _ => {
                    println!("t_c_return");
                    return Err(Error::new_info(
                        token_buffer.get_error_info(Some(token.clone())),
                        ErrorKind::UnknownToken(token),
                        Some(ErrorHint::Function),
                    ));
                }
            }?;
        }

        println!("return");

        let token = token_buffer.next();

        Err(Error::new_info(
            token_buffer.get_error_info(token.clone()),
            ErrorKind::UnknownToken(token.unwrap()),
            Some(ErrorHint::Function),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: Identifier,
    pub kind: Token,
    pub value: String,
}

impl Definition {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let name = token_buffer.want_identifier(ErrorHint::Definition)?;
        let kind = token_buffer.want_definition(ErrorHint::Definition)?;
        let value = token_buffer
            .want(TokenKind::String, ErrorHint::Definition)?
            .class
            .inner_string();

        Ok(Self { name, kind, value })
    }
}

#[derive(Debug, Clone)]
pub struct Invocation {
    pub name: Identifier,
    pub list: Vec<String>,
}

impl Invocation {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let name = token_buffer.want_identifier(ErrorHint::Invocation)?;
        let mut list = Vec::new();

        token_buffer.want(TokenKind::ParenthesisBegin, ErrorHint::Invocation)?;

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::ParenthesisClose {
                break;
            }

            if token.class.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(
                token_buffer
                    .want(TokenKind::String, ErrorHint::Invocation)?
                    .class
                    .inner_string(),
            );
        }

        token_buffer.want(TokenKind::ParenthesisClose, ErrorHint::Invocation)?;

        Ok(Self { name, list })
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Block {
    pub code: Vec<Instruction>,
    pub block: Vec<Box<Block>>,
}

impl Block {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let mut code = Vec::new();
        let mut block = Vec::new();

        token_buffer.want(TokenKind::CurlyBegin, ErrorHint::Function)?;

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::CurlyBegin {
                block.push(Box::new(Self::parse_token(token_buffer)?));
            } else if token.class.kind() == TokenKind::CurlyClose {
                break;
            } else {
                code.push(Instruction::parse_token(token_buffer)?);
            }
        }

        token_buffer.want(TokenKind::CurlyClose, ErrorHint::Function)?;

        Ok(Self { code, block })
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name_structure: Option<Identifier>,
    pub name: Identifier,
    pub enter: Vec<Variable>,
    pub leave: Option<Identifier>,
    pub block: Block,
}

impl Function {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.want(TokenKind::Function, ErrorHint::Function)?;

        let mut name_structure = None;
        let mut name = token_buffer.want_identifier(ErrorHint::Function)?;
        let mut enter = Vec::new();
        let mut leave = None;

        if token_buffer.want_peek(TokenKind::Dot) {
            token_buffer.want(TokenKind::Dot, ErrorHint::Function)?;
            name_structure = Some(name);
            name = token_buffer.want_identifier(ErrorHint::Function)?;
        }

        token_buffer.want(TokenKind::ParenthesisBegin, ErrorHint::Function)?;

        // No argument branch.
        if token_buffer.want_peek(TokenKind::ParenthesisClose) {
            token_buffer.want(TokenKind::ParenthesisClose, ErrorHint::Function)?;
        } else {
            while let Some(token) = token_buffer.peek() {
                if token.class.kind() == TokenKind::ParenthesisClose {
                    break;
                }

                if token.class.kind() == TokenKind::Comma {
                    token_buffer.next();
                }

                enter.push(Variable::parse_token(token_buffer)?);
            }

            token_buffer.want(TokenKind::ParenthesisClose, ErrorHint::Function)?;
        }

        if token_buffer.want_peek(TokenKind::Colon) {
            token_buffer.want(TokenKind::Colon, ErrorHint::Function)?;
            leave = Some(token_buffer.want_identifier(ErrorHint::Function)?);
        }

        let block = Block::parse_token(token_buffer)?;

        Ok(Self {
            name_structure,
            name,
            enter,
            leave,
            block,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Identifier,
    pub kind: Identifier,
    pub reference: bool,
}

impl Variable {
    fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let name = token_buffer.want_identifier(ErrorHint::Variable)?;
        token_buffer.want(TokenKind::Colon, ErrorHint::Variable)?;

        let reference = if token_buffer.want_peek(TokenKind::Ampersand) {
            token_buffer.want(TokenKind::Ampersand, ErrorHint::Variable)?;
            true
        } else {
            false
        };

        let kind = token_buffer.want_identifier(ErrorHint::Variable)?;

        Ok(Self {
            name,
            kind,
            reference,
        })
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Structure {
    pub name: Identifier,
    pub list: Vec<Variable>,
}

impl Structure {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let mut list = Vec::new();

        token_buffer.want(TokenKind::Structure, ErrorHint::Structure)?;

        let name = token_buffer.want_identifier(ErrorHint::Structure)?;

        token_buffer.want(TokenKind::CurlyBegin, ErrorHint::Structure)?;

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::CurlyClose {
                break;
            }

            if token.class.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(Variable::parse_token(token_buffer)?);
        }

        token_buffer.want(TokenKind::CurlyClose, ErrorHint::Structure)?;

        Ok(Self { name, list })
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Enumerate {
    pub name: Identifier,
    pub list: Vec<Identifier>,
}

impl Enumerate {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let mut list = Vec::new();

        token_buffer.want(TokenKind::Enumerate, ErrorHint::Enumerate)?;

        let name = token_buffer.want_identifier(ErrorHint::Enumerate)?;

        token_buffer.want(TokenKind::CurlyBegin, ErrorHint::Enumerate)?;

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::CurlyClose {
                break;
            }

            if token.class.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(token_buffer.want_identifier(ErrorHint::Enumerate)?);
        }

        token_buffer.want(TokenKind::CurlyClose, ErrorHint::Enumerate)?;

        Ok(Self { name, list })
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Use {
    pub path: Path,
}

impl Use {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let mut path = Path::default();

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::String {
                path.push(token_buffer.want_identifier(ErrorHint::Use)?);
            } else if token.class.kind() == TokenKind::Dot {
                token_buffer.want(TokenKind::Dot, ErrorHint::Use)?;
            } else {
                break;
            }
        }

        Ok(Self { path })
    }
}
