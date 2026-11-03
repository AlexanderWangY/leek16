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
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialRegister {
    Sp,
    Pc,
}

impl SpecialRegister {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sp => "SP",
            Self::Pc => "PC",
        }
    }
}

impl RegisterKind {
    pub fn to_u16(&self) -> u16 {
        match self {
            Self::R0 => 0,
            Self::R1 => 1,
            Self::R2 => 2,
            Self::R3 => 3,
            Self::R4 => 4,
            Self::R5 => 5,
            Self::R6 => 6,
            Self::R7 => 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Register(RegisterKind),
    Special(SpecialRegister),

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
