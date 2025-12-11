use std::{
    collections::VecDeque,
    io::{Write, stdout},
};

use crate::devices::terminal::Terminal;

pub const UART_BASE: u64 = 0x10000000;
pub const UART_SIZE: u64 = 8;

const UART_RBR: u32 = 0;
const UART_THR: u32 = 1;
const UART_IER: u32 = 2;
const UART_IIR: u32 = 3;
const UART_FCR: u32 = 4;
const UART_LCR: u32 = 5;
const UART_LSR: u32 = 6;
const UART_SCR: u32 = 7;

const LSR_DR: u32 = 0x1;
const LSR_THRE: u32 = 0x1 << 5;
const LSR_TEMT: u32 = 0x1 << 6;

const IER_RX_ENABLE: u32 = 0x1;
const IER_TX_ENABLE: u32 = 0x1 << 1;

const IIR_NO_INTERRUPT: u32 = 0x1;
const IIR_RX_DATA: u32 = 0x1;
const IIR_THR_EMPTY: u32 = 0x1;
const IIR_FIFO_ENABLED: u32 = 0x1;

pub struct Uart {
    rx_fifo: VecDeque<u8>,
    tx_fifo: VecDeque<u8>,
    tsr: Option<u8>,
    ier: u32,
    iir: u32,
    fcr: u32,
    lcr: u32,
    lsr: u32,
    scr: u32,
    terminal: Box<dyn Terminal>,
}

impl Uart {
    pub fn new(terminal: Box<dyn Terminal>) -> Self {
        Self {
            rx_fifo: VecDeque::new(),
            tx_fifo: VecDeque::new(),
            tsr: None,
            ier: 0,
            iir: 0,
            fcr: 0,
            lcr: 0,
            lsr: LSR_TEMT | LSR_THRE,
            scr: 0,
            terminal: terminal,
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
    use crate::devices::terminal::tests::MockTerminal;

    fn create_uart() -> Uart {
        Uart::new(Box::new(MockTerminal::new()))
    }

    #[test]
    fn test_uart_new() {
        let uart = create_uart();
        // LSR 초기값: THRE | TEMT (TX 준비 완료)
        assert_eq!(uart.lsr, LSR_THRE | LSR_TEMT);
        // FIFO는 빈 상태
        assert!(uart.tx_fifo.is_empty());
        assert!(uart.rx_fifo.is_empty());
        // TSR은 비어있음
        assert!(uart.tsr.is_none());
    }

    #[test]
    fn test_uart_initial_registers() {
        let uart = create_uart();
        assert_eq!(uart.ier, 0);
        assert_eq!(uart.iir, 0);
        assert_eq!(uart.fcr, 0);
        assert_eq!(uart.lcr, 0);
        assert_eq!(uart.scr, 0);
    }
}
