pub const CLINT_BASE: u64 = 0x200_0000;
pub const CLINT_SIZE: u64 = 0x10000;

const MSIP_OFFSET: u64 = 0x0000;
const MTIMECMP_OFFSET: u64 = 0x4000;
const MTIME_OFFSET: u64 = 0xBFF8;

pub struct Clint {
    mtime: u64,
    mtimecmp: u64,
    msip: bool,
}

impl Clint {
    pub fn new() -> Self {
        Clint {
            mtime: 0,
            mtimecmp: 0,
            msip: false,
        }
    }

    pub fn read32(&self, offset: u64) -> u32 {
        match offset {
            MSIP_OFFSET => self.msip as u32,
            _ => panic!("Not Implemented"),
        }
    }

    pub fn read64(&self, offset: u64) -> u64 {
        match offset {
            MTIMECMP_OFFSET => self.mtimecmp,
            MTIME_OFFSET => self.mtime,
            _ => panic!("Not Implemented"),
        }
    }

    pub fn write32(&mut self, offset: u64, value: u32) {
        match offset {
            MSIP_OFFSET => {
                self.msip = value > 0;
            }
            _ => panic!("Not Implemented"),
        }
    }

    pub fn write64(&mut self, offset: u64, value: u64) {
        match offset {
            MTIMECMP_OFFSET => {
                self.mtimecmp = value;
            }
            MTIME_OFFSET => {
                self.mtime = value;
            }
            _ => panic!("Not Implemented"),
        }
    }

    pub fn tick(&mut self) {
        self.mtime += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clint_new() {
        let clint = Clint::new();
        assert_eq!(clint.read64(MTIME_OFFSET), 0);
        assert_eq!(clint.read64(MTIMECMP_OFFSET), 0);
        assert_eq!(clint.read32(MSIP_OFFSET), 0);
    }

    #[test]
    fn test_clint_mtime_read_write() {
        let mut clint = Clint::new();
        clint.write64(MTIME_OFFSET, 12345);
        assert_eq!(clint.read64(MTIME_OFFSET), 12345);
    }

    #[test]
    fn test_clint_mtimecmp_read_write() {
        let mut clint = Clint::new();
        clint.write64(MTIMECMP_OFFSET, 99999);
        assert_eq!(clint.read64(MTIMECMP_OFFSET), 99999);
    }

    #[test]
    fn test_clint_msip_read_write() {
        let mut clint = Clint::new();
        assert_eq!(clint.read32(MSIP_OFFSET), 0);

        clint.write32(MSIP_OFFSET, 1);
        assert_eq!(clint.read32(MSIP_OFFSET), 1);

        clint.write32(MSIP_OFFSET, 0);
        assert_eq!(clint.read32(MSIP_OFFSET), 0);
    }

    #[test]
    fn test_clint_msip_any_nonzero() {
        let mut clint = Clint::new();
        clint.write32(MSIP_OFFSET, 0xFF);
        assert_eq!(clint.read32(MSIP_OFFSET), 1); // bool이라 1로 변환
    }

    #[test]
    fn test_clint_64bit_values() {
        let mut clint = Clint::new();
        let large_value: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        clint.write64(MTIME_OFFSET, large_value);
        assert_eq!(clint.read64(MTIME_OFFSET), large_value);
    }

    #[test]
    fn test_clint_tick() {
        let mut clint = Clint::new();
        assert_eq!(clint.read64(MTIME_OFFSET), 0);

        clint.tick();
        assert_eq!(clint.read64(MTIME_OFFSET), 1);

        clint.tick();
        clint.tick();
        assert_eq!(clint.read64(MTIME_OFFSET), 3);
    }

    #[test]
    fn test_clint_tick_multiple() {
        let mut clint = Clint::new();
        for _ in 0..100 {
            clint.tick();
        }
        assert_eq!(clint.read64(MTIME_OFFSET), 100);
    }
}
