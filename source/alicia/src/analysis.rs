use crate::{
    construct::ExpressionKind,
    error::*,
    machine::{Argument, Machine, Value, ValueKind},
    scope::*,
};
use alicia_macro::function;
use alicia_macro::function_add;

struct FunctionMeta {
    enter: NativeArgument,
    leave: ValueType,
}

impl FunctionMeta {
    const fn new(enter: NativeArgument, leave: ValueType) {}
}

pub struct Analysis {}

impl Analysis {
    #[function]
    fn test(a: String, b: i64, c: f64) -> Option<Value> {
        None
    }

    fn format_internal(mut argument: Argument) -> Result<String, Error> {
        let string = argument.next().unwrap().as_string();
        let mut string = string.chars();
        let mut result = String::default();

        while let Some(character) = string.next() {
            match character {
                '{' => {
                    let mut identifier = String::default();

                    while let Some(character) = string.next() {
                        if character == '}' {
                            break;
                        }

                        identifier.push(character);
                    }

                    //if identifier.is_empty() {
                    if let Some(value) = argument.next() {
                        result.push_str(&value.to_string());
                    }

                    // TO-DO unavailable until a look-up table is made between String -> stack slot..
                    /*
                    } else {
                        let split: Vec<&str> = identifier.split(".").collect();

                        if split.len() == 1 {
                            if let Some(value) = argument.memory.get(&identifier) {
                                result.push_str(&value.to_string());
                            } else {
                                panic!("No variable by the name of {identifier}.")
                            }
                        } else {
                            let mut current = None;

                            for access in split {
                                if let Some(a_current) = &current {
                                    match a_current {
                                        Value::Structure(structure) => {
                                            if let Some(value) = structure.data.get(access) {
                                                current = Some(value.clone());
                                            } else {
                                                panic!(
                                                    "No field in structure {} by the name of {access}.",
                                                    structure.kind
                                                )
                                            }
                                        }
                                        x => current = Some(x.clone()),
                                    }
                                } else {
                                    if let Some(value) = argument.memory.get(access) {
                                        current = Some(value.clone());
                                    } else {
                                        panic!("No variable by the name of {access}.")
                                    }
                                }
                            }

                            if let Some(current) = current {
                                result.push_str(&current.to_string());
                            }
                        }
                    }
                    */
                }
                _ => result.push(character),
            }
        }

        Ok(result)
    }

    fn print(machine: &mut Machine, argument: Argument) -> Option<Value> {
        println!("{}", Self::format_internal(argument).unwrap());

        None
    }

    #[rustfmt::skip]
    pub fn analyze_tree(mut scope: Scope) -> Result<Scope, Error> {
        scope.symbol.insert("print".to_string(), Declaration::FunctionNative(FunctionNative {
            name: "print".to_string(),
            call: Self::print,
            enter: NativeArgument::Variable,
            leave: ExpressionKind::Null,
        }));
        scope.symbol.insert("test".to_string(), Declaration::FunctionNative(FunctionNative {
            name: "test".to_string(),
            call: Self::test,
            enter: NativeArgument::Variable,
            leave: ExpressionKind::Boolean,
        }));

        //function_add!(scope.clone(), "print", Self::print);

        let mut scope_clone = scope.clone();

        for (_, value) in scope.symbol.clone() {

            // TO-DO this is quite bad. I think in a future design we should make a difference
            // between pre-analyze declaration and post-analyze declaration rather than modify
            // everything in-place.
            match value {
                Declaration::Function(mut function) => {
                    function.analyze(&mut scope_clone)?;
                    scope_clone.set_declaration(function.name.clone(), Declaration::Function(function));
                },
                Declaration::Structure(mut structure) => {
                    structure.analyze(&mut scope_clone)?;
                    scope_clone.set_declaration(structure.name.clone(), Declaration::Structure(structure));
                },
                Declaration::Enumerate(mut enumerate) => {
                    enumerate.analyze(&mut scope_clone)?;
                    scope_clone.set_declaration(enumerate.name.clone(), Declaration::Enumerate(enumerate));
                },
                Declaration::Definition(mut definition) => {
                    definition.analyze(&mut scope_clone)?;
                    scope_clone.set_declaration(definition.name.clone(), Declaration::Definition(definition));
                },
                _ => {}
            }
        }

        Ok(scope_clone)
    }
}
