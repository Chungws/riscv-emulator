use crate::devices;

pub struct Bus {
    memory: devices::Memory,
    uart: devices::Uart,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: devices::Memory::new(),
            uart: devices::Uart::new(),
        }
    }

    pub fn read8(&self, addr: u32) -> u8 {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8()
        } else if addr >= devices::DRAM_BASE {
            self.memory.read8(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn read16(&self, addr: u32) -> u16 {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8() as u16
        } else if addr >= devices::DRAM_BASE {
            self.memory.read16(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn read32(&self, addr: u32) -> u32 {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8() as u32
        } else if addr >= devices::DRAM_BASE {
            self.memory.read32(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write8(&mut self, addr: u32, value: u8) {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.write8(value);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write8(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write16(&mut self, addr: u32, value: u16) {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.write8(value as u8);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write16(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write32(&mut self, addr: u32, value: u32) {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.write8(value as u8);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write32(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
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

    // 잘못된 주소 테스트
    #[test]
    #[should_panic(expected = "Invalid address")]
    fn test_bus_invalid_address() {
        let bus = Bus::new();
        bus.read8(0x00000000); // DRAM도 UART도 아닌 주소
    }
}
