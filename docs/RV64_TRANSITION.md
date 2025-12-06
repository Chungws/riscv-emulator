# RV32I → RV64I 전환 가이드

에뮬레이터를 32비트에서 64비트 RISC-V로 전환한다.

---

## RV32 vs RV64 차이점

| 항목 | RV32I | RV64I |
|------|-------|-------|
| 레지스터 크기 | 32비트 | 64비트 |
| 주소 공간 | 4GB | 16EB (이론상) |
| 기본 연산 | 32비트 | 64비트 |
| XLEN | 32 | 64 |

---

## 새로 추가되는 명령어

RV64I는 RV32I의 모든 명령어 + 64비트 전용 명령어:

### 로드/스토어

| 명령어 | 설명 |
|--------|------|
| LD | Load Doubleword (64비트 로드) |
| SD | Store Doubleword (64비트 스토어) |
| LWU | Load Word Unsigned (32비트 → 64비트 제로 확장) |

### 32비트 연산 (W 접미사)

64비트 레지스터에서 하위 32비트만 연산 후 부호 확장:

| 명령어 | 설명 |
|--------|------|
| ADDIW | Add Immediate Word |
| SLLIW | Shift Left Logical Immediate Word |
| SRLIW | Shift Right Logical Immediate Word |
| SRAIW | Shift Right Arithmetic Immediate Word |
| ADDW | Add Word |
| SUBW | Subtract Word |
| SLLW | Shift Left Logical Word |
| SRLW | Shift Right Logical Word |
| SRAW | Shift Right Arithmetic Word |

### 시프트 변경

- RV32: shamt = 5비트 (0-31)
- RV64: shamt = 6비트 (0-63)

---

## 전환 단계

### Step 1: 타입 변경

**목표:** 모든 32비트 타입을 64비트로 변경

**cpu.rs:**
```rust
// Before
pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
    // ...
}

// After
pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    // ...
}
```

**memory.rs:**
```rust
// Before
pub const DRAM_BASE: u32 = 0x80000000;

// After
pub const DRAM_BASE: u64 = 0x80000000;
```

**체크리스트:**
- [x] `cpu.rs`: regs `[u32; 32]` → `[u64; 32]`
- [x] `cpu.rs`: pc `u32` → `u64`
- [x] `devices/memory.rs`: DRAM_BASE, DRAM_SIZE `u32` → `u64`
- [x] `devices/uart.rs`: UART_BASE, UART_SIZE `u32` → `u64`
- [x] `bus.rs`: 주소 파라미터 `u32` → `u64`
- [x] `cargo build` (많은 에러 예상)

---

### Step 2: 메모리 인터페이스 확장

**목표:** 64비트 읽기/쓰기 추가

**memory.rs:**
```rust
pub fn read64(&self, addr: u64) -> u64 {
    let index = (addr - DRAM_BASE) as usize;
    u64::from_le_bytes([
        self.data[index],
        self.data[index + 1],
        self.data[index + 2],
        self.data[index + 3],
        self.data[index + 4],
        self.data[index + 5],
        self.data[index + 6],
        self.data[index + 7],
    ])
}

pub fn write64(&mut self, addr: u64, value: u64) {
    let index = (addr - DRAM_BASE) as usize;
    let bytes = value.to_le_bytes();
    for i in 0..8 {
        self.data[index + i] = bytes[i];
    }
}
```

**bus.rs:**
```rust
pub fn read64(&self, addr: u64) -> u64 { ... }
pub fn write64(&mut self, addr: u64, value: u64) { ... }
```

**체크리스트:**
- [x] `memory.rs`: read64, write64 추가
- [x] `bus.rs`: read64, write64 추가
- [x] 기존 read8/16/32, write8/16/32의 addr 타입 변경
- [x] `cargo build` 확인

---

### Step 3: 기존 명령어 수정

**목표:** 32비트 연산을 64비트로 변경

**주요 변경:**

1. **즉시값 부호 확장**
```rust
// Before (32비트)
let imm = decoder::imm_i(inst);  // i32
self.write_reg(rd, rs1_val.wrapping_add(imm as u32));

// After (64비트)
let imm = decoder::imm_i(inst) as i64;  // i32 → i64 부호 확장
self.write_reg(rd, rs1_val.wrapping_add(imm as u64));
```

2. **시프트 연산**
```rust
// Before
let shamt = (imm as u32) & 0x1F;  // 5비트

// After
let shamt = (imm as u64) & 0x3F;  // 6비트
```

3. **LW (Load Word)**
```rust
// Before
let val = self.bus.read32(addr);

// After - 부호 확장 필요
let val = self.bus.read32(addr) as i32 as i64 as u64;
```

**체크리스트:**
- [x] 모든 즉시값 부호 확장 검토
- [x] 시프트 연산 shamt 비트 수정 (5→6비트)
- [x] LW: 32비트 → 64비트 부호 확장
- [x] LH/LB: 부호 확장 검토
- [x] `cargo build` 확인

---

### Step 4: 새 명령어 추가 - LD/SD

**목표:** 64비트 로드/스토어 구현

**인코딩:**
```
LD:  funct3 = 011, opcode = 0000011 (LOAD)
SD:  funct3 = 011, opcode = 0100011 (STORE)
LWU: funct3 = 110, opcode = 0000011 (LOAD)
```

**구현:**
```rust
// LOAD (opcode = 0x03)
0x3 => {  // LD
    let val = self.bus.read64(addr);
    self.write_reg(rd, val);
}
0x6 => {  // LWU
    let val = self.bus.read32(addr) as u64;  // 제로 확장
    self.write_reg(rd, val);
}

// STORE (opcode = 0x23)
0x3 => {  // SD
    self.bus.write64(addr, rs2_val);
}
```

**체크리스트:**
- [x] LD 구현
- [x] SD 구현
- [x] LWU 구현
- [x] 테스트 코드 작성
- [x] `cargo test` 확인

---

### Step 5: 새 명령어 추가 - W 접미사 연산

**목표:** 32비트 연산 후 64비트 부호 확장

**핵심 패턴:**
```rust
fn sign_extend_32(val: u64) -> u64 {
    val as i32 as i64 as u64
}
```

**OP-IMM-32 (opcode = 0x1B):**
```rust
// ADDIW
let result = (rs1_val as i32).wrapping_add(imm as i32);
self.write_reg(rd, result as i64 as u64);

// SLLIW (shamt = 5비트)
let result = (rs1_val as u32) << shamt;
self.write_reg(rd, result as i32 as i64 as u64);

// SRLIW
let result = (rs1_val as u32) >> shamt;
self.write_reg(rd, result as i32 as i64 as u64);

// SRAIW
let result = (rs1_val as i32) >> shamt;
self.write_reg(rd, result as i64 as u64);
```

**OP-32 (opcode = 0x3B):**
```rust
// ADDW
let result = (rs1_val as i32).wrapping_add(rs2_val as i32);
self.write_reg(rd, result as i64 as u64);

// SUBW
let result = (rs1_val as i32).wrapping_sub(rs2_val as i32);
self.write_reg(rd, result as i64 as u64);

// SLLW, SRLW, SRAW 유사
```

**체크리스트:**
- [x] OP_IMM_32 (0x1B) 상수 추가
- [x] OP_32 (0x3B) 상수 추가
- [x] ADDIW 구현
- [x] SLLIW, SRLIW, SRAIW 구현
- [x] ADDW, SUBW 구현
- [x] SLLW, SRLW, SRAW 구현
- [x] 테스트 코드 작성
- [x] `cargo test` 확인

---

### Step 6: 테스트 수정

**목표:** 기존 테스트를 64비트에 맞게 수정

**주요 변경:**
```rust
// Before
assert_eq!(cpu.read_reg(1), 0xFFFFFFFF);  // -1 as u32

// After
assert_eq!(cpu.read_reg(1), 0xFFFFFFFFFFFFFFFF);  // -1 as u64
```

**체크리스트:**
- [x] 모든 테스트의 예상값 수정
- [x] 부호 확장 관련 테스트 추가
- [x] 64비트 전용 명령어 테스트 추가
- [x] `cargo test` 전체 통과

---

### Step 7: main.rs 테스트 프로그램

**목표:** 64비트 동작 확인

```rust
// 간단한 64비트 테스트
let program: Vec<u32> = vec![
    // x1 = 0xFFFFFFFF (32비트 -1)
    0xFFF00093,  // addi x1, x0, -1

    // x2 = x1 (64비트로 부호 확장되어 0xFFFFFFFFFFFFFFFF)
    0x00008113,  // addi x2, x1, 0

    // 64비트 연산 테스트
    // ...

    0x00000073,  // ecall
];
```

**체크리스트:**
- [x] 테스트 프로그램 작성
- [x] `cargo run` 확인
- [x] 결과값 검증

---

## 요약: 새 명령어 목록

| opcode | funct3 | funct7 | 명령어 |
|--------|--------|--------|--------|
| 0x03 | 0x3 | - | LD |
| 0x03 | 0x6 | - | LWU |
| 0x23 | 0x3 | - | SD |
| 0x1B | 0x0 | - | ADDIW |
| 0x1B | 0x1 | 0x00 | SLLIW |
| 0x1B | 0x5 | 0x00 | SRLIW |
| 0x1B | 0x5 | 0x20 | SRAIW |
| 0x3B | 0x0 | 0x00 | ADDW |
| 0x3B | 0x0 | 0x20 | SUBW |
| 0x3B | 0x1 | 0x00 | SLLW |
| 0x3B | 0x5 | 0x00 | SRLW |
| 0x3B | 0x5 | 0x20 | SRAW |

**총 12개 새 명령어**

---

## 참고

- [RISC-V Spec Volume 1](https://riscv.org/specifications/) - Chapter 5: RV64I
- 기존 RV32I 명령어는 모두 동작 (64비트 확장)
