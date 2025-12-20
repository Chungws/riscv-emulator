# A Extension 구현 가이드

RISC-V A Extension (Atomic) 구현 가이드.

**설계 방향**: 멀티코어 확장 가능한 구조

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

## 2. 멀티코어 확장 가능 아키텍처

### 2.1 설계 원칙

싱글코어로 시작하되, 멀티코어 확장이 용이한 구조로 설계.

**핵심**: 예약(reservation)을 CPU가 아닌 Bus에서 관리

### 2.2 구조 변경

```
현재 구조:
┌─────┐     ┌─────┐
│ CPU │────▶│ Bus │────▶ Memory
└─────┘     └─────┘

멀티코어 확장 가능 구조:
┌─────┐     ┌─────────────────────┐
│ CPU │────▶│        Bus          │
│ id=0│     │  ┌───────────────┐  │
└─────┘     │  │ Reservations  │  │────▶ Memory
            │  │ {hart_id: addr}│  │
            │  └───────────────┘  │
            └─────────────────────┘

멀티코어 확장 시:
┌─────┐
│ CPU │──┐  ┌─────────────────────┐
│ id=0│  │  │        Bus          │
└─────┘  ├─▶│  ┌───────────────┐  │
┌─────┐  │  │  │ Reservations  │  │────▶ Memory
│ CPU │──┘  │  │ {0: addr_a,   │  │
│ id=1│     │  │  1: addr_b}   │  │
└─────┘     │  └───────────────┘  │
            └─────────────────────┘
```

### 2.3 예약 무효화 (핵심!)

```
Hart 0: LR.W a0, (0x1000)     # reservations[0] = 0x1000
Hart 1: SW zero, (0x1000)     # 쓰기 시 0x1000 예약한 hart들 무효화!
Hart 0: SC.W a1, a0, (0x1000) # 예약 없음 → 실패 (a1 = 1)
```

**규칙**: 어떤 hart든 메모리에 쓸 때, 해당 주소를 예약한 모든 hart의 예약 무효화

### 2.4 왜 이 설계인가?

| 싱글코어 전용 | 멀티코어 확장 가능 |
|--------------|-------------------|
| CPU 내부에 예약 저장 | Bus에 예약 저장 |
| 멀티코어 시 대규모 리팩토링 | 새 CPU 추가만 하면 됨 |
| 실제 HW 동작과 다름 | 실제 HW 동작과 유사 |

---

## 3. LR/SC 명령어

### 3.1 Load-Reserved (LR)

| 명령어 | funct5 | 설명 |
|--------|--------|------|
| LR.W | 0x02 | 32비트 load + 예약 |
| LR.D | 0x02 | 64비트 load + 예약 |

**동작**:
1. 메모리에서 값 읽기
2. Bus에 예약 등록: `reservations[hart_id] = addr`

### 3.2 Store-Conditional (SC)

| 명령어 | funct5 | 설명 |
|--------|--------|------|
| SC.W | 0x03 | 32비트 조건부 store |
| SC.D | 0x03 | 64비트 조건부 store |

**동작**:
1. Bus에서 예약 확인: `reservations[hart_id] == addr?`
2. 예약 유효: store 수행, rd = 0 (성공)
3. 예약 무효: store 안 함, rd = 1 (실패)
4. 예약 클리어: `reservations.remove(hart_id)`

### 3.3 예약 무효화 조건

- 다른 hart가 해당 주소에 write
- SC 명령어 실행 (성공/실패 무관)
- 인터럽트/예외 발생 (선택적)

---

## 4. AMO 명령어

### 4.1 AMO 명령어 목록

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

### 4.2 AMO 동작

모든 AMO 명령어는:
1. 메모리에서 값 읽기 (rd에 저장)
2. rs2와 연산 수행
3. 결과를 메모리에 쓰기
4. **해당 주소의 예약 무효화** (쓰기이므로)

**원자적**: 1-3이 다른 hart에게 분리되어 보이지 않음

### 4.3 W vs D suffix

| suffix | funct3 | 비트 폭 |
|--------|--------|---------|
| .W | 0x2 | 32비트 (sign-extend to 64) |
| .D | 0x3 | 64비트 |

---

## 5. 구현 단계

### Step 0: 아키텍처 준비 ✅

**목표**: 멀티코어 확장 가능한 예약 시스템 구축

- [x] CPU에 `hart_id: u64` 추가
- [x] Bus에 예약 테이블 추가
  ```
  reservations: HashMap<u64, u64>  // hart_id -> reserved_addr
  ```
- [x] Bus에 예약 API 추가
  - `reserve(hart_id, addr)`: 예약 등록
  - `check_reservation(hart_id, addr) -> bool`: 예약 확인
  - `clear_reservation(hart_id)`: 예약 클리어
  - `invalidate_reservations(addr)`: 해당 주소 예약 무효화
- [x] 메모리 쓰기 시 `invalidate_reservations(addr)` 호출

**검증**: 기존 테스트 통과 확인 ✅

---

### Step 1: LR/SC 구현

**목표**: 예약 기반 원자적 읽기-수정-쓰기

- [ ] AMO opcode (0x2F) 핸들러 추가
- [ ] decoder에 funct5 추가
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

## 6. Bus 예약 API 상세

### 6.1 데이터 구조

```
Bus {
    memory: Memory,
    uart: Uart,
    clint: Clint,
    reservations: HashMap<u64, u64>,  // hart_id -> addr
}
```

### 6.2 API

```
reserve(hart_id: u64, addr: u64)
  - reservations.insert(hart_id, addr)
  - 기존 예약 덮어쓰기 (hart당 1개만 유지)

check_reservation(hart_id: u64, addr: u64) -> bool
  - reservations.get(hart_id) == Some(addr)

clear_reservation(hart_id: u64)
  - reservations.remove(hart_id)

invalidate_reservations(addr: u64)
  - reservations에서 value == addr인 모든 항목 제거
  - 여러 hart가 같은 주소 예약했을 수 있음
```

### 6.3 메모리 쓰기 통합

```
write8/16/32/64(addr, value) {
    self.invalidate_reservations(addr);  // 핵심!
    // 기존 쓰기 로직...
}
```

---

## 7. 테스트 케이스

### LR/SC 기본
```
LR.W a0, (a1)      # a0 = mem[a1], reserve a1
ADDI a0, a0, 1     # a0++
SC.W a2, a0, (a1)  # if reserved: mem[a1] = a0, a2 = 0
                   # else: a2 = 1
```

### LR/SC 실패 케이스
```
LR.W a0, (a1)      # 예약
SW zero, (a1)      # 같은 주소에 쓰기 → 예약 무효화
SC.W a2, a0, (a1)  # 실패 (a2 = 1)
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

## 8. 참고 자료

- RISC-V Specification Chapter 8: "A" Standard Extension
- https://riscv.org/specifications/

---

## 9. 구현 우선순위

xv6 실행을 위한 최소 구현:
1. **Step 0**: 예약 시스템 (필수)
2. **AMOSWAP.W** (필수) - spinlock
3. **LR.W / SC.W** (필수) - CAS 연산
4. 나머지는 필요시 추가

**aq/rl 비트**: 싱글코어에서는 무시 가능. 멀티코어 시 메모리 배리어로 구현.
