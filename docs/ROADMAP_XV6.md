# xv6-riscv 실행 로드맵

xv6-riscv를 에뮬레이터에서 실행하기 위한 구현 계획.

---

## 중요 결정 사항

### RV32 vs RV64

**xv6-riscv 공식 버전은 RV64 사용.**

| 옵션 | 장점 | 단점 |
|------|------|------|
| RV64로 전환 | xv6 그대로 사용 가능 | 레지스터, 메모리 연산 수정 필요 |
| RV32 유지 + xv6 포팅 | 현재 코드 유지 | xv6 수정 필요 |
| RV32 xv6 찾기 | 둘 다 유지 | 커뮤니티 버전 의존 |

**권장: RV64로 전환** - xv6를 수정하는 것보다 에뮬레이터를 64비트로 바꾸는 게 쉬움.

---

## 전체 로드맵

```
Phase 1: 기반 확장
├── UART 완성 (현재)
├── Timer (CLINT)
└── RV64 전환

Phase 2: 특권 모드
├── CSR 명령어 (Zicsr)
├── M/S/U 모드
├── 트랩/예외 처리
└── 기본 CSR 레지스터

Phase 3: 메모리 관리
├── M 확장 (곱셈/나눗셈)
├── A 확장 (atomic)
└── MMU (Sv39)

Phase 4: 디바이스
├── PLIC (인터럽트 컨트롤러)
├── Virtio-blk (디스크)
└── Device Tree

Phase 5: xv6 부팅
├── OpenSBI 또는 직접 부트
├── xv6 커널 로드
└── 셸 실행
```

---

## Phase 1: 기반 확장

### 1.1 UART 완성

현재 진행 중. 송수신 버퍼와 타이밍 구현.

**파일:** `src/devices/uart.rs`

### 1.2 Timer (CLINT)

RISC-V Core Local Interruptor.

**레지스터:**
```
CLINT_BASE = 0x2000000

0x0000: msip      - 소프트웨어 인터럽트
0xBFF8: mtime     - 현재 시간 (64비트)
0x4000: mtimecmp  - 비교값 (64비트)
```

**동작:**
- `mtime`은 매 사이클 증가
- `mtime >= mtimecmp`이면 타이머 인터럽트 발생

### 1.3 RV64 전환

**변경 사항:**
- 레지스터: `u32` → `u64`
- 메모리 연산: `read64`, `write64` 추가
- 주소: 32비트 → 64비트
- 새 명령어: `LD`, `SD`, `ADDIW`, `SLLIW` 등

---

## Phase 2: 특권 모드

### 2.1 CSR 명령어 (Zicsr)

6개 명령어:
```
CSRRW  - CSR 읽고 쓰기
CSRRS  - CSR 읽고 비트 셋
CSRRC  - CSR 읽고 비트 클리어
CSRRWI - 즉시값으로 위 동작
CSRRSI
CSRRCI
```

### 2.2 특권 모드

| 모드 | 레벨 | 용도 |
|------|------|------|
| M (Machine) | 3 | 펌웨어, 부트로더 |
| S (Supervisor) | 1 | OS 커널 |
| U (User) | 0 | 유저 프로그램 |

### 2.3 기본 CSR 레지스터

**Machine 모드:**
```
mstatus  - 상태 (인터럽트 활성화, 이전 모드 등)
mtvec    - 트랩 벡터 (예외 발생 시 점프할 주소)
mepc     - 예외 발생 시 PC 저장
mcause   - 예외 원인
mtval    - 예외 관련 값 (잘못된 주소 등)
mie      - 인터럽트 활성화 비트
mip      - 인터럽트 펜딩 비트
medeleg  - 예외 위임 (S모드로)
mideleg  - 인터럽트 위임
```

**Supervisor 모드:**
```
sstatus, stvec, sepc, scause, stval, sie, sip
satp     - 페이지 테이블 베이스 주소
```

### 2.4 트랩/예외 처리

**트랩 종류:**
- 인터럽트: 타이머, 외부, 소프트웨어
- 예외: 잘못된 명령어, 페이지 폴트, 환경 호출(ecall)

**트랩 발생 시:**
1. `mepc` ← 현재 PC
2. `mcause` ← 원인
3. `mstatus` 업데이트 (이전 모드 저장)
4. PC ← `mtvec`
5. 모드 → M (또는 위임된 경우 S)

---

## Phase 3: 메모리 관리

### 3.1 M 확장 (곱셈/나눗셈)

8개 명령어:
```
MUL, MULH, MULHSU, MULHU  - 곱셈
DIV, DIVU, REM, REMU      - 나눗셈
```

### 3.2 A 확장 (Atomic)

락 구현에 필요:
```
LR.W/D    - Load Reserved
SC.W/D    - Store Conditional
AMO*.W/D  - Atomic Memory Operations
```

### 3.3 MMU (Sv39)

**가상 주소 변환:**
```
가상 주소 (39비트)
    ↓
페이지 테이블 워크 (3단계)
    ↓
물리 주소 (56비트)
```

**satp 레지스터:**
```
MODE (4비트) | ASID (16비트) | PPN (44비트)
```

---

## Phase 4: 디바이스

### 4.1 PLIC

Platform-Level Interrupt Controller.
외부 인터럽트 관리 (UART 등).

### 4.2 Virtio-blk

가상 블록 디바이스. 디스크 읽기/쓰기.
xv6 파일시스템에 필요.

### 4.3 Device Tree

하드웨어 구성 정보. Linux/xv6가 부팅 시 읽음.

---

## Phase 5: xv6 부팅

### 5.1 부트 순서

```
1. 에뮬레이터 시작
2. OpenSBI 로드 (M모드)
3. xv6 커널 로드 (S모드)
4. 초기화 (메모리, 디바이스)
5. 첫 유저 프로세스 (init)
6. 셸 실행
```

### 5.2 테스트

```bash
# xv6-riscv 빌드
git clone https://github.com/mit-pdos/xv6-riscv
cd xv6-riscv
make

# 에뮬레이터로 실행
./riscv-emulator kernel/kernel fs.img
```

---

## 예상 작업량

| Phase | 예상 파일 수 | 복잡도 |
|-------|-------------|--------|
| 1. 기반 확장 | 5-10 | 중간 |
| 2. 특권 모드 | 10-15 | 어려움 |
| 3. 메모리 관리 | 5-10 | 어려움 |
| 4. 디바이스 | 5-10 | 중간 |
| 5. xv6 부팅 | 2-3 | 통합 |

---

## 다음 단계

**지금 바로 할 것:**
1. UART 완성 (현재 문서 따라가기)
2. Timer (CLINT) 구현

**결정 필요:**
- RV64 전환 시점 (UART 후? Timer 후?)

---

## 참고 자료

- [xv6-riscv GitHub](https://github.com/mit-pdos/xv6-riscv)
- [xv6 book](https://pdos.csail.mit.edu/6.828/2023/xv6/book-riscv-rev3.pdf)
- [RISC-V Privileged Spec](https://riscv.org/specifications/privileged-isa/)
- [RISC-V Unprivileged Spec](https://riscv.org/specifications/)
