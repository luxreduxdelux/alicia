use crate::{
    helper::error::*,
    stage_1::helper::{Identifier, Point},
    stage_2::{construct::*, scope::*},
    stage_4::buffer::*,
};

pub struct Analysis {}

impl Analysis {
    fn format_internal(mut argument: ArgumentBuffer) -> Result<String, Error> {
        let mut string = argument.next().unwrap().as_string()?;

        while argument.peek().is_some() {
            let value = argument.next().unwrap();
            string = string.replacen("{}", &format!("{value}"), 1);
        }

        Ok(string)
    }

    fn print(argument: ArgumentBuffer) -> Result<Option<Value>, Error> {
        println!("{}", Self::format_internal(argument)?);
        Ok(None)
    }

    fn format(argument: ArgumentBuffer) -> Result<Option<Value>, Error> {
        Ok(Some(Value::String(Self::format_internal(argument)?)))
    }

    #[rustfmt::skip]
    pub fn analyze_tree(scope: &mut Scope) -> Result<(), Error> {
        // TO-DO probably not the place to be adding the standard library?
        scope.set_declaration(
            Identifier::from_string("print".to_string(), Point::new(0, 0)).unwrap(),
            Declaration::FunctionNative(FunctionNative::new(Self::print, Vec::default(), None)),
        );

        scope.set_declaration(
            Identifier::from_string("format".to_string(), Point::new(0, 0)).unwrap(),
            Declaration::FunctionNative(FunctionNative::new(Self::format, Vec::default(), None)),
        );

        for value in scope.symbol.clone().values() {
            match value {
                Declaration::Function(function)    => function.analyze(scope)?,
                Declaration::Structure(structure)  => structure.analyze(scope)?,
                Declaration::Definition(definition) => {
                    definition.analyze(scope)?;
                    definition.execute(scope)?;
                },
                //Declaration::Enumerate(enumerate) => enumerate.analyze(&scope)?,
                _ => {}
            }
        }

        Ok(())
    }
}
