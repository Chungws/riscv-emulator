# Bus 아키텍처 구현 가이드

이 문서는 Bus 아키텍처를 구현하여 여러 장치를 에뮬레이션하는 가이드입니다.

---

## 개요

### 현재 구조
```
CPU → Memory (DRAM만)
```

### 목표 구조
```
CPU → Bus → Memory (DRAM: 0x80000000~)
         → UART   (0x10000000)
         → (확장 가능: Timer, GPIO, etc.)
```

### 메모리 맵
| 주소 범위 | 장치 | 크기 |
|-----------|------|------|
| 0x10000000 | UART | 1 byte |
| 0x80000000 ~ 0x87FFFFFF | DRAM | 128MB |

---

## Step 1: 폴더 구조 변경

### 목표
devices 폴더를 만들어 장치들을 모듈화합니다.

### 작업 내용
1. `src/devices/` 폴더 생성
2. `src/memory.rs` → `src/devices/memory.rs` 이동
3. `src/devices/mod.rs` 생성

### 파일 구조
```
src/
  lib.rs
  cpu.rs
  bus.rs          ← 새로 생성
  decoder.rs
  devices/
    mod.rs        ← 새로 생성
    memory.rs     ← 이동
    uart.rs       ← 새로 생성
```

### devices/mod.rs
```rust
pub mod memory;
pub mod uart;

pub use memory::Memory;
pub use uart::Uart;
```

### lib.rs 수정
```rust
pub mod cpu;
pub mod decoder;
pub mod bus;
pub mod devices;

pub use cpu::Cpu;
// ...
```

### 체크리스트
- [x] `src/devices/` 폴더 생성
- [x] `memory.rs` 이동
- [x] `devices/mod.rs` 작성
- [x] `lib.rs` 수정
- [x] `cargo test` 통과 확인

---

## Step 2: UART 구현

### 목표
문자 출력이 가능한 UART 장치를 구현합니다.

### 구현 내용
- UART_BASE: 0x10000000
- write8: 문자 출력 (print!)
- read8: 0 반환 (또는 상태)

### devices/uart.rs
```rust
use std::io::{Write, stdout};

pub const UART_BASE: u32 = 0x10000000;
pub const UART_SIZE: u32 = 1;

pub struct Uart;

impl Uart {
    pub fn new() -> Self {
        Uart
    }

    pub fn read8(&self, _addr: u32) -> u8 {
        0 // 입력 미구현
    }

    pub fn write8(&mut self, _addr: u32, value: u8) {
        print!("{}", value as char);
        stdout().flush().unwrap();
    }
}
```

### 테스트
```rust
#[test]
fn test_uart_write() {
    let mut uart = Uart::new();
    uart.write8(0, b'H');
    uart.write8(0, b'i');
    // 출력: "Hi"
}
```

### 체크리스트
- [x] `devices/uart.rs` 생성
- [x] `Uart` 구조체 구현
- [x] `devices/mod.rs`에 추가
- [x] 테스트 통과

---

## Step 3: Bus 구현

### 목표
주소에 따라 적절한 장치로 라우팅하는 Bus를 구현합니다.

### 구현 내용
- Memory와 Uart 소유
- 주소 범위 체크하여 라우팅
- read8/16/32, write8/16/32 메서드

### bus.rs
```rust
use crate::devices::{Memory, Uart};
use crate::devices::uart::{UART_BASE, UART_SIZE};
use crate::DRAM_BASE;

pub struct Bus {
    memory: Memory,
    uart: Uart,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            uart: Uart::new(),
        }
    }

    pub fn read8(&self, addr: u32) -> u8 {
        if addr >= UART_BASE && addr < UART_BASE + UART_SIZE {
            self.uart.read8(addr - UART_BASE)
        } else if addr >= DRAM_BASE {
            self.memory.read8(addr)
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    pub fn write8(&mut self, addr: u32, value: u8) {
        if addr >= UART_BASE && addr < UART_BASE + UART_SIZE {
            self.uart.write8(addr - UART_BASE, value);
        } else if addr >= DRAM_BASE {
            self.memory.write8(addr, value);
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    }

    // read16, read32, write16, write32도 유사하게 구현
}
```

### 체크리스트
- [x] `bus.rs` 생성
- [x] Bus 구조체 정의
- [x] 주소 라우팅 로직 구현
- [x] read/write 메서드 구현
- [x] 테스트 통과

---

## Step 4: CPU 연결 변경

### 목표
CPU가 Memory 대신 Bus를 사용하도록 변경합니다.

### 변경 내용

#### cpu.rs (Before)
```rust
pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
    pub memory: Memory,
    pub halted: bool,
}
```

#### cpu.rs (After)
```rust
pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
    pub bus: Bus,
    pub halted: bool,
}
```

#### 메서드 변경
- `self.memory.read32(...)` → `self.bus.read32(...)`
- `self.memory.write32(...)` → `self.bus.write32(...)`

### 체크리스트
- [x] Cpu 구조체 수정 (memory → bus)
- [x] 모든 memory 참조를 bus로 변경
- [x] `cargo test` 통과 확인

---

## Step 5: Hello World 테스트

### 목표
UART로 문자열을 출력하는 프로그램을 실행합니다.

### 어셈블리 프로그램
```asm
# "Hi!\n" 출력
# UART_BASE = 0x10000000

    lui  x1, 0x10000      # x1 = 0x10000000 (UART 주소)
    addi x2, x0, 'H'      # x2 = 'H' (72)
    sb   x2, 0(x1)        # UART에 'H' 출력
    addi x2, x0, 'i'      # x2 = 'i' (105)
    sb   x2, 0(x1)        # UART에 'i' 출력
    addi x2, x0, '!'      # x2 = '!' (33)
    sb   x2, 0(x1)        # UART에 '!' 출력
    addi x2, x0, 10       # x2 = '\n' (10)
    sb   x2, 0(x1)        # UART에 '\n' 출력
    ecall                 # 종료
```

### 기계어
```
0x100000B7  // lui x1, 0x10000
0x04800113  // addi x2, x0, 72 ('H')
0x00208023  // sb x2, 0(x1)
0x06900113  // addi x2, x0, 105 ('i')
0x00208023  // sb x2, 0(x1)
0x02100113  // addi x2, x0, 33 ('!')
0x00208023  // sb x2, 0(x1)
0x00A00113  // addi x2, x0, 10 ('\n')
0x00208023  // sb x2, 0(x1)
0x00000073  // ecall
```

### main.rs
```rust
fn main() {
    let mut cpu = Cpu::new();
    let program: Vec<u32> = vec![
        0x100000B7,  // lui x1, 0x10000
        0x04800113,  // addi x2, x0, 'H'
        0x00208023,  // sb x2, 0(x1)
        // ... 나머지
        0x00000073,  // ecall
    ];
    cpu.load_program(&program);
    cpu.run();
}
```

### 기대 출력
```
Hi!
```

### 체크리스트
- [x] 프로그램 기계어 변환
- [x] main.rs 수정
- [x] `cargo run` 실행
- [x] "Hi!" 출력 확인

---

## 완료 후 확장 아이디어

- [ ] UART 입력 지원 (키보드)
- [ ] Timer 장치 추가
- [ ] 인터럽트 지원
- [ ] VirtIO 장치

---

## 참고: 메모리 맵 확장 예시

나중에 장치를 추가할 때:

| 주소 범위 | 장치 |
|-----------|------|
| 0x00001000 ~ 0x00001FFF | Boot ROM |
| 0x02000000 ~ 0x0200FFFF | CLINT (Timer) |
| 0x0C000000 ~ 0x0FFFFFFF | PLIC (Interrupt) |
| 0x10000000 ~ 0x10000FFF | UART |
| 0x80000000 ~ | DRAM |

이 맵은 QEMU virt 머신 기준입니다.
