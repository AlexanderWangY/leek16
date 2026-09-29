use crate::token::RegisterKind;

pub struct Label(pub String);

pub enum Address {
    Register(RegisterKind), // [R1]
    Immediate(u16),         // [0x1000]
}

pub enum Operand {
    Register(RegisterKind),
    Immediate(u16),
}

pub enum BinaryOp {
    Add,
    Sub,
    And,
    Or,
    ShiftL,
    ShiftR,
}

pub enum UnaryOp {
    Not,
}

pub enum CompareOp {
    Greater,
    Less,
    GreaterEq,
    LessEq,
    Equal,
    NotEqual,
}

pub struct Function {
    pub label: Label,
    pub instructions: Vec<Instruction>,
}

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

pub enum Item {
    Function(Function),
    Instruction(Instruction),
}

pub struct Ast {
    pub items: Vec<Item>,
}

impl Ast {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}
