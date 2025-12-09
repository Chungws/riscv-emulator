use crate::devices::{self, CLINT_BASE};

pub struct Bus {
    clint: devices::Clint,
    memory: devices::Memory,
    uart: devices::Uart,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            clint: devices::Clint::new(),
            memory: devices::Memory::new(),
            uart: devices::Uart::new(),
        }
    }

    pub fn read8(&self, addr: u64) -> u8 {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8()
        } else if addr >= devices::DRAM_BASE {
            self.memory.read8(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn read16(&self, addr: u64) -> u16 {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8() as u16
        } else if addr >= devices::DRAM_BASE {
            self.memory.read16(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn read32(&self, addr: u64) -> u32 {
        if addr >= devices::CLINT_BASE && addr < devices::CLINT_BASE + devices::CLINT_SIZE {
            self.clint.read32(addr - CLINT_BASE)
        } else if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8() as u32
        } else if addr >= devices::DRAM_BASE {
            self.memory.read32(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn read64(&self, addr: u64) -> u64 {
        if addr >= devices::CLINT_BASE && addr < devices::CLINT_BASE + devices::CLINT_SIZE {
            self.clint.read64(addr - CLINT_BASE)
        } else if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8() as u64
        } else if addr >= devices::DRAM_BASE {
            self.memory.read64(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }
    pub fn write8(&mut self, addr: u64, value: u8) {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.write8(value);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write8(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write16(&mut self, addr: u64, value: u16) {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.write8(value as u8);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write16(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write32(&mut self, addr: u64, value: u32) {
        if addr >= devices::CLINT_BASE && addr < devices::CLINT_BASE + devices::CLINT_SIZE {
            self.clint.write32(addr - CLINT_BASE, value);
        } else if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.write8(value as u8);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write32(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write64(&mut self, addr: u64, value: u64) {
        if addr >= devices::CLINT_BASE && addr < devices::CLINT_BASE + devices::CLINT_SIZE {
            self.clint.write64(addr - CLINT_BASE, value);
        } else if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.write8(value as u8);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write64(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn tick(&mut self) {
        self.clint.tick();
    }

    pub fn check_timer_interrupt(&self) -> bool {
        self.clint.check_timer_interrupt()
    }

    pub fn check_software_interrupt(&self) -> bool {
        self.clint.check_software_interrupt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_new() {
        let _bus = Bus::new();
    }

    // DRAM 테스트
    #[test]
    fn test_bus_dram_read_write8() {
        let mut bus = Bus::new();
        bus.write8(0x80000000, 0xAB);
        assert_eq!(bus.read8(0x80000000), 0xAB);
    }

    #[test]
    fn test_bus_dram_read_write16() {
        let mut bus = Bus::new();
        bus.write16(0x80000000, 0xABCD);
        assert_eq!(bus.read16(0x80000000), 0xABCD);
    }

    #[test]
    fn test_bus_dram_read_write32() {
        let mut bus = Bus::new();
        bus.write32(0x80000000, 0xDEADBEEF);
        assert_eq!(bus.read32(0x80000000), 0xDEADBEEF);
    }

    // UART 테스트
    #[test]
    fn test_bus_uart_read8() {
        let bus = Bus::new();
        assert_eq!(bus.read8(0x10000000), 0); // UART read returns 0
    }

    #[test]
    fn test_bus_uart_write8() {
        let mut bus = Bus::new();
        bus.write8(0x10000000, b'X'); // 출력됨
    }

    // CLINT 테스트
    #[test]
    fn test_bus_clint_mtime() {
        let mut bus = Bus::new();
        bus.write64(0x200BFF8, 12345); // CLINT_BASE + MTIME_OFFSET
        assert_eq!(bus.read64(0x200BFF8), 12345);
    }

    #[test]
    fn test_bus_clint_mtimecmp() {
        let mut bus = Bus::new();
        bus.write64(0x2004000, 99999); // CLINT_BASE + MTIMECMP_OFFSET
        assert_eq!(bus.read64(0x2004000), 99999);
    }

    #[test]
    fn test_bus_clint_msip() {
        let mut bus = Bus::new();
        bus.write32(0x2000000, 1); // CLINT_BASE + MSIP_OFFSET
        assert_eq!(bus.read32(0x2000000), 1);
    }

    // 잘못된 주소 테스트
    #[test]
    #[should_panic(expected = "Invalid address")]
    fn test_bus_invalid_address() {
        let bus = Bus::new();
        bus.read8(0x00000000); // DRAM도 UART도 아닌 주소
    }
}
