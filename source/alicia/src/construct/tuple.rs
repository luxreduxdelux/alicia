use super::expression::*;
use super::statement::*;

//================================================================

use crate::buffer::*;
use crate::error::*;
use crate::scope::*;
use crate::token::*;

//================================================================

#[derive(Debug, Clone)]
pub struct TupleD {
    pub list: Vec<Expression>,
}

impl TupleD {
    pub fn new(list: Vec<Expression>) -> Self {
        Self { list }
    }

    pub fn analyze(
        &self,
        scope: &Scope,
        infer: Option<ExpressionKind>,
    ) -> Result<ExpressionKind, Error> {
        let mut list = Vec::default();

        for expression in &self.list {
            list.push(expression.analyze(scope, None)?);
        }

        Ok(ExpressionKind::Tuple(list))
    }
}
