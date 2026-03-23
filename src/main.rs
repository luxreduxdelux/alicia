mod helper;
mod machine;
mod parse;
mod split;

//================================================================

use crate::machine::instance::*;

//================================================================

fn run() -> Result<(), crate::helper::error::Error> {
    let mut instance = Instance::new();
    instance.load_file("src/test.alicia")?;

    if let Some(main) = instance.get_value("main") {
        instance.call_function(&main.as_function()?, Vec::default())?;
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
    }
}
