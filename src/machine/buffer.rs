use super::error::*;
use super::value::*;

//================================================================

pub struct ArgumentBuffer {
    buffer: Vec<String>,
    cursor: usize,
}

impl ArgumentBuffer {
    pub fn new(buffer: Vec<String>) -> Self {
        Self {
            buffer,
            cursor: usize::default(),
        }
    }

    pub fn want(&mut self, kind: ValueKind) -> Result<Value, Error> {
        if let Some(next) = self.buffer.get(self.cursor) {
            self.cursor += 1;

            if let Ok(value) = Value::parse_kind(kind, next) {
                if value.kind() == kind {
                    return Ok(value);
                } else {
                    return Err(Error::IncorrectKind(kind, value.kind()));
                }
            }
        }

        Err(Error::IncorrectKind(kind, ValueKind::Null))
    }

    pub fn peek(&self) -> bool {
        self.buffer.get(self.cursor).is_some()
    }

    pub fn size(&self) -> usize {
        self.buffer.iter().len()
    }
}
