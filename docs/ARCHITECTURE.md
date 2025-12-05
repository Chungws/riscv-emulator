# RV32I 에뮬레이터 아키텍처

## 개요

이 프로젝트는 RISC-V RV32I ISA를 구현하는 학습용 베어메탈 에뮬레이터입니다.

- **언어**: Rust
- **타겟 ISA**: RV32I (기본 정수 명령어셋)
- **방식**: 베어메탈 에뮬레이션
- **테스트**: 단계별 유닛 테스트 → 최종적으로 riscv-tests

---

## RV32I 기본 구조

### 레지스터

| 레지스터 | ABI 이름 | 설명 |
|---------|---------|------|
| x0 | zero | 항상 0 (쓰기 무시) |
| x1 | ra | 리턴 주소 |
| x2 | sp | 스택 포인터 |
| x3 | gp | 글로벌 포인터 |
| x4 | tp | 스레드 포인터 |
| x5-7 | t0-t2 | 임시 레지스터 |
| x8 | s0/fp | 저장 레지스터 / 프레임 포인터 |
| x9 | s1 | 저장 레지스터 |
| x10-11 | a0-a1 | 함수 인자 / 리턴값 |
| x12-17 | a2-a7 | 함수 인자 |
| x18-27 | s2-s11 | 저장 레지스터 |
| x28-31 | t3-t6 | 임시 레지스터 |

- 총 32개 레지스터, 각 32비트
- PC (Program Counter): 현재 실행 중인 명령어 주소

### 명령어 포맷

모든 명령어는 **32비트 고정 길이**입니다.

```
R-type: 레지스터 간 연산
31       25 24   20 19   15 14  12 11    7 6      0
┌─────────┬───────┬───────┬──────┬───────┬────────┐
│ funct7  │  rs2  │  rs1  │funct3│  rd   │ opcode │
│  7bit   │ 5bit  │ 5bit  │ 3bit │ 5bit  │  7bit  │
└─────────┴───────┴───────┴──────┴───────┴────────┘

I-type: 즉시값 연산, 로드
31          20 19   15 14  12 11    7 6      0
┌─────────────┬───────┬──────┬───────┬────────┐
│  imm[11:0]  │  rs1  │funct3│  rd   │ opcode │
│    12bit    │ 5bit  │ 3bit │ 5bit  │  7bit  │
└─────────────┴───────┴──────┴───────┴────────┘

S-type: 스토어
31       25 24   20 19   15 14  12 11    7 6      0
┌─────────┬───────┬───────┬──────┬───────┬────────┐
│imm[11:5]│  rs2  │  rs1  │funct3│imm[4:0]│ opcode │
│  7bit   │ 5bit  │ 5bit  │ 3bit │  5bit  │  7bit  │
└─────────┴───────┴───────┴──────┴───────┴────────┘

B-type: 분기
31    30     25 24  20 19  15 14 12 11     8 7    6      0
┌────┬────────┬──────┬──────┬─────┬────────┬────┬────────┐
│[12]│[10:5]  │ rs2  │ rs1  │funct3│[4:1]  │[11]│ opcode │
└────┴────────┴──────┴──────┴─────┴────────┴────┴────────┘

U-type: 상위 즉시값
31                  12 11    7 6      0
┌─────────────────────┬───────┬────────┐
│     imm[31:12]      │  rd   │ opcode │
│       20bit         │ 5bit  │  7bit  │
└─────────────────────┴───────┴────────┘

J-type: 점프
31    30       21 20   19        12 11   7 6      0
┌────┬──────────┬────┬────────────┬──────┬────────┐
│[20]│ [10:1]   │[11]│  [19:12]   │  rd  │ opcode │
└────┴──────────┴────┴────────────┴──────┴────────┘
```

### Opcode 맵

| opcode (6:0) | 포맷 | 명령어 |
|--------------|------|--------|
| 0110011 | R | add, sub, and, or, xor, sll, srl, sra, slt, sltu |
| 0010011 | I | addi, andi, ori, xori, slti, sltiu, slli, srli, srai |
| 0000011 | I | lb, lh, lw, lbu, lhu |
| 0100011 | S | sb, sh, sw |
| 1100011 | B | beq, bne, blt, bge, bltu, bgeu |
| 1101111 | J | jal |
| 1100111 | I | jalr |
| 0110111 | U | lui |
| 0010111 | U | auipc |
| 1110011 | I | ecall, ebreak |

### 명령어 세부 구분 (funct3, funct7)

같은 opcode 내에서 funct3, funct7으로 명령어를 구분합니다.

**R-type (opcode = 0110011)**

| funct3 | funct7 | 명령어 | 동작 |
|--------|--------|--------|------|
| 000 | 0000000 | ADD | rd = rs1 + rs2 |
| 000 | 0100000 | SUB | rd = rs1 - rs2 |
| 001 | 0000000 | SLL | rd = rs1 << rs2 |
| 010 | 0000000 | SLT | rd = (rs1 < rs2) ? 1 : 0 (signed) |
| 011 | 0000000 | SLTU | rd = (rs1 < rs2) ? 1 : 0 (unsigned) |
| 100 | 0000000 | XOR | rd = rs1 ^ rs2 |
| 101 | 0000000 | SRL | rd = rs1 >> rs2 (logical) |
| 101 | 0100000 | SRA | rd = rs1 >> rs2 (arithmetic) |
| 110 | 0000000 | OR | rd = rs1 \| rs2 |
| 111 | 0000000 | AND | rd = rs1 & rs2 |

**I-type 산술 (opcode = 0010011)**

| funct3 | imm[11:5] | 명령어 | 동작 |
|--------|-----------|--------|------|
| 000 | - | ADDI | rd = rs1 + imm |
| 010 | - | SLTI | rd = (rs1 < imm) ? 1 : 0 (signed) |
| 011 | - | SLTIU | rd = (rs1 < imm) ? 1 : 0 (unsigned) |
| 100 | - | XORI | rd = rs1 ^ imm |
| 110 | - | ORI | rd = rs1 \| imm |
| 111 | - | ANDI | rd = rs1 & imm |
| 001 | 0000000 | SLLI | rd = rs1 << imm[4:0] |
| 101 | 0000000 | SRLI | rd = rs1 >> imm[4:0] (logical) |
| 101 | 0100000 | SRAI | rd = rs1 >> imm[4:0] (arithmetic) |

**I-type 로드 (opcode = 0000011)**

| funct3 | 명령어 | 동작 |
|--------|--------|------|
| 000 | LB | rd = sign_ext(mem[rs1+imm][7:0]) |
| 001 | LH | rd = sign_ext(mem[rs1+imm][15:0]) |
| 010 | LW | rd = mem[rs1+imm][31:0] |
| 100 | LBU | rd = zero_ext(mem[rs1+imm][7:0]) |
| 101 | LHU | rd = zero_ext(mem[rs1+imm][15:0]) |

**S-type 스토어 (opcode = 0100011)**

| funct3 | 명령어 | 동작 |
|--------|--------|------|
| 000 | SB | mem[rs1+imm][7:0] = rs2[7:0] |
| 001 | SH | mem[rs1+imm][15:0] = rs2[15:0] |
| 010 | SW | mem[rs1+imm][31:0] = rs2 |

**B-type 분기 (opcode = 1100011)**

| funct3 | 명령어 | 동작 |
|--------|--------|------|
| 000 | BEQ | if (rs1 == rs2) PC += imm |
| 001 | BNE | if (rs1 != rs2) PC += imm |
| 100 | BLT | if (rs1 < rs2) PC += imm (signed) |
| 101 | BGE | if (rs1 >= rs2) PC += imm (signed) |
| 110 | BLTU | if (rs1 < rs2) PC += imm (unsigned) |
| 111 | BGEU | if (rs1 >= rs2) PC += imm (unsigned) |

**시스템 (opcode = 1110011)**

| imm[11:0] | 명령어 |
|-----------|--------|
| 0 | ECALL |
| 1 | EBREAK |

---

## RV32I 명령어 목록 (37개)

### 산술 연산

| 명령어 | 형식 | 설명 |
|--------|------|------|
| ADD | R | rd = rs1 + rs2 |
| SUB | R | rd = rs1 - rs2 |
| ADDI | I | rd = rs1 + imm |

### 논리 연산

| 명령어 | 형식 | 설명 |
|--------|------|------|
| AND | R | rd = rs1 & rs2 |
| OR | R | rd = rs1 \| rs2 |
| XOR | R | rd = rs1 ^ rs2 |
| ANDI | I | rd = rs1 & imm |
| ORI | I | rd = rs1 \| imm |
| XORI | I | rd = rs1 ^ imm |

### 시프트 연산

| 명령어 | 형식 | 설명 |
|--------|------|------|
| SLL | R | rd = rs1 << rs2 (논리 좌측) |
| SRL | R | rd = rs1 >> rs2 (논리 우측) |
| SRA | R | rd = rs1 >> rs2 (산술 우측) |
| SLLI | I | rd = rs1 << imm |
| SRLI | I | rd = rs1 >> imm (논리) |
| SRAI | I | rd = rs1 >> imm (산술) |

### 비교 연산

| 명령어 | 형식 | 설명 |
|--------|------|------|
| SLT | R | rd = (rs1 < rs2) ? 1 : 0 (부호있음) |
| SLTU | R | rd = (rs1 < rs2) ? 1 : 0 (부호없음) |
| SLTI | I | rd = (rs1 < imm) ? 1 : 0 (부호있음) |
| SLTIU | I | rd = (rs1 < imm) ? 1 : 0 (부호없음) |

### 로드 (메모리 → 레지스터)

| 명령어 | 형식 | 설명 |
|--------|------|------|
| LB | I | rd = sign_extend(mem[rs1+imm][7:0]) |
| LH | I | rd = sign_extend(mem[rs1+imm][15:0]) |
| LW | I | rd = mem[rs1+imm][31:0] |
| LBU | I | rd = zero_extend(mem[rs1+imm][7:0]) |
| LHU | I | rd = zero_extend(mem[rs1+imm][15:0]) |

### 스토어 (레지스터 → 메모리)

| 명령어 | 형식 | 설명 |
|--------|------|------|
| SB | S | mem[rs1+imm][7:0] = rs2[7:0] |
| SH | S | mem[rs1+imm][15:0] = rs2[15:0] |
| SW | S | mem[rs1+imm][31:0] = rs2[31:0] |

### 분기 (조건부 점프)

| 명령어 | 형식 | 설명 |
|--------|------|------|
| BEQ | B | if (rs1 == rs2) PC += imm |
| BNE | B | if (rs1 != rs2) PC += imm |
| BLT | B | if (rs1 < rs2) PC += imm (부호있음) |
| BGE | B | if (rs1 >= rs2) PC += imm (부호있음) |
| BLTU | B | if (rs1 < rs2) PC += imm (부호없음) |
| BGEU | B | if (rs1 >= rs2) PC += imm (부호없음) |

### 점프

| 명령어 | 형식 | 설명 |
|--------|------|------|
| JAL | J | rd = PC+4; PC += imm |
| JALR | I | rd = PC+4; PC = (rs1+imm) & ~1 |

### 상위 즉시값

| 명령어 | 형식 | 설명 |
|--------|------|------|
| LUI | U | rd = imm << 12 |
| AUIPC | U | rd = PC + (imm << 12) |

### 시스템

| 명령어 | 형식 | 설명 |
|--------|------|------|
| ECALL | I | 환경 호출 (시스템 콜) |
| EBREAK | I | 디버거 브레이크포인트 |

---

## 메모리 레이아웃

```
0x00000000 ┌─────────────────┐
           │    (Reserved)   │
0x80000000 ├─────────────────┤  ← DRAM_BASE (프로그램 시작점)
           │                 │
           │      RAM        │
           │                 │
0x88000000 └─────────────────┘  ← DRAM_BASE + DRAM_SIZE (128MB)
```

- 프로그램은 `0x80000000`에 로드됨 (QEMU virt 머신 기준)
- 초기 PC = `0x80000000`

---

## 실행 사이클

```
┌─────────────────────────────────────────────────┐
│                  CPU Loop                       │
│                                                 │
│  ┌─────────┐   ┌──────────┐   ┌───────────┐   │
│  │  Fetch  │ → │  Decode  │ → │  Execute  │   │
│  └─────────┘   └──────────┘   └───────────┘   │
│       │                             │          │
│       │         PC Update           │          │
│       └─────────────────────────────┘          │
│                                                 │
└─────────────────────────────────────────────────┘

1. Fetch:   instruction = memory.read32(PC)
2. Decode:  명령어 비트 필드 파싱 (opcode, rd, rs1, rs2, imm, funct3, funct7)
3. Execute: 연산 수행, 레지스터/메모리 갱신
4. PC 업데이트: 분기/점프가 아니면 PC += 4
```

---

## 프로젝트 구조

```
riscv-emulator/
├── Cargo.toml
├── docs/
│   ├── ARCHITECTURE.md     # 이 문서
│   └── IMPLEMENTATION.md   # 단계별 구현 가이드
├── src/
│   ├── main.rs             # 진입점, CLI
│   ├── lib.rs              # 라이브러리 루트
│   ├── cpu.rs              # CPU 구조체, 실행 루프
│   ├── memory.rs           # 메모리 시스템
│   ├── instruction.rs      # 명령어 enum 정의
│   └── decoder.rs          # 명령어 디코딩
└── tests/
    └── integration/        # 통합 테스트
```
