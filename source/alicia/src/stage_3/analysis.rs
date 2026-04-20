use crate::{
    helper::error::*,
    stage_2::{construct::ExpressionKind, scope::*},
    stage_4::machine::Argument,
};

pub struct Analysis {}

impl Analysis {
    fn format_internal(mut argument: Argument) -> Result<String, Error> {
        let mut string = argument.next().unwrap().as_string();

        while !argument.is_empty() {
            let value = argument.next().unwrap();
            string = string.replacen("{}", &format!("{value}"), 1);
        }

        Ok(string)
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

        for value in scope.symbol.clone().values() {
            match value {
                Declaration::Function(function)     => function.analyze(scope)?,
                Declaration::Structure(structure)   => structure.analyze(scope)?,
                //Declaration::Enumerate(enumerate)   => enumerate.analyze(&scope)?,
                Declaration::Definition(definition) => { definition.analyze(scope)?; },
                _ => {}
            }
        }

        Ok(())
    }
}
