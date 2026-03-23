mod machine;
mod parse;
mod split;
mod utility;

//================================================================

use crate::machine::instance::*;

//================================================================

fn main() -> Result<(), crate::utility::error::Error> {
    let mut instance = Instance::new();
    instance.load_file("test.alicia")?;

    if let Some(main) = instance.get_value("main") {
        instance.call_function(&main.as_function()?, Vec::default())?;
    }

    Ok(())
}
