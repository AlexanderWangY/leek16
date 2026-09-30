use crate::token::TokenKind;

#[derive(Debug)]
pub enum LexError {
    UnexpectedCharacter {
        ch: u8,
        line: usize,
        column: usize,
    },
    InvalidNumber {
        text: String,
        line: usize,
        column: usize,
    },
}

#[derive(Debug)]
pub enum ParseError {
    MissingToken {
        expected: &'static str,
        line: usize,
        column: usize,
    },
    MissingFunctionEnd {
        name: String,
        line: usize,
        column: usize,
    },
    UnexpectedToken {
        found: TokenKind,
        expected: &'static str,
        line: usize,
        column: usize,
    },
    UnmatchedFunctionEnd {
        line: usize,
        column: usize,
    },
    NestedFunction {
        outer: String,
        line: usize,
        column: usize,
    },
    ImmediateCompare {
        lhs: u16,
        rhs: u16,
        line: usize,
        column: usize,
    },
    LoadIntoImmediate {
        value: u16,
        line: usize,
        column: usize,
    },
    StoreImmediateToImmediate {
        value: u16,
        addr: u16,
        line: usize,
        column: usize,
    },
}
