use crate::{
    helper::error::*,
    stage_1::helper::{Identifier, Point},
    stage_2::{construct::*, scope::*},
    stage_4::buffer::*,
};

pub struct Analysis {}

impl Analysis {
    fn print(mut argument: ArgumentBuffer) -> Result<Option<Value>, Error> {
        let mut string = String::default();

        while argument.peek().is_some() {
            let value = argument.next().unwrap();
            string.push_str(&format!("{value} "));
        }

        println!("{string}");

        Ok(None)
    }

    fn test_get(_: ArgumentBuffer) -> Result<Option<Value>, Error> {
        println!("call test_get");

        Ok(Some(Value::Integer(1337)))
    }

    fn test_set(mut argument: ArgumentBuffer) -> Result<Option<Value>, Error> {
        println!("call test_set");

        let integer = argument.next().unwrap();

        println!("{integer}");

        Ok(None)
    }

    pub fn analyze_tree(scope: &mut Scope) -> Result<(), Error> {
        // TO-DO probably not the place to be adding the standard library?
        scope.set_declaration(
            Identifier::from_string("print".to_string(), Point::new(0, 0)).unwrap(),
            Declaration::FunctionNative(FunctionNative::new(Self::print, Vec::default(), None)),
        );

        scope.set_declaration(
            Identifier::from_string("test_get".to_string(), Point::new(0, 0)).unwrap(),
            Declaration::FunctionNative(FunctionNative::new(
                Self::test_get,
                Vec::default(),
                Some(ExpressionKind::Integer),
            )),
        );

        scope.set_declaration(
            Identifier::from_string("test_set".to_string(), Point::new(0, 0)).unwrap(),
            Declaration::FunctionNative(FunctionNative::new(
                Self::test_set,
                vec![ExpressionKind::Integer],
                None,
            )),
        );

        for value in scope.symbol.values() {
            match value {
                Declaration::Function(function) => function.analyze(scope)?,
                Declaration::Structure(structure) => structure.analyze(scope)?,
                //Declaration::Enumerate(enumerate) => enumerate.analyze(&scope)?,
                _ => {}
            }
        }

        Ok(())
    }
}
