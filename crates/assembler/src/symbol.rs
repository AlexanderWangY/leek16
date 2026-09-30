use std::collections::HashMap;

use crate::{
    ast::{Address, Ast, BinaryOp, Instruction, Item, Label, Operand},
    error::SymbolError,
};

#[derive(Debug)]
pub struct SymbolTable {
    data: HashMap<Label, u16>,
}

impl SymbolTable {
    pub fn build(ast: &Ast) -> Result<Self, SymbolError> {
        let mut map: HashMap<Label, u16> = HashMap::new();
        let mut pc = 0;

        for item in ast.items.iter() {
            match item {
                Item::Function(func) => {
                    if map.insert(func.label.clone(), pc).is_some() {
                        return Err(SymbolError::DuplicateFunction {
                            label: func.label.0.clone(),
                        });
                    }

                    pc = Self::increment_instructions(&func.instructions, pc);
                }
                Item::Instruction(instr) => {
                    pc = Self::increment_instruction(instr, pc);
                }
            }
        }

        Ok(Self { data: map })
    }

    pub fn get(&self, label: &Label) -> Option<u16> {
        self.data.get(label).copied()
    }

    pub fn increment_instruction(instruction: &Instruction, pc: u16) -> u16 {
        let size = match instruction {
            Instruction::Move {
                src: Operand::Immediate(_),
                ..
            } => 4,
            Instruction::Binary {
                op: BinaryOp::Sub | BinaryOp::ShiftL | BinaryOp::ShiftR,
                lhs: Operand::Immediate(_),
                rhs: Operand::Register(_),
                ..
            } => 6,
            Instruction::Binary {
                lhs: Operand::Immediate(_),
                ..
            }
            | Instruction::Binary {
                rhs: Operand::Immediate(_),
                ..
            } => 4,
            Instruction::Load {
                addr: Address::Immediate(_),
                ..
            } => 4,
            Instruction::Store {
                addr: Address::Immediate(_),
                ..
            }
            | Instruction::Store {
                src: Operand::Immediate(_),
                ..
            } => 4,
            Instruction::Compare {
                lhs: Operand::Immediate(_),
                ..
            }
            | Instruction::Compare {
                rhs: Operand::Immediate(_),
                ..
            } => 4,
            Instruction::Jump { .. }
            | Instruction::BranchTrue { .. }
            | Instruction::BranchFalse { .. } => 4,
            Instruction::Move { .. }
            | Instruction::Binary { .. }
            | Instruction::Unary { .. }
            | Instruction::Load { .. }
            | Instruction::Store { .. }
            | Instruction::Compare { .. } => 2,
        };

        pc + size
    }

    pub fn increment_instructions(instructions: &[Instruction], pc: u16) -> u16 {
        instructions.iter().fold(pc, |pc, instruction| {
            Self::increment_instruction(instruction, pc)
        })
    }
}
