mod compiler;
mod error;
mod lexer;
mod machine;
mod parser;

//================================================================

use crate::lexer::*;
use crate::parser::*;

//================================================================

fn main() {
    if let Err(e) = run() {
        println!("{e}");
    }
}

fn run() -> anyhow::Result<()> {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let file = std::fs::read_to_string("test.alicia")?;
    let token = TokenBuffer::from_text("test.alicia", &file)?;

    //println!("{token:#?}");

    let _ = DefinitionBuffer::from_token(token)?;

    Ok(())
}
