mod parser;
mod runtime;

//================================================================

use crate::parser::error::*;
use crate::runtime::instance::*;

//================================================================

fn main() -> Result<(), AliciaError> {
    let mut instance = Instance::new();
    instance.load_file("test.alicia")?;
    instance.execute_function(
        &instance.get_value("main").unwrap().as_function()?,
        Vec::default(),
    )?;

    Ok(())
}
