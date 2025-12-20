use std::collections::HashMap;

use crate::devices;
use crate::devices::stdioterminal::StdioTerminal;

pub struct Bus {
    clint: devices::Clint,
    memory: devices::Memory,
    uart: devices::Uart,
    reservations: HashMap<u64, u64>,
}

impl Bus {
    pub fn new() -> Self {
        let (terminal, _handle) = StdioTerminal::new();
        Self {
            clint: devices::Clint::new(),
            memory: devices::Memory::new(),
            uart: devices::Uart::new(Box::new(terminal)),
            reservations: HashMap::new(),
        }
    }

    pub fn read8(&mut self, addr: u64) -> u8 {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8((addr - devices::UART_BASE) as u8)
        } else if addr >= devices::DRAM_BASE {
            self.memory.read8(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn read16(&mut self, addr: u64) -> u16 {
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8((addr - devices::UART_BASE) as u8) as u16
        } else if addr >= devices::DRAM_BASE {
            self.memory.read16(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn read32(&mut self, addr: u64) -> u32 {
        if addr >= devices::CLINT_BASE && addr < devices::CLINT_BASE + devices::CLINT_SIZE {
            self.clint.read32(addr - devices::CLINT_BASE)
        } else if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8((addr - devices::UART_BASE) as u8) as u32
        } else if addr >= devices::DRAM_BASE {
            self.memory.read32(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn read64(&mut self, addr: u64) -> u64 {
        if addr >= devices::CLINT_BASE && addr < devices::CLINT_BASE + devices::CLINT_SIZE {
            self.clint.read64(addr - devices::CLINT_BASE)
        } else if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.read8((addr - devices::UART_BASE) as u8) as u64
        } else if addr >= devices::DRAM_BASE {
            self.memory.read64(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }
    pub fn write8(&mut self, addr: u64, value: u8) {
        self.invalidate_reservations(addr);
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart.write8((addr - devices::UART_BASE) as u8, value);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write8(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write16(&mut self, addr: u64, value: u16) {
        self.invalidate_reservations(addr);
        if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart
                .write8((addr - devices::UART_BASE) as u8, value as u8);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write16(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write32(&mut self, addr: u64, value: u32) {
        self.invalidate_reservations(addr);
        if addr >= devices::CLINT_BASE && addr < devices::CLINT_BASE + devices::CLINT_SIZE {
            self.clint.write32(addr - devices::CLINT_BASE, value);
        } else if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart
                .write8((addr - devices::UART_BASE) as u8, value as u8);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write32(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write64(&mut self, addr: u64, value: u64) {
        self.invalidate_reservations(addr);
        if addr >= devices::CLINT_BASE && addr < devices::CLINT_BASE + devices::CLINT_SIZE {
            self.clint.write64(addr - devices::CLINT_BASE, value);
        } else if addr >= devices::UART_BASE && addr < devices::UART_BASE + devices::UART_SIZE {
            self.uart
                .write8((addr - devices::UART_BASE) as u8, value as u8);
        } else if addr >= devices::DRAM_BASE {
            self.memory.write64(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn reserve(&mut self, hart_id: u64, addr: u64) {
        self.reservations.insert(hart_id, addr);
    }

    pub fn check_reservation(&self, hart_id: u64, addr: u64) -> bool {
        match self.reservations.get(&hart_id) {
            Some(val) => *val == addr,
            None => false,
        }
    }

    pub fn clear_reservation(&mut self, hart_id: u64) {
        self.reservations.remove(&hart_id);
    }

    pub fn invalidate_reservations(&mut self, addr: u64) {
        self.reservations.retain(|_key, value| *value != addr);
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

    pub fn check_uart_interrupt(&self) -> bool {
        self.uart.check_interrupt()
    }

    pub fn push_uart_input(&mut self, data: u8) {
        self.uart.push_input(data);
    }

    pub fn receive_uart_input(&mut self) {
        self.uart.receive_input();
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
        let mut bus = Bus::new();
        assert_eq!(bus.read8(0x10000000), 0); // UART read returns 0
    }

    #[test]
    fn test_bus_uart_write8() {
        let mut bus = Bus::new();
        bus.write8(0x10000000, b'X'); // 출력됨
    }

    #[test]
    fn test_bus_check_uart_interrupt() {
        let mut bus = Bus::new();

        // 초기: 인터럽트 없음
        assert!(!bus.check_uart_interrupt());

        // IER에 RX 인터럽트 활성화 (UART_BASE + 1)
        bus.write8(0x10000001, 0x01);

        // push_input으로 데이터 주입 → 인터럽트 발생
        bus.uart.push_input(b'A');

        assert!(bus.check_uart_interrupt());
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
        let mut bus = Bus::new();
        bus.read8(0x00000000); // DRAM도 UART도 아닌 주소
    }

    // Reservation 테스트
    #[test]
    fn test_reserve_and_check() {
        let mut bus = Bus::new();
        let hart_id = 0;
        let addr = 0x80001000;

        // 예약 전에는 false
        assert!(!bus.check_reservation(hart_id, addr));

        // 예약 후에는 true
        bus.reserve(hart_id, addr);
        assert!(bus.check_reservation(hart_id, addr));

        // 다른 주소는 false
        assert!(!bus.check_reservation(hart_id, 0x80002000));
    }

    #[test]
    fn test_clear_reservation() {
        let mut bus = Bus::new();
        let hart_id = 0;
        let addr = 0x80001000;

        bus.reserve(hart_id, addr);
        assert!(bus.check_reservation(hart_id, addr));

        bus.clear_reservation(hart_id);
        assert!(!bus.check_reservation(hart_id, addr));
    }

    #[test]
    fn test_invalidate_reservations_on_write() {
        let mut bus = Bus::new();
        let hart_id = 0;
        let addr = 0x80001000;

        bus.reserve(hart_id, addr);
        assert!(bus.check_reservation(hart_id, addr));

        // 같은 주소에 write하면 예약 무효화
        bus.write32(addr, 0x12345678);
        assert!(!bus.check_reservation(hart_id, addr));
    }

    #[test]
    fn test_invalidate_does_not_affect_other_addresses() {
        let mut bus = Bus::new();
        let hart_id = 0;
        let addr1 = 0x80001000;
        let addr2 = 0x80002000;

        bus.reserve(hart_id, addr1);

        // 다른 주소에 write해도 예약 유지
        bus.write32(addr2, 0x12345678);
        assert!(bus.check_reservation(hart_id, addr1));
    }

    #[test]
    fn test_multi_hart_reservations() {
        let mut bus = Bus::new();
        let hart0 = 0;
        let hart1 = 1;
        let addr = 0x80001000;

        // 두 hart가 같은 주소 예약
        bus.reserve(hart0, addr);
        bus.reserve(hart1, addr);

        assert!(bus.check_reservation(hart0, addr));
        assert!(bus.check_reservation(hart1, addr));

        // write하면 둘 다 무효화
        bus.write32(addr, 0xDEADBEEF);
        assert!(!bus.check_reservation(hart0, addr));
        assert!(!bus.check_reservation(hart1, addr));
    }

    #[test]
    fn test_reserve_overwrites_previous() {
        let mut bus = Bus::new();
        let hart_id = 0;
        let addr1 = 0x80001000;
        let addr2 = 0x80002000;

        bus.reserve(hart_id, addr1);
        assert!(bus.check_reservation(hart_id, addr1));

        // 같은 hart가 다른 주소 예약하면 이전 예약 덮어씀
        bus.reserve(hart_id, addr2);
        assert!(!bus.check_reservation(hart_id, addr1));
        assert!(bus.check_reservation(hart_id, addr2));
    }

    #[test]
    fn test_write8_invalidates_reservation() {
        let mut bus = Bus::new();
        bus.reserve(0, 0x80001000);
        bus.write8(0x80001000, 0xFF);
        assert!(!bus.check_reservation(0, 0x80001000));
    }

    #[test]
    fn test_write16_invalidates_reservation() {
        let mut bus = Bus::new();
        bus.reserve(0, 0x80001000);
        bus.write16(0x80001000, 0xFFFF);
        assert!(!bus.check_reservation(0, 0x80001000));
    }

    #[test]
    fn test_write64_invalidates_reservation() {
        let mut bus = Bus::new();
        bus.reserve(0, 0x80001000);
        bus.write64(0x80001000, 0xFFFFFFFFFFFFFFFF);
        assert!(!bus.check_reservation(0, 0x80001000));
    }
}
