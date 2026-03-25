mod helper;
mod stage_1; // Lexer stage.
mod stage_2; // Parser stage.
mod stage_3; // Analyzer stage.
mod stage_4; // Run-time stage.

//================================================================

use helper::error::*;
use stage_1::{buffer::TokenBuffer, helper::Source};
use stage_2::scope::*;

//================================================================

fn run() -> Result<(), Error> {
    Scope::new(TokenBuffer::new(Source::new_file("src/test.alicia")?))?;

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
    }
}
