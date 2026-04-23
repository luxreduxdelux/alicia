use crate::{
    helper::error::*,
    stage_2::{construct::ExpressionKind, scope::*},
    stage_4::machine::{Argument, Value},
};

pub struct Analysis {}

impl Analysis {
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

                    if identifier.is_empty() {
                        if let Some(value) = argument.next() {
                            result.push_str(&value.to_string());
                        }
                    } else {
                        let split: Vec<&str> = identifier.split(".").collect();

                        if split.len() == 1 {
                            if let Some(value) = argument.local.get(&identifier) {
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
                                    if let Some(value) = argument.local.get(access) {
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
                }
                _ => result.push(character),
            }
        }

        Ok(result)
    }

    fn print(argument: Argument) {
        println!("{}", Self::format_internal(argument).unwrap());
    }

    #[rustfmt::skip]
    pub fn analyze_tree(scope: &mut Scope) -> Result<(), Error> {
        scope.symbol.insert("print".to_string(), Declaration::FunctionNative(FunctionNative {
            name: "print".to_string(),
            call: Self::print,
            enter: NativeArgument::Variable,
            leave: ExpressionKind::Null,
        }));

        let scope_borrow = scope as *mut Scope;

        for value in scope.symbol.values_mut() {
            let scope = unsafe { &mut (*scope_borrow) };

            // TO-DO this is quite bad. I think in a future design we should make a difference
            // between pre-analyze declaration and post-analyze declaration rather than modify
            // everything in-place.
            match value {
                Declaration::Function(function)     => function.analyze(scope)?,
                Declaration::Structure(structure)   => structure.analyze(scope)?,
                //Declaration::Enumerate(enumerate) => enumerate.analyze(&scope)?,
                Declaration::Definition(definition) => { definition.analyze(scope)?; },
                _ => {}
            }
        }

        Ok(())
    }
}
