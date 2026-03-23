use crate::split::buffer::*;
use crate::split::token::*;

//================================================================

#[derive(Debug, Clone)]
pub enum Instruction {
    Assignment(Assignment),
    Invocation(Invocation),
}

impl Instruction {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::utility::error::Error> {
        if token_buffer.want_peek(TokenKind::Let) {
            return Ok(Self::Assignment(Assignment::parse_token(token_buffer)?));
        } else if token_buffer.want_peek(TokenKind::String) {
            return Ok(Self::Invocation(Invocation::parse_token(token_buffer)?));
        }

        token_buffer.print_state();

        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub variable: Variable,
    pub value: String,
}

impl Assignment {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::utility::error::Error> {
        token_buffer.want(TokenKind::Let)?;
        let variable = Variable::parse_token(token_buffer)?;
        token_buffer.want(TokenKind::Assignment)?;
        let value = token_buffer.want(TokenKind::String)?.data.inner_string();

        Ok(Self { variable, value })
    }
}

#[derive(Debug, Clone)]
pub struct Invocation {
    pub name: String,
    pub list: Vec<String>,
}

impl Invocation {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::utility::error::Error> {
        let name = token_buffer.want(TokenKind::String)?.data.inner_string();
        let mut list = Vec::new();

        token_buffer.want(TokenKind::ParenthesisBegin)?;

        while let Some(token) = token_buffer.peek() {
            if token.data.kind() == TokenKind::ParenthesisClose {
                break;
            }

            if token.data.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(token_buffer.want(TokenKind::String)?.data.inner_string());
        }

        token_buffer.want(TokenKind::ParenthesisClose)?;

        Ok(Self { name, list })
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub list: Vec<Variable>,
    pub code: Vec<Instruction>,
}

impl Function {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::utility::error::Error> {
        let mut list = Vec::new();
        let mut code = Vec::new();

        let name = token_buffer.want(TokenKind::String)?.data.inner_string();
        token_buffer.want(TokenKind::ParenthesisBegin)?;

        // No argument branch.
        if token_buffer.want_peek(TokenKind::ParenthesisClose) {
            token_buffer.want(TokenKind::ParenthesisClose)?;
        } else {
            while let Some(token) = token_buffer.peek() {
                if token.data.kind() == TokenKind::ParenthesisClose {
                    break;
                }

                if token.data.kind() == TokenKind::Comma {
                    token_buffer.next();
                }

                list.push(Variable::parse_token(token_buffer)?);
            }

            token_buffer.want(TokenKind::ParenthesisClose)?;
        }

        token_buffer.want(TokenKind::BracketBegin)?;

        let mut bracket_begin = 1;

        while let Some(token) = token_buffer.peek() {
            if token.data.kind() == TokenKind::BracketBegin {
                bracket_begin -= 1;
            }

            if token.data.kind() == TokenKind::BracketClose {
                bracket_begin -= 1;
            }

            if bracket_begin == 0 {
                break;
            }

            code.push(Instruction::parse_token(token_buffer)?);
        }

        Ok(Self { name, list, code })
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub kind: String,
}

impl Variable {
    fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, crate::utility::error::Error> {
        let name = token_buffer.want(TokenKind::String)?.data.inner_string();
        token_buffer.want(TokenKind::Colon)?;
        let kind = token_buffer.want(TokenKind::String)?.data.inner_string();

        Ok(Self { name, kind })
    }
}

//================================================================

#[derive(Debug)]
pub struct Structure {
    pub name: String,
    pub list: Vec<Variable>,
}

impl Structure {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::utility::error::Error> {
        let mut list = Vec::new();

        let name = token_buffer.want(TokenKind::String)?.data.inner_string();

        token_buffer.want(TokenKind::BracketBegin)?;

        while let Some(token) = token_buffer.peek() {
            if token.data.kind() == TokenKind::BracketClose {
                break;
            }

            if token.data.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(Variable::parse_token(token_buffer)?);
        }

        token_buffer.want(TokenKind::BracketClose)?;

        Ok(Self { name, list })
    }
}

//================================================================

#[derive(Debug)]
pub struct Enumerate {
    pub name: String,
    pub list: Vec<Variable>,
}
