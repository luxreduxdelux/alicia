use crate::helper::error::*;
use crate::stage_2::construct::*;
use crate::stage_2::scope::*;

//================================================================

pub struct ArgumentBuffer {
    buffer: Vec<ExpressionValue>,
    cursor: usize,
}

impl ArgumentBuffer {
    pub fn new(expression_list: Vec<Expression>, scope: &Scope) -> Result<Self, Error> {
        let mut buffer = Vec::new();

        for expression in expression_list {
            buffer.push(expression.evaluate(scope)?);
        }

        Ok(Self {
            buffer,
            cursor: usize::default(),
        })
    }

    pub fn next(&mut self) -> Option<&ExpressionValue> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;
            return Some(next);
        }

        None
    }

    pub fn peek(&self) -> Option<&ExpressionValue> {
        if let Some(next) = self.buffer.get(self.cursor) {
            return Some(next);
        }

        None
    }

    // TO-DO add fn want

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
