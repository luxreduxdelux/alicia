use super::{error::*, token::*};

//================================================================

#[derive(Debug)]
pub enum Declaration {
    Function(Function),
    // Structure(Structure)
}

#[derive(Debug)]
pub enum Instruction {
    Assignment(Assignment),
    Invocation(Invocation),
}

impl Instruction {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, AliciaError> {
        if token_buffer.want_peek(TokenKind::Let) {
            return Ok(Self::Assignment(Assignment::parse_token(token_buffer)?));
        } else if token_buffer.want_peek(TokenKind::String) {
            return Ok(Self::Invocation(Invocation::parse_token(token_buffer)?));
        }

        token_buffer.print_state();

        todo!()
    }
}

#[derive(Debug)]
pub struct Assignment {
    pub variable: Variable,
    pub value: String,
}

impl Assignment {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, AliciaError> {
        token_buffer.want(TokenKind::Let)?;
        let variable = Variable::parse_token(token_buffer)?;
        token_buffer.want(TokenKind::Assignment)?;
        let value = token_buffer.want(TokenKind::String)?.inner_string();

        Ok(Self { variable, value })
    }
}

#[derive(Debug)]
pub struct Invocation {
    pub name: String,
    pub list: Vec<String>,
}

impl Invocation {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, AliciaError> {
        let name = token_buffer.want(TokenKind::String)?.inner_string();
        let mut list = Vec::new();

        token_buffer.want(TokenKind::ParenthesisBegin)?;

        while let Some(token) = token_buffer.peek() {
            if token.kind() == TokenKind::ParenthesisClose {
                break;
            }

            if token.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(token_buffer.want(TokenKind::String)?.inner_string());
        }

        token_buffer.want(TokenKind::ParenthesisClose)?;

        Ok(Self { name, list })
    }
}

//================================================================

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub list: Vec<Variable>,
    pub code: Vec<Instruction>,
}

impl Function {
    pub fn parse_token(token: &Token, token_buffer: &mut TokenBuffer) -> Result<Self, AliciaError> {
        let mut list = Vec::new();
        let mut code = Vec::new();

        let name = token_buffer.want(TokenKind::String)?.inner_string();
        token_buffer.want(TokenKind::ParenthesisBegin)?;

        // No argument branch.
        if token_buffer.want_peek(TokenKind::ParenthesisClose) {
            token_buffer.want(TokenKind::ParenthesisClose)?;
        } else {
            while let Some(token) = token_buffer.peek() {
                if token.kind() == TokenKind::ParenthesisClose {
                    break;
                }

                if token.kind() == TokenKind::Comma {
                    token_buffer.next();
                }

                list.push(Variable::parse_token(token_buffer)?);
            }

            token_buffer.want(TokenKind::ParenthesisClose)?;
        }

        token_buffer.want(TokenKind::BracketBegin)?;

        let mut bracket_begin = 1;

        while let Some(token) = token_buffer.peek() {
            if token.kind() == TokenKind::BracketBegin {
                bracket_begin -= 1;
            }

            if token.kind() == TokenKind::BracketClose {
                bracket_begin -= 1;
            }

            if bracket_begin == 0 {
                break;
            }

            code.push(Instruction::parse_token(token_buffer)?);
        }

        Ok(Self { name, list, code })

        /*
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
        */
    }
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub kind: String,
}

impl Variable {
    fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, AliciaError> {
        let name = token_buffer.want(TokenKind::String)?.inner_string();
        token_buffer.want(TokenKind::Colon)?;
        let kind = token_buffer.want(TokenKind::String)?.inner_string();

        Ok(Self { name, kind })
    }
}
