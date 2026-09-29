use std::{env, fs, io::Error};

use crate::error::{LexError, ParseError};
use crate::lexer::Lexer;
use crate::parser::Parser;

mod ast;
mod error;
mod lexer;
mod parser;
mod token;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let path: String = args[1].clone();

    let source = fs::read_to_string(path)?;

    let mut lexer = Lexer::new(source.as_bytes());

    let tokens = match lexer.lex() {
        Ok(t) => t,
        Err(LexError::UnexpectedCharacter { ch, line, column }) => {
            println!(
                "unexpected character {} found at line {} col {}",
                ch as char, line, column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "unexpected character",
            ));
        }
        Err(LexError::InvalidNumber { text, line, column }) => {
            println!(
                "invalid number {} found at line {} col {}",
                text, line, column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid number",
            ));
        }
    };

    println!("{:?}", tokens);
    println!("Parsing tokens now...");

    let mut parser = Parser::new(&tokens);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(ParseError::MissingToken) => {
            println!("Missing token, abort parse!");
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing tokens",
            ));
        }
        Err(ParseError::MissingFunctionEnd) => {
            println!("Missing function end, abort parse!");
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing function end",
            ));
        }
        Err(ParseError::UnexpectedToken) => {
            println!("Unexpected token, abort parse!");
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unexpected tokens",
            ));
        }
    };

    println!("{:?}", ast);

    Ok(())
}
