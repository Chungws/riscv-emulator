# S-mode 인터럽트 구현 가이드

xv6 실행을 위한 Supervisor Mode 인터럽트 지원 구현.

---

## 1. 배경

### 1.1 현재 상태

- M-mode 인터럽트만 지원 (MTIP, MSIP, MEIP)
- xv6는 S-mode에서 실행되며 S-mode 인터럽트 필요
- 타이머 인터럽트가 S-mode로 전달되지 않아 scheduler가 동작 안 함

### 1.2 xv6 타이머 인터럽트 흐름

```
┌─────────────────────────────────────────────────────────────┐
│  1. mtime >= mtimecmp                                       │
│     → CLINT가 MTIP 설정                                     │
├─────────────────────────────────────────────────────────────┤
│  2. M-mode trap handler (timervec)                          │
│     → MIP.STIP = 1 설정 (S-mode에 알림)                     │
│     → mtimecmp += interval (다음 타이머 설정)               │
│     → MRET                                                  │
├─────────────────────────────────────────────────────────────┤
│  3. S-mode에서 STIP 감지                                    │
│     → S-mode trap handler (kernelvec)                       │
│     → scheduler 실행                                        │
│     → MIP.STIP = 0 클리어                                   │
│     → SRET                                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. CSR 확장

### 2.1 추가할 CSR 주소

```rust
// csr.rs에 추가

// Supervisor Mode CSRs (추가)
pub const SIE: u16 = 0x104;      // Supervisor Interrupt Enable
pub const SIP: u16 = 0x144;      // Supervisor Interrupt Pending
pub const SSCRATCH: u16 = 0x140; // Supervisor Scratch

// Machine Mode CSRs (추가)
pub const MEDELEG: u16 = 0x302;  // Machine Exception Delegation
pub const MIDELEG: u16 = 0x303;  // Machine Interrupt Delegation
pub const MCOUNTEREN: u16 = 0x306; // Machine Counter Enable
pub const SCOUNTEREN: u16 = 0x106; // Supervisor Counter Enable
```

### 2.2 추가할 비트 마스크

```rust
// S-mode Interrupt Enable bits (SIE)
pub const SIE_SSIE: u64 = 1 << 1;  // Supervisor Software Interrupt Enable
pub const SIE_STIE: u64 = 1 << 5;  // Supervisor Timer Interrupt Enable
pub const SIE_SEIE: u64 = 1 << 9;  // Supervisor External Interrupt Enable

// S-mode Interrupt Pending bits (SIP)
pub const SIP_SSIP: u64 = 1 << 1;  // Supervisor Software Interrupt Pending
pub const SIP_STIP: u64 = 1 << 5;  // Supervisor Timer Interrupt Pending
pub const SIP_SEIP: u64 = 1 << 9;  // Supervisor External Interrupt Pending

// MIE에 S-mode 비트 추가
pub const MIE_SSIE: u64 = 1 << 1;
pub const MIE_STIE: u64 = 1 << 5;
pub const MIE_SEIE: u64 = 1 << 9;

// MIP에 S-mode 비트 추가
pub const MIP_SSIP: u64 = 1 << 1;
pub const MIP_STIP: u64 = 1 << 5;
pub const MIP_SEIP: u64 = 1 << 9;

// S-mode Interrupt codes
pub const INTERRUPT_S_SOFTWARE: u64 = 1;
pub const INTERRUPT_S_TIMER: u64 = 5;
pub const INTERRUPT_S_EXTERNAL: u64 = 9;
```

---

## 3. CSR 별칭(Aliasing) 처리

### 3.1 SSTATUS ↔ MSTATUS

SSTATUS는 MSTATUS의 일부 비트만 보여주는 뷰(view):

```
MSTATUS:  [SD][...][TSR][TW][TVM][MXR][SUM][MPRV][XS][FS][MPP][VS][SPP][MPIE][UBE][SPIE][...][MIE][...][SIE][...]
SSTATUS:  [SD][...][  ][  ][   ][MXR][SUM][    ][XS][FS][   ][VS][SPP][    ][UBE][SPIE][...][   ][...][SIE][...]
```

SSTATUS에서 접근 가능한 비트:
- SIE (bit 1)
- SPIE (bit 5)
- UBE (bit 6)
- SPP (bit 8)
- VS (bits 10:9)
- FS (bits 14:13)
- XS (bits 16:15)
- SUM (bit 18)
- MXR (bit 19)
- SD (bit 63)

```rust
// SSTATUS 마스크
pub const SSTATUS_MASK: u64 =
    MSTATUS_SIE | MSTATUS_SPIE | MSTATUS_SPP |
    (0x3 << 13) |  // FS
    (0x3 << 15) |  // XS
    (1 << 18) |    // SUM
    (1 << 19) |    // MXR
    (1u64 << 63);  // SD
```

### 3.2 SIE ↔ MIE

SIE는 MIE의 S-mode 비트만 보여줌:

```rust
pub const SIE_MASK: u64 = SIE_SSIE | SIE_STIE | SIE_SEIE;
```

### 3.3 SIP ↔ MIP

SIP는 MIP의 S-mode 비트만 보여줌:

```rust
pub const SIP_MASK: u64 = SIP_SSIP | SIP_STIP | SIP_SEIP;
// 참고: STIP, SEIP는 S-mode에서 읽기 전용
pub const SIP_WRITABLE_MASK: u64 = SIP_SSIP;
```

---

## 4. 구현 단계

### Step 1: CSR 상수 추가

- [ ] SIE, SIP, SSCRATCH, MEDELEG, MIDELEG, MCOUNTEREN, SCOUNTEREN 상수
- [ ] S-mode 인터럽트 비트 마스크
- [ ] SSTATUS_MASK, SIE_MASK, SIP_MASK 마스크

```rust
// csr.rs 추가

// S-mode CSR 주소
pub const SIE: u16 = 0x104;
pub const SIP: u16 = 0x144;
pub const SSCRATCH: u16 = 0x140;
pub const MEDELEG: u16 = 0x302;
pub const MIDELEG: u16 = 0x303;
pub const MCOUNTEREN: u16 = 0x306;
pub const SCOUNTEREN: u16 = 0x106;

// S-mode 인터럽트 비트
pub const MIE_SSIE: u64 = 1 << 1;
pub const MIE_STIE: u64 = 1 << 5;
pub const MIE_SEIE: u64 = 1 << 9;

pub const MIP_SSIP: u64 = 1 << 1;
pub const MIP_STIP: u64 = 1 << 5;
pub const MIP_SEIP: u64 = 1 << 9;

// 별칭용 마스크
pub const SSTATUS_MASK: u64 = 0x800000030001e2;
pub const SIE_MASK: u64 = 0x222;
pub const SIP_MASK: u64 = 0x222;
pub const SIP_WRITABLE_MASK: u64 = 0x2;

// S-mode 인터럽트 코드
pub const INTERRUPT_S_SOFTWARE: u64 = 1;
pub const INTERRUPT_S_TIMER: u64 = 5;
pub const INTERRUPT_S_EXTERNAL: u64 = 9;
```

---

### Step 2: CSR 읽기/쓰기 별칭 처리

SSTATUS, SIE, SIP 접근 시 MSTATUS, MIE, MIP를 통해 처리:

```rust
// cpu.rs의 CSR 읽기 로직 수정

fn read_csr(&self, addr: u16) -> u64 {
    match addr {
        csr::SSTATUS => self.csr.read(csr::MSTATUS) & csr::SSTATUS_MASK,
        csr::SIE => self.csr.read(csr::MIE) & csr::SIE_MASK,
        csr::SIP => self.csr.read(csr::MIP) & csr::SIP_MASK,
        _ => self.csr.read(addr),
    }
}

fn write_csr(&mut self, addr: u16, value: u64) {
    match addr {
        csr::SSTATUS => {
            let mstatus = self.csr.read(csr::MSTATUS);
            let new_val = (mstatus & !csr::SSTATUS_MASK) | (value & csr::SSTATUS_MASK);
            self.csr.write(csr::MSTATUS, new_val);
        }
        csr::SIE => {
            let mie = self.csr.read(csr::MIE);
            let new_val = (mie & !csr::SIE_MASK) | (value & csr::SIE_MASK);
            self.csr.write(csr::MIE, new_val);
        }
        csr::SIP => {
            let mip = self.csr.read(csr::MIP);
            // SSIP만 쓰기 가능
            let new_val = (mip & !csr::SIP_WRITABLE_MASK) | (value & csr::SIP_WRITABLE_MASK);
            self.csr.write(csr::MIP, new_val);
        }
        _ => self.csr.write(addr, value),
    }
}
```

---

### Step 3: S-mode 인터럽트 처리

`check_pending_interrupts()`에 S-mode 인터럽트 처리 추가:

```rust
fn check_pending_interrupts(&mut self) -> bool {
    // 기존 M-mode 인터럽트 펜딩 업데이트 (MTIP, MSIP, MEIP)
    // ... 기존 코드 ...

    let mstatus = self.csr.read(csr::MSTATUS);
    let mip = self.csr.read(csr::MIP);
    let mie = self.csr.read(csr::MIE);
    let mideleg = self.csr.read(csr::MIDELEG);

    // M-mode 인터럽트 처리 (mideleg 안 된 것들)
    if mstatus & csr::MSTATUS_MIE != 0 {
        // MSIP (Software)
        if (mip & csr::MIP_MSIP != 0) && (mie & csr::MIE_MSIE != 0)
           && (mideleg & csr::MIP_MSIP == 0) {
            self.trap(csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_SOFTWARE, 0);
            return true;
        }
        // MTIP (Timer)
        if (mip & csr::MIP_MTIP != 0) && (mie & csr::MIE_MTIE != 0)
           && (mideleg & csr::MIP_MTIP == 0) {
            self.trap(csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_TIMER, 0);
            return true;
        }
        // MEIP (External)
        if (mip & csr::MIP_MEIP != 0) && (mie & csr::MIE_MEIE != 0)
           && (mideleg & csr::MIP_MEIP == 0) {
            self.trap(csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_EXTERNAL, 0);
            return true;
        }
    }

    // S-mode 인터럽트 처리 (현재 S-mode이고 SIE가 활성화된 경우)
    let in_s_mode = self.privilege == PrivilegeMode::Supervisor;
    let sie_enabled = mstatus & csr::MSTATUS_SIE != 0;

    if in_s_mode && sie_enabled {
        // SSIP
        if (mip & csr::MIP_SSIP != 0) && (mie & csr::MIE_SSIE != 0) {
            self.trap_to_s_mode(csr::INTERRUPT_BIT | csr::INTERRUPT_S_SOFTWARE, 0);
            return true;
        }
        // STIP
        if (mip & csr::MIP_STIP != 0) && (mie & csr::MIE_STIE != 0) {
            self.trap_to_s_mode(csr::INTERRUPT_BIT | csr::INTERRUPT_S_TIMER, 0);
            return true;
        }
        // SEIP
        if (mip & csr::MIP_SEIP != 0) && (mie & csr::MIE_SEIE != 0) {
            self.trap_to_s_mode(csr::INTERRUPT_BIT | csr::INTERRUPT_S_EXTERNAL, 0);
            return true;
        }
    }

    false
}
```

---

### Step 4: S-mode Trap 처리

S-mode로 트랩하는 함수 추가:

```rust
fn trap_to_s_mode(&mut self, cause: u64, tval: u64) {
    // SEPC = 현재 PC
    self.csr.write(csr::SEPC, self.pc);

    // SCAUSE = 원인
    self.csr.write(csr::SCAUSE, cause);

    // STVAL = 추가 정보
    self.csr.write(csr::STVAL, tval);

    // SSTATUS 업데이트
    let mstatus = self.csr.read(csr::MSTATUS);

    // SPIE = SIE (이전 인터럽트 활성화 상태 저장)
    let spie = if mstatus & csr::MSTATUS_SIE != 0 {
        csr::MSTATUS_SPIE
    } else {
        0
    };

    // SPP = 현재 권한 (S=1, U=0)
    let spp = if self.privilege == PrivilegeMode::Supervisor {
        csr::MSTATUS_SPP
    } else {
        0
    };

    // SIE = 0 (인터럽트 비활성화)
    let new_mstatus = (mstatus & !csr::MSTATUS_SIE & !csr::MSTATUS_SPIE & !csr::MSTATUS_SPP)
                      | spie | spp;
    self.csr.write(csr::MSTATUS, new_mstatus);

    // PC = STVEC
    self.pc = self.csr.read(csr::STVEC) & !0x3;

    // 권한 = S-mode
    self.privilege = PrivilegeMode::Supervisor;
}
```

---

### Step 5: SRET 명령어 구현

SRET이 이미 구현되어 있다면 확인, 없다면 추가:

```rust
// SYSTEM 명령어 핸들러에서
0x102 => {
    // SRET
    let mstatus = self.csr.read(csr::MSTATUS);

    // SIE = SPIE
    let sie = if mstatus & csr::MSTATUS_SPIE != 0 {
        csr::MSTATUS_SIE
    } else {
        0
    };

    // 권한 = SPP (1=S, 0=U)
    self.privilege = if mstatus & csr::MSTATUS_SPP != 0 {
        PrivilegeMode::Supervisor
    } else {
        PrivilegeMode::User
    };

    // SPIE = 1, SPP = 0
    let new_mstatus = (mstatus & !csr::MSTATUS_SIE & !csr::MSTATUS_SPP)
                      | sie | csr::MSTATUS_SPIE;
    self.csr.write(csr::MSTATUS, new_mstatus);

    // PC = SEPC
    self.pc = self.csr.read(csr::SEPC);
}
```

---

### Step 6: WFI (Wait For Interrupt) 명령어

WFI는 인터럽트가 발생할 때까지 CPU를 대기 상태로 만드는 명령어:

```
31      25 24  20 19  15 14  12 11   7 6      0
[0000100 ][ 00101][ 00000][ 000 ][ 00000][ 1110011]
  funct7    rs2     rs1   funct3   rd     SYSTEM
```

- **인코딩**: funct7=0x08, rs2=0x05, funct3=0x0
- **동작**: 인터럽트 pending될 때까지 대기 (또는 NOP처럼 동작)

```rust
// SYSTEM 명령어 핸들러에서 (funct3 == 0x0)
(0x08, 0x05) => {
    // WFI - Wait For Interrupt
    debug_log!("WFI");
    // 간단한 구현: 인터럽트가 pending될 때까지 대기
    // 싱글코어에서는 NOP처럼 동작해도 됨
    // (다음 step에서 인터럽트 체크됨)
    false
}
```

**참고**:
- 실제 하드웨어에서는 전력 절약을 위해 CPU를 sleep 상태로 전환
- 에뮬레이터에서는 간단히 NOP으로 구현 가능
- 인터럽트가 pending 상태면 즉시 리턴

---

### Step 7: 테스트

- [ ] CSR 별칭 테스트 (SSTATUS ↔ MSTATUS)
- [ ] S-mode 타이머 인터럽트 테스트
- [ ] SRET 테스트
- [ ] WFI 테스트
- [ ] xv6 부팅 테스트

---

## 5. 체크리스트

### Step 1: CSR 상수
- [ ] SIE, SIP, SSCRATCH 주소
- [ ] MEDELEG, MIDELEG, MCOUNTEREN, SCOUNTEREN 주소
- [ ] S-mode 인터럽트 비트 마스크
- [ ] 별칭 마스크 (SSTATUS_MASK, SIE_MASK, SIP_MASK)

### Step 2: CSR 별칭 처리
- [ ] SSTATUS 읽기 → MSTATUS & SSTATUS_MASK
- [ ] SSTATUS 쓰기 → MSTATUS 수정
- [ ] SIE 읽기/쓰기 → MIE
- [ ] SIP 읽기/쓰기 → MIP

### Step 3: S-mode 인터럽트 처리
- [ ] SSIP, STIP, SEIP 체크
- [ ] 권한 레벨 확인 (S-mode인지)
- [ ] SIE 활성화 확인

### Step 4: S-mode Trap
- [ ] trap_to_s_mode() 함수
- [ ] SEPC, SCAUSE, STVAL 설정
- [ ] SSTATUS 업데이트 (SPIE, SPP, SIE)
- [ ] PC = STVEC

### Step 5: SRET
- [ ] SRET 명령어 확인/구현
- [ ] SSTATUS 복원
- [ ] PC = SEPC

### Step 6: WFI
- [ ] WFI 명령어 추가 (funct7=0x08, rs2=0x05)
- [ ] NOP처럼 동작 (싱글코어)

### Step 7: 테스트
- [ ] CSR 별칭 테스트
- [ ] S-mode 인터럽트 테스트
- [ ] WFI 테스트
- [ ] xv6 테스트

---

## 6. 참고

### 6.1 인터럽트 우선순위

높음 → 낮음:
1. MEI (Machine External)
2. MSI (Machine Software)
3. MTI (Machine Timer)
4. SEI (Supervisor External)
5. SSI (Supervisor Software)
6. STI (Supervisor Timer)

### 6.2 xv6에서 사용하는 CSR

```
M-mode:
- mstatus, mtvec, mepc, mcause, mtval
- mie, mip
- mhartid
- mscratch

S-mode:
- sstatus, stvec, sepc, scause, stval
- sie, sip (실제로는 mie, mip의 별칭)
- sscratch
```

### 6.3 RISC-V Spec 참고

- Volume II: Privileged Architecture
- Chapter 4: Supervisor-Level ISA
- Section 4.1: Supervisor CSRs
