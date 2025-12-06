# UART 구현 (16550 호환)

실제 16550 UART 칩을 모방하여 송수신 버퍼와 상태 관리를 구현한다.

---

## 배경 지식

### UART란?

**U**niversal **A**synchronous **R**eceiver/**T**ransmitter

직렬 통신 장치로, 한 번에 1비트씩 데이터를 송수신한다.
컴퓨터와 터미널, 모뎀, 센서 등을 연결할 때 사용.

### 실제 하드웨어 동작

```
[송신]
CPU write → THR → Shift Register → TX 핀 → (비트 단위 전송) → 상대방
                   ↑
                   전송 완료까지 시간 소요 (baud rate에 따라)

[수신]
상대방 → RX 핀 → Shift Register → RBR → CPU read
                  ↑
                  수신 완료 시 Data Ready 플래그 설정
```

### 왜 버퍼가 필요한가?

1. **송신**: CPU는 빠르고, 직렬 전송은 느림 → 버퍼에 쌓아두고 천천히 전송
2. **수신**: 데이터가 언제 올지 모름 → 버퍼에 쌓아두고 CPU가 나중에 읽기

### Baud Rate

직렬 통신 속도. 예: 9600 baud = 초당 9600비트 ≈ 초당 960바이트
우리는 시뮬레이션이므로 "CPU 사이클 N번 = 1바이트 전송"으로 단순화.

---

## 16550 레지스터 맵

| 오프셋 | 읽기 | 쓰기 | 설명 |
|--------|------|------|------|
| 0 | RBR | THR | 데이터 수신/송신 |
| 1 | IER | IER | 인터럽트 활성화 (생략) |
| 2 | IIR | FCR | 인터럽트 상태/FIFO 제어 (생략) |
| 3 | LCR | LCR | 라인 제어 (생략) |
| 4 | MCR | MCR | 모뎀 제어 (생략) |
| 5 | LSR | - | 라인 상태 (읽기 전용) |
| 6 | MSR | - | 모뎀 상태 (생략) |
| 7 | SCR | SCR | 스크래치 (생략) |

우리가 구현할 것: **RBR, THR, LSR** (최소한의 동작에 필요)

---

## LSR (Line Status Register) 비트

| 비트 | 이름 | 의미 |
|------|------|------|
| 0 | DR (Data Ready) | 1 = RBR에 읽을 데이터 있음 |
| 1 | OE (Overrun Error) | 1 = 버퍼 오버플로우 발생 (선택) |
| 5 | THRE (THR Empty) | 1 = THR이 비어서 쓰기 가능 |
| 6 | TEMT (Transmitter Empty) | 1 = 송신 완전히 완료 |

---

## 구현 계획

### 구조체

```
Uart {
    rx_buffer: VecDeque<u8>    // 수신 버퍼 (외부 입력 저장)
    tx_buffer: VecDeque<u8>    // 송신 버퍼 (출력 대기)
    tx_busy_cycles: u32        // 현재 전송 중인 바이트의 남은 사이클
}
```

### 상수

```
UART_BASE: u32 = 0x10000000
UART_SIZE: u32 = 8

// 오프셋
RBR: u32 = 0  // Receive Buffer Register (read)
THR: u32 = 0  // Transmit Holding Register (write)
LSR: u32 = 5  // Line Status Register (read)

// LSR 비트
LSR_DR:   u8 = 0x01  // Data Ready
LSR_THRE: u8 = 0x20  // THR Empty
LSR_TEMT: u8 = 0x40  // Transmitter Empty

// 타이밍 (1바이트 전송에 필요한 CPU 사이클)
TX_CYCLES_PER_BYTE: u32 = 100
```

---

## Step 1: 구조체와 상수 정의

### 목표
- UART 구조체에 rx_buffer, tx_buffer, tx_busy_cycles 추가
- 레지스터 오프셋과 LSR 비트 상수 정의

### 체크리스트
- [ ] `use std::collections::VecDeque` 추가
- [ ] `UART_SIZE`를 8로 변경
- [ ] 레지스터 오프셋 상수 정의 (RBR, THR, LSR)
- [ ] LSR 비트 상수 정의 (DR, THRE, TEMT)
- [ ] `TX_CYCLES_PER_BYTE` 상수 정의
- [ ] `Uart` 구조체에 필드 추가
- [ ] `new()`에서 초기화
- [ ] `cargo build` 확인

---

## Step 2: LSR 읽기 구현

### 목표
- 현재 상태를 LSR 비트로 반환

### 로직
```
LSR 읽기:
    status = 0

    if rx_buffer가 비어있지 않으면:
        status |= LSR_DR (0x01)

    if tx_buffer가 비어있으면:
        status |= LSR_THRE (0x20)

    if tx_buffer가 비어있고 tx_busy_cycles == 0이면:
        status |= LSR_TEMT (0x40)

    return status
```

### 체크리스트
- [ ] `read_lsr(&self) -> u8` 메서드 구현
- [ ] 테스트: 초기 상태에서 THRE와 TEMT가 1인지 확인
- [ ] 테스트: rx_buffer에 데이터 넣으면 DR이 1인지 확인
- [ ] `cargo test` 확인

---

## Step 3: RBR 읽기 구현 (수신)

### 목표
- rx_buffer에서 데이터를 꺼내서 반환

### 로직
```
RBR 읽기:
    if rx_buffer가 비어있지 않으면:
        return rx_buffer.pop_front()
    else:
        return 0
```

### 체크리스트
- [ ] `read_rbr(&mut self) -> u8` 메서드 구현
- [ ] 테스트: push 후 read하면 같은 값 나오는지
- [ ] 테스트: 빈 버퍼에서 read하면 0 나오는지
- [ ] `cargo test` 확인

---

## Step 4: THR 쓰기 구현 (송신)

### 목표
- tx_buffer에 데이터를 넣기
- THRE 상태 업데이트

### 로직
```
THR 쓰기:
    tx_buffer.push_back(value)
```

주의: 실제로는 THRE=0일 때 쓰면 안 되지만, 우리는 무한 버퍼로 단순화.

### 체크리스트
- [ ] `write_thr(&mut self, value: u8)` 메서드 구현
- [ ] 테스트: 쓰기 후 tx_buffer에 들어갔는지 확인
- [ ] `cargo test` 확인

---

## Step 5: tick() 구현 (송신 타이밍)

### 목표
- CPU 사이클마다 호출되어 실제 송신 처리
- tx_buffer에서 꺼내서 화면에 출력

### 로직
```
tick():
    if tx_busy_cycles > 0:
        tx_busy_cycles -= 1
        if tx_busy_cycles == 0:
            // 전송 완료, 다음 바이트 준비 가능

    if tx_busy_cycles == 0 and tx_buffer가 비어있지 않으면:
        byte = tx_buffer.pop_front()
        print(byte as char)
        stdout.flush()
        tx_busy_cycles = TX_CYCLES_PER_BYTE
```

### 체크리스트
- [ ] `tick(&mut self)` 메서드 구현
- [ ] 테스트: tick 호출 없이는 출력 안 됨
- [ ] 테스트: 충분한 tick 후 출력됨
- [ ] `cargo test` 확인

---

## Step 6: read8/write8 통합

### 목표
- offset에 따라 적절한 메서드 호출

### 로직
```
read8(offset):
    match offset:
        0 (RBR) => read_rbr()
        5 (LSR) => read_lsr()
        _ => 0

write8(offset, value):
    match offset:
        0 (THR) => write_thr(value)
        _ => 무시
```

### 체크리스트
- [ ] `read8(&mut self, offset: u32) -> u8` 구현
- [ ] `write8(&mut self, offset: u32, value: u8)` 구현
- [ ] `cargo build` 확인

---

## Step 7: 외부 입력 메서드

### 목표
- 외부(키보드)에서 rx_buffer에 데이터 넣기

### 메서드
```
push_input(c: u8):
    rx_buffer.push_back(c)

has_input() -> bool:
    !rx_buffer.is_empty()
```

### 체크리스트
- [ ] `push_input(&mut self, c: u8)` 구현
- [ ] `has_input(&self) -> bool` 구현
- [ ] `cargo test` 확인

---

## Step 8: Bus 연결

### 목표
- Bus에서 UART 호출 시 offset 계산하여 전달
- Bus의 uart 필드를 `&mut`로 접근 가능하게

### 변경
```
// Bus read8
if addr in UART 범위:
    offset = addr - UART_BASE
    self.uart.read8(offset)

// Bus write8
if addr in UART 범위:
    offset = addr - UART_BASE
    self.uart.write8(offset, value)
```

### 체크리스트
- [ ] Bus read8/write8에서 offset 계산
- [ ] `cargo test` 확인

---

## Step 9: CPU에서 tick 호출

### 목표
- CPU step()마다 UART tick() 호출

### 변경
```
// cpu.rs step() 마지막에
self.bus.uart.tick();
```

또는 Bus에 tick() 추가:
```
// bus.rs
pub fn tick(&mut self) {
    self.uart.tick();
}

// cpu.rs
self.bus.tick();
```

### 체크리스트
- [ ] tick 호출 위치 결정 (CPU 또는 Bus)
- [ ] 구현
- [ ] `cargo test` 확인

---

## Step 10: 테스트 프로그램

### 간단한 출력 테스트

```rust
// "Hi!" 출력 - 기존과 동일하게 동작해야 함
let program: Vec<u32> = vec![
    0x100002B7,  // lui t0, 0x10000
    0x04800313,  // addi t1, x0, 'H'
    0x00628023,  // sb t1, 0(t0)
    // ... 생략
];
```

단, tick()이 충분히 호출되어야 실제 출력됨.

### 체크리스트
- [ ] 기존 "Hi!" 프로그램 실행
- [ ] 출력 확인 (tick 때문에 약간 지연될 수 있음)

---

## Step 11: 에코 프로그램

### 어셈블리
```asm
loop:
    lui   t0, 0x10000       # t0 = UART_BASE

wait_input:
    lbu   t1, 5(t0)         # LSR 읽기
    andi  t1, t1, 1         # DR 비트 확인
    beq   t1, zero, wait_input

    lbu   t2, 0(t0)         # RBR에서 문자 읽기

wait_output:
    lbu   t1, 5(t0)         # LSR 읽기
    andi  t1, t1, 0x20      # THRE 비트 확인
    beq   t1, zero, wait_output

    sb    t2, 0(t0)         # THR에 문자 쓰기

    j     loop
```

### 체크리스트
- [ ] 어셈블리 → 기계어 변환
- [ ] main.rs에서 키보드 입력 처리
- [ ] 에코 동작 확인

---

## Step 12: 키보드 입력 처리 (main.rs)

### 방법: 별도 스레드

```rust
use std::thread;
use std::sync::mpsc;
use std::io::{stdin, Read};

fn main() {
    let mut cpu = Cpu::new();
    cpu.load_program(&program);

    let (tx, rx) = mpsc::channel();

    // 입력 스레드
    thread::spawn(move || {
        let stdin = stdin();
        for byte in stdin.lock().bytes() {
            if let Ok(b) = byte {
                tx.send(b).unwrap();
            }
        }
    });

    // 메인 루프
    while !cpu.halted {
        // 입력 확인 (non-blocking)
        if let Ok(c) = rx.try_recv() {
            cpu.bus.uart.push_input(c);
        }
        cpu.step();
    }
}
```

### 체크리스트
- [ ] 스레드와 채널 설정
- [ ] 입력 스레드 구현
- [ ] 메인 루프에서 입력 처리
- [ ] 에코 테스트

---

## 완료 후 확장 아이디어

1. **Raw 터미널 모드**: Enter 없이 즉시 입력 (`termios` 크레이트)
2. **버퍼 오버플로우 처리**: OE 비트 설정
3. **인터럽트**: 데이터 수신 시 인터럽트 발생
4. **FIFO 크기 제한**: 실제 16550은 16바이트 FIFO
5. **Baud rate 설정**: DLL/DLM 레지스터로 속도 조절
