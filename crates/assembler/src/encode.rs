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

const MOV_OPCODE: u16 = 0;
const LOADI_OPCODE: u16 = 1;
const LOADR_OPCODE: u16 = 2;
const LOADA_OPCODE: u16 = 3;
const STORER_OPCODE: u16 = 4;
const STOREI_OPCODE: u16 = 5;
const STOREA_OPCODE: u16 = 6;
const ADD_OPCODE: u16 = 7;
const ADDI_OPCODE: u16 = 8;
const SUB_OPCODE: u16 = 9;
const SUBI_OPCODE: u16 = 10;
const AND_OPCODE: u16 = 11;
const ANDI_OPCODE: u16 = 12;
const OR_OPCODE: u16 = 13;
const ORI_OPCODE: u16 = 14;
const NOT_OPCODE: u16 = 15;
const SHL_OPCODE: u16 = 16;
const SHLI_OPCODE: u16 = 17;
const SHR_OPCODE: u16 = 18;
const SHRI_OPCODE: u16 = 19;
const CMPEQ_OPCODE: u16 = 20;
const CMPNE_OPCODE: u16 = 21;
const CMPGT_OPCODE: u16 = 22;
const CMPLT_OPCODE: u16 = 23;
const CMPGE_OPCODE: u16 = 24;
const CMPLE_OPCODE: u16 = 25;
const CMPEQI_OPCODE: u16 = 26;
const CMPNEI_OPCODE: u16 = 27;
const CMPGTI_OPCODE: u16 = 28;
const CMPLTI_OPCODE: u16 = 29;
const CMPGEI_OPCODE: u16 = 30;
const CMPLEI_OPCODE: u16 = 31;
const RDSP_OPCODE: u16 = 32;
const WRSP_OPCODE: u16 = 33;
const RDPC_OPCODE: u16 = 34;
const JMP_OPCODE: u16 = 35;
const BRT_OPCODE: u16 = 36;
const BRF_OPCODE: u16 = 37;

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
                Item::Instruction(instruction) => {
                    self.encode_instruction(instruction, &mut writer)?
                }
            }
        }

        writer.flush().map_err(|_| EncodeError::FailedToWrite)?;

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
            Instruction::Compare { op, lhs, rhs } => self.encode_compare(op, lhs, rhs, writer)?,
            Instruction::Jump { target } => self.encode_jump(target, writer)?,
            Instruction::BranchTrue { target } => self.encode_branch_if(target, true, writer)?,
            Instruction::BranchFalse { target } => self.encode_branch_if(target, false, writer)?,
            Instruction::ReadSp { dst } => {
                self.encode_special(RDSP_OPCODE, dst.to_u16() << 6, writer)?
            }
            Instruction::WriteSp { src } => {
                self.encode_special(WRSP_OPCODE, src.to_u16() << 3, writer)?
            }
            Instruction::ReadPc { dst } => {
                self.encode_special(RDPC_OPCODE, dst.to_u16() << 6, writer)?
            }
        }
        Ok(())
    }

    fn encode_move<W: Write>(
        &self,
        dst: &RegisterKind,
        src: &Operand,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        // Write the opcode
        // write the dst
        // decide based on src whether to just append the src reg or add 16 bit instruction for immediate
        let opcode: u16 = match src {
            Operand::Register(_) => MOV_OPCODE,
            Operand::Immediate(_) => LOADI_OPCODE,
        };

        let dest_bits = dst.to_u16() << 6;

        let src_bits = match src {
            Operand::Register(reg) => reg.to_u16() << 3,
            Operand::Immediate(_) => 0,
        };

        // First instruction
        let word = (opcode << 9) | dest_bits | src_bits;

        // Write to file
        writer
            .write_all(&word.to_be_bytes())
            .map_err(|_| EncodeError::FailedToWrite)?;

        // Check for immediate
        if let Operand::Immediate(imm) = src {
            writer
                .write_all(&imm.to_be_bytes())
                .map_err(|_| EncodeError::FailedToWrite)?;
        }

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
        let (reg_opcode, imm_opcode) = match op {
            BinaryOp::Add => (ADD_OPCODE, ADDI_OPCODE),
            BinaryOp::Sub => (SUB_OPCODE, SUBI_OPCODE),
            BinaryOp::And => (AND_OPCODE, ANDI_OPCODE),
            BinaryOp::Or => (OR_OPCODE, ORI_OPCODE),
            BinaryOp::ShiftL => (SHL_OPCODE, SHLI_OPCODE),
            BinaryOp::ShiftR => (SHR_OPCODE, SHRI_OPCODE),
        };

        match (lhs, rhs) {
            (Operand::Register(lhs), Operand::Register(rhs)) => {
                let word = (reg_opcode << 9) | dst.to_u16() << 6 | lhs.to_u16() << 3 | rhs.to_u16();
                writer
                    .write_all(&word.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)?;
            }
            (Operand::Register(lhs), Operand::Immediate(rhs)) => {
                let word = (imm_opcode << 9) | dst.to_u16() << 6 | lhs.to_u16() << 3; // ord is implicitly 0 here
                let encoded = ((word as u32) << 16) | (*rhs as u32);
                writer
                    .write_all(&encoded.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)?;
            }
            (Operand::Immediate(lhs), Operand::Register(rhs)) => {
                let word = (imm_opcode << 9) | dst.to_u16() << 6 | rhs.to_u16() << 3 | 1;
                let encoded = ((word as u32) << 16) | (*lhs as u32);
                writer
                    .write_all(&encoded.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)?;
            }
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                let result = match op {
                    BinaryOp::Add => lhs.wrapping_add(*rhs),
                    BinaryOp::Sub => lhs.wrapping_sub(*rhs),
                    BinaryOp::And => lhs & *rhs,
                    BinaryOp::Or => lhs | *rhs,
                    BinaryOp::ShiftL => lhs.checked_shl(u32::from(*rhs)).unwrap_or(0),
                    BinaryOp::ShiftR => lhs.checked_shr(u32::from(*rhs)).unwrap_or(0),
                };

                let word = (LOADI_OPCODE << 9) | dst.to_u16() << 6;
                let encoded = ((word as u32) << 16) | (result as u32);
                writer
                    .write_all(&encoded.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)?;
            }
        }

        Ok(())
    }

    fn encode_unary<W: Write>(
        &self,
        dst: &RegisterKind,
        op: &UnaryOp,
        src: &RegisterKind,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        let opcode = match op {
            UnaryOp::Not => NOT_OPCODE,
        };
        let word = (opcode << 9) | (dst.to_u16() << 6) | (src.to_u16() << 3);

        writer
            .write_all(&word.to_be_bytes())
            .map_err(|_| EncodeError::FailedToWrite)?;

        Ok(())
    }

    fn encode_load<W: Write>(
        &self,
        dst: &RegisterKind,
        addr: &Address,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match addr {
            Address::Register(reg) => {
                let word = (LOADR_OPCODE << 9) | dst.to_u16() << 6 | reg.to_u16() << 3;
                writer
                    .write_all(&word.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)
            }
            Address::Immediate(imm) => {
                let word = (LOADA_OPCODE << 9) | dst.to_u16() << 6;
                let encoded = ((word as u32) << 16) | *imm as u32;
                writer
                    .write_all(&encoded.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)
            }
        }
    }

    fn encode_store<W: Write>(
        &self,
        addr: &Address,
        src: &Operand,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match (addr, src) {
            (Address::Register(addr), Operand::Register(src)) => {
                let word = (STORER_OPCODE << 9) | (addr.to_u16() << 3) | src.to_u16();
                writer
                    .write_all(&word.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)
            }
            (Address::Immediate(addr), Operand::Register(src)) => {
                let word = (STOREA_OPCODE << 9) | src.to_u16() << 3;
                let encoded = ((word as u32) << 16) | (*addr as u32);
                writer
                    .write_all(&encoded.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)
            }
            (Address::Register(addr), Operand::Immediate(src)) => {
                let word = (STOREI_OPCODE << 9) | (addr.to_u16() << 3);
                let encoded = ((word as u32) << 16) | (*src as u32);
                writer
                    .write_all(&encoded.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)
            }
            (Address::Immediate(_), Operand::Immediate(_)) => {
                unreachable!("parser rejects immediate-to-immediate stores")
            }
        }
    }

    fn encode_compare<W: Write>(
        &self,
        op: &CompareOp,
        lhs: &Operand,
        rhs: &Operand,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        let (reg_opcode, imm_opcode) = match op {
            CompareOp::Equal => (CMPEQ_OPCODE, CMPEQI_OPCODE),
            CompareOp::NotEqual => (CMPNE_OPCODE, CMPNEI_OPCODE),
            CompareOp::Greater => (CMPGT_OPCODE, CMPGTI_OPCODE),
            CompareOp::Less => (CMPLT_OPCODE, CMPLTI_OPCODE),
            CompareOp::GreaterEq => (CMPGE_OPCODE, CMPGEI_OPCODE),
            CompareOp::LessEq => (CMPLE_OPCODE, CMPLEI_OPCODE),
        };

        match (lhs, rhs) {
            (Operand::Register(lhs), Operand::Register(rhs)) => {
                let word = (reg_opcode << 9) | lhs.to_u16() << 3 | rhs.to_u16();
                writer
                    .write_all(&word.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)
            }
            (Operand::Register(lhs), Operand::Immediate(rhs)) => {
                let word = (imm_opcode << 9) | lhs.to_u16() << 3; // ord is implicitly 0 here
                let encoded = ((word as u32) << 16) | (*rhs as u32);
                writer
                    .write_all(&encoded.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)
            }
            (Operand::Immediate(lhs), Operand::Register(rhs)) => {
                let word = (imm_opcode << 9) | rhs.to_u16() << 3 | 1;
                let encoded = ((word as u32) << 16) | (*lhs as u32);
                writer
                    .write_all(&encoded.to_be_bytes())
                    .map_err(|_| EncodeError::FailedToWrite)
            }
            (Operand::Immediate(_), Operand::Immediate(_)) => {
                unreachable!("parser rejects immediate-to-immediate compares")
            }
        }
    }

    fn encode_special<W: Write>(
        &self,
        opcode: u16,
        reg_bits: u16,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        let word = (opcode << 9) | reg_bits;
        writer
            .write_all(&word.to_be_bytes())
            .map_err(|_| EncodeError::FailedToWrite)
    }

    fn encode_jump<W: Write>(&self, target: &Label, writer: &mut W) -> Result<(), EncodeError> {
        self.encode_control_flow(JMP_OPCODE, target, writer)
    }

    fn encode_branch_if<W: Write>(
        &self,
        target: &Label,
        cond: bool,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        let opcode = if cond { BRT_OPCODE } else { BRF_OPCODE };
        self.encode_control_flow(opcode, target, writer)
    }

    fn encode_control_flow<W: Write>(
        &self,
        opcode: u16,
        target: &Label,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        let addr = self
            .symbols
            .get(target)
            .ok_or_else(|| EncodeError::UndefinedLabel {
                label: target.0.clone(),
            })?;

        let word = opcode << 9;
        let encoded = ((word as u32) << 16) | (addr as u32);
        writer
            .write_all(&encoded.to_be_bytes())
            .map_err(|_| EncodeError::FailedToWrite)
    }
}
