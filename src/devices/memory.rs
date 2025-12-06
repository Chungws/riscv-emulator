pub const DRAM_BASE: u32 = 0x80000000;
pub const DRAM_SIZE: usize = 0x8000000;
pub struct Memory {
    dram: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            dram: vec![0; DRAM_SIZE],
        }
    }

    pub fn read8(&self, addr: u32) -> u8 {
        let index = (addr - DRAM_BASE) as usize;
        self.dram[index]
    }

    pub fn read16(&self, addr: u32) -> u16 {
        let index = (addr - DRAM_BASE) as usize;
        u16::from_le_bytes([self.dram[index], self.dram[index + 1]])
    }

    pub fn read32(&self, addr: u32) -> u32 {
        let index = (addr - DRAM_BASE) as usize;
        let slice = &self.dram[index..(index + 4)];
        u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
    }

    pub fn write8(&mut self, addr: u32, value: u8) {
        let index = (addr - DRAM_BASE) as usize;
        self.dram[index] = value;
    }

    pub fn write16(&mut self, addr: u32, value: u16) {
        let index = (addr - DRAM_BASE) as usize;
        let bytes = value.to_le_bytes();
        for i in 0..2 {
            self.dram[index + i] = bytes[i]
        }
    }

    pub fn write32(&mut self, addr: u32, value: u32) {
        let index = (addr - DRAM_BASE) as usize;
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            self.dram[index + i] = bytes[i]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_read_write_byte() {
        let mut mem = Memory::new();
        mem.write8(0x80000000, 0xAB);
        assert_eq!(mem.read8(0x80000000), 0xAB);
    }

    #[test]
    fn test_memory_read_write_half() {
        let mut mem = Memory::new();
        mem.write16(0x80000000, 0xBEEF);
        assert_eq!(mem.read16(0x80000000), 0xBEEF);
    }

    #[test]
    fn test_memory_read_write_word() {
        let mut mem = Memory::new();
        mem.write32(0x80000000, 0xDEADBEEF);
        assert_eq!(mem.read32(0x80000000), 0xDEADBEEF);
    }

    #[test]
    fn test_memory_half_little_endian() {
        let mut mem = Memory::new();
        mem.write16(0x80000000, 0xABCD);
        assert_eq!(mem.read8(0x80000000), 0xCD); // LSB
        assert_eq!(mem.read8(0x80000001), 0xAB); // MSB
    }

    #[test]
    fn test_memory_little_endian() {
        let mut mem = Memory::new();
        mem.write32(0x80000000, 0x12345678);
        assert_eq!(mem.read8(0x80000000), 0x78); // LSB first
        assert_eq!(mem.read8(0x80000001), 0x56);
        assert_eq!(mem.read8(0x80000002), 0x34);
        assert_eq!(mem.read8(0x80000003), 0x12); // MSB last
    }
}
