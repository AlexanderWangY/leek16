#[derive(Debug, Clone, PartialEq)]
pub enum RegisterKind {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    Sp,
    Pc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Register(RegisterKind),

    Number(u16),

    Identifier(String),

    Assign,
    ArrowLeft,
    ArrowRight,

    Plus,
    Minus,
    And,
    Or,
    Not,

    ShiftLeft,
    ShiftRight,

    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEq,
    LessEq,

    LBracket,
    RBracket,
    Colon,
    Func,
    FuncEnd,

    Jmp,
    Brt,
    Brf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}
