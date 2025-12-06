use std::{
    collections::VecDeque,
    io::{Write, stdout},
};

pub const UART_BASE: u64 = 0x10000000;
pub const UART_SIZE: u64 = 8;

const UART_RBR: u32 = 0; // Receive Buffer Register
const UART_THR: u32 = 0; // Transmit Holding Register
const UART_LSR: u32 = 5; // Line Status Register

const LSR_DATA_READY: u8 = 1 << 0;
const LSR_THR_EMPTY: u8 = 1 << 5;

pub struct Uart {
    rx_buffer: VecDeque<u8>,
}

impl Uart {
    pub fn new() -> Self {
        Self {
            rx_buffer: VecDeque::new(),
        }
    }

    pub fn read8(&self) -> u8 {
        0
    }

    pub fn write8(&mut self, value: u8) {
        print!("{}", value as char);
        stdout().flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uart_new() {
        let _uart = Uart::new();
    }

    #[test]
    fn test_uart_read8_returns_zero() {
        let uart = Uart::new();
        assert_eq!(uart.read8(), 0);
    }

    #[test]
    fn test_uart_write8() {
        let mut uart = Uart::new();
        // 실제 출력은 터미널에 나감 (테스트에서 캡처 어려움)
        uart.write8(b'H');
        uart.write8(b'i');
    }
}
