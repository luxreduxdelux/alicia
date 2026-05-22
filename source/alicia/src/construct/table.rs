use super::expression::*;
use super::statement::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct TableD {
    pub list: Vec<(Expression, Expression)>,
}

impl TableD {
    pub fn parse_token(token_buffer: &mut TokenBuffer) -> Result<Self, Error> {
        // TO-DO use Hint::Table
        token_buffer.parse(ErrorHint::Array, |token_buffer| {
            let mut list = Vec::new();

            token_buffer.want(TokenKind::CurlyBegin)?;

            Statement::parse_comma(token_buffer, TokenKind::CurlyClose, |token_buffer| {
                let k = Expression::parse_token(token_buffer, 0.0)?;
                token_buffer.want(TokenKind::DefinitionVariable)?;
                let v = Expression::parse_token(token_buffer, 0.0)?;

                list.push((k, v));

                Ok(())
            })?;

            token_buffer.want(TokenKind::CurlyClose)?;

            Ok(Self { list })
        })
    }

    pub fn analyze(
        &self,
        scope: &Scope,
        infer: Option<ExpressionKind>,
    ) -> Result<ExpressionKind, Error> {
        let (i_a, i_b) = if let Some(infer) = infer {
            match infer {
                ExpressionKind::Table(a, b) => (Some(*a), Some(*b)),
                x => panic!("non-table kind for table definition {x:?}"),
            }
        } else {
            (None, None)
        };

        let mut c_a = i_a;
        let mut c_b = i_b;

        for (e_a, e_b) in &self.list {
            let k_a = e_a.analyze(scope, c_a.clone())?;
            let k_b = e_b.analyze(scope, c_b.clone())?;

            if let Some(ref c_a) = c_a {
                if k_a != *c_a {
                    panic!("type mis-match in array literal ({k_a:?} != {c_a:?})")
                }
            } else {
                c_a = Some(k_a)
            }

            if let Some(ref c_b) = c_b {
                if k_b != *c_b {
                    panic!("type mis-match in array literal ({k_b:?} != {c_b:?})")
                }
            } else {
                c_b = Some(k_b)
            }
        }

        Ok(ExpressionKind::Table(
            Box::new(c_a.unwrap()),
            Box::new(c_b.unwrap()),
        ))
    }
}
