use super::expression::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::machine::Function as MFunction;
use crate::machine::Instruction;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Option<Expression>,
}

impl Return {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Return, |token_buffer| {
            token_buffer.want(TokenKind::Return)?;

            let value = if token_buffer.want_peek(TokenKind::ColonSemi) {
                None
            } else {
                Some(Expression::parse_token(token_buffer, 0.0)?)
            };

            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self { value })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<ExpressionKind, Error> {
        if let Some(value) = &self.value {
            value.analyze(scope, None)
        } else {
            Ok(ExpressionKind::Null)
        }
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        if let Some(value) = &self.value {
            value.compile(scope, function)?;
            function.push(Instruction::Return(true));
        } else {
            function.push(Instruction::Return(false));
        }

        Ok(())
    }
}
