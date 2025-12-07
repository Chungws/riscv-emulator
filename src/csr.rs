use std::collections::HashMap;

pub const MSTATUS: u16 = 0x300;
pub const MTVEC: u16 = 0x305;
pub const MEPC: u16 = 0x341;
pub const MCAUSE: u16 = 0x342;
pub const MTVAL: u16 = 0x343;
pub const MHARTID: u16 = 0xF14;

pub const SSTATUS: u16 = 0x100;
pub const STVEC: u16 = 0x105;
pub const SEPC: u16 = 0x141;
pub const SCAUSE: u16 = 0x142;
pub const STVAL: u16 = 0x143;

pub const MSTATUS_MIE: u64 = 0x8;
pub const MSTATUS_MPIE: u64 = 0x80;
pub const MSTATUS_MPP: u64 = 0x1800;
pub const MSTATUS_SIE: u64 = 0x2;
pub const MSTATUS_SPIE: u64 = 0x20;
pub const MSTATUS_SPP: u64 = 0x100;

pub struct Csr {
    data: HashMap<u16, u64>,
}

impl Csr {
    pub fn new() -> Self {
        Csr {
            data: HashMap::new(),
        }
    }

    pub fn read(&self, addr: u16) -> u64 {
        if self.data.contains_key(&addr) {
            self.data[&addr]
        } else {
            0
        }
    }

    pub fn write(&mut self, addr: u16, value: u64) {
        self.data.insert(addr, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csr_new() {
        let csr = Csr::new();
        assert_eq!(csr.read(0x300), 0);
    }

    #[test]
    fn test_csr_read_uninitialized() {
        let csr = Csr::new();
        assert_eq!(csr.read(0x000), 0);
        assert_eq!(csr.read(0xFFF), 0);
        assert_eq!(csr.read(0x300), 0);
    }

    #[test]
    fn test_csr_write_read() {
        let mut csr = Csr::new();
        csr.write(0x300, 0x1234);
        assert_eq!(csr.read(0x300), 0x1234);
    }

    #[test]
    fn test_csr_overwrite() {
        let mut csr = Csr::new();
        csr.write(0x300, 0x1111);
        csr.write(0x300, 0x2222);
        assert_eq!(csr.read(0x300), 0x2222);
    }

    #[test]
    fn test_csr_multiple_registers() {
        let mut csr = Csr::new();
        csr.write(0x300, 0xAAAA);
        csr.write(0x341, 0xBBBB);
        csr.write(0x342, 0xCCCC);

        assert_eq!(csr.read(0x300), 0xAAAA);
        assert_eq!(csr.read(0x341), 0xBBBB);
        assert_eq!(csr.read(0x342), 0xCCCC);
    }

    #[test]
    fn test_csr_64bit_value() {
        let mut csr = Csr::new();
        let large_value: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        csr.write(0x300, large_value);
        assert_eq!(csr.read(0x300), large_value);
    }
}
