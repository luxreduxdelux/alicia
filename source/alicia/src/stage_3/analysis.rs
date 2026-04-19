use crate::{helper::error::*, stage_2::scope::*};

pub struct Analysis {}

impl Analysis {
    fn print() {
        println!("Hello, world, part 2.");
    }

    #[rustfmt::skip]
    pub fn analyze_tree(scope: &mut Scope) -> Result<(), Error> {
        scope.symbol.insert("print".to_string(), Declaration::FunctionNative(FunctionNative {
            name: "print".to_string(),
            call: Self::print
        }));

        for value in scope.symbol.clone().values() {
            match value {
                Declaration::Function(function)     => function.analyze(scope)?,
                Declaration::Structure(structure)   => structure.analyze(scope)?,
                //Declaration::Enumerate(enumerate)   => enumerate.analyze(&scope)?,
                Declaration::Definition(definition) => definition.analyze(scope)?,
                _ => {}
            }
        }

        Ok(())
    }
}
