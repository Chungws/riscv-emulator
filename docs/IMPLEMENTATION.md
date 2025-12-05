# 단계별 구현 가이드

이 문서는 RV32I 에뮬레이터를 단계별로 구현하는 가이드입니다.
각 단계는 명확한 목표와 테스트를 가지고 있어서, 테스트를 통과하면 다음 단계로 넘어갈 수 있습니다.

---

## Step 1: 프로젝트 설정 & CPU 기본 구조

### 목표
프로젝트를 초기화하고 CPU의 기본 구조를 만듭니다.

### 구현 내용
- Cargo 프로젝트 생성
- `Cpu` 구조체 정의
  - 32개의 32비트 레지스터 (`regs: [u32; 32]`)
  - Program Counter (`pc: u32`)
- x0 레지스터는 항상 0 유지

### 테스트
```rust
#[test]
fn test_cpu_init() {
    let cpu = Cpu::new();
    // 모든 레지스터가 0으로 초기화
    for i in 0..32 {
        assert_eq!(cpu.regs[i], 0);
    }
    // PC는 DRAM_BASE(0x80000000)에서 시작
    assert_eq!(cpu.pc, 0x80000000);
}

#[test]
fn test_x0_always_zero() {
    let mut cpu = Cpu::new();
    cpu.write_reg(0, 100);  // x0에 쓰기 시도
    assert_eq!(cpu.read_reg(0), 0);  // 여전히 0
}
```

### 체크리스트
- [x] `Cargo.toml` 생성
- [x] `src/cpu.rs` - Cpu 구조체
- [x] `Cpu::new()` 생성자
- [x] `Cpu::read_reg()` / `Cpu::write_reg()` 메서드
- [x] 테스트 통과

---

## Step 2: 메모리 시스템

### 목표
간단한 메모리 시스템을 구현합니다.

### 구현 내용
- `Memory` 구조체
  - DRAM_BASE: `0x80000000`
  - DRAM_SIZE: `128MB` (0x8000000)
- 바이트/하프워드/워드 읽기/쓰기

### 테스트
```rust
#[test]
fn test_memory_read_write_byte() {
    let mut mem = Memory::new();
    mem.write8(0x80000000, 0xAB);
    assert_eq!(mem.read8(0x80000000), 0xAB);
}

#[test]
fn test_memory_read_write_word() {
    let mut mem = Memory::new();
    mem.write32(0x80000000, 0xDEADBEEF);
    assert_eq!(mem.read32(0x80000000), 0xDEADBEEF);
}

#[test]
fn test_memory_little_endian() {
    let mut mem = Memory::new();
    mem.write32(0x80000000, 0x12345678);
    assert_eq!(mem.read8(0x80000000), 0x78);      // LSB first
    assert_eq!(mem.read8(0x80000001), 0x56);
    assert_eq!(mem.read8(0x80000002), 0x34);
    assert_eq!(mem.read8(0x80000003), 0x12);      // MSB last
}
```

### 체크리스트
- [x] `src/memory.rs` - Memory 구조체
- [x] `read8`, `read16`, `read32` 구현
- [x] `write8`, `write16`, `write32` 구현
- [x] 리틀 엔디안 처리
- [x] 테스트 통과

---

## Step 3: Fetch (명령어 읽기)

### 목표
PC 위치에서 32비트 명령어를 읽어옵니다.

### 구현 내용
- CPU에 Memory 연결
- `Cpu::fetch()` 메서드

### 테스트
```rust
#[test]
fn test_fetch() {
    let mut cpu = Cpu::new();
    // ADDI x1, x0, 42를 메모리에 로드
    // addi x1, x0, 42 → 0x02A00093
    cpu.memory.write32(0x80000000, 0x02A00093);

    let instruction = cpu.fetch();
    assert_eq!(instruction, 0x02A00093);
}
```

### 체크리스트
- [x] CPU에 Memory 필드 추가
- [x] `Cpu::fetch()` 구현
- [x] 테스트 통과

---

## Step 4: Decode (명령어 디코딩)

### 목표
32비트 명령어를 파싱하여 의미 있는 필드로 분리합니다.

### 구현 내용
- 명령어 필드 추출 함수들
  - `opcode`, `rd`, `rs1`, `rs2`, `funct3`, `funct7`
  - 각 포맷별 immediate 추출 (I, S, B, U, J)
- `Instruction` enum 정의 (선택적)

### 테스트
```rust
#[test]
fn test_decode_r_type() {
    // ADD x3, x1, x2 → 0x002080B3
    let inst = 0x002080B3_u32;
    assert_eq!(opcode(inst), 0b0110011);  // R-type
    assert_eq!(rd(inst), 3);
    assert_eq!(funct3(inst), 0);
    assert_eq!(rs1(inst), 1);
    assert_eq!(rs2(inst), 2);
    assert_eq!(funct7(inst), 0);
}

#[test]
fn test_decode_i_type_imm() {
    // ADDI x1, x0, -1 → 0xFFF00093
    let inst = 0xFFF00093_u32;
    let imm = imm_i(inst);
    assert_eq!(imm, -1_i32);  // sign-extended
}

#[test]
fn test_decode_s_type_imm() {
    // SW x2, 8(x1) → 0x00212423
    let inst = 0x00212423_u32;
    let imm = imm_s(inst);
    assert_eq!(imm, 8);
}

#[test]
fn test_decode_b_type_imm() {
    // BEQ x1, x2, 16 → 0x00208863
    let inst = 0x00208863_u32;
    let imm = imm_b(inst);
    assert_eq!(imm, 16);
}
```

### 체크리스트
- [x] `src/decoder.rs` 생성
- [x] `opcode()`, `rd()`, `rs1()`, `rs2()`, `funct3()`, `funct7()` 구현
- [x] `imm_i()`, `imm_s()`, `imm_b()`, `imm_u()`, `imm_j()` 구현
- [x] 부호 확장 (sign extension) 올바르게 처리
- [x] 테스트 통과

---

## Step 5: Execute - 산술 연산 (ADD, SUB, ADDI)

### 목표
첫 번째 명령어들을 실행합니다!

### 구현 내용
- `Cpu::execute()` 메서드 골격
- ADD, SUB, ADDI 구현

### 테스트
```rust
#[test]
fn test_addi() {
    let mut cpu = Cpu::new();
    // ADDI x1, x0, 42
    cpu.memory.write32(0x80000000, 0x02A00093);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 42);
    assert_eq!(cpu.pc, 0x80000004);
}

#[test]
fn test_add() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 10);
    cpu.write_reg(2, 20);
    // ADD x3, x1, x2
    cpu.memory.write32(0x80000000, 0x002080B3);
    cpu.step();
    assert_eq!(cpu.read_reg(3), 30);
}

#[test]
fn test_sub() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 100);
    cpu.write_reg(2, 30);
    // SUB x3, x1, x2
    cpu.memory.write32(0x80000000, 0x402080B3);
    cpu.step();
    assert_eq!(cpu.read_reg(3), 70);
}
```

### 체크리스트
- [x] `Cpu::step()` 또는 `Cpu::execute()` 구현
- [x] ADDI 구현 (opcode=0010011, funct3=000)
- [x] ADD 구현 (opcode=0110011, funct3=000, funct7=0000000)
- [x] SUB 구현 (opcode=0110011, funct3=000, funct7=0100000)
- [x] 테스트 통과

---

## Step 6: Execute - 논리 연산 (AND, OR, XOR, ANDI, ORI, XORI)

### 목표
비트 논리 연산을 구현합니다.

### 테스트
```rust
#[test]
fn test_and() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 0b1100);
    cpu.write_reg(2, 0b1010);
    // AND x3, x1, x2
    cpu.memory.write32(0x80000000, 0x002070B3);
    cpu.step();
    assert_eq!(cpu.read_reg(3), 0b1000);
}

#[test]
fn test_andi() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 0xFF);
    // ANDI x2, x1, 0x0F
    cpu.memory.write32(0x80000000, 0x00F0F113);
    cpu.step();
    assert_eq!(cpu.read_reg(2), 0x0F);
}

// OR, XOR, ORI, XORI도 유사하게 테스트
```

### 체크리스트
- [ ] AND (funct3=111, funct7=0)
- [ ] OR (funct3=110, funct7=0)
- [ ] XOR (funct3=100, funct7=0)
- [ ] ANDI (opcode=0010011, funct3=111)
- [ ] ORI (opcode=0010011, funct3=110)
- [ ] XORI (opcode=0010011, funct3=100)
- [ ] 테스트 통과

---

## Step 7: Execute - 시프트 연산 (SLL, SRL, SRA, SLLI, SRLI, SRAI)

### 목표
비트 시프트 연산을 구현합니다.

### 테스트
```rust
#[test]
fn test_slli() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 1);
    // SLLI x2, x1, 4
    cpu.memory.write32(0x80000000, 0x00409113);
    cpu.step();
    assert_eq!(cpu.read_reg(2), 16);  // 1 << 4 = 16
}

#[test]
fn test_srai() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 0xFFFFFF00_u32);  // -256 in signed
    // SRAI x2, x1, 4
    cpu.memory.write32(0x80000000, 0x40409113);
    cpu.step();
    assert_eq!(cpu.read_reg(2), 0xFFFFFFF0);  // sign preserved
}
```

### 체크리스트
- [ ] SLL (funct3=001, funct7=0)
- [ ] SRL (funct3=101, funct7=0000000)
- [ ] SRA (funct3=101, funct7=0100000)
- [ ] SLLI (funct3=001, imm[11:5]=0)
- [ ] SRLI (funct3=101, imm[11:5]=0)
- [ ] SRAI (funct3=101, imm[11:5]=0100000)
- [ ] 테스트 통과

---

## Step 8: Execute - 비교 연산 (SLT, SLTU, SLTI, SLTIU)

### 목표
비교 연산을 구현합니다.

### 테스트
```rust
#[test]
fn test_slt_signed() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, (-5_i32) as u32);  // -5
    cpu.write_reg(2, 5);
    // SLT x3, x1, x2 (is -5 < 5? yes → 1)
    cpu.memory.write32(0x80000000, 0x0020A0B3);
    cpu.step();
    assert_eq!(cpu.read_reg(3), 1);
}

#[test]
fn test_sltu_unsigned() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, (-5_i32) as u32);  // 0xFFFFFFFB (very large unsigned)
    cpu.write_reg(2, 5);
    // SLTU x3, x1, x2 (is 0xFFFFFFFB < 5? no → 0)
    cpu.memory.write32(0x80000000, 0x0020B0B3);
    cpu.step();
    assert_eq!(cpu.read_reg(3), 0);
}
```

### 체크리스트
- [ ] SLT (funct3=010, funct7=0) - 부호 있는 비교
- [ ] SLTU (funct3=011, funct7=0) - 부호 없는 비교
- [ ] SLTI (funct3=010) - 부호 있는 즉시값 비교
- [ ] SLTIU (funct3=011) - 부호 없는 즉시값 비교
- [ ] 테스트 통과

---

## Step 9: Execute - 로드/스토어 (LW, SW, LB, LH, SB, SH, LBU, LHU)

### 목표
메모리 접근 명령어를 구현합니다.

### 테스트 (기본)
```rust
#[test]
fn test_sw_lw() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 0x80001000);  // base address
    cpu.write_reg(2, 0xDEADBEEF);  // value to store

    // SW x2, 0(x1)
    cpu.memory.write32(0x80000000, 0x0020A023);
    cpu.step();

    // LW x3, 0(x1)
    cpu.memory.write32(0x80000004, 0x0000A183);
    cpu.step();

    assert_eq!(cpu.read_reg(3), 0xDEADBEEF);
}

#[test]
fn test_lb_sign_extend() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 0x80001000);
    cpu.memory.write8(0x80001000, 0x80);  // -128 in signed byte

    // LB x2, 0(x1)
    cpu.memory.write32(0x80000000, 0x00008103);
    cpu.step();

    assert_eq!(cpu.read_reg(2), 0xFFFFFF80);  // sign extended
}

#[test]
fn test_lbu_zero_extend() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 0x80001000);
    cpu.memory.write8(0x80001000, 0x80);

    // LBU x2, 0(x1)
    cpu.memory.write32(0x80000000, 0x0000C103);
    cpu.step();

    assert_eq!(cpu.read_reg(2), 0x00000080);  // zero extended
}
```

### 체크리스트
- [ ] LW (opcode=0000011, funct3=010)
- [ ] LH (funct3=001) - 부호 확장
- [ ] LB (funct3=000) - 부호 확장
- [ ] LHU (funct3=101) - 제로 확장
- [ ] LBU (funct3=100) - 제로 확장
- [ ] SW (opcode=0100011, funct3=010)
- [ ] SH (funct3=001)
- [ ] SB (funct3=000)
- [ ] 테스트 통과

---

## Step 10: Execute - 분기 (BEQ, BNE, BLT, BGE, BLTU, BGEU)

### 목표
조건부 분기를 구현합니다.

### 테스트
```rust
#[test]
fn test_beq_taken() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 42);
    cpu.write_reg(2, 42);
    // BEQ x1, x2, 8
    cpu.memory.write32(0x80000000, 0x00208463);
    cpu.step();
    assert_eq!(cpu.pc, 0x80000008);  // branched
}

#[test]
fn test_beq_not_taken() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 42);
    cpu.write_reg(2, 100);
    // BEQ x1, x2, 8
    cpu.memory.write32(0x80000000, 0x00208463);
    cpu.step();
    assert_eq!(cpu.pc, 0x80000004);  // not branched, PC += 4
}

#[test]
fn test_blt_signed() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, (-10_i32) as u32);  // -10
    cpu.write_reg(2, 5);
    // BLT x1, x2, 16 (is -10 < 5? yes)
    cpu.memory.write32(0x80000000, 0x0020C863);
    cpu.step();
    assert_eq!(cpu.pc, 0x80000010);  // branched
}
```

### 체크리스트
- [ ] BEQ (funct3=000)
- [ ] BNE (funct3=001)
- [ ] BLT (funct3=100) - 부호 있는 비교
- [ ] BGE (funct3=101) - 부호 있는 비교
- [ ] BLTU (funct3=110) - 부호 없는 비교
- [ ] BGEU (funct3=111) - 부호 없는 비교
- [ ] 테스트 통과

---

## Step 11: Execute - 점프 (JAL, JALR)

### 목표
무조건 점프를 구현합니다.

### 테스트
```rust
#[test]
fn test_jal() {
    let mut cpu = Cpu::new();
    // JAL x1, 16
    cpu.memory.write32(0x80000000, 0x010000EF);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0x80000004);  // return address
    assert_eq!(cpu.pc, 0x80000010);           // jumped to PC+16
}

#[test]
fn test_jalr() {
    let mut cpu = Cpu::new();
    cpu.write_reg(1, 0x80001000);
    // JALR x2, x1, 4
    cpu.memory.write32(0x80000000, 0x00408167);
    cpu.step();
    assert_eq!(cpu.read_reg(2), 0x80000004);  // return address
    assert_eq!(cpu.pc, 0x80001004);           // jumped to x1+4
}
```

### 체크리스트
- [ ] JAL (opcode=1101111) - rd = PC+4, PC += imm
- [ ] JALR (opcode=1100111) - rd = PC+4, PC = (rs1+imm) & ~1
- [ ] 테스트 통과

---

## Step 12: Execute - 상위 즉시값 (LUI, AUIPC)

### 목표
상위 20비트 즉시값 명령어를 구현합니다.

### 테스트
```rust
#[test]
fn test_lui() {
    let mut cpu = Cpu::new();
    // LUI x1, 0x12345
    cpu.memory.write32(0x80000000, 0x123450B7);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0x12345000);
}

#[test]
fn test_auipc() {
    let mut cpu = Cpu::new();
    // AUIPC x1, 0x00001
    cpu.memory.write32(0x80000000, 0x00001097);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0x80001000);  // PC + 0x1000
}
```

### 체크리스트
- [ ] LUI (opcode=0110111) - rd = imm << 12
- [ ] AUIPC (opcode=0010111) - rd = PC + (imm << 12)
- [ ] 테스트 통과

---

## Step 13: Execute - 시스템 (ECALL, EBREAK)

### 목표
시스템 명령어를 구현합니다. 베어메탈 1단계에서는 간단히 처리합니다.

### 구현 내용
- ECALL: 프로그램 종료 신호로 사용
- EBREAK: 디버거 브레이크포인트 (일단 무시 또는 정지)

### 테스트
```rust
#[test]
fn test_ecall_halts() {
    let mut cpu = Cpu::new();
    cpu.write_reg(10, 0);  // a0 = 0 (success)
    // ECALL
    cpu.memory.write32(0x80000000, 0x00000073);
    let result = cpu.step();
    assert!(result.is_halt());  // or check cpu.halted flag
}
```

### 체크리스트
- [ ] ECALL 인식 (opcode=1110011, imm=0)
- [ ] EBREAK 인식 (opcode=1110011, imm=1)
- [ ] 종료 상태 반환
- [ ] 테스트 통과

---

## Step 14: 통합 - 프로그램 실행

### 목표
여러 명령어로 이루어진 프로그램을 실행합니다.

### 테스트 프로그램 1: 1부터 10까지 합
```asm
# 1 + 2 + 3 + ... + 10 = 55
# 결과는 a0(x10)에 저장
        addi x10, x0, 0     # sum = 0
        addi x11, x0, 1     # i = 1
        addi x12, x0, 10    # limit = 10
loop:   add  x10, x10, x11  # sum += i
        addi x11, x11, 1    # i++
        ble  x11, x12, loop # if i <= limit, goto loop
        ecall               # exit
```

### 테스트
```rust
#[test]
fn test_sum_1_to_10() {
    let mut cpu = Cpu::new();
    let program: Vec<u32> = vec![
        0x00000513,  // addi x10, x0, 0
        0x00100593,  // addi x11, x0, 1
        0x00A00613,  // addi x12, x0, 10
        0x00B50533,  // add x10, x10, x11
        0x00158593,  // addi x11, x11, 1
        0xFEC5D6E3,  // bge x11, x12, -8 (wait, need ble which is bge with swapped operands)
        0x00000073,  // ecall
    ];
    cpu.load_program(&program);
    cpu.run();
    assert_eq!(cpu.read_reg(10), 55);
}
```

### 체크리스트
- [ ] `Cpu::load_program()` 구현
- [ ] `Cpu::run()` 루프 구현 (ecall까지 실행)
- [ ] 합계 프로그램 테스트 통과
- [ ] 추가 테스트 프로그램 작성 및 통과

---

## 다음 단계 (1단계 완료 후)

1단계를 완료하면 다음을 고려할 수 있습니다:

- **riscv-tests 연동**: 공식 테스트 스위트로 검증
- **ELF 로더**: 실제 컴파일된 바이너리 로드
- **특권 ISA**: CSR, 트랩 핸들링 추가
- **M 확장**: 곱셈/나눗셈 명령어
- **디버거/트레이서**: 명령어별 로깅, 레지스터 덤프

---

## 유용한 참고 자료

- [RISC-V Spec (Volume 1)](https://riscv.org/technical/specifications/) - 명령어 정의
- [RISC-V Green Card](https://www.cl.cam.ac.uk/teaching/1617/ECAD+Arch/files/docs/RISCVGreenCardv8-20151013.pdf) - 빠른 참조용
- [riscv-tests](https://github.com/riscv-software-src/riscv-tests) - 공식 테스트
- [rvemu (Rust)](https://github.com/d0iasm/rvemu) - 참고할 만한 Rust 구현체
