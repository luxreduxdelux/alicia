use crate::parser::instruction::*;
use std::collections::HashMap;

pub struct Machine {}

impl Machine {
    pub fn execute_function(function: Function) {
        let mut scope = Scope::default();

        for instruction in function.code {
            match instruction {
                Instruction::Assignment(assignment) => {
                    scope.set_value(
                        &assignment.variable.name,
                        Value::parse_text(&assignment.variable.kind, &assignment.value),
                    );
                }
                Instruction::Invocation(invocation) => {
                    // look into the global or local scope for calling into a function.
                }
            }
        }

        println!("{scope:?}");
    }
}

#[derive(Debug, Default)]
struct Scope {
    symbol: HashMap<String, Value>,
}

impl Scope {
    fn set_value(&mut self, name: &str, value: Value) {
        self.symbol.insert(name.to_string(), value);
    }

    fn get_value(&self, name: &str) -> Option<&Value> {
        self.symbol.get(name)
    }
}

#[derive(Debug)]
enum Value {
    String(String),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
}

impl Value {
    fn parse_text(kind: &str, text: &str) -> Self {
        let kind = ValueKind::parse_text(kind);

        match kind {
            ValueKind::String => {
                return Self::String(text.to_string());
            }
            ValueKind::Integer => {
                if let Ok(integer) = text.parse::<i32>() {
                    return Self::Integer(integer);
                }
            }
            ValueKind::Decimal => {
                if let Ok(decimal) = text.parse::<f32>() {
                    return Self::Decimal(decimal);
                }
            }
            ValueKind::Boolean => {
                if text == "true" {
                    return Self::Boolean(true);
                } else if text == "false" {
                    return Self::Boolean(false);
                }
            }
        }

        panic!("Value::parse_text: Unknown value.")
    }
}

#[derive(Debug)]
enum ValueKind {
    String,
    Integer,
    Decimal,
    Boolean,
}

impl ValueKind {
    fn parse_text(kind: &str) -> Self {
        match kind {
            "String" => Self::String,
            "Integer" => Self::Integer,
            "Decimal" => Self::Decimal,
            "Boolean" => Self::Boolean,
            _ => panic!("ValueKind::parse_text(): Unknown kind."),
        }
    }
}
