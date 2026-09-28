use crate::ram::Ram;

pub struct Cpu {
    registers: [u16; 8],
    pc: u16,
    ic: u16,
    ram: Ram,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: [0; 8],
            pc: 0,
            ic: 0,
            ram: Ram::new(),
        }
    }
}
