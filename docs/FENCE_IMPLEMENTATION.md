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

| 비트 | 의미 |
|------|------|
| I (bit 3) | Device Input |
| O (bit 2) | Device Output |
| R (bit 1) | Memory Read |
| W (bit 0) | Memory Write |

예시:
- `FENCE RW, RW` (pred=0011, succ=0011): 모든 읽기/쓰기 순서 보장
- `FENCE W, W` (pred=0001, succ=0001): 쓰기 순서만 보장
- `FENCE.TSO` (pred=0011, succ=0011, fm=1000): Total Store Order

### 2.3 FENCE.I

```
31                20 19   15 14  12 11   7 6      0
[       imm        ][ rs1  ][funct3][ rd  ][ opcode]
   000000000000        0       1      0     0001111
```

- **funct3**: `0x1` (FENCE.I)

---

## 3. 구현 단계

### Step 1: Decoder 확장

- [ ] `fence_pred(inst)` 함수 추가
- [ ] `fence_succ(inst)` 함수 추가

```
pub fn fence_pred(inst: u32) -> u32 {
    (inst >> 24) & 0xF
}

pub fn fence_succ(inst: u32) -> u32 {
    (inst >> 20) & 0xF
}
```

### Step 2: MISC_MEM 핸들러 추가

- [ ] `const MISC_MEM: u32 = 0x0F;` 추가
- [ ] `step()`에 MISC_MEM 핸들러 추가
- [ ] FENCE (funct3=0x0) 처리
- [ ] FENCE.I (funct3=0x1) 처리

```
MISC_MEM => {
    let funct3 = decoder::funct3(inst);
    match funct3 {
        0x0 => {
            // FENCE
            let _pred = decoder::fence_pred(inst);
            let _succ = decoder::fence_succ(inst);
            debug_log!("FENCE pred={:#x}, succ={:#x}", _pred, _succ);
            // 싱글코어: NOP
            // TODO: 멀티코어 시 memory_barrier 구현
        }
        0x1 => {
            // FENCE.I
            debug_log!("FENCE.I");
            // 캐시 없으므로 NOP
        }
        _ => panic!("Unknown MISC_MEM funct3: {:#x}", funct3)
    }
}
```

### Step 3: 테스트

- [ ] FENCE 기본 테스트 (NOP으로 동작 확인)
- [ ] FENCE.I 테스트

---

## 4. 멀티코어 확장 시

### 4.1 Bus에 memory_barrier 추가

```
impl Bus {
    pub fn memory_barrier(&mut self, hart_id: u64, pred: u32, succ: u32) {
        // 해당 hart의 pending 메모리 연산을 flush
        // pred 종류의 연산이 완료될 때까지 succ 종류 연산 대기
    }
}
```

### 4.2 FENCE 호출 수정

```
0x0 => {
    let pred = decoder::fence_pred(inst);
    let succ = decoder::fence_succ(inst);
    self.bus.memory_barrier(self.hart_id, pred, succ);
}
```

---

## 5. xv6에서의 사용

### 5.1 spinlock release

```c
void release(struct spinlock *lk) {
    __sync_synchronize();  // → FENCE RW, RW
    lk->locked = 0;
}
```

FENCE가 없으면 `lk->locked = 0`이 critical section 내 쓰기보다 먼저 보일 수 있음.

### 5.2 spinlock acquire

```c
void acquire(struct spinlock *lk) {
    while(__sync_lock_test_and_set(&lk->locked, 1) != 0)
        ;
    __sync_synchronize();  // → FENCE RW, RW
}
```

FENCE가 critical section 진입 전에 이전 코어의 쓰기가 보이도록 보장.

---

## 6. 싱글코어에서 NOP인 이유

싱글코어 에뮬레이터는:
1. **순차 실행**: 명령어가 순서대로 실행됨
2. **즉시 반영**: 메모리 쓰기가 즉시 보임
3. **캐시 없음**: 캐시 일관성 문제 없음

따라서 메모리 순서가 항상 보장되어 FENCE가 NOP이어도 정확함.

---

## 7. 참고 자료

- RISC-V Specification Chapter 2.7: "Memory Ordering Instructions"
- RISC-V Specification Chapter 8: "Zifencei" Extension
