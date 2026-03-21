mod parser;
mod runtime;

//================================================================

use crate::parser::source::*;

//================================================================

fn main() {
    if let Err(error) = Source::parse("test.alicia") {
        println!("{error}");
    }
}
