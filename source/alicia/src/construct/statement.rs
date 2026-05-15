use super::assignment::*;
use super::block::*;
use super::condition::*;
use super::definition::*;
use super::enumerate::*;
use super::expression::*;
use super::function::*;
use super::iteration::*;
use super::resolve::*;
use super::structure::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub enum Statement {
    Function(Function),
    Structure(Structure),
    Enumerate(Enumerate),
    Definition(Definition),
    Assignment(Assignment),
    Expression(Expression),
    Condition(Condition),
    Iteration(Iteration),
    Block(Block),
    Skip,
    Exit,
    Return(Return),
}

impl Statement {
    fn parse_identifier(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        let expression = Expression::parse_token(token_buffer, 0.0)?;

        if token_buffer.want_peek(TokenKind::ColonSemi) {
            token_buffer.want(TokenKind::ColonSemi)?;
            return Ok(Self::Expression(expression));
        }

        Ok(Self::Assignment(Assignment::parse_token(
            token_buffer,
            expression,
        )?))
    }

    pub fn parse_comma<F: FnMut(&mut TokenBuffer) -> Result<(), Error>>(
        token_buffer: &mut TokenBuffer,
        delimiter: TokenKind,
        mut call: F,
    ) -> Result<(), Error> {
        while let Some(token) = token_buffer.peek() {
            if token.class.kind() == delimiter {
                break;
            }

            call(token_buffer)?;

            if let Some(token) = token_buffer.peek()
                && token.class.kind() == TokenKind::Comma
            {
                token_buffer.next();
            } else {
                break;
            }
        }

        Ok(())
    }

    #[rustfmt::skip]
    pub fn parse_token(token: Token, token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        match token.class {
            TokenClass::Function  => Ok(Self::Function(Function::parse_token(token_buffer, None)?)),
            TokenClass::Structure => Ok(Self::Structure(Structure::parse_token(token_buffer)?)),
            TokenClass::Enumerate => Ok(Self::Enumerate(Enumerate::parse_token(token_buffer)?)),
            TokenClass::Let       => Ok(Self::Definition(Definition::parse_token(token_buffer)?)),
            TokenClass::If        => Ok(Self::Condition(Condition::parse_token(
                token_buffer,
                false,
            )?)),
            TokenClass::Loop => Ok(Self::Iteration(Iteration::parse_token(token_buffer)?)),
            TokenClass::Skip => {
                token_buffer.want(TokenKind::Skip)?;
                Ok(Self::Skip)
            }
            TokenClass::Exit => {
                token_buffer.want(TokenKind::Exit)?;
                Ok(Self::Exit)
            }
            TokenClass::Return        => Ok(Self::Return(Return::parse_token(token_buffer)?)),
            TokenClass::Identifier(_) => Ok(Self::parse_identifier(token_buffer)?),
            TokenClass::CurlyBegin    => Ok(Self::Block(Block::parse_token(token_buffer)?)),
            _ => Error::new_info(
                token_buffer.get_error_info(Some(token.clone())),
                ErrorKind::UnknownToken(token),
                Some(ErrorHint::Function),
            ),
        }
    }
}
