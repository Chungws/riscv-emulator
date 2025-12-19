# A Extension 구현 가이드

RISC-V A Extension (Atomic) 구현 가이드.

---

## 1. A Extension 개요

### 1.1 A Extension이란?

A Extension은 RISC-V의 원자적 메모리 연산 확장.
- LR/SC: Load-Reserved / Store-Conditional
- AMO: Atomic Memory Operations

### 1.2 왜 필요한가?

- 멀티코어 동기화 (spinlock, mutex)
- xv6의 `acquire()` / `release()` 함수가 사용
- 싱글코어에서도 인터럽트 동시성 처리에 사용

### 1.3 명령어 인코딩

모든 A Extension 명령어:
- opcode: `0x2F` (AMO)
- funct3: `0x2` (32비트) 또는 `0x3` (64비트)

```
31    27 26 25 24   20 19   15 14  12 11   7 6      0
[funct5][aq][rl][ rs2 ][ rs1 ][funct3][ rd  ][ opcode]
```

- **aq** (acquire): 이후 메모리 연산이 이 연산 이전으로 재배치되지 않음
- **rl** (release): 이전 메모리 연산이 이 연산 이후로 재배치되지 않음

---

## 2. LR/SC 명령어

### 2.1 Load-Reserved (LR)

| 명령어 | funct5 | 설명 |
|--------|--------|------|
| LR.W | 0x02 | 32비트 load + 예약 |
| LR.D | 0x02 | 64비트 load + 예약 |

**동작**:
1. 메모리에서 값 읽기
2. 해당 주소를 "예약" 상태로 마킹

### 2.2 Store-Conditional (SC)

| 명령어 | funct5 | 설명 |
|--------|--------|------|
| SC.W | 0x03 | 32비트 조건부 store |
| SC.D | 0x03 | 64비트 조건부 store |

**동작**:
1. 예약이 유효하면: store 수행, rd = 0 (성공)
2. 예약이 무효하면: store 안 함, rd = 1 (실패)

**예약 무효화 조건**:
- 다른 hart가 해당 주소에 write
- SC 명령어 실행 (성공/실패 무관)
- 인터럽트/예외 발생 (선택적)

### 2.3 싱글코어 구현

싱글코어에서는 간단하게 구현 가능:
```
reservation_addr: Option<u64>

LR:
  reservation_addr = Some(addr)
  return memory[addr]

SC:
  if reservation_addr == Some(addr):
    memory[addr] = value
    reservation_addr = None
    return 0  // 성공
  else:
    return 1  // 실패
```

---

## 3. AMO 명령어

### 3.1 AMO 명령어 목록

| 명령어 | funct5 | 연산 |
|--------|--------|------|
| AMOSWAP | 0x01 | swap: rd = mem; mem = rs2 |
| AMOADD | 0x00 | add: rd = mem; mem = mem + rs2 |
| AMOXOR | 0x04 | xor: rd = mem; mem = mem ^ rs2 |
| AMOAND | 0x0C | and: rd = mem; mem = mem & rs2 |
| AMOOR | 0x08 | or: rd = mem; mem = mem \| rs2 |
| AMOMIN | 0x10 | min (signed): rd = mem; mem = min(mem, rs2) |
| AMOMAX | 0x14 | max (signed): rd = mem; mem = max(mem, rs2) |
| AMOMINU | 0x18 | min (unsigned) |
| AMOMAXU | 0x1C | max (unsigned) |

### 3.2 AMO 동작

모든 AMO 명령어는:
1. 메모리에서 값 읽기 (rd에 저장)
2. rs2와 연산 수행
3. 결과를 메모리에 쓰기

**원자적**: 1-3이 다른 hart에게 분리되어 보이지 않음

### 3.3 W vs D suffix

| suffix | funct3 | 비트 폭 |
|--------|--------|---------|
| .W | 0x2 | 32비트 (sign-extend to 64) |
| .D | 0x3 | 64비트 |

---

## 4. 구현 단계

### Step 1: LR/SC 구현

**목표**: 기본 예약 메커니즘 구현

- [ ] CPU에 `reservation_addr: Option<u64>` 추가
- [ ] LR.W: 32비트 load + 예약
- [ ] LR.D: 64비트 load + 예약
- [ ] SC.W: 32비트 조건부 store
- [ ] SC.D: 64비트 조건부 store

**검증**: LR/SC 테스트

---

### Step 2: AMOSWAP 구현

**목표**: 스핀락의 핵심 명령어

- [ ] AMOSWAP.W
- [ ] AMOSWAP.D

xv6의 `acquire()`:
```c
while(__sync_lock_test_and_set(&lk->locked, 1) != 0)
  ;
```
이것이 `amoswap.w` 명령어로 컴파일됨.

**검증**: AMOSWAP 테스트

---

### Step 3: 산술 AMO 구현

**목표**: 원자적 산술 연산

- [ ] AMOADD.W / AMOADD.D
- [ ] AMOAND.W / AMOAND.D
- [ ] AMOOR.W / AMOOR.D
- [ ] AMOXOR.W / AMOXOR.D

**검증**: 산술 AMO 테스트

---

### Step 4: 비교 AMO 구현

**목표**: 원자적 min/max

- [ ] AMOMIN.W / AMOMIN.D
- [ ] AMOMAX.W / AMOMAX.D
- [ ] AMOMINU.W / AMOMINU.D
- [ ] AMOMAXU.W / AMOMAXU.D

**검증**: 비교 AMO 테스트

---

### Step 5: xv6 테스트

**목표**: xv6 커널로 통합 테스트

- [ ] xv6 커널 실행
- [ ] 다음 에러 확인 및 대응

**검증**: xv6가 더 진행되는지 확인

---

## 5. 참고 자료

- RISC-V Specification Chapter 8: "A" Standard Extension
- https://riscv.org/specifications/

---

## 6. 테스트 케이스

### LR/SC
```
LR.W a0, (a1)      # a0 = mem[a1], reserve a1
ADDI a0, a0, 1     # a0++
SC.W a2, a0, (a1)  # if reserved: mem[a1] = a0, a2 = 0
                   # else: a2 = 1
```

### AMOSWAP (spinlock)
```
li t0, 1
amoswap.w.aq t1, t0, (a0)  # t1 = old, mem[a0] = 1
bnez t1, spin              # if old != 0, retry
```

### AMOADD
```
li t0, 5
amoadd.w t1, t0, (a0)  # t1 = old, mem[a0] = old + 5
```

---

## 7. 구현 우선순위

xv6 실행을 위한 최소 구현:
1. **AMOSWAP.W** (필수) - spinlock
2. **LR.W / SC.W** (필수) - CAS 연산
3. 나머지는 필요시 추가

싱글코어에서는 aq/rl 비트 무시 가능 (메모리 순서 보장 불필요).
