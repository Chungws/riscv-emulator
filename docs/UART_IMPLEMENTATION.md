# UART (16550) 구현 가이드

RISC-V 에뮬레이터의 시리얼 통신을 위한 16550 UART 구현 가이드.

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

## 2. 레지스터 맵

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

## 3. 주요 레지스터 상세

### LSR (Line Status Register) - 오프셋 5

| 비트 | 이름 | 의미 |
|------|------|------|
| 0 | DR | 수신 데이터 있음 |
| 5 | THRE | 송신 레지스터 비어있음 |
| 6 | TEMT | 송신기 비어있음 |

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

## 4. 동작 메커니즘

### 4.1 송신 흐름

1. 소프트웨어가 LSR.THRE 확인
2. THR에 데이터 쓰기
3. UART가 stdout으로 출력 (에뮬레이터)
4. 즉시 완료 → LSR.THRE = 1 유지

### 4.2 수신 흐름

1. 외부 입력 → rx_fifo에 저장
2. LSR.DR = 1 설정
3. IER.ERBFI가 1이면 인터럽트 발생
4. 소프트웨어가 RBR 읽기
5. FIFO 비면 LSR.DR = 0

### 4.3 인터럽트 우선순위

1. 수신 데이터 가용 (높음)
2. THR 비어있음 (낮음)

---

## 5. 구현 단계

### Step 1: 구조체와 상수 정의

**목표**: UART 상태를 저장할 구조체 정의

- [ ] 레지스터 오프셋 상수 (RBR, THR, IER, IIR, FCR, LCR, LSR, SCR)
- [ ] LSR 비트 상수 (DR, THRE, TEMT)
- [ ] IER 비트 상수 (RX_ENABLE, TX_ENABLE)
- [ ] IIR 비트 상수 (NO_INTERRUPT, RX_DATA, THR_EMPTY, FIFO_ENABLED)
- [ ] Uart 구조체 필드 추가 (ier, iir, fcr, lcr, lsr, scr, rx_fifo)
- [ ] `new()`: lsr = THRE | TEMT로 초기화

**검증**: 구조체 생성 테스트

---

### Step 2: LSR 상태 관리

**목표**: rx_fifo 상태에 따라 LSR 동적 업데이트

- [ ] `update_lsr()` 메서드 추가
- [ ] rx_fifo 비어있지 않으면 DR = 1
- [ ] rx_fifo 비어있으면 DR = 0
- [ ] THRE, TEMT는 항상 1 (즉시 송신 모델)

**검증**: 초기 상태 THRE=1, TEMT=1, DR=0 확인

---

### Step 3: 레지스터 읽기

**목표**: 오프셋별 읽기 구현

- [ ] `read8(offset)` 메서드 구현
- [ ] RBR (오프셋 0): rx_fifo.pop_front()
- [ ] IER (오프셋 1): ier 반환
- [ ] IIR (오프셋 2): iir 반환
- [ ] LCR (오프셋 3): lcr 반환
- [ ] LSR (오프셋 5): update_lsr() 후 lsr 반환
- [ ] SCR (오프셋 7): scr 반환

**검증**: 각 레지스터 읽기 테스트

---

### Step 4: 레지스터 쓰기

**목표**: 오프셋별 쓰기 구현

- [ ] `write8(offset, value)` 메서드 구현
- [ ] THR (오프셋 0): stdout 출력
- [ ] IER (오프셋 1): 하위 4비트만 저장
- [ ] FCR (오프셋 2): FIFO 리셋 처리
- [ ] LCR (오프셋 3): lcr에 저장
- [ ] SCR (오프셋 7): scr에 저장

**검증**: THR 쓰기 → stdout 출력 확인

---

### Step 5: 인터럽트 상태 관리

**목표**: IIR 업데이트 및 인터럽트 감지

- [ ] `update_iir()` 메서드 추가
- [ ] RX 인터럽트: IER.RX_ENABLE && !rx_fifo.is_empty()
- [ ] TX 인터럽트: IER.TX_ENABLE && LSR.THRE
- [ ] 인터럽트 우선순위: RX > TX
- [ ] `check_interrupt()` 메서드 추가

**검증**: 수신 데이터 있을 때 인터럽트 발생 테스트

---

### Step 6: 외부 입력 메서드

**목표**: 키보드 입력을 rx_fifo에 추가

- [ ] `push_input(c)` 메서드 추가
- [ ] rx_fifo.push_back(c)
- [ ] update_lsr() 호출
- [ ] update_iir() 호출

**검증**: push_input 후 LSR.DR = 1 확인

---

### Step 7: Bus 연결 업데이트

**목표**: Bus에서 UART offset 전달

- [ ] Bus read8에서 UART offset 계산하여 전달
- [ ] Bus write8에서 UART offset 계산하여 전달
- [ ] `check_uart_interrupt()` 메서드 추가

**검증**: Bus를 통한 UART 레지스터 접근 테스트

---

### Step 8: 인터럽트 통합

**목표**: UART 인터럽트를 CPU로 전달

- [ ] 방법 결정 (직접 연결 or PLIC)
- [ ] CPU check_pending_interrupts에서 UART 인터럽트 체크
- [ ] MIP.MEIP 비트 업데이트
- [ ] 외부 인터럽트 트랩 처리

**검증**: 수신 데이터 → 인터럽트 핸들러 진입 테스트

---

### Step 9: 키보드 입력 처리

**목표**: 실시간 키보드 입력

- [ ] 별도 스레드에서 stdin 읽기
- [ ] mpsc 채널로 메인 스레드에 전달
- [ ] 메인 루프에서 try_recv()로 확인
- [ ] push_input()으로 UART에 전달

**검증**: 에코 프로그램 테스트

---

## 6. xv6 UART 사용 패턴

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

## 7. 테스트 시나리오

### 기본 송신

1. LSR 읽기 → THRE = 1
2. THR에 'A' 쓰기
3. stdout에 'A' 출력 확인

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

## 8. 주의사항

### DLAB 비트

- LCR.DLAB = 1이면 오프셋 0, 1이 보레이트 설정용
- 에뮬레이터에서는 보레이트 무시 가능
- xv6는 보레이트 설정 안 함

### 송신 타이밍

- 실제 하드웨어: 보레이트에 따른 지연
- 에뮬레이터: 즉시 완료 (THRE 항상 1)

### FIFO 크기

- 실제 16550: 16바이트
- 에뮬레이터: 무제한 또는 적당한 크기

---

## 9. 참고 자료

- 16550 UART 데이터시트
- xv6 kernel/uart.c
- https://wiki.osdev.org/Serial_Ports
