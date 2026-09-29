#[derive(Default)]
pub struct GeneralRegisters {
    pub r0: u16,
    pub r1: u16,
    pub r2: u16,
    pub r3: u16,
    pub r4: u16,
    pub r5: u16,
    pub r6: u16,
    pub r7: u16,
}

pub struct Cpu {
    pub reg: GeneralRegisters,
    pub pc: u16,
    pub sp: u16,
    pub cnd: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            reg: GeneralRegisters::default(),
            pc: 0, // 0x0000
            sp: 0xFFFE,
            cnd: false,
        }
    }
}
