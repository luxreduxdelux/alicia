use crate::split::buffer::*;
use crate::split::helper::*;
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
    ) -> Result<Self, crate::helper::error::Error> {
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
    ) -> Result<Self, crate::helper::error::Error> {
        token_buffer.want(TokenKind::Let)?;
        let variable = Variable::parse_token(token_buffer)?;
        token_buffer.want(TokenKind::Assignment)?;
        let value = token_buffer.want(TokenKind::String)?.class.inner_string();

        Ok(Self { variable, value })
    }
}

#[derive(Debug, Clone)]
pub struct Invocation {
    pub name: Identifier,
    pub list: Vec<String>,
}

impl Invocation {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::helper::error::Error> {
        let name = token_buffer.want(TokenKind::String)?.class.inner_string();
        let mut list = Vec::new();

        token_buffer.want(TokenKind::ParenthesisBegin)?;

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::ParenthesisClose {
                break;
            }

            if token.class.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(token_buffer.want(TokenKind::String)?.class.inner_string());
        }

        token_buffer.want(TokenKind::ParenthesisClose)?;

        Ok(Self {
            name: name.try_into()?,
            list,
        })
    }
}

//================================================================

#[derive(Debug, Clone)]
pub struct Function {
    pub name_structure: Option<Identifier>,
    pub name: Identifier,
    pub enter: Vec<Variable>,
    pub leave: Option<Identifier>,
    pub code: Vec<Instruction>,
}

impl Function {
    #[rustfmt::skip]
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::helper::error::Error> {
        let mut name_structure = None;
        let mut name           = token_buffer.want(TokenKind::String)?.class.inner_string();
        let mut enter          = Vec::new();
        let mut leave          = None;
        let mut code           = Vec::new();

        if token_buffer.want_peek(TokenKind::Dot) {
            token_buffer.want(TokenKind::Dot)?;
            name_structure = Some(name.try_into()?);
            name = token_buffer.want(TokenKind::String)?.class.inner_string();
        }

        token_buffer.want(TokenKind::ParenthesisBegin)?;

        // No argument branch.
        if token_buffer.want_peek(TokenKind::ParenthesisClose) {
            token_buffer.want(TokenKind::ParenthesisClose)?;
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

            token_buffer.want(TokenKind::ParenthesisClose)?;
        }

        if token_buffer.want_peek(TokenKind::Colon) {
            token_buffer.want(TokenKind::Colon)?;
            leave = Some(
                token_buffer
                    .want(TokenKind::String)?
                    .class
                    .inner_string()
                    .try_into()?,
            );
        }

        token_buffer.want(TokenKind::CurlyBegin)?;

        let mut bracket_begin = 1;

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::CurlyBegin {
                bracket_begin -= 1;
            }

            if token.class.kind() == TokenKind::CurlyClose {
                bracket_begin -= 1;
            }

            if bracket_begin == 0 {
                break;
            }

            code.push(Instruction::parse_token(token_buffer)?);
        }

        token_buffer.want(TokenKind::CurlyClose)?;

        Ok(Self {
            name_structure,
            name: name.try_into()?,
            enter,
            leave,
            code,
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
    fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, crate::helper::error::Error> {
        let name = token_buffer.want(TokenKind::String)?.class.inner_string();
        token_buffer.want(TokenKind::Colon)?;

        let reference = if token_buffer.want_peek(TokenKind::Ampersand) {
            token_buffer.want(TokenKind::Ampersand)?;
            true
        } else {
            false
        };

        let kind = token_buffer.want(TokenKind::String)?.class.inner_string();

        Ok(Self {
            name: name.try_into()?,
            kind: kind.try_into()?,
            reference,
        })
    }
}

//================================================================

#[derive(Debug)]
pub struct Structure {
    pub name: Identifier,
    pub list: Vec<Variable>,
}

impl Structure {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::helper::error::Error> {
        let mut list = Vec::new();

        let name = token_buffer.want(TokenKind::String)?.class.inner_string();

        token_buffer.want(TokenKind::CurlyBegin)?;

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::CurlyClose {
                break;
            }

            if token.class.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(Variable::parse_token(token_buffer)?);
        }

        token_buffer.want(TokenKind::CurlyClose)?;

        Ok(Self {
            name: name.try_into()?,
            list,
        })
    }
}

//================================================================

#[derive(Debug)]
pub struct Enumerate {
    pub name: Identifier,
    pub list: Vec<Identifier>,
}

impl Enumerate {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::helper::error::Error> {
        let mut list = Vec::new();

        let name = token_buffer.want(TokenKind::String)?.class.inner_string();

        token_buffer.want(TokenKind::CurlyBegin)?;

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::CurlyClose {
                break;
            }

            if token.class.kind() == TokenKind::Comma {
                token_buffer.next();
            }

            list.push(
                token_buffer
                    .want(TokenKind::String)?
                    .class
                    .inner_string()
                    .try_into()?,
            );
        }

        token_buffer.want(TokenKind::CurlyClose)?;

        Ok(Self {
            name: name.try_into()?,
            list,
        })
    }
}

//================================================================

#[derive(Debug)]
pub struct Use {
    pub path: Path,
}

impl Use {
    pub fn parse_token(
        token_buffer: &mut TokenBuffer,
    ) -> Result<Self, crate::helper::error::Error> {
        let mut path = Path::default();

        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == TokenKind::String {
                path.push(token_buffer.want(TokenKind::String)?.class.inner_string())?;
            } else if token.class.kind() == TokenKind::Dot {
                token_buffer.want(TokenKind::Dot)?;
            } else {
                break;
            }
        }

        Ok(Self { path })
    }
}
