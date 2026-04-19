use alicia::helper::error::Error;
use alicia::stage_1::{buffer::TokenBuffer, helper::Source};
use alicia::stage_2::scope::*;
use alicia::stage_3::analysis::*;
use alicia::stage_4::machine::*;

//================================================================

fn run() -> Result<(), Error> {
    let mut scope = Scope::new(None);
    scope.parse_buffer(TokenBuffer::new(Source::new_file("src/test.alicia")?)?)?;
    Analysis::analyze_tree(&mut scope)?;
    let mut machine = Machine::new(&scope)?;

    let main = machine.function.get("main").unwrap().clone();

    if let FunctionKind::Function(function) = main {
        function.execute(&mut machine)
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        println!("{error}");
    }
}
