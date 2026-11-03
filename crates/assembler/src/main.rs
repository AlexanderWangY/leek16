use std::{env, fs, io::Error, path::PathBuf};

use crate::encode::Encoder;
use crate::error::{EncodeError, LexError, ParseError, SymbolError};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::symbol::SymbolTable;

mod ast;
mod encode;
mod error;
mod lexer;
mod parser;
mod symbol;
mod token;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let path: String = args[1].clone();
    let out: PathBuf = match args.get(2) {
        Some(out) => PathBuf::from(out),
        None => PathBuf::from(&path).with_extension("bin"),
    };

    let source = fs::read_to_string(&path)?;

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
        Err(ParseError::MissingToken {
            expected,
            line,
            column,
        }) => {
            println!(
                "expected {} but reached end of input after line {} col {}",
                expected, line, column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing tokens",
            ));
        }
        Err(ParseError::MissingFunctionEnd { name, line, column }) => {
            println!(
                "function {} starting at line {} col {} is missing END",
                name, line, column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing function end",
            ));
        }
        Err(ParseError::UnexpectedToken {
            found,
            expected,
            line,
            column,
        }) => {
            println!(
                "unexpected token {:?} found at line {} col {}, expected {}",
                found, line, column, expected
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unexpected tokens",
            ));
        }
        Err(ParseError::UnmatchedFunctionEnd { line, column }) => {
            println!(
                "END without matching FUNC found at line {} col {}",
                line, column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "unmatched function end",
            ));
        }
        Err(ParseError::NestedFunction {
            outer,
            line,
            column,
        }) => {
            println!(
                "nested FUNC found at line {} col {} inside function {}",
                line, column, outer
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "nested function",
            ));
        }
        Err(ParseError::ImmediateCompare {
            lhs,
            rhs,
            line,
            column,
        }) => {
            println!(
                "cannot compare two immediates {} and {} at line {} col {}",
                lhs, rhs, line, column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "immediate compare",
            ));
        }
        Err(ParseError::LoadIntoImmediate {
            value,
            line,
            column,
        }) => {
            println!(
                "cannot load into immediate {} at line {} col {}",
                value, line, column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "load into immediate",
            ));
        }
        Err(ParseError::StoreImmediateToImmediate {
            value,
            addr,
            line,
            column,
        }) => {
            println!(
                "cannot store immediate {} to immediate address {} at line {} col {}",
                value, addr, line, column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "store immediate to immediate",
            ));
        }
        Err(ParseError::WriteToPc { line, column }) => {
            println!(
                "cannot write to PC at line {} col {}, use JMP instead",
                line, column
            );
            return Err(Error::new(std::io::ErrorKind::InvalidInput, "write to PC"));
        }
        Err(ParseError::SpecialRegisterMisuse { reg, line, column }) => {
            println!(
                "{} can only be used as `R := SP`, `SP := R` or `R := PC`, found at line {} col {}",
                reg.name(),
                line,
                column
            );
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "special register misuse",
            ));
        }
    };

    println!("{:?}", ast);

    let symbol_table = match SymbolTable::build(&ast) {
        Ok(s) => s,
        Err(SymbolError::DuplicateFunction { label }) => {
            println!("duplicate function {}", label);
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "duplicate function",
            ));
        }
    };

    println!("{:?}", symbol_table);

    let mut encoder = Encoder::new(ast, symbol_table);
    match encoder.encode(&out) {
        Ok(()) => {}
        Err(EncodeError::UndefinedLabel { label }) => {
            println!("undefined label {}", label);
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "undefined label",
            ));
        }
        Err(EncodeError::FailedCreateParentPath) => {
            println!("failed to create parent directory for {}", out.display());
            return Err(Error::other("failed to create parent directory"));
        }
        Err(EncodeError::FailedCreateFile) => {
            println!("failed to create output file {}", out.display());
            return Err(Error::other("failed to create output file"));
        }
        Err(EncodeError::FailedToWrite) => {
            println!("failed to write to {}", out.display());
            return Err(Error::other("failed to write output"));
        }
    }

    println!("Wrote {}", out.display());

    Ok(())
}
