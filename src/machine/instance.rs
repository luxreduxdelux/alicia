use super::buffer::*;
use super::error::*;
use super::scope::*;
use super::value::*;
use crate::parse::construct::*;
use crate::split::buffer::*;
use crate::split::token::*;
use crate::split::utility::*;

//================================================================

pub struct Instance<'a> {
    scope: Scope<'a>,
}

impl<'a> Instance<'a> {
    pub fn new() -> Self {
        let mut scope = Scope::default();

        scope.set_value(
            "print",
            Value::FunctionNative(Box::new(|mut argument| {
                let mut format = argument.want(ValueKind::String)?.as_string()?;
                let mut buffer = Vec::new();

                while argument.peek() {
                    buffer.push(argument.want(ValueKind::String)?.as_string()?);
                }

                for (i, replace) in buffer.iter().enumerate() {
                    format = format.replacen("{}", replace, i + 1);
                }

                println!("{}", format);

                //println!("{}", argument.want(ValueKind::String)?.as_string()?);

                Ok(Value::Null)
            })),
        );

        Self { scope }
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), crate::utility::error::Error> {
        let mut token_buffer = TokenBuffer::new(Source::new_file(path)?);

        while let Some(token) = token_buffer.next() {
            match token.data {
                TokenData::Function => {
                    let value = Function::parse_token(&mut token_buffer)?;
                    self.scope
                        .set_value(&value.name.clone(), Value::Function(value));
                }
                /*
                TokenData::Enumerate => {
                    let value = Enumerate::parse_token(&mut token_buffer)?;
                    scope.set_value(&value.name.clone(), Value::Enumerate(value));
                }
                */
                TokenData::Structure => {
                    let value = Structure::parse_token(&mut token_buffer)?;
                    self.scope
                        .set_value(&value.name.clone(), Value::Structure(value));
                }
                _ => {}
            };
        }

        println!("scope: {:?}", self.scope);

        Ok(())
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.scope.get_value(name)
    }

    pub fn call_function(
        &self,
        function: &Function,
        argument_list: Vec<String>,
    ) -> Result<Value, Error> {
        Self::call_function_aux(&self.scope, function, argument_list)
    }

    fn call_function_aux(
        scope: &Scope,
        function: &Function,
        argument_list: Vec<String>,
    ) -> Result<Value, Error> {
        let mut local = Scope::new(Some(scope));

        for instruction in &function.code {
            match instruction {
                Instruction::Assignment(assignment) => {
                    local.set_value(
                        &assignment.variable.name,
                        Value::parse_text(&assignment.variable.kind, &assignment.value)?,
                    );
                }
                Instruction::Invocation(invocation) => {
                    if let Some(function) = local.get_value(&invocation.name) {
                        match function {
                            Value::Function(function) => {
                                return Self::call_function_aux(
                                    &local,
                                    function,
                                    argument_list.clone(),
                                );
                            }
                            Value::FunctionNative(function) => {
                                return function(ArgumentBuffer::new(invocation.list.clone()));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(Value::Null)
    }
}
