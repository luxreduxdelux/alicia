use super::expression::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::machine::Function as MFunction;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct Assignment {
    pub span: TokenSpan,
    pub path: Expression,
    pub kind: Token,
    pub value: Expression,
}

impl Assignment {
    pub fn parse_token(token_buffer: &mut TokenBuffer, path: Expression) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Assignment, |token_buffer| {
            let kind = token_buffer.want_definition()?;
            let value = Expression::parse_token(token_buffer, 0.0)?;
            token_buffer.want(TokenKind::ColonSemi)?;

            Ok(Self {
                span: token_buffer.get_span(),
                path: path.clone(),
                kind,
                value,
            })
        })
    }

    pub fn analyze(&self, scope: &Scope) -> Result<(), Error> {
        self.value.analyze(scope, None)?;

        // TO-DO analyze if it's correct to load the value onto our path.

        Ok(())
    }

    pub fn compile(&self, scope: &Scope, function: &mut MFunction) -> Result<(), Error> {
        self.value.compile(scope, function)?;
        self.path.compile_l(scope, function, false)?;

        Ok(())
    }
}
