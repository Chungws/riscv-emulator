# M Extension 구현 가이드

RISC-V M Extension (곱셈/나눗셈) 구현 가이드.

---

## 1. M Extension 개요

### 1.1 M Extension이란?

M Extension은 RISC-V의 표준 곱셈/나눗셈 확장.
- 곱셈: MUL, MULH, MULHSU, MULHU
- 나눗셈: DIV, DIVU
- 나머지: REM, REMU

### 1.2 왜 필요한가?

- xv6-riscv가 rv64g (= rv64imafd)로 빌드됨
- 곱셈/나눗셈 없이는 실행 불가

### 1.3 명령어 인코딩

모든 M Extension 명령어:
- opcode: `0x33` (OP) 또는 `0x3B` (OP-32)
- funct7: `0x01`

---

## 2. RV64M 명령어 (64비트)

### 2.1 곱셈 명령어

| 명령어 | funct3 | funct7 | 설명 |
|--------|--------|--------|------|
| MUL    | 0x0    | 0x01   | rd = (rs1 × rs2)[63:0] |
| MULH   | 0x1    | 0x01   | rd = (rs1 × rs2)[127:64] (signed × signed) |
| MULHSU | 0x2    | 0x01   | rd = (rs1 × rs2)[127:64] (signed × unsigned) |
| MULHU  | 0x3    | 0x01   | rd = (rs1 × rs2)[127:64] (unsigned × unsigned) |

### 2.2 나눗셈 명령어

| 명령어 | funct3 | funct7 | 설명 |
|--------|--------|--------|------|
| DIV    | 0x4    | 0x01   | rd = rs1 ÷ rs2 (signed) |
| DIVU   | 0x5    | 0x01   | rd = rs1 ÷ rs2 (unsigned) |
| REM    | 0x6    | 0x01   | rd = rs1 % rs2 (signed) |
| REMU   | 0x7    | 0x01   | rd = rs1 % rs2 (unsigned) |

---

## 3. RV64M 명령어 (32비트 연산)

opcode: `0x3B` (OP-32)

| 명령어 | funct3 | funct7 | 설명 |
|--------|--------|--------|------|
| MULW   | 0x0    | 0x01   | rd = sign_extend((rs1 × rs2)[31:0]) |
| DIVW   | 0x4    | 0x01   | rd = sign_extend(rs1[31:0] ÷ rs2[31:0]) |
| DIVUW  | 0x5    | 0x01   | rd = sign_extend(rs1[31:0] ÷ rs2[31:0]) (unsigned) |
| REMW   | 0x6    | 0x01   | rd = sign_extend(rs1[31:0] % rs2[31:0]) |
| REMUW  | 0x7    | 0x01   | rd = sign_extend(rs1[31:0] % rs2[31:0]) (unsigned) |

---

## 4. 특수 케이스 처리

### 4.1 0으로 나누기

| 연산 | 결과 |
|------|------|
| DIV by 0 | -1 (all bits set) |
| DIVU by 0 | 2^64 - 1 (all bits set) |
| REM by 0 | 피제수 (rs1) |
| REMU by 0 | 피제수 (rs1) |

### 4.2 오버플로우

| 연산 | 조건 | 결과 |
|------|------|------|
| DIV | -2^63 ÷ -1 | -2^63 |
| REM | -2^63 % -1 | 0 |

---

## 5. 구현 단계

### Step 1: MUL 구현 ✅

**목표**: 기본 곱셈 명령어 구현

- [x] OP (0x33)에서 funct7=0x01 분기 추가
- [x] MUL (funct3=0x0): 하위 64비트 결과

**검증**: MUL 테스트 (6개 통과)

---

### Step 2: MULH, MULHSU, MULHU 구현 ✅

**목표**: 상위 64비트 곱셈 구현

- [x] MULH (funct3=0x1): signed × signed
- [x] MULHSU (funct3=0x2): signed × unsigned
- [x] MULHU (funct3=0x3): unsigned × unsigned

**힌트**: 128비트 곱셈 필요 (i128 또는 u128 사용)

**검증**: MULH, MULHSU, MULHU 테스트 (11개 통과)

---

### Step 3: DIV, DIVU 구현 ✅

**목표**: 나눗셈 구현

- [x] DIV (funct3=0x4): signed 나눗셈
- [x] DIVU (funct3=0x5): unsigned 나눗셈
- [x] 0으로 나누기 처리
- [x] 오버플로우 처리 (DIV만)

**검증**: DIV, DIVU 테스트 (10개 통과)

---

### Step 4: REM, REMU 구현 ✅

**목표**: 나머지 연산 구현

- [x] REM (funct3=0x6): signed 나머지
- [x] REMU (funct3=0x7): unsigned 나머지
- [x] 0으로 나누기 처리
- [x] 오버플로우 처리 (REM만)

**검증**: REM, REMU 테스트 (10개 통과)

---

### Step 5: 32비트 연산 (W suffix) ✅

**목표**: OP-32 (0x3B)에 M extension 추가

- [x] MULW (funct3=0x0)
- [x] DIVW (funct3=0x4)
- [x] DIVUW (funct3=0x5)
- [x] REMW (funct3=0x6)
- [x] REMUW (funct3=0x7)

**검증**: 32비트 연산 테스트 (21개 통과, sign-extend 엣지케이스 포함)

---

### Step 6: xv6 테스트

**목표**: xv6 커널로 통합 테스트

- [ ] xv6 커널 실행
- [ ] 다음 에러 확인 및 대응

**검증**: xv6가 더 진행되는지 확인

---

## 6. 참고 자료

- RISC-V Specification Chapter 7: "M" Standard Extension
- https://riscv.org/specifications/

---

## 7. 테스트 케이스

### MUL
- 3 × 7 = 21
- -1 × 5 = -5
- 0x7FFFFFFFFFFFFFFF × 2 (오버플로우)

### DIV/REM
- 20 ÷ 3 = 6, 20 % 3 = 2
- -20 ÷ 3 = -6, -20 % 3 = -2
- 7 ÷ 0 = -1, 7 % 0 = 7
- MIN_I64 ÷ -1 = MIN_I64, MIN_I64 % -1 = 0
