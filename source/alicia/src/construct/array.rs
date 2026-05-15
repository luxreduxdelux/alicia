use super::expression::*;
use super::statement::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct ArrayD {
    pub list: Vec<Expression>,
}

impl ArrayD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        token_buffer.parse(ErrorHint::Array, |token_buffer| {
            let mut list = Vec::new();

            token_buffer.want(TokenKind::SquareBegin)?;

            Statement::parse_comma(token_buffer, TokenKind::SquareClose, |token_buffer| {
                list.push(Expression::parse_token(token_buffer, 0.0)?);
                Ok(())
            })?;

            token_buffer.want(TokenKind::SquareClose)?;

            Ok(Self { list })
        })
    }

    pub fn analyze(
        &self,
        scope: &Scope,
        infer: Option<ExpressionKind>,
    ) -> Result<ExpressionKind, Error> {
        let infer = if let Some(infer) = infer {
            match infer {
                ExpressionKind::Array(kind) => Some(*kind),
                x => panic!("non-array kind for array definition {x:?}"),
            }
        } else {
            None
        };

        let mut current = infer;

        for expression in &self.list {
            let kind = expression.analyze(scope, current.clone())?;

            if let Some(ref current) = current {
                if kind != *current {
                    panic!("type mis-match in array literal ({kind:?} != {current:?})")
                }
            } else {
                current = Some(kind)
            }
        }

        Ok(ExpressionKind::Array(Box::new(
            current.expect("could not infer type for array"),
        )))
    }
}
