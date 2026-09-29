pub struct Ram {
    data: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        self.data[addr as usize] = val;
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        u16::from_be_bytes([self.data[addr as usize], self.data[(addr + 1) as usize]])
    }
}
