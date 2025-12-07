# CSR 및 트랩 구현 가이드

특권 모드와 트랩 처리를 구현한다.

---

## 개요

### 왜 필요한가?

- **OS 실행**: xv6는 커널(S모드)과 유저(U모드) 분리 필요
- **인터럽트**: 타이머, 키보드 등 비동기 이벤트 처리
- **예외 처리**: 잘못된 명령어, 페이지 폴트 등
- **시스템 콜**: 유저 프로그램이 OS 기능 요청 (`ecall`)

### 구현 순서

```
Step 1: CSR 저장소
Step 2: CSR 명령어 (6개)
Step 3: 기본 CSR 레지스터
Step 4: 특권 모드 (M/S/U)
Step 5: 트랩 진입
Step 6: 트랩 복귀 (mret/sret)
Step 7: 테스트
```

---

## Step 1: CSR 저장소

**목표:** CSR 값을 저장하고 읽는 구조 만들기

### 구현

**src/csr.rs:**
```rust
use std::collections::HashMap;

pub struct Csr {
    data: HashMap<u16, u64>,
}

impl Csr {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn read(&self, addr: u16) -> u64 {
        *self.data.get(&addr).unwrap_or(&0)
    }

    pub fn write(&mut self, addr: u16, value: u64) {
        self.data.insert(addr, value);
    }
}
```

**cpu.rs에 추가:**
```rust
pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
    pub csr: Csr,      // 추가
    pub halted: bool,
}
```

### 체크리스트

- [x] `src/csr.rs` 파일 생성
- [x] `Csr` 구조체 구현
- [x] `lib.rs`에 모듈 추가
- [x] `Cpu`에 `csr` 필드 추가
- [x] `cargo build` 확인

---

## Step 2: CSR 명령어

**목표:** Zicsr 확장 명령어 6개 구현

### 명령어 인코딩

opcode = `0x73` (SYSTEM), funct3로 구분:

| funct3 | 명령어 | 동작 |
|--------|--------|------|
| 0x1 | CSRRW | t=CSR; CSR=rs1; rd=t |
| 0x2 | CSRRS | t=CSR; CSR=t\|rs1; rd=t |
| 0x3 | CSRRC | t=CSR; CSR=t&~rs1; rd=t |
| 0x5 | CSRRWI | t=CSR; CSR=zimm; rd=t |
| 0x6 | CSRRSI | t=CSR; CSR=t\|zimm; rd=t |
| 0x7 | CSRRCI | t=CSR; CSR=t&~zimm; rd=t |

### 인코딩 형식

```
31        20 19    15 14  12 11   7 6      0
[  csr(12) ][ rs1/z ][ f3  ][  rd  ][ 0x73 ]
```

- `csr`: CSR 주소 (12비트)
- `rs1`: 소스 레지스터 (또는 zimm: 5비트 즉시값)
- `zimm`: rs1 필드를 즉시값으로 사용 (제로 확장)

### 구현

**decoder.rs에 추가:**
```rust
pub fn csr_addr(inst: u32) -> u16 {
    ((inst >> 20) & 0xFFF) as u16
}
```

**cpu.rs:**
```rust
SYSTEM => {
    let funct3 = decoder::funct3(inst);
    let rd = decoder::rd(inst);
    let rs1 = decoder::rs1(inst);
    let csr_addr = decoder::csr_addr(inst);

    match funct3 {
        0x0 => {
            // ECALL/EBREAK (기존 코드)
            let imm = decoder::imm_i(inst);
            match imm {
                0x000 => { /* ECALL */ }
                0x001 => { /* EBREAK */ }
                _ => {}
            }
        }
        0x1 => {
            // CSRRW
            let old = self.csr.read(csr_addr);
            self.csr.write(csr_addr, self.read_reg(rs1));
            self.write_reg(rd, old);
        }
        0x2 => {
            // CSRRS
            let old = self.csr.read(csr_addr);
            if rs1 != 0 {
                self.csr.write(csr_addr, old | self.read_reg(rs1));
            }
            self.write_reg(rd, old);
        }
        0x3 => {
            // CSRRC
            let old = self.csr.read(csr_addr);
            if rs1 != 0 {
                self.csr.write(csr_addr, old & !self.read_reg(rs1));
            }
            self.write_reg(rd, old);
        }
        0x5 => {
            // CSRRWI
            let zimm = rs1 as u64;  // 5비트 즉시값
            let old = self.csr.read(csr_addr);
            self.csr.write(csr_addr, zimm);
            self.write_reg(rd, old);
        }
        0x6 => {
            // CSRRSI
            let zimm = rs1 as u64;
            let old = self.csr.read(csr_addr);
            if zimm != 0 {
                self.csr.write(csr_addr, old | zimm);
            }
            self.write_reg(rd, old);
        }
        0x7 => {
            // CSRRCI
            let zimm = rs1 as u64;
            let old = self.csr.read(csr_addr);
            if zimm != 0 {
                self.csr.write(csr_addr, old & !zimm);
            }
            self.write_reg(rd, old);
        }
        _ => {}
    }
}
```

### 체크리스트

- [x] `decoder::csr_addr()` 추가
- [x] CSRRW 구현
- [x] CSRRS 구현
- [x] CSRRC 구현
- [x] CSRRWI 구현
- [x] CSRRSI 구현
- [x] CSRRCI 구현
- [x] 테스트 작성
- [x] `cargo test` 확인

---

## Step 3: 기본 CSR 레지스터

**목표:** Machine/Supervisor 모드 CSR 정의

### CSR 주소 상수

**src/csr.rs에 추가:**
```rust
// Machine Information
pub const MVENDORID: u16 = 0xF11;
pub const MARCHID: u16 = 0xF12;
pub const MIMPID: u16 = 0xF13;
pub const MHARTID: u16 = 0xF14;

// Machine Trap Setup
pub const MSTATUS: u16 = 0x300;
pub const MISA: u16 = 0x301;
pub const MEDELEG: u16 = 0x302;
pub const MIDELEG: u16 = 0x303;
pub const MIE: u16 = 0x304;
pub const MTVEC: u16 = 0x305;

// Machine Trap Handling
pub const MSCRATCH: u16 = 0x340;
pub const MEPC: u16 = 0x341;
pub const MCAUSE: u16 = 0x342;
pub const MTVAL: u16 = 0x343;
pub const MIP: u16 = 0x344;

// Supervisor Trap Setup
pub const SSTATUS: u16 = 0x100;
pub const SIE: u16 = 0x104;
pub const STVEC: u16 = 0x105;

// Supervisor Trap Handling
pub const SSCRATCH: u16 = 0x140;
pub const SEPC: u16 = 0x141;
pub const SCAUSE: u16 = 0x142;
pub const STVAL: u16 = 0x143;
pub const SIP: u16 = 0x144;

// Supervisor Protection
pub const SATP: u16 = 0x180;
```

### mstatus 비트 필드

```
63    62    38 37  36  35 34 33 32  22 21 20 19 18 17  13 12 11  9 8  7  6  5  4  3  2  1  0
[SD][WPRI][MBE|SBE|SXL|UXL][WPRI][TSR|TW|TVM|MXR|SUM|MPRV][XS][FS][MPP][VS][SPP|MPIE|UBE|SPIE][WPRI|MIE][WPRI|SIE][WPRI]
```

주요 필드:
| 비트 | 이름 | 설명 |
|------|------|------|
| 12-11 | MPP | Machine Previous Privilege (이전 모드) |
| 8 | SPP | Supervisor Previous Privilege |
| 7 | MPIE | Machine Previous Interrupt Enable |
| 5 | SPIE | Supervisor Previous Interrupt Enable |
| 3 | MIE | Machine Interrupt Enable |
| 1 | SIE | Supervisor Interrupt Enable |

### mstatus 비트 마스크

```rust
pub const MSTATUS_MIE: u64 = 1 << 3;
pub const MSTATUS_MPIE: u64 = 1 << 7;
pub const MSTATUS_MPP_MASK: u64 = 0x3 << 11;
pub const MSTATUS_MPP_M: u64 = 0x3 << 11;
pub const MSTATUS_MPP_S: u64 = 0x1 << 11;
pub const MSTATUS_MPP_U: u64 = 0x0 << 11;

pub const MSTATUS_SIE: u64 = 1 << 1;
pub const MSTATUS_SPIE: u64 = 1 << 5;
pub const MSTATUS_SPP: u64 = 1 << 8;
```

### 체크리스트

- [x] CSR 주소 상수 정의
- [x] mstatus 비트 마스크 정의
- [ ] 초기값 설정 (misa 등)
- [x] `cargo build` 확인

---

## Step 4: 특권 모드

**목표:** M/S/U 모드 구분

### 모드 정의

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrivilegeMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}
```

### CPU에 추가

```rust
pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
    pub csr: Csr,
    pub mode: PrivilegeMode,  // 추가
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: DRAM_BASE,
            bus: Bus::new(),
            csr: Csr::new(),
            mode: PrivilegeMode::Machine,  // 부팅 시 M모드
            halted: false,
        }
    }
}
```

### 체크리스트

- [x] `PrivilegeMode` enum 정의
- [x] `Cpu`에 `mode` 필드 추가
- [x] 초기 모드를 Machine으로 설정
- [x] `cargo build` 확인

---

## Step 5: 트랩 진입

**목표:** 예외/인터럽트 발생 시 핸들러로 점프

### 트랩 원인 (mcause)

**인터럽트 (최상위 비트 = 1):**
| 코드 | 이름 |
|------|------|
| 1 | Supervisor software interrupt |
| 3 | Machine software interrupt |
| 5 | Supervisor timer interrupt |
| 7 | Machine timer interrupt |
| 9 | Supervisor external interrupt |
| 11 | Machine external interrupt |

**예외 (최상위 비트 = 0):**
| 코드 | 이름 |
|------|------|
| 0 | Instruction address misaligned |
| 1 | Instruction access fault |
| 2 | Illegal instruction |
| 3 | Breakpoint |
| 4 | Load address misaligned |
| 5 | Load access fault |
| 6 | Store address misaligned |
| 7 | Store access fault |
| 8 | Environment call from U-mode |
| 9 | Environment call from S-mode |
| 11 | Environment call from M-mode |
| 12 | Instruction page fault |
| 13 | Load page fault |
| 15 | Store page fault |

### 트랩 상수

```rust
// 인터럽트 (bit 63 = 1)
pub const INTERRUPT_BIT: u64 = 1 << 63;

pub const SUPERVISOR_SOFTWARE_INTERRUPT: u64 = INTERRUPT_BIT | 1;
pub const MACHINE_SOFTWARE_INTERRUPT: u64 = INTERRUPT_BIT | 3;
pub const SUPERVISOR_TIMER_INTERRUPT: u64 = INTERRUPT_BIT | 5;
pub const MACHINE_TIMER_INTERRUPT: u64 = INTERRUPT_BIT | 7;
pub const SUPERVISOR_EXTERNAL_INTERRUPT: u64 = INTERRUPT_BIT | 9;
pub const MACHINE_EXTERNAL_INTERRUPT: u64 = INTERRUPT_BIT | 11;

// 예외
pub const INSTRUCTION_ADDRESS_MISALIGNED: u64 = 0;
pub const ILLEGAL_INSTRUCTION: u64 = 2;
pub const BREAKPOINT: u64 = 3;
pub const ECALL_FROM_U: u64 = 8;
pub const ECALL_FROM_S: u64 = 9;
pub const ECALL_FROM_M: u64 = 11;
```

### 트랩 처리 함수

```rust
impl Cpu {
    pub fn trap(&mut self, cause: u64, tval: u64) {
        // 현재 모드에 따라 M모드 또는 S모드로 트랩
        // (위임 레지스터 medeleg/mideleg 확인 필요, 일단 M모드로)

        let is_interrupt = (cause >> 63) == 1;

        // 1. mepc에 현재 PC 저장
        //    예외: 예외 발생 명령어 주소
        //    인터럽트: 다음 명령어 주소
        if is_interrupt {
            self.csr.write(MEPC, self.pc);
        } else {
            self.csr.write(MEPC, self.pc);  // 현재 명령어
        }

        // 2. mcause에 원인 저장
        self.csr.write(MCAUSE, cause);

        // 3. mtval에 추가 정보 저장
        self.csr.write(MTVAL, tval);

        // 4. mstatus 업데이트
        let mut mstatus = self.csr.read(MSTATUS);

        // MPIE = MIE (이전 인터럽트 활성화 상태 저장)
        let mie = (mstatus & MSTATUS_MIE) != 0;
        if mie {
            mstatus |= MSTATUS_MPIE;
        } else {
            mstatus &= !MSTATUS_MPIE;
        }

        // MIE = 0 (인터럽트 비활성화)
        mstatus &= !MSTATUS_MIE;

        // MPP = 현재 모드
        mstatus &= !MSTATUS_MPP_MASK;
        mstatus |= (self.mode as u64) << 11;

        self.csr.write(MSTATUS, mstatus);

        // 5. 모드를 Machine으로 변경
        self.mode = PrivilegeMode::Machine;

        // 6. mtvec으로 점프
        let mtvec = self.csr.read(MTVEC);
        let mode = mtvec & 0x3;
        let base = mtvec & !0x3;

        if mode == 0 {
            // Direct mode: 모든 트랩이 base로
            self.pc = base;
        } else {
            // Vectored mode: base + 4*cause (인터럽트만)
            if is_interrupt {
                self.pc = base + 4 * (cause & 0x3FF);
            } else {
                self.pc = base;
            }
        }
    }
}
```

### ECALL 수정

```rust
0x000 => {
    // ECALL
    let cause = match self.mode {
        PrivilegeMode::User => ECALL_FROM_U,
        PrivilegeMode::Supervisor => ECALL_FROM_S,
        PrivilegeMode::Machine => ECALL_FROM_M,
    };
    self.trap(cause, 0);
    return;  // pc += 4 하지 않음
}
```

### 체크리스트

- [x] 트랩 원인 상수 정의
- [x] `trap()` 함수 구현
- [x] ECALL에서 트랩 호출
- [x] EBREAK에서 트랩 호출
- [x] 테스트 작성
- [x] `cargo test` 확인

---

## Step 6: 트랩 복귀 (mret/sret)

**목표:** 핸들러에서 원래 코드로 복귀

### 명령어 인코딩

```
MRET: 0x30200073 (funct7=0011000, rs2=00010, ...)
SRET: 0x10200073 (funct7=0001000, rs2=00010, ...)
```

둘 다 opcode=0x73, funct3=0x0, 상위 비트로 구분

### 구현

```rust
0x0 => {
    let funct7 = decoder::funct7(inst);
    let rs2 = decoder::rs2(inst);

    match (funct7, rs2) {
        (0x00, 0x00) => {
            // ECALL
            let cause = match self.mode {
                PrivilegeMode::User => ECALL_FROM_U,
                PrivilegeMode::Supervisor => ECALL_FROM_S,
                PrivilegeMode::Machine => ECALL_FROM_M,
            };
            self.trap(cause, 0);
            return;
        }
        (0x00, 0x01) => {
            // EBREAK
            self.trap(BREAKPOINT, 0);
            return;
        }
        (0x18, 0x02) => {
            // MRET
            self.mret();
            return;
        }
        (0x08, 0x02) => {
            // SRET
            self.sret();
            return;
        }
        _ => {}
    }
}
```

### mret 구현

```rust
impl Cpu {
    pub fn mret(&mut self) {
        // 1. mepc에서 PC 복원
        self.pc = self.csr.read(MEPC);

        // 2. mstatus에서 이전 상태 복원
        let mut mstatus = self.csr.read(MSTATUS);

        // MIE = MPIE
        let mpie = (mstatus & MSTATUS_MPIE) != 0;
        if mpie {
            mstatus |= MSTATUS_MIE;
        } else {
            mstatus &= !MSTATUS_MIE;
        }

        // MPIE = 1
        mstatus |= MSTATUS_MPIE;

        // 모드 = MPP
        let mpp = (mstatus & MSTATUS_MPP_MASK) >> 11;
        self.mode = match mpp {
            0 => PrivilegeMode::User,
            1 => PrivilegeMode::Supervisor,
            3 => PrivilegeMode::Machine,
            _ => PrivilegeMode::Machine,
        };

        // MPP = U (또는 지원하는 가장 낮은 모드)
        mstatus &= !MSTATUS_MPP_MASK;

        self.csr.write(MSTATUS, mstatus);
    }

    pub fn sret(&mut self) {
        // 1. sepc에서 PC 복원
        self.pc = self.csr.read(SEPC);

        // 2. sstatus에서 이전 상태 복원
        let mut sstatus = self.csr.read(SSTATUS);

        // SIE = SPIE
        let spie = (sstatus & MSTATUS_SPIE) != 0;
        if spie {
            sstatus |= MSTATUS_SIE;
        } else {
            sstatus &= !MSTATUS_SIE;
        }

        // SPIE = 1
        sstatus |= MSTATUS_SPIE;

        // 모드 = SPP
        let spp = (sstatus & MSTATUS_SPP) != 0;
        self.mode = if spp {
            PrivilegeMode::Supervisor
        } else {
            PrivilegeMode::User
        };

        // SPP = U
        sstatus &= !MSTATUS_SPP;

        self.csr.write(SSTATUS, sstatus);
    }
}
```

### 체크리스트

- [x] MRET 명령어 디코딩
- [x] SRET 명령어 디코딩
- [x] `mret()` 함수 구현
- [x] `sret()` 함수 구현
- [x] 테스트 작성
- [x] `cargo test` 확인

---

## Step 7: 통합 테스트

**목표:** 트랩 → 핸들러 → 복귀 전체 흐름 테스트

### 테스트 프로그램

```rust
#[test]
fn test_ecall_trap() {
    let mut cpu = Cpu::new();

    // mtvec 설정 (핸들러 주소)
    cpu.csr.write(MTVEC, 0x80001000);

    // ECALL 실행
    cpu.bus.write32(DRAM_BASE, 0x00000073);  // ecall
    cpu.step();

    // 트랩 발생 확인
    assert_eq!(cpu.pc, 0x80001000);  // mtvec으로 점프
    assert_eq!(cpu.csr.read(MEPC), DRAM_BASE);  // 원래 PC 저장
    assert_eq!(cpu.csr.read(MCAUSE), ECALL_FROM_M);  // 원인
    assert_eq!(cpu.mode, PrivilegeMode::Machine);
}

#[test]
fn test_mret() {
    let mut cpu = Cpu::new();

    // 트랩 상태 설정
    cpu.csr.write(MEPC, 0x80002000);
    cpu.csr.write(MSTATUS, MSTATUS_MPIE | MSTATUS_MPP_S);

    // MRET 실행
    cpu.bus.write32(DRAM_BASE, 0x30200073);  // mret
    cpu.step();

    // 복귀 확인
    assert_eq!(cpu.pc, 0x80002000);  // mepc로 복귀
    assert_eq!(cpu.mode, PrivilegeMode::Supervisor);  // MPP 모드로

    let mstatus = cpu.csr.read(MSTATUS);
    assert!((mstatus & MSTATUS_MIE) != 0);  // MIE = MPIE
}
```

### 체크리스트

- [x] ECALL 트랩 테스트
- [x] MRET 복귀 테스트
- [x] SRET 복귀 테스트
- [x] mstatus 비트 변경 테스트
- [x] ECALL → MRET 라운드트립 테스트
- [x] UART 출력 통합 테스트
- [x] Sum 1-10 루프 테스트
- [x] `cargo test` 전체 통과 (139 tests)

---

## 요약: 새 명령어

| 명령어 | 인코딩 | 설명 |
|--------|--------|------|
| CSRRW | funct3=0x1 | CSR 읽고 쓰기 |
| CSRRS | funct3=0x2 | CSR 읽고 비트 셋 |
| CSRRC | funct3=0x3 | CSR 읽고 비트 클리어 |
| CSRRWI | funct3=0x5 | 즉시값으로 CSRRW |
| CSRRSI | funct3=0x6 | 즉시값으로 CSRRS |
| CSRRCI | funct3=0x7 | 즉시값으로 CSRRC |
| MRET | 0x30200073 | M모드 트랩에서 복귀 |
| SRET | 0x10200073 | S모드 트랩에서 복귀 |

**총 8개 새 명령어**

---

## 다음 단계

Phase 2 완료 후:
1. Timer (CLINT) 구현 → 타이머 인터럽트 연결
2. UART 수신 구현 → 외부 인터럽트 연결
3. 인터럽트 펜딩/활성화 로직 (mip/mie)

---

## 참고

- [RISC-V Privileged Spec](https://riscv.org/specifications/privileged-isa/)
- Chapter 3: Machine-Level ISA
- Chapter 4: Supervisor-Level ISA
