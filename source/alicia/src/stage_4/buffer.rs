use crate::helper::error::*;
use crate::stage_2::construct::*;
use crate::stage_2::scope::*;

//================================================================

pub struct ArgumentBuffer {
    buffer: Vec<Value>,
    cursor: usize,
}

impl ArgumentBuffer {
    pub fn new(expression_list: Vec<Expression>, scope: &mut Scope) -> Result<Self, Error> {
        let mut buffer = Vec::new();

        for expression in expression_list {
            if let Some(expression) = expression.evaluate(scope)? {
                buffer.push(expression);
            }
        }

        Ok(Self {
            buffer,
            cursor: usize::default(),
        })
    }

    pub fn next(&mut self) -> Option<Value> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;
            return Some(next.clone());
        }

        None
    }

    pub fn peek(&self) -> Option<&Value> {
        if let Some(next) = self.buffer.get(self.cursor) {
            return Some(next);
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
