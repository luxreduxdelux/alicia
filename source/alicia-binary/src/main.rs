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
        function.execute(
            &mut machine,
            vec![
                Value::String("foo".to_string()),
                Value::Integer(1),
                Value::Decimal(1.5),
                Value::Boolean(true),
            ],
        )
    }

    Ok(())
}

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(error) = run() {
        println!("{error}");
    }
}
