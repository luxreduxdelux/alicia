use crate::{helper::error::*, stage_2::scope::*};

pub struct Analysis {}

impl Analysis {
    pub fn analyze_tree(scope: Scope) -> Result<(), Error> {
        for value in scope.symbol.values() {
            match value {
                Declaration::Function(function) => function.analyze(&scope)?,
                Declaration::Structure(structure) => structure.analyze(&scope)?,
                //Declaration::Enumerate(enumerate) => enumerate.analyze(&scope)?,
                _ => {}
            }
        }

        Ok(())
    }
}
