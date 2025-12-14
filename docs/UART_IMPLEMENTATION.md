# UART (16550) 구현 가이드

RISC-V 에뮬레이터의 시리얼 통신을 위한 16550 UART 구현 가이드.
QEMU 스타일의 아키텍처를 참고하여 Terminal 백엔드 분리 구조로 구현.

---

## 1. UART 개요

### 1.1 16550 UART란?

16550은 가장 널리 사용되는 시리얼 통신 칩.
QEMU virt 머신과 대부분의 RISC-V 보드에서 콘솔 I/O에 사용.

### 1.2 왜 필요한가?

- OS 부팅 메시지 출력
- 디버그 콘솔
- 사용자 입력 처리
- xv6, Linux 등 OS와의 상호작용

### 1.3 주소

```
UART_BASE = 0x1000_0000
UART_SIZE = 0x100
UART_IRQ  = 10 (PLIC용)
```

---

## 2. 아키텍처 (QEMU 스타일)

### 2.1 전체 구조

```
[OS/Software]
     │
     ▼ write(THR)
┌─────────────────────────────────────────┐
│                 UART                    │
│  ┌──────────┐      ┌─────┐             │
│  │ TX FIFO  │─────→│ TSR │─────────────────→ Terminal Backend
│  │(16 bytes)│      └─────┘             │           │
│  └──────────┘                          │           ├── StdioTerminal
│                                        │           ├── FileTerminal
│  ┌──────────┐                          │           └── (확장 가능)
│  │ RX FIFO  │←─────────────────────────────────────┘
│  │(16 bytes)│      read(RBR)           │
│  └──────────┘                          │
└─────────────────────────────────────────┘
```

### 2.2 구성 요소

**UART**
- TX FIFO: 송신 대기 버퍼 (16바이트)
- RX FIFO: 수신 대기 버퍼 (16바이트)
- TSR (Transmit Shift Register): 현재 송신 중인 바이트
- 각종 제어/상태 레지스터

**Terminal (trait)**
- I/O 백엔드 추상화
- 구현체 교체로 다양한 출력 대상 지원 (stdout, 파일, 소켓 등)

**StdioTerminal**
- 기본 구현체
- stdout으로 출력, stdin에서 입력

### 2.3 QEMU와의 비교

| 항목 | QEMU | 우리 구현 |
|------|------|-----------|
| TX FIFO | Fifo8 (16바이트) | VecDeque (16바이트) |
| RX FIFO | Fifo8 (16바이트) | VecDeque (16바이트) |
| TSR | 있음 | 있음 |
| 백엔드 | chardev | Terminal trait |
| 타이밍 | 재시도/watch | 즉시 전송 (단순화) |

---

## 3. 레지스터 맵

| 오프셋 | 읽기 | 쓰기 | 설명 |
|--------|------|------|------|
| 0 | RBR | THR | 수신/송신 데이터 |
| 1 | IER | IER | 인터럽트 활성화 |
| 2 | IIR | FCR | 인터럽트 ID / FIFO 제어 |
| 3 | LCR | LCR | 라인 제어 |
| 4 | MCR | MCR | 모뎀 제어 (생략 가능) |
| 5 | LSR | - | 라인 상태 |
| 6 | MSR | - | 모뎀 상태 (생략 가능) |
| 7 | SCR | SCR | 스크래치 |

### 구현 우선순위

1. **필수**: RBR, THR, LSR (기본 송수신)
2. **권장**: IER, IIR, FCR, LCR (인터럽트, xv6 호환)
3. **선택**: MCR, MSR (모뎀 제어)

---

## 4. 주요 레지스터 상세

### LSR (Line Status Register) - 오프셋 5

| 비트 | 이름 | 의미 |
|------|------|------|
| 0 | DR | 수신 데이터 있음 (RX FIFO not empty) |
| 5 | THRE | TX FIFO 비어있음 |
| 6 | TEMT | TX FIFO와 TSR 모두 비어있음 |

### IER (Interrupt Enable Register) - 오프셋 1

| 비트 | 이름 | 의미 |
|------|------|------|
| 0 | ERBFI | 수신 인터럽트 활성화 |
| 1 | ETBEI | 송신 인터럽트 활성화 |

### IIR (Interrupt Identification Register) - 오프셋 2 읽기

| 비트 0 | 의미 |
|--------|------|
| 1 | 인터럽트 없음 |
| 0 | 인터럽트 펜딩 |

**인터럽트 ID (비트 3-1):**
- 010: 수신 데이터 가용
- 001: THR 비어있음

### FCR (FIFO Control Register) - 오프셋 2 쓰기

| 비트 | 의미 |
|------|------|
| 0 | FIFO 활성화 |
| 1 | RX FIFO 리셋 |
| 2 | TX FIFO 리셋 |

### LCR (Line Control Register) - 오프셋 3

| 비트 | 의미 |
|------|------|
| 1-0 | 워드 길이 (11 = 8비트) |
| 7 | DLAB |

---

## 5. 동작 메커니즘

### 5.1 송신 흐름 (TX)

```
1. 소프트웨어가 LSR.THRE 확인 (TX FIFO 여유 있는지)
2. THR에 데이터 쓰기
3. 데이터가 TX FIFO에 들어감
4. transmit() 호출:
   - TSR이 비어있으면 TX FIFO에서 pop → TSR에 로드
   - TSR 데이터를 Terminal로 출력
   - TSR 비움
5. TX FIFO 비면 LSR.THRE = 1
6. TX FIFO와 TSR 모두 비면 LSR.TEMT = 1
```

### 5.2 수신 흐름 (RX)

```
1. Terminal에서 입력 → RX FIFO에 저장
2. LSR.DR = 1 설정
3. IER.ERBFI가 1이면 인터럽트 발생
4. 소프트웨어가 RBR 읽기 → RX FIFO에서 pop
5. FIFO 비면 LSR.DR = 0
```

### 5.3 인터럽트 우선순위

1. 수신 데이터 가용 (높음)
2. THR 비어있음 (낮음)

---

## 6. 구현 단계

### Step 1: Terminal trait 정의

**목표**: I/O 백엔드 추상화

- [x] Terminal trait 정의
  - `fn write(&mut self, data: u8)` - 1바이트 출력
  - `fn read(&mut self) -> Option<u8>` - 1바이트 입력 (non-blocking)
- [x] StdioTerminal 구현체
  - write: stdout으로 출력
  - read: 입력 버퍼에서 읽기 (나중에 스레드와 연결)

**검증**: ~~StdioTerminal로 문자 출력 테스트~~ MockTerminal로 테스트 완료

---

### Step 2: UART 구조체와 상수 정의

**목표**: UART 상태를 저장할 구조체 정의

- [x] 레지스터 오프셋 상수 (RBR, THR, IER, IIR, FCR, LCR, LSR, SCR)
- [x] LSR 비트 상수 (DR, THRE, TEMT)
- [x] IER 비트 상수 (RX_ENABLE, TX_ENABLE)
- [x] IIR 비트 상수 (NO_INTERRUPT, RX_DATA, THR_EMPTY, FIFO_ENABLED)
- [x] Uart 구조체 필드:
  - `tx_fifo: VecDeque<u8>` (16바이트)
  - `rx_fifo: VecDeque<u8>` (16바이트)
  - `tsr: Option<u8>` (Transmit Shift Register)
  - `ier, iir, fcr, lcr, lsr, scr` (레지스터들)
  - `terminal: Box<dyn Terminal>` (백엔드)
- [x] `new(terminal)`: 초기 상태 설정

**검증**: 구조체 생성 테스트 완료

---

### Step 3: FIFO 및 TSR 관리

**목표**: TX/RX FIFO와 TSR 동작 구현

- [x] `tx_fifo_push(data)`: TX FIFO에 추가 (최대 16바이트)
- [x] `tx_fifo_pop() -> Option<u8>`: TX FIFO에서 꺼내기
- [x] `rx_fifo_push(data)`: RX FIFO에 추가
- [x] `rx_fifo_pop() -> Option<u8>`: RX FIFO에서 꺼내기
- [x] `transmit()`: TX FIFO → TSR → Terminal 전송
  - TSR이 비어있으면 FIFO에서 로드
  - Terminal.write() 호출
  - TSR 비움

**검증**: FIFO push/pop, transmit 동작 테스트 완료

---

### Step 4: LSR 상태 관리

**목표**: FIFO/TSR 상태에 따라 LSR 동적 업데이트

- [x] `update_lsr()` 메서드 추가
- [x] DR: RX FIFO가 비어있지 않으면 1
- [x] THRE: TX FIFO가 비어있으면 1
- [x] TEMT: TX FIFO와 TSR 모두 비어있으면 1

**검증**: 각 상태 전이 테스트 완료

---

### Step 5: 레지스터 읽기

**목표**: 오프셋별 읽기 구현

- [x] `read8(offset)` 메서드 구현
- [x] RBR (오프셋 0): rx_fifo.pop_front(), update_lsr()
- [x] IER (오프셋 1): ier 반환
- [x] IIR (오프셋 2): iir 반환
- [x] LCR (오프셋 3): lcr 반환
- [x] LSR (오프셋 5): update_lsr() 후 lsr 반환
- [x] SCR (오프셋 7): scr 반환

**검증**: 각 레지스터 읽기 테스트 완료

---

### Step 6: 레지스터 쓰기

**목표**: 오프셋별 쓰기 구현

- [x] `write8(offset, value)` 메서드 구현
- [x] THR (오프셋 0): tx_fifo에 push, transmit() 호출
- [x] IER (오프셋 1): 하위 4비트만 저장
- [x] FCR (오프셋 2): FIFO 리셋 처리
- [x] LCR (오프셋 3): lcr에 저장
- [x] SCR (오프셋 7): scr에 저장

**검증**: THR 쓰기 → Terminal 출력 테스트 완료

---

### Step 7: 인터럽트 상태 관리

**목표**: IIR 업데이트 및 인터럽트 감지

- [x] `update_iir()` 메서드 추가
- [x] RX 인터럽트: IER.RX_ENABLE && !rx_fifo.is_empty()
- [x] TX 인터럽트: IER.TX_ENABLE && tx_fifo.is_empty()
- [x] 인터럽트 우선순위: RX > TX
- [x] `check_interrupt() -> bool` 메서드 추가
- [x] IIR_FIFO_ENABLED (0xC0) 항상 설정
- [x] fcr 필드 제거 (FIFO 항상 활성화)

**검증**: 인터럽트 발생/해제 테스트 완료

---

### Step 8: 외부 입력 메서드

**목표**: Terminal 입력을 RX FIFO에 추가

- [x] `receive_input()` 메서드 추가
  - Terminal.read() 호출
  - 데이터 있으면 rx_fifo에 push
- [x] `push_input(c)` 메서드 (직접 주입용)
  - rx_fifo_push() 호출 (내부에서 update_lsr(), update_iir() 호출)
- [x] pub 정리: check_interrupt() pub, transmit() private

**검증**: push_input, receive_input 테스트 완료

---

### Step 9: Bus 연결 업데이트

**목표**: Bus에서 UART offset 전달

- [x] Bus read8에서 UART offset 계산하여 전달 (Step 5에서 완료)
- [x] Bus write8에서 UART offset 계산하여 전달 (Step 6에서 완료)
- [x] `check_uart_interrupt()` 메서드 추가
- [x] UART 초기 iir 값 수정 (IIR_FIFO_ENABLED | IIR_NO_INTERRUPT)

**검증**: Bus를 통한 UART 인터럽트 테스트 완료

---

### Step 10: 인터럽트 통합

**목표**: UART 인터럽트를 CPU로 전달

- [x] 방법 결정: CPU 직접 연결 (PLIC는 나중에)
- [x] CPU check_pending_interrupts에서 UART 인터럽트 체크
- [x] MIP.MEIP 비트 업데이트
- [x] 외부 인터럽트 트랩 처리 (우선순위: Software > Timer > External)
- [x] Bus::push_uart_input() 추가

**검증**: UART 외부 인터럽트 테스트 완료

---

### Step 11: 키보드 입력 처리

**목표**: 실시간 키보드 입력

- [ ] 별도 스레드에서 stdin 읽기
- [ ] mpsc 채널로 메인 스레드에 전달
- [ ] 메인 루프에서 try_recv()로 확인
- [ ] StdioTerminal에 입력 전달

**검증**: 에코 프로그램 테스트

---

## 7. xv6 UART 사용 패턴

### 초기화

1. FCR에 FIFO 활성화 + 리셋 쓰기
2. IER에 수신 인터럽트 활성화 쓰기
3. LCR에 8비트 모드 쓰기

### 송신 (폴링)

1. LSR 읽기
2. THRE = 0이면 대기
3. THR에 문자 쓰기

### 수신 (인터럽트)

1. 인터럽트 발생
2. LSR.DR 확인
3. RBR에서 문자 읽기

---

## 8. 테스트 시나리오

### 기본 송신

1. LSR 읽기 → THRE = 1, TEMT = 1
2. THR에 'A' 쓰기
3. TX FIFO → TSR → Terminal 출력
4. LSR 읽기 → THRE = 1, TEMT = 1

### TX FIFO 동작

1. THR에 연속으로 16바이트 쓰기
2. 17번째 쓰기 시 THRE = 0 (FIFO full)
3. transmit() 후 THRE = 1

### 기본 수신

1. push_input('H')
2. LSR 읽기 → DR = 1
3. RBR 읽기 → 'H'
4. LSR 읽기 → DR = 0

### 인터럽트

1. IER에 RX_ENABLE 쓰기
2. push_input('X')
3. check_interrupt() → true
4. RBR 읽기
5. check_interrupt() → false

---

## 9. 주의사항

### DLAB 비트

- LCR.DLAB = 1이면 오프셋 0, 1이 보레이트 설정용
- 에뮬레이터에서는 보레이트 무시 가능
- xv6는 보레이트 설정 안 함

### 송신 타이밍

- 실제 하드웨어: 보레이트에 따른 지연, TSR에서 비트 단위 송출
- 에뮬레이터: transmit() 호출 시 즉시 완료 (단순화)

### FIFO 크기

- 실제 16550: 16바이트
- 우리 구현: 16바이트 (동일)

---

## 10. 참고 자료

- 16550 UART 데이터시트
- xv6 kernel/uart.c
- https://wiki.osdev.org/Serial_Ports
- QEMU hw/char/serial.c: https://github.com/qemu/qemu/blob/master/hw/char/serial.c
