use alicia::prelude::*;

//================================================================

fn run() -> Result<(), Error> {
    let instance = Builder::default().with_file("src/test.alicia".to_string())?;
    let mut instance = instance.build()?;

    if let Some(function) = instance.machine.function.get("main").cloned()
        && let FunctionKind::Function(function) = function
    {
        function.execute(&mut instance.machine, vec![]);
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
    }
}
