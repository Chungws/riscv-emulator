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

    pub fn transmit(&mut self) {
        if self.tsr.is_none() {
            self.tsr = self.tx_fifo_pop();
        }

        if let Some(data) = self.tsr.take() {
            self.terminal.write(data);
        }
    }

    fn update_lsr(&mut self) {
        if !self.rx_fifo.is_empty() {
            self.lsr |= LSR_DR;
        } else {
            self.lsr &= !LSR_DR;
        }

        if self.tx_fifo.is_empty() {
            self.lsr |= LSR_THRE;
            if self.tsr.is_none() {
                self.lsr |= LSR_TEMT;
            } else {
                self.lsr &= !LSR_TEMT;
            }
        } else {
            self.lsr &= !(LSR_THRE | LSR_TEMT);
        }
    }

    fn tx_fifo_push(&mut self, data: u8) {
        if self.tx_fifo.len() < 16 {
            self.tx_fifo.push_back(data);
            self.update_lsr();
        }
    }

    fn tx_fifo_pop(&mut self) -> Option<u8> {
        let res = self.tx_fifo.pop_front();
        self.update_lsr();
        res
    }

    fn rx_fifo_push(&mut self, data: u8) {
        if self.rx_fifo.len() < 16 {
            self.rx_fifo.push_back(data);
            self.update_lsr();
        }
    }

    fn rx_fifo_pop(&mut self) -> Option<u8> {
        let res = self.rx_fifo.pop_front();
        self.update_lsr();
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::terminal::tests::MockTerminal;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// 테스트용 SharedMockTerminal - Rc<RefCell>로 MockTerminal을 감싸서 공유 가능하게 함
    struct SharedMockTerminal(Rc<RefCell<MockTerminal>>);

    impl Terminal for SharedMockTerminal {
        fn write(&mut self, data: u8) {
            self.0.borrow_mut().output.push(data);
        }
        fn read(&mut self) -> Option<u8> {
            self.0.borrow_mut().input.pop_front()
        }
    }

    fn create_uart() -> Uart {
        Uart::new(Box::new(MockTerminal::new()))
    }

    fn create_uart_with_mock() -> (Uart, Rc<RefCell<MockTerminal>>) {
        let mock = Rc::new(RefCell::new(MockTerminal::new()));
        let uart = Uart::new(Box::new(SharedMockTerminal(mock.clone())));
        (uart, mock)
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

    // Step 3: FIFO 테스트
    #[test]
    fn test_tx_fifo_push_pop() {
        let mut uart = create_uart();

        uart.tx_fifo_push(b'A');
        uart.tx_fifo_push(b'B');

        assert_eq!(uart.tx_fifo.len(), 2);
        assert_eq!(uart.tx_fifo_pop(), Some(b'A'));
        assert_eq!(uart.tx_fifo_pop(), Some(b'B'));
        assert_eq!(uart.tx_fifo_pop(), None);
    }

    #[test]
    fn test_tx_fifo_max_16_bytes() {
        let mut uart = create_uart();

        // 16바이트까지만 저장
        for i in 0..20 {
            uart.tx_fifo_push(i);
        }

        assert_eq!(uart.tx_fifo.len(), 16);
    }

    #[test]
    fn test_rx_fifo_push_pop() {
        let mut uart = create_uart();

        uart.rx_fifo_push(b'X');
        uart.rx_fifo_push(b'Y');

        assert_eq!(uart.rx_fifo.len(), 2);
        assert_eq!(uart.rx_fifo_pop(), Some(b'X'));
        assert_eq!(uart.rx_fifo_pop(), Some(b'Y'));
        assert_eq!(uart.rx_fifo_pop(), None);
    }

    #[test]
    fn test_rx_fifo_max_16_bytes() {
        let mut uart = create_uart();

        for i in 0..20 {
            uart.rx_fifo_push(i);
        }

        assert_eq!(uart.rx_fifo.len(), 16);
    }

    // Step 3: transmit 테스트
    #[test]
    fn test_transmit_from_tx_fifo() {
        let (mut uart, mock) = create_uart_with_mock();

        uart.tx_fifo_push(b'H');
        uart.tx_fifo_push(b'i');

        // transmit 호출 - TX FIFO → TSR → Terminal
        uart.transmit();
        uart.transmit();

        // Terminal에 출력 확인
        assert_eq!(mock.borrow().output_as_string(), "Hi");
    }

    #[test]
    fn test_transmit_empty_fifo() {
        let (mut uart, mock) = create_uart_with_mock();

        // 빈 FIFO에서 transmit - 아무것도 출력 안 됨
        uart.transmit();

        assert_eq!(mock.borrow().output.len(), 0);
    }

    #[test]
    fn test_transmit_clears_tsr() {
        let (mut uart, _mock) = create_uart_with_mock();

        uart.tx_fifo_push(b'A');
        uart.transmit();

        // transmit 후 TSR은 비어있어야 함
        assert!(uart.tsr.is_none());
    }

    // Step 4: LSR 상태 관리 테스트
    #[test]
    fn test_lsr_initial_state() {
        let uart = create_uart();
        // 초기: TX FIFO 비어있음, TSR 비어있음
        assert_eq!(uart.lsr & LSR_THRE, LSR_THRE); // THRE = 1
        assert_eq!(uart.lsr & LSR_TEMT, LSR_TEMT); // TEMT = 1
        assert_eq!(uart.lsr & LSR_DR, 0); // DR = 0
    }

    #[test]
    fn test_lsr_dr_set_when_rx_has_data() {
        let mut uart = create_uart();

        // RX FIFO에 데이터 추가
        uart.rx_fifo_push(b'A');

        // DR = 1
        assert_eq!(uart.lsr & LSR_DR, LSR_DR);
    }

    #[test]
    fn test_lsr_dr_clear_when_rx_empty() {
        let mut uart = create_uart();

        uart.rx_fifo_push(b'A');
        assert_eq!(uart.lsr & LSR_DR, LSR_DR); // DR = 1

        uart.rx_fifo_pop();
        assert_eq!(uart.lsr & LSR_DR, 0); // DR = 0
    }

    #[test]
    fn test_lsr_thre_clear_when_tx_has_data() {
        let mut uart = create_uart();

        // TX FIFO에 데이터 추가
        uart.tx_fifo_push(b'A');

        // THRE = 0 (TX FIFO에 데이터 있음)
        assert_eq!(uart.lsr & LSR_THRE, 0);
    }

    #[test]
    fn test_lsr_thre_set_when_tx_empty() {
        let mut uart = create_uart();

        uart.tx_fifo_push(b'A');
        assert_eq!(uart.lsr & LSR_THRE, 0); // THRE = 0

        uart.tx_fifo_pop();
        assert_eq!(uart.lsr & LSR_THRE, LSR_THRE); // THRE = 1
    }

    #[test]
    fn test_lsr_temt_clear_when_tsr_has_data() {
        let mut uart = create_uart();

        // TSR에 직접 데이터 설정
        uart.tsr = Some(b'X');
        uart.update_lsr();

        // TEMT = 0 (TSR에 데이터 있음)
        assert_eq!(uart.lsr & LSR_TEMT, 0);
    }

    #[test]
    fn test_lsr_temt_set_when_both_empty() {
        let mut uart = create_uart();

        // TX FIFO와 TSR 모두 비어있음
        assert_eq!(uart.lsr & LSR_TEMT, LSR_TEMT); // TEMT = 1

        // TX FIFO에 데이터 추가
        uart.tx_fifo_push(b'A');
        assert_eq!(uart.lsr & LSR_TEMT, 0); // TEMT = 0

        // TX FIFO 비우기
        uart.tx_fifo_pop();
        assert_eq!(uart.lsr & LSR_TEMT, LSR_TEMT); // TEMT = 1
    }
}
