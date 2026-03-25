use parse::construct::*;
use split::{buffer::TokenBuffer, helper::Source, token::*};

mod helper;
mod parse;
mod split;

//================================================================

//================================================================

#[derive(Debug)]
enum Declaration {
    Function(Function),
    Structure(Structure),
    Enumerate(Enumerate),
}

fn run() -> Result<(), crate::helper::error::Error> {
    let mut buffer = TokenBuffer::new(Source::new_file("src/test.alicia")?);
    let mut vector = Vec::new();

    while let Some(token) = buffer.next() {
        match token.class {
            TokenClass::Function => {
                vector.push(Declaration::Function(Function::parse_token(&mut buffer)?));
            }
            TokenClass::Structure => {
                vector.push(Declaration::Structure(Structure::parse_token(&mut buffer)?));
            }
            TokenClass::Enumerate => {
                vector.push(Declaration::Enumerate(Enumerate::parse_token(&mut buffer)?));
            }
            TokenClass::Use => {
                let value = Use::parse_token(&mut buffer)?;
                println!("use: {value:?}");
            }
            _ => {
                //return Err(Error::new());
            }
        };
    }

    println!("{vector:#?}");

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
    }
}
