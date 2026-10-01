use std::{
    fs::{File, create_dir_all},
    io::{BufWriter, Write},
    path::Path,
};

use crate::{
    ast::{
        Address, Ast, BinaryOp, CompareOp, Function, Instruction, Item, Label, Operand, UnaryOp,
    },
    error::EncodeError,
    symbol::SymbolTable,
    token::RegisterKind,
};

pub struct Encoder {
    ast: Ast,
    symbols: SymbolTable,
}

impl Encoder {
    pub fn new(ast: Ast, symbols: SymbolTable) -> Self {
        Self { ast, symbols }
    }

    pub fn encode(&mut self, out: impl AsRef<Path>) -> Result<(), EncodeError> {
        let path = out.as_ref();

        if let Some(parent) = path.parent() {
            create_dir_all(parent).map_err(|_| EncodeError::FailedCreateParentPath)?;
        }

        let file = File::create(path).map_err(|_| EncodeError::FailedCreateFile)?;

        // Go through the AST and start writing

        let mut writer = BufWriter::new(file);

        for item in self.ast.items.iter() {
            match item {
                Item::Function(func) => self.encode_function(func, &mut writer)?,
                Item::Instruction(_) => todo!(),
            }
        }

        Ok(())
    }

    fn encode_function<W: Write>(
        &self,
        func: &Function,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        for instruction in func.instructions.iter() {
            self.encode_instruction(instruction, writer)?;
        }
        Ok(())
    }

    fn encode_instruction<W: Write>(
        &self,
        instruction: &Instruction,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match instruction {
            Instruction::Move { dst, src } => self.encode_move(dst, src, writer)?,
            Instruction::Binary { dst, op, lhs, rhs } => {
                self.encode_binary(dst, op, lhs, rhs, writer)?
            }
            Instruction::Unary { dst, op, src } => self.encode_unary(dst, op, src, writer)?,
            Instruction::Load { dst, addr } => self.encode_load(dst, addr, writer)?,
            Instruction::Store { addr, src } => self.encode_store(addr, src, writer)?,
            Instruction::Compare { op, lhs, rhs } => self.encode_compare(op, rhs, lhs, writer)?,
            Instruction::Jump { target } => self.encode_jump(target, writer)?,
            Instruction::BranchTrue { target } => self.encode_branch_if(target, true, writer)?,
            Instruction::BranchFalse { target } => self.encode_branch_if(target, false, writer)?,
        }
        Ok(())
    }

    fn encode_move<W: Write>(
        &self,
        dst: &RegisterKind,
        src: &Operand,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(())
    }

    fn encode_binary<W: Write>(
        &self,
        dst: &RegisterKind,
        op: &BinaryOp,
        lhs: &Operand,
        rhs: &Operand,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(())
    }

    fn encode_unary<W: Write>(
        &self,
        dst: &RegisterKind,
        op: &UnaryOp,
        src: &RegisterKind,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(())
    }

    fn encode_load<W: Write>(
        &self,
        dst: &RegisterKind,
        addr: &Address,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(())
    }

    fn encode_store<W: Write>(
        &self,
        addr: &Address,
        src: &Operand,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(())
    }

    fn encode_compare<W: Write>(
        &self,
        op: &CompareOp,
        rhs: &Operand,
        lhs: &Operand,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(())
    }

    fn encode_jump<W: Write>(&self, target: &Label, writer: &mut W) -> Result<(), EncodeError> {
        Ok(())
    }

    fn encode_branch_if<W: Write>(
        &self,
        target: &Label,
        cond: bool,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(())
    }
}
