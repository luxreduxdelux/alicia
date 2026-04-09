use crate::{
    helper::error::*,
    stage_1::helper::{Identifier, Point},
    stage_2::{construct::*, scope::*},
    stage_4::buffer::*,
};

pub struct Analysis {}

impl Analysis {
    fn print(mut argument: ArgumentBuffer) {
        let string = argument.next().unwrap();

        match string {
            ExpressionValue::String(string) => println!("{}", string),
            _ => {}
        }
    }

    pub fn analyze_tree(scope: &mut Scope) -> Result<(), Error> {
        // TO-DO probably not the place to be adding the standard library?
        scope.set_declaration(
            Identifier::from_string("print".to_string(), Point::new(0, 0)).unwrap(),
            Declaration::FunctionNative(Box::new(Self::print)),
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
