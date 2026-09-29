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
    MissingToken,
    MissingFunctionEnd,
    UnexpectedToken,
}
