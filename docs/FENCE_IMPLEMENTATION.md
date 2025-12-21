# FENCE 구현 가이드

RISC-V FENCE 명령어 구현 가이드.

**설계 방향**: 멀티코어 확장 가능한 구조

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

## 3. 멀티코어 확장 가능 아키텍처

### 3.1 설계 원칙

A Extension과 동일하게, 싱글코어로 시작하되 멀티코어 확장이 용이한 구조로 설계.

**핵심**: Write Buffer를 Bus에서 관리

### 3.2 Write Buffer란?

실제 CPU는 쓰기 성능 향상을 위해 Write Buffer 사용:

```
┌─────┐     ┌──────────────┐     ┌────────┐
│ CPU │────▶│ Write Buffer │────▶│ Memory │
└─────┘     └──────────────┘     └────────┘
              (비동기 flush)
```

- CPU가 쓰기 명령 실행 → Write Buffer에 저장 → 나중에 메모리에 반영
- 다른 코어는 Write Buffer 내용을 못 봄 → 순서 문제 발생

### 3.3 멀티코어 구조

```
싱글코어 (현재):
┌─────┐     ┌─────────────────────┐
│ CPU │────▶│        Bus          │
│ id=0│     │  write_buffers: {}  │────▶ Memory
└─────┘     └─────────────────────┘

멀티코어 확장 시:
┌─────┐
│ CPU │──┐  ┌─────────────────────────────┐
│ id=0│  │  │            Bus              │
└─────┘  ├─▶│  write_buffers: {           │
┌─────┐  │  │    0: [(addr, val), ...],   │────▶ Memory
│ CPU │──┘  │    1: [(addr, val), ...]    │
│ id=1│     │  }                          │
└─────┘     └─────────────────────────────┘
```

### 3.4 FENCE 동작

```
FENCE W, R 실행 시:
1. 해당 hart의 Write Buffer를 Memory에 flush
2. 다른 hart들이 변경 내용을 볼 수 있게 됨
```

**예시**:
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

### Step 0: 아키텍처 준비 (멀티코어 확장용)

**목표**: Write Buffer 인프라 구축

싱글코어에서는 즉시 메모리에 쓰므로 Write Buffer가 비어있음.
멀티코어 확장 시 실제 버퍼링 로직 추가.

- [ ] Bus에 Write Buffer 타입 정의
- [ ] Bus에 `fence()` API 추가 (현재는 NOP)

```rust
// bus.rs
use std::collections::HashMap;

// Write Buffer entry
pub struct WriteBufferEntry {
    pub addr: u64,
    pub value: u64,
    pub size: u8,  // 1, 2, 4, 8 bytes
}

pub struct Bus {
    // 기존 필드들...
    reservations: HashMap<u64, u64>,

    // 멀티코어용 Write Buffer (현재는 사용 안 함)
    write_buffers: HashMap<u64, Vec<WriteBufferEntry>>,
}

impl Bus {
    // FENCE 처리
    pub fn fence(&mut self, hart_id: u64, pred: u32, succ: u32) {
        // 싱글코어: 즉시 메모리 반영이므로 NOP
        // 멀티코어: write_buffers[hart_id] flush

        let _ = (hart_id, pred, succ);  // 현재는 사용 안 함

        // TODO: 멀티코어 시 구현
        // if pred & FENCE_W != 0 {
        //     self.flush_write_buffer(hart_id);
        // }
    }
}
```

**검증**: 기존 테스트 통과 확인

---

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

### Step 2: MISC_MEM 핸들러 추가

- [ ] `const MISC_MEM: u32 = 0x0F;` 추가
- [ ] `step()`에 MISC_MEM 핸들러 추가
- [ ] FENCE (funct3=0x0) 처리
- [ ] FENCE.I (funct3=0x1) 처리

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

### Step 3: 테스트

- [ ] FENCE 기본 테스트 (NOP으로 동작 확인)
- [ ] FENCE.I 테스트
- [ ] xv6 테스트

---

## 5. 멀티코어 확장 시 추가 구현

### 5.1 Write Buffer 활성화

```rust
impl Bus {
    pub fn write32_buffered(&mut self, hart_id: u64, addr: u64, value: u32) {
        // Write Buffer에 추가
        self.write_buffers
            .entry(hart_id)
            .or_insert_with(Vec::new)
            .push(WriteBufferEntry {
                addr,
                value: value as u64,
                size: 4,
            });
    }

    pub fn flush_write_buffer(&mut self, hart_id: u64) {
        if let Some(buffer) = self.write_buffers.remove(&hart_id) {
            for entry in buffer {
                match entry.size {
                    1 => self.write8_direct(entry.addr, entry.value as u8),
                    2 => self.write16_direct(entry.addr, entry.value as u16),
                    4 => self.write32_direct(entry.addr, entry.value as u32),
                    8 => self.write64_direct(entry.addr, entry.value),
                    _ => unreachable!(),
                }
            }
        }
    }
}
```

### 5.2 FENCE 구현 완성

```rust
pub fn fence(&mut self, hart_id: u64, pred: u32, succ: u32) {
    const FENCE_W: u32 = 0b0001;

    // pred에 W가 있으면 Write Buffer flush
    if pred & FENCE_W != 0 {
        self.flush_write_buffer(hart_id);
    }

    // succ에 R이 있으면 다른 hart의 Write Buffer도 flush 필요
    // (실제 HW에서는 cache invalidation)
}
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

FENCE가 없으면 `lk->locked = 0`이 critical section 내 쓰기보다 먼저 보일 수 있음.

### 6.2 spinlock acquire

```c
void acquire(struct spinlock *lk) {
    while(__sync_lock_test_and_set(&lk->locked, 1) != 0)
        ;
    __sync_synchronize();  // → FENCE RW, RW
}
```

FENCE가 critical section 진입 전에 이전 코어의 쓰기가 보이도록 보장.

---

## 7. 싱글코어에서 NOP인 이유

싱글코어 에뮬레이터는:
1. **순차 실행**: 명령어가 순서대로 실행됨
2. **즉시 반영**: 메모리 쓰기가 즉시 보임 (Write Buffer 없음)
3. **캐시 없음**: 캐시 일관성 문제 없음

따라서 메모리 순서가 항상 보장되어 FENCE가 NOP이어도 정확함.

---

## 8. 체크리스트

### Step 0: 아키텍처 준비
- [ ] Bus에 `fence()` API 추가

### Step 1: Decoder 확장
- [ ] `fence_pred()` 추가
- [ ] `fence_succ()` 추가

### Step 2: CPU 핸들러
- [ ] MISC_MEM opcode 상수
- [ ] `execute_misc_mem()` 함수
- [ ] FENCE 처리
- [ ] FENCE.I 처리

### Step 3: 테스트
- [ ] FENCE 테스트
- [ ] xv6 테스트

---

## 9. 참고 자료

- RISC-V Specification Chapter 2.7: "Memory Ordering Instructions"
- RISC-V Specification Chapter 8: "Zifencei" Extension
- RISC-V Memory Consistency Model (RVWMO)
