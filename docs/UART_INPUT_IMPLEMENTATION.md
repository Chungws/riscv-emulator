# UART 입력 구현

UART에 키보드 입력 기능을 추가하여 에코 프로그램을 실행할 수 있게 한다.

---

## 배경 지식

### UART 16550 레지스터

실제 UART 칩은 여러 레지스터를 가진다:

| 오프셋 | 읽기 | 쓰기 | 설명 |
|--------|------|------|------|
| 0 | RBR (Receive Buffer) | THR (Transmit Holding) | 데이터 송수신 |
| 5 | LSR (Line Status) | - | 상태 확인 |

### LSR (Line Status Register) 비트

| 비트 | 이름 | 설명 |
|------|------|------|
| 0 | Data Ready | 1이면 수신 데이터 있음 |
| 5 | THR Empty | 1이면 송신 가능 |

### 폴링 방식 입력

```c
// 데이터가 올 때까지 대기
while ((read(UART_BASE + 5) & 1) == 0) {
    // 폴링
}
// 데이터 읽기
char c = read(UART_BASE + 0);
```

---

## Step 1: UART 구조체 수정

### 목표
- 수신 버퍼 추가
- UART_SIZE 확장

### 변경 사항

```rust
use std::collections::VecDeque;

pub const UART_SIZE: u32 = 8;  // 1 → 8

pub struct Uart {
    rx_buffer: VecDeque<u8>,
}

impl Uart {
    pub fn new() -> Self {
        Uart {
            rx_buffer: VecDeque::new(),
        }
    }
}
```

### 체크리스트
- [ ] `use std::collections::VecDeque` 추가
- [ ] `UART_SIZE`를 8로 변경
- [ ] `Uart` 구조체에 `rx_buffer` 필드 추가
- [ ] `new()`에서 빈 VecDeque 초기화
- [ ] `cargo build` 확인

---

## Step 2: 레지스터 상수 정의

### 목표
- 레지스터 오프셋 상수 정의
- LSR 비트 상수 정의

### 변경 사항

```rust
// 레지스터 오프셋
const UART_RBR: u32 = 0;  // Receive Buffer Register
const UART_THR: u32 = 0;  // Transmit Holding Register
const UART_LSR: u32 = 5;  // Line Status Register

// LSR 비트
const LSR_DATA_READY: u8 = 1 << 0;
const LSR_THR_EMPTY: u8 = 1 << 5;
```

### 체크리스트
- [ ] 레지스터 오프셋 상수 추가
- [ ] LSR 비트 상수 추가
- [ ] `cargo build` 확인

---

## Step 3: read8/write8에 offset 파라미터 추가

### 목표
- 메서드 시그니처 변경
- offset에 따른 동작 분기

### UART 변경

```rust
pub fn read8(&mut self, offset: u32) -> u8 {
    match offset {
        UART_RBR => self.rx_buffer.pop_front().unwrap_or(0),
        UART_LSR => {
            let mut status = LSR_THR_EMPTY;
            if !self.rx_buffer.is_empty() {
                status |= LSR_DATA_READY;
            }
            status
        }
        _ => 0,
    }
}

pub fn write8(&mut self, offset: u32, value: u8) {
    if offset == UART_THR {
        print!("{}", value as char);
        stdout().flush().unwrap();
    }
}
```

### 주의
- `read8`이 `&self` → `&mut self`로 변경됨 (버퍼에서 pop하므로)

### 체크리스트
- [ ] `read8` 시그니처 변경 및 구현
- [ ] `write8` 시그니처 변경 및 구현
- [ ] `cargo build` 확인 (Bus에서 에러 날 것임 - 다음 스텝에서 수정)

---

## Step 4: Bus에서 UART offset 처리

### 목표
- UART 주소 범위 내에서 offset 계산
- UART 메서드 호출 시 offset 전달

### 변경 사항

```rust
// Bus read8
if addr >= UART_BASE && addr < UART_BASE + UART_SIZE {
    let offset = addr - UART_BASE;
    self.uart.read8(offset)
}

// Bus write8
if addr >= UART_BASE && addr < UART_BASE + UART_SIZE {
    let offset = addr - UART_BASE;
    self.uart.write8(offset, value)
}
```

### 체크리스트
- [ ] `read8`에서 offset 계산하여 전달
- [ ] `write8`에서 offset 계산하여 전달
- [ ] `cargo test` 통과 확인

---

## Step 5: 입력 메서드 추가

### 목표
- 외부에서 UART 버퍼에 입력을 넣을 수 있게 함

### 변경 사항

```rust
impl Uart {
    /// 키보드 입력을 버퍼에 추가
    pub fn push_input(&mut self, c: u8) {
        self.rx_buffer.push_back(c);
    }

    /// 입력 데이터가 있는지 확인
    pub fn has_input(&self) -> bool {
        !self.rx_buffer.is_empty()
    }
}
```

### 체크리스트
- [ ] `push_input` 메서드 추가
- [ ] `has_input` 메서드 추가
- [ ] 테스트 코드 작성
- [ ] `cargo test` 통과 확인

---

## Step 6: 에코 프로그램 작성

### 목표
- 입력받은 문자를 그대로 출력하는 프로그램

### 어셈블리

```asm
# 에코 프로그램
# UART_BASE = 0x10000000
# LSR = UART_BASE + 5
# 입력 받아서 그대로 출력

loop:
    lui  t0, 0x10000      # t0 = UART_BASE

wait_input:
    lbu  t1, 5(t0)        # t1 = LSR 읽기
    andi t1, t1, 1        # Data Ready 비트 확인
    beq  t1, zero, wait_input  # 데이터 없으면 대기

    lbu  t2, 0(t0)        # t2 = 데이터 읽기
    sb   t2, 0(t0)        # 에코 출력

    j    loop             # 반복
```

### 기계어 (직접 변환해보기)

```rust
let program: Vec<u32> = vec![
    // loop:
    0x100002B7,  // lui t0, 0x10000
    // wait_input:
    0x0052C303,  // lbu t1, 5(t0)
    0x0013F313,  // andi t1, t1, 1
    0xFE030CE3,  // beq t1, zero, wait_input (-8)
    0x0002C383,  // lbu t2, 0(t0)
    0x00728023,  // sb t2, 0(t0)
    0xFE9FF06F,  // j loop (-24)
];
```

### 체크리스트
- [ ] 어셈블리 → 기계어 변환
- [ ] main.rs에 프로그램 추가
- [ ] `cargo run` 실행
- [ ] 키보드 입력이 에코되는지 확인

---

## Step 7: 메인 루프에서 키보드 입력 처리

### 목표
- 터미널에서 키보드 입력을 받아 UART로 전달
- non-blocking 입력 처리

### 방법 1: 간단한 동기 방식

```rust
use std::io::{stdin, Read};

fn main() {
    let mut cpu = Cpu::new();
    cpu.load_program(&program);

    // stdin을 raw mode로 설정 필요
    let mut buffer = [0u8; 1];
    loop {
        // 입력 확인
        if let Ok(_) = stdin().read(&mut buffer) {
            cpu.bus.uart.push_input(buffer[0]);
        }

        // CPU 실행
        if !cpu.halted {
            cpu.step();
        }
    }
}
```

### 방법 2: 별도 스레드 사용

```rust
use std::thread;
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();

    // 입력 스레드
    thread::spawn(move || {
        let mut buffer = [0u8; 1];
        loop {
            stdin().read_exact(&mut buffer).unwrap();
            tx.send(buffer[0]).unwrap();
        }
    });

    // 메인 루프
    loop {
        // 입력 확인 (non-blocking)
        if let Ok(c) = rx.try_recv() {
            cpu.bus.uart.push_input(c);
        }

        if !cpu.halted {
            cpu.step();
        }
    }
}
```

### 참고: Raw 터미널 모드
기본 터미널은 Enter를 눌러야 입력이 전달됨.
즉시 입력을 받으려면 `termios` 크레이트로 raw mode 설정 필요.

### 체크리스트
- [ ] 입력 처리 방식 선택 (동기/스레드)
- [ ] main.rs 수정
- [ ] 에코 테스트

---

## 완료 후 확장 아이디어

1. **Raw 터미널 모드**: `termios` 크레이트로 즉시 입력
2. **특수 키 처리**: Ctrl+C로 종료
3. **입력 버퍼 크기 제한**: 오버플로우 방지
4. **인터럽트 기반 입력**: 폴링 대신 인터럽트 사용
