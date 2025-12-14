use std::collections::VecDeque;

use crate::devices::terminal::Terminal;

pub const UART_BASE: u64 = 0x10000000;
pub const UART_SIZE: u64 = 8;

const UART_RBR: u8 = 0;
const UART_THR: u8 = 0;
const UART_IER: u8 = 1;
const UART_IIR: u8 = 2;
const UART_FCR: u8 = 2;
const UART_LCR: u8 = 3;
const UART_LSR: u8 = 5;
const UART_SCR: u8 = 7;

const LSR_DR: u8 = 0x1;
const LSR_THRE: u8 = 0x1 << 5;
const LSR_TEMT: u8 = 0x1 << 6;

const IER_RX_ENABLE: u8 = 0x1;
const IER_TX_ENABLE: u8 = 0x1 << 1;

const IIR_NO_INTERRUPT: u8 = 0x01;
const IIR_RX_INTERRUPT: u8 = 0x04;
const IIR_TX_INTERRUPT: u8 = 0x02;
const IIR_FIFO_ENABLED: u8 = 0xC0;

pub struct Uart {
    rx_fifo: VecDeque<u8>,
    tx_fifo: VecDeque<u8>,
    tsr: Option<u8>,
    ier: u8,
    iir: u8,
    lcr: u8,
    lsr: u8,
    scr: u8,
    terminal: Box<dyn Terminal>,
}

impl Uart {
    pub fn new(terminal: Box<dyn Terminal>) -> Self {
        Self {
            rx_fifo: VecDeque::new(),
            tx_fifo: VecDeque::new(),
            tsr: None,
            ier: 0,
            iir: IIR_FIFO_ENABLED | IIR_NO_INTERRUPT,
            lcr: 0,
            lsr: LSR_TEMT | LSR_THRE,
            scr: 0,
            terminal: terminal,
        }
    }

    pub fn read8(&mut self, offset: u8) -> u8 {
        match offset {
            UART_RBR => {
                if let Some(data) = self.rx_fifo_pop() {
                    self.update_lsr();
                    self.update_iir();
                    return data;
                }
                0
            }
            UART_IER => self.ier as u8,
            UART_IIR => self.iir as u8,
            UART_LCR => self.lcr as u8,
            UART_LSR => {
                self.update_lsr();
                self.lsr as u8
            }
            UART_SCR => self.scr as u8,
            _ => panic!("Not Supported Offset"),
        }
    }

    pub fn write8(&mut self, offset: u8, value: u8) {
        match offset {
            UART_THR => {
                self.tx_fifo_push(value);
                self.transmit();
                self.update_iir();
            }
            UART_IER => {
                self.ier = (self.ier & 0xF0) | (value & 0x0F);
                self.update_iir();
            }
            UART_FCR => {
                if value & 0x2 != 0 {
                    self.rx_fifo.clear();
                }
                if value & 0x4 != 0 {
                    self.tx_fifo.clear();
                }
            }
            UART_LCR => {
                self.lcr = value;
            }
            UART_SCR => {
                self.scr = value;
            }
            _ => panic!("Not Supported Offset"),
        }
    }

    pub fn receive_input(&mut self) {
        if let Some(data) = self.terminal.read() {
            self.rx_fifo_push(data);
        }
    }

    pub fn push_input(&mut self, data: u8) {
        self.rx_fifo_push(data);
    }

    pub fn check_interrupt(&self) -> bool {
        self.iir & IIR_NO_INTERRUPT == 0
    }

    fn transmit(&mut self) {
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

    fn update_iir(&mut self) {
        if self.ier & IER_RX_ENABLE != 0 && !self.rx_fifo.is_empty() {
            self.iir = IIR_FIFO_ENABLED | IIR_RX_INTERRUPT;
            return;
        }
        if self.ier & IER_TX_ENABLE != 0 && self.tx_fifo.is_empty() {
            self.iir = IIR_FIFO_ENABLED | IIR_TX_INTERRUPT;
            return;
        }
        self.iir = IIR_FIFO_ENABLED | IIR_NO_INTERRUPT;
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
            self.update_iir();
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
        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_NO_INTERRUPT);
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

    // Step 5: 레지스터 읽기 테스트
    #[test]
    fn test_read8_rbr() {
        let mut uart = create_uart();

        // RX FIFO에 데이터 추가
        uart.rx_fifo_push(b'H');
        uart.rx_fifo_push(b'i');

        // RBR 읽기
        assert_eq!(uart.read8(UART_RBR), b'H');
        assert_eq!(uart.read8(UART_RBR), b'i');
        assert_eq!(uart.read8(UART_RBR), 0); // 빈 FIFO
    }

    #[test]
    fn test_read8_rbr_updates_lsr() {
        let mut uart = create_uart();

        uart.rx_fifo_push(b'A');
        assert_eq!(uart.lsr & LSR_DR, LSR_DR); // DR = 1

        uart.read8(UART_RBR); // RBR 읽기
        assert_eq!(uart.lsr & LSR_DR, 0); // DR = 0 (FIFO 비어짐)
    }

    #[test]
    fn test_read8_ier() {
        let mut uart = create_uart();
        uart.ier = 0x0F;
        assert_eq!(uart.read8(UART_IER), 0x0F);
    }

    #[test]
    fn test_read8_iir() {
        let mut uart = create_uart();
        uart.iir = 0x01;
        assert_eq!(uart.read8(UART_IIR), 0x01);
    }

    #[test]
    fn test_read8_lcr() {
        let mut uart = create_uart();
        uart.lcr = 0x03;
        assert_eq!(uart.read8(UART_LCR), 0x03);
    }

    #[test]
    fn test_read8_lsr() {
        let mut uart = create_uart();

        // 초기 LSR: THRE | TEMT
        let lsr = uart.read8(UART_LSR);
        assert_eq!(lsr & (LSR_THRE as u8), LSR_THRE as u8);
        assert_eq!(lsr & (LSR_TEMT as u8), LSR_TEMT as u8);
    }

    #[test]
    fn test_read8_scr() {
        let mut uart = create_uart();
        uart.scr = 0xAB;
        assert_eq!(uart.read8(UART_SCR), 0xAB);
    }

    // Step 6: 레지스터 쓰기 테스트
    #[test]
    fn test_write8_thr() {
        let (mut uart, mock) = create_uart_with_mock();

        // THR에 쓰기 → TX FIFO → transmit → Terminal
        uart.write8(UART_THR, b'H');
        uart.write8(UART_THR, b'i');

        assert_eq!(mock.borrow().output_as_string(), "Hi");
    }

    #[test]
    fn test_write8_ier() {
        let mut uart = create_uart();

        // 하위 4비트만 저장
        uart.write8(UART_IER, 0xFF);
        assert_eq!(uart.ier, 0x0F);

        uart.write8(UART_IER, 0x03);
        assert_eq!(uart.ier, 0x03);
    }

    #[test]
    fn test_write8_fcr_rx_reset() {
        let mut uart = create_uart();

        // RX FIFO에 데이터 추가
        uart.rx_fifo_push(b'A');
        uart.rx_fifo_push(b'B');
        assert_eq!(uart.rx_fifo.len(), 2);

        // FCR bit 1: RX FIFO 리셋
        uart.write8(UART_FCR, 0x02);
        assert!(uart.rx_fifo.is_empty());
    }

    #[test]
    fn test_write8_fcr_tx_reset() {
        let mut uart = create_uart();

        // TX FIFO에 데이터 추가 (transmit 호출 안 하고 직접)
        uart.tx_fifo.push_back(b'A');
        uart.tx_fifo.push_back(b'B');
        assert_eq!(uart.tx_fifo.len(), 2);

        // FCR bit 2: TX FIFO 리셋
        uart.write8(UART_FCR, 0x04);
        assert!(uart.tx_fifo.is_empty());
    }

    #[test]
    fn test_write8_lcr() {
        let mut uart = create_uart();

        uart.write8(UART_LCR, 0x03);
        assert_eq!(uart.lcr, 0x03);
    }

    #[test]
    fn test_write8_scr() {
        let mut uart = create_uart();

        uart.write8(UART_SCR, 0xCD);
        assert_eq!(uart.scr, 0xCD);
    }

    // Step 7: 인터럽트 상태 관리 테스트
    #[test]
    fn test_update_iir_no_interrupt() {
        let mut uart = create_uart();

        // IER = 0 (인터럽트 비활성)
        uart.update_iir();

        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_NO_INTERRUPT);
        assert!(!uart.check_interrupt());
    }

    #[test]
    fn test_update_iir_rx_interrupt() {
        let mut uart = create_uart();

        // RX 인터럽트 활성화
        uart.write8(UART_IER, IER_RX_ENABLE);

        // RX FIFO에 데이터 추가
        uart.rx_fifo_push(b'A');

        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_RX_INTERRUPT);
        assert!(uart.check_interrupt());
    }

    #[test]
    fn test_update_iir_tx_interrupt() {
        let mut uart = create_uart();

        // TX 인터럽트 활성화
        uart.write8(UART_IER, IER_TX_ENABLE);

        // TX FIFO 비어있음 → TX 인터럽트 발생
        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_TX_INTERRUPT);
        assert!(uart.check_interrupt());
    }

    #[test]
    fn test_update_iir_rx_priority_over_tx() {
        let mut uart = create_uart();

        // RX, TX 둘 다 활성화
        uart.write8(UART_IER, IER_RX_ENABLE | IER_TX_ENABLE);

        // TX FIFO 비어있고, RX FIFO에 데이터 있음
        uart.rx_fifo_push(b'A');

        // RX 우선순위가 높음
        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_RX_INTERRUPT);
    }

    #[test]
    fn test_iir_clear_after_rbr_read() {
        let mut uart = create_uart();

        // RX 인터럽트 활성화 + 데이터 추가
        uart.write8(UART_IER, IER_RX_ENABLE);
        uart.rx_fifo_push(b'A');

        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_RX_INTERRUPT);
        assert!(uart.check_interrupt());

        // RBR 읽기 → RX FIFO 비워짐 → 인터럽트 해제
        uart.read8(UART_RBR);

        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_NO_INTERRUPT);
        assert!(!uart.check_interrupt());
    }

    #[test]
    fn test_iir_tx_clear_after_thr_write() {
        let (mut uart, _mock) = create_uart_with_mock();

        // TX 인터럽트 활성화 (TX FIFO 비어있으므로 인터럽트 발생)
        uart.write8(UART_IER, IER_TX_ENABLE);
        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_TX_INTERRUPT);

        // THR에 쓰기 → TX FIFO에 데이터 들어감
        // 하지만 transmit()이 즉시 호출되어 비워지므로 다시 TX 인터럽트
        uart.write8(UART_THR, b'X');

        // transmit 후 TX FIFO 비어있으므로 TX 인터럽트 유지
        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_TX_INTERRUPT);
    }

    #[test]
    fn test_check_interrupt_returns_correct_value() {
        let mut uart = create_uart();

        // 초기: 인터럽트 없음
        uart.update_iir();
        assert!(!uart.check_interrupt());

        // RX 인터럽트 활성화 + 데이터 추가
        uart.write8(UART_IER, IER_RX_ENABLE);
        uart.rx_fifo_push(b'Z');

        assert!(uart.check_interrupt());
    }

    // Step 8: 외부 입력 메서드 테스트
    #[test]
    fn test_push_input() {
        let mut uart = create_uart();

        // push_input으로 데이터 주입
        uart.push_input(b'H');
        uart.push_input(b'i');

        // RX FIFO에 데이터 들어감
        assert_eq!(uart.rx_fifo.len(), 2);

        // LSR.DR = 1
        assert_eq!(uart.lsr & LSR_DR, LSR_DR);

        // RBR로 읽기
        assert_eq!(uart.read8(UART_RBR), b'H');
        assert_eq!(uart.read8(UART_RBR), b'i');
    }

    #[test]
    fn test_push_input_triggers_interrupt() {
        let mut uart = create_uart();

        // RX 인터럽트 활성화
        uart.write8(UART_IER, IER_RX_ENABLE);

        // 인터럽트 없음
        assert!(!uart.check_interrupt());

        // push_input → 인터럽트 발생
        uart.push_input(b'X');

        assert!(uart.check_interrupt());
        assert_eq!(uart.iir, IIR_FIFO_ENABLED | IIR_RX_INTERRUPT);
    }

    #[test]
    fn test_receive_input_from_terminal() {
        let (mut uart, mock) = create_uart_with_mock();

        // Terminal에 입력 데이터 설정
        mock.borrow_mut().input.push_back(b'A');
        mock.borrow_mut().input.push_back(b'B');

        // receive_input 호출
        uart.receive_input();
        uart.receive_input();

        // RX FIFO에 데이터 들어감
        assert_eq!(uart.rx_fifo.len(), 2);
        assert_eq!(uart.read8(UART_RBR), b'A');
        assert_eq!(uart.read8(UART_RBR), b'B');
    }

    #[test]
    fn test_receive_input_empty_terminal() {
        let (mut uart, _mock) = create_uart_with_mock();

        // Terminal에 입력 없음
        uart.receive_input();

        // RX FIFO 비어있음
        assert!(uart.rx_fifo.is_empty());
        assert_eq!(uart.lsr & LSR_DR, 0);
    }
}
