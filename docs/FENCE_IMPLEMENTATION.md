# FENCE 구현 가이드

RISC-V FENCE 명령어 구현 가이드.

**설계 방향**: 멀티코어 기반 구조

---

## 1. FENCE 개요

### 1.1 FENCE란?

메모리 배리어(Memory Barrier) 명령어. 메모리 연산의 순서를 보장.

- **FENCE**: 데이터 메모리 배리어
- **FENCE.I**: 명령어 캐시 동기화 (Zifencei extension)

### 1.2 왜 필요한가?

현대 CPU는 성능 최적화를 위해 메모리 연산 순서를 바꿀 수 있음:

```c
// 코드 순서
data = 42;      // (1)
ready = 1;      // (2)

// 실제 실행 (CPU 최적화)
ready = 1;      // (2) 먼저 실행될 수 있음
data = 42;      // (1)
```

멀티코어에서 다른 코어가 `ready = 1`을 보고 `data`를 읽으면 잘못된 값을 얻을 수 있음.

### 1.3 소속

| 명령어 | 소속 |
|--------|------|
| FENCE | RV32I/RV64I (Base ISA) |
| FENCE.I | Zifencei extension |

---

## 2. 명령어 인코딩

### 2.1 FENCE

```
31    28 27  24 23  20 19   15 14  12 11   7 6      0
[  fm  ][ pred][ succ][ rs1  ][funct3][ rd  ][ opcode]
   4       4      4       5       3      5       7
         IORW   IORW     0       0      0     0001111
```

- **opcode**: `0x0F` (MISC-MEM)
- **funct3**: `0x0` (FENCE)
- **pred**: 이전 연산 종류 (어떤 연산이 완료되어야 하는지)
- **succ**: 이후 연산 종류 (어떤 연산이 기다려야 하는지)
- **fm**: fence mode (TSO용, 일반적으로 0)

### 2.2 pred/succ 비트

| 비트 | 의미 | 설명 |
|------|------|------|
| I (bit 3) | Device Input | 디바이스 읽기 (MMIO) |
| O (bit 2) | Device Output | 디바이스 쓰기 (MMIO) |
| R (bit 1) | Memory Read | 메모리 읽기 |
| W (bit 0) | Memory Write | 메모리 쓰기 |

예시:
- `FENCE RW, RW` (pred=0011, succ=0011): 모든 읽기/쓰기 순서 보장
- `FENCE W, W` (pred=0001, succ=0001): 쓰기 순서만 보장
- `FENCE.TSO` (pred=0011, succ=0011, fm=1000): Total Store Order
- `FENCE W, R` (pred=0001, succ=0010): Store-Load 배리어

### 2.3 FENCE.I

```
31                20 19   15 14  12 11   7 6      0
[       imm        ][ rs1  ][funct3][ rd  ][ opcode]
   000000000000        0       1      0     0001111
```

- **funct3**: `0x1` (FENCE.I)

---

## 3. 멀티코어 아키텍처

### 3.1 Write Buffer

실제 CPU는 쓰기 성능 향상을 위해 Write Buffer 사용:

```
┌─────┐     ┌──────────────┐     ┌────────┐
│ CPU │────▶│ Write Buffer │────▶│ Memory │
└─────┘     └──────────────┘     └────────┘
              (비동기 flush)
```

- CPU가 쓰기 명령 실행 → Write Buffer에 저장 → 나중에 메모리에 반영
- 다른 코어는 Write Buffer 내용을 못 봄 → 순서 문제 발생

### 3.2 Bus 기반 Write Buffer 관리

```
┌─────┐
│ CPU │──┐  ┌─────────────────────────────┐
│ id=0│  │  │            Bus              │
└─────┘  ├─▶│  write_buffers: {           │
┌─────┐  │  │    0: [(addr, val), ...],   │────▶ Memory
│ CPU │──┘  │    1: [(addr, val), ...]    │
│ id=1│     │  }                          │
└─────┘     └─────────────────────────────┘
```

### 3.3 FENCE 동작

```
Hart 0:                         Hart 1:
store data, 42
  → write_buffer[0] += (data, 42)
store ready, 1
  → write_buffer[0] += (ready, 1)
FENCE W, W
  → flush write_buffer[0]
  → Memory[data] = 42
  → Memory[ready] = 1
                                load ready  → 1
                                FENCE R, R
                                load data   → 42 ✓
```

---

## 4. 구현 단계

### Step 1: Decoder 확장

- [ ] `fence_pred(inst)` 함수 추가
- [ ] `fence_succ(inst)` 함수 추가

```rust
// decoder.rs
pub fn fence_pred(inst: u32) -> u32 {
    (inst >> 24) & 0xF
}

pub fn fence_succ(inst: u32) -> u32 {
    (inst >> 20) & 0xF
}
```

---

### Step 2: Bus에 Write Buffer 추가

- [ ] `WriteBufferEntry` 구조체 추가
- [ ] Bus에 `write_buffers: HashMap<u64, Vec<WriteBufferEntry>>` 추가
- [ ] `fence()` API 추가
- [ ] `flush_write_buffer()` API 추가

```rust
// bus.rs
use std::collections::HashMap;

pub struct WriteBufferEntry {
    pub addr: u64,
    pub value: u64,
    pub size: u8,  // 1, 2, 4, 8 bytes
}

pub struct Bus {
    // 기존 필드들...
    clint: devices::Clint,
    memory: devices::Memory,
    uart: devices::Uart,
    reservations: HashMap<u64, u64>,
    write_buffers: HashMap<u64, Vec<WriteBufferEntry>>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            // 기존...
            write_buffers: HashMap::new(),
        }
    }

    pub fn fence(&mut self, hart_id: u64, pred: u32, _succ: u32) {
        const FENCE_W: u32 = 0b0001;

        // pred에 W가 있으면 Write Buffer flush
        if pred & FENCE_W != 0 {
            self.flush_write_buffer(hart_id);
        }
    }

    pub fn flush_write_buffer(&mut self, hart_id: u64) {
        if let Some(buffer) = self.write_buffers.remove(&hart_id) {
            for entry in buffer {
                match entry.size {
                    1 => self.write8(entry.addr, entry.value as u8),
                    2 => self.write16(entry.addr, entry.value as u16),
                    4 => self.write32(entry.addr, entry.value as u32),
                    8 => self.write64(entry.addr, entry.value),
                    _ => unreachable!(),
                }
            }
        }
    }
}
```

**참고**: 현재 싱글코어에서는 write_buffers가 항상 비어있으므로 flush해도 아무 일 없음.

---

### Step 3: CPU 핸들러 추가

- [ ] `const MISC_MEM: u32 = 0x0F;` 추가
- [ ] `step()`에 MISC_MEM 핸들러 추가
- [ ] `execute_misc_mem()` 함수 구현

```rust
// cpu.rs
const MISC_MEM: u32 = 0x0F;

// step() 내부
MISC_MEM => {
    self.execute_misc_mem(inst);
}

fn execute_misc_mem(&mut self, inst: u32) {
    let funct3 = decoder::funct3(inst);

    match funct3 {
        0x0 => {
            // FENCE
            let pred = decoder::fence_pred(inst);
            let succ = decoder::fence_succ(inst);
            debug_log!("FENCE pred={:#x}, succ={:#x}", pred, succ);
            self.bus.fence(self.hart_id, pred, succ);
        }
        0x1 => {
            // FENCE.I - 명령어 캐시 동기화
            debug_log!("FENCE.I");
            // 캐시 없으므로 NOP
        }
        _ => panic!("Unknown MISC_MEM funct3: {:#x}", funct3)
    }
}
```

---

### Step 4: 테스트

- [ ] FENCE 기본 테스트 (NOP 동작 확인)
- [ ] FENCE.I 테스트
- [ ] xv6 테스트

---

## 5. 멀티코어 확장 시 추가 작업

현재 구현은 write를 즉시 메모리에 반영하므로 write_buffers가 항상 비어있음.

멀티코어로 확장 시:

### 5.1 Buffered Write 추가

```rust
impl Bus {
    pub fn write32_buffered(&mut self, hart_id: u64, addr: u64, value: u32) {
        self.write_buffers
            .entry(hart_id)
            .or_insert_with(Vec::new)
            .push(WriteBufferEntry {
                addr,
                value: value as u64,
                size: 4,
            });
    }
}
```

### 5.2 CPU에서 buffered write 사용

```rust
// 멀티코어 시 store 명령어에서
self.bus.write32_buffered(self.hart_id, addr, value);
// 대신
// self.bus.write32(addr, value);
```

---

## 6. xv6에서의 사용

### 6.1 spinlock release

```c
void release(struct spinlock *lk) {
    __sync_synchronize();  // → FENCE RW, RW
    lk->locked = 0;
}
```

### 6.2 spinlock acquire

```c
void acquire(struct spinlock *lk) {
    while(__sync_lock_test_and_set(&lk->locked, 1) != 0)
        ;
    __sync_synchronize();  // → FENCE RW, RW
}
```

---

## 7. 체크리스트

### Step 1: Decoder 확장
- [ ] `fence_pred()` 추가
- [ ] `fence_succ()` 추가

### Step 2: Bus 확장
- [ ] `WriteBufferEntry` 구조체
- [ ] `write_buffers` 필드
- [ ] `fence()` API
- [ ] `flush_write_buffer()` API

### Step 3: CPU 핸들러
- [ ] MISC_MEM opcode 상수
- [ ] `execute_misc_mem()` 함수
- [ ] FENCE 처리
- [ ] FENCE.I 처리

### Step 4: 테스트
- [ ] FENCE 테스트
- [ ] xv6 테스트

---

## 8. 참고 자료

- RISC-V Specification Chapter 2.7: "Memory Ordering Instructions"
- RISC-V Specification Chapter 8: "Zifencei" Extension
- RISC-V Memory Consistency Model (RVWMO)
