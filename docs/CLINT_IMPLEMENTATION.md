# CLINT (Core Local Interruptor) 구현 가이드

RISC-V 타이머 및 소프트웨어 인터럽트 구현을 위한 단계별 가이드.

---

## 1. CLINT 개요

### 1.1 CLINT란?

CLINT는 RISC-V의 코어 로컬 인터럽트 컨트롤러로, 두 가지 기능을 담당:
- **타이머 인터럽트**: mtime과 mtimecmp 비교
- **소프트웨어 인터럽트**: msip 레지스터

### 1.2 왜 필요한가?

- OS 스케줄러의 타임슬라이스 구현
- 주기적인 작업 처리 (틱 카운터)
- xv6는 타이머 인터럽트로 프로세스 전환

### 1.3 QEMU virt 머신 기준 주소

```
CLINT_BASE = 0x200_0000
CLINT_SIZE = 0x10000 (64KB)
```

---

## 2. 메모리 맵 레지스터

### 2.1 레지스터 레이아웃

| 오프셋 | 이름 | 크기 | 설명 |
|--------|------|------|------|
| 0x0000 | msip | 4바이트 | 소프트웨어 인터럽트 펜딩 |
| 0x4000 | mtimecmp | 8바이트 | 타이머 비교값 |
| 0xBFF8 | mtime | 8바이트 | 현재 타이머 값 |

### 2.2 msip (Machine Software Interrupt Pending)

- 비트 0만 사용 (나머지는 0)
- 1을 쓰면 소프트웨어 인터럽트 발생
- 0을 쓰면 인터럽트 클리어

### 2.3 mtime (Machine Time)

- 64비트 읽기 전용 카운터
- 일정 주기로 증가 (실제 하드웨어는 고정 주파수)
- 에뮬레이터에서는 명령어 실행마다 또는 별도 주기로 증가

### 2.4 mtimecmp (Machine Time Compare)

- 64비트 읽기/쓰기
- `mtime >= mtimecmp`이면 타이머 인터럽트 발생
- 새 값을 쓰면 인터럽트 조건 재평가

---

## 3. 인터럽트 메커니즘

### 3.1 타이머 인터럽트 발생 조건

```
mtime >= mtimecmp
AND mstatus.MIE = 1 (인터럽트 전역 활성화)
AND mie.MTIE = 1 (타이머 인터럽트 활성화)
```

### 3.2 인터럽트 처리 흐름

1. 매 사이클(또는 스텝)마다 `mtime` 증가
2. `mtime >= mtimecmp` 체크
3. 조건 만족 시 `mip.MTIP = 1` 설정
4. 인터럽트 활성화 상태면 트랩 발생
5. `mcause = 0x8000000000000007` (타이머 인터럽트)
6. 핸들러에서 `mtimecmp`를 다음 값으로 업데이트

### 3.3 관련 CSR 비트

**mie (Machine Interrupt Enable):**
- 비트 3: MSIE (소프트웨어 인터럽트)
- 비트 7: MTIE (타이머 인터럽트)
- 비트 11: MEIE (외부 인터럽트)

**mip (Machine Interrupt Pending):**
- 비트 3: MSIP
- 비트 7: MTIP
- 비트 11: MEIP

**mcause 인터럽트 코드:**
- 3: 소프트웨어 인터럽트
- 7: 타이머 인터럽트
- 11: 외부 인터럽트

---

## 4. 구현 단계

### Step 1: CSR 확장

**목표**: 인터럽트 관련 CSR 추가

- [x] `mie` (0x304) 레지스터 상수 추가
- [x] `mip` (0x344) 레지스터 상수 추가
- [x] 비트 마스크 상수 정의 (MTIE, MTIP 등)

**검증**: CSR 읽기/쓰기 테스트

---

### Step 2: CLINT 디바이스 생성

**목표**: CLINT 메모리 맵 디바이스 구현

- [x] `src/devices/clint.rs` 파일 생성
- [x] CLINT 구조체 정의 (mtime, mtimecmp, msip 필드)
- [x] `new()` 함수: 초기값 설정
- [x] `read64(offset)`: mtime, mtimecmp 읽기
- [x] `write64(offset, value)`: mtimecmp 쓰기
- [x] `read32(offset)`: msip 읽기
- [x] `write32(offset, value)`: msip 쓰기

**검증**: 레지스터 읽기/쓰기 단위 테스트 ✅

---

### Step 3: Bus에 CLINT 연결

**목표**: 메모리 맵에 CLINT 추가

- [x] Bus 구조체에 CLINT 필드 추가
- [x] 주소 범위 상수 정의 (0x200_0000 ~ 0x200_FFFF)
- [x] `read32`, `read64`에서 CLINT 주소 분기
- [x] `write32`, `write64`에서 CLINT 주소 분기

**검증**: Bus를 통한 CLINT 접근 테스트 ✅

---

### Step 4: 시간 증가 구현

**목표**: mtime 카운터 증가 로직

- [x] CLINT에 `tick()` 메서드 추가
- [x] CPU의 `step()` 에서 `tick()` 호출
- [x] mtime 값 1씩 증가

**검증**: 여러 스텝 후 mtime 값 확인 ✅

---

### Step 5: 인터럽트 펜딩 체크

**목표**: 타이머 인터럽트 조건 확인

- [x] CLINT에 `check_timer_interrupt()` 메서드 추가
- [x] `mtime >= mtimecmp` 비교
- [x] 결과 반환 (bool)

**검증**: mtimecmp 설정 후 조건 확인 테스트 ✅

---

### Step 6: 인터럽트 처리 통합

**목표**: CPU에서 인터럽트 처리

- [x] CPU에 `check_pending_interrupts()` 메서드 추가
- [x] mip.MTIP 업데이트 로직 (CLINT 타이머 상태 반영)
- [x] mip.MSIP 업데이트 로직 (CLINT msip 상태 반영)
- [x] 인터럽트 활성화 조건 확인 (mstatus.MIE && mie.MTIE/MSIE)
- [x] 조건 만족 시 트랩 호출 (cause = 인터럽트 비트 | 7 또는 3)
- [x] `step()` 시작 시 인터럽트 체크
- [x] 소프트웨어 인터럽트가 타이머보다 우선순위 높음

**검증**: 타이머 인터럽트 발생 및 핸들러 진입 테스트 ✅

---

### Step 7: 인터럽트 클리어

**목표**: 인터럽트 클리어 메커니즘

- [x] mtimecmp에 새 값 쓰면 인터럽트 조건 재평가
- [x] `mtime < mtimecmp`이면 인터럽트 클리어

**검증**: 인터럽트 핸들러에서 mtimecmp 업데이트 후 복귀 테스트 ✅

---

### Step 8: 통합 테스트

**목표**: 전체 흐름 검증

- [x] 타이머 설정 → 인터럽트 대기 → 핸들러 실행 → 복귀
- [x] 주기적 인터럽트 발생 테스트 (여러 번)
- [x] 소프트웨어 인터럽트 테스트 (msip)

---

## 5. 테스트 시나리오

### 5.1 기본 타이머 테스트

1. mtimecmp = 10 설정
2. mie.MTIE = 1, mstatus.MIE = 1
3. mtvec 설정
4. 10 스텝 이상 실행
5. 트랩 발생 확인 (mcause = 타이머 인터럽트)

### 5.2 인터럽트 비활성화 테스트

1. mtimecmp = 5 설정
2. mie.MTIE = 0 (비활성화)
3. 10 스텝 실행
4. 트랩 발생하지 않음 확인

### 5.3 주기적 인터럽트 테스트

1. mtimecmp = 10 설정
2. 인터럽트 핸들러에서 mtimecmp += 10
3. 100 스텝 실행
4. 인터럽트 10번 발생 확인

### 5.4 소프트웨어 인터럽트 테스트

1. mie.MSIE = 1, mstatus.MIE = 1
2. msip = 1 쓰기
3. 소프트웨어 인터럽트 발생 확인
4. msip = 0으로 클리어

---

## 6. 주의사항

### 6.1 mtime 증가 주기

- 실제 하드웨어: 고정 주파수 (예: 10MHz)
- 에뮬레이터: 명령어당 1 증가 (단순화)
- 정확한 타이밍이 필요하면 호스트 시간 기반 구현

### 6.2 64비트 읽기/쓰기 원자성

- RV32에서는 mtime을 32비트 두 번에 나눠 읽음
- RV64에서는 한 번에 64비트 읽기/쓰기 가능
- 우리는 RV64이므로 단순하게 처리

### 6.3 인터럽트 우선순위

여러 인터럽트가 동시에 펜딩된 경우:
1. 외부 인터럽트 (MEI) - 최고 우선순위
2. 소프트웨어 인터럽트 (MSI)
3. 타이머 인터럽트 (MTI) - 최저 우선순위

### 6.4 인터럽트 vs 예외

- 인터럽트: 비동기, mcause 최상위 비트 = 1
- 예외: 동기 (명령어 실행 중), mcause 최상위 비트 = 0

---

## 7. 참고 자료

- RISC-V Privileged Spec: 3.1.9 Machine Timer Registers
- SiFive CLINT: https://sifive.cdn.prismic.io/sifive/0d163928-2128-42be-a75a-464df65e04e0_sifive-interrupt-cookbook.pdf
- xv6 타이머 코드: `kernel/start.c`, `kernel/trap.c`
