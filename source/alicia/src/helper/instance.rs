use crate::helper::error::Error;
use crate::stage_1::buffer::TokenBuffer;
use crate::stage_2::scope::*;
use crate::stage_3::analysis::Analysis;
use crate::{stage_1::helper::Source, stage_4::machine::Machine};

//================================================================

pub struct Instance {
    pub machine: Machine,
}

#[derive(Default)]
pub struct Builder {
    source: Option<Source>,
    function: Vec<FunctionNative>,
}

impl Builder {
    pub fn with_file(mut self, path: String) -> Result<Self, Error> {
        self.source = Some(Source::new_file(&path)?);
        Ok(self)
    }

    pub fn add_function(mut self, function: FunctionNative) -> Result<Self, Error> {
        self.function.push(function);
        Ok(self)
    }

    pub fn build(self) -> Result<Instance, Error> {
        let source = self.source.unwrap();

        //================================================================

        let mut scope = Scope::new(None);
        scope.parse_buffer(TokenBuffer::new(source)?)?;

        for function in self.function {
            scope.symbol.insert(
                function.name.to_string(),
                Declaration::FunctionNative(function),
            );
        }

        Analysis::analyze_tree(&mut scope)?;

        //================================================================

        Ok(Instance {
            machine: Machine::new(&scope)?,
        })
    }
}
