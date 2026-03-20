use super::instruction::*;
use super::token::*;

pub struct Source {}

impl Source {
    pub fn parse(path: &str) {
        let file = std::fs::read_to_string("test.alicia").unwrap();
        let mut list: Vec<Token> = Vec::new();

        for line in file.lines() {
            Token::parse_line(line, &mut list);
        }

        println!("token list: {list:?}");

        let mut iterator = list.iter();
        let mut list: Vec<Instruction> = Vec::new();

        while let Some(token) = iterator.next() {
            Instruction::parse_token(token, &mut iterator, &mut list);
        }

        println!("inst. list: {list:?}");
    }
}
