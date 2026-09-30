use crate::token::RegisterKind;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub String);

#[derive(Debug, Clone)]
pub enum Address {
    Register(RegisterKind), // [R1]
    Immediate(u16),         // [0x1000]
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(RegisterKind),
    Immediate(u16),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    And,
    Or,
    ShiftL,
    ShiftR,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, Clone)]
pub enum CompareOp {
    Greater,
    Less,
    GreaterEq,
    LessEq,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub label: Label,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Move {
        dst: RegisterKind,
        src: Operand,
    },

    Binary {
        dst: RegisterKind,
        op: BinaryOp,
        lhs: Operand,
        rhs: Operand,
    },

    Unary {
        dst: RegisterKind,
        op: UnaryOp,
        src: RegisterKind,
    },

    Load {
        dst: RegisterKind,
        addr: Address,
    },

    Store {
        addr: Address,
        src: Operand,
    },

    Compare {
        op: CompareOp,
        lhs: Operand,
        rhs: Operand,
    },

    Jump {
        target: Label,
    },

    BranchTrue {
        target: Label,
    },

    BranchFalse {
        target: Label,
    },
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Instruction(Instruction),
}

#[derive(Debug, Clone)]
pub struct Ast {
    pub items: Vec<Item>,
}

impl Ast {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}
