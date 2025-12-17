# xv6 실행 가이드

RISC-V 에뮬레이터에서 xv6 커널을 실행하는 방법.

---

## 1. 사전 준비

### 1.1 RISC-V 툴체인 설치

```bash
brew install riscv64-elf-gcc
```

### 1.2 xv6 빌드

```bash
cd xv6-riscv
make TOOLPREFIX=riscv64-elf- kernel/kernel
```

**참고**: Makefile이 `rv64g` (-march=rv64g)로 수정되어 있어야 함 (RVC 비활성화)

---

## 2. 실행

```bash
cargo run -- xv6-riscv/kernel/kernel
```

---

## 3. 현재 상태

### 3.1 구현 완료
- ELF 로더
- RV64I 기본 명령어
- UART (16550)
- CLINT (타이머)

### 3.2 구현 필요
- M Extension (곱셈/나눗셈) - xv6 실행에 필요
- A Extension (원자적 연산) - 멀티코어 동기화
- PLIC (외부 인터럽트 컨트롤러)
- VirtIO (블록 디바이스)
- MMU/가상 메모리

---

## 4. 디버깅

### 4.1 디버그 로그 활성화

`src/cpu/mod.rs`에서 `debug_log!` 매크로 활성화:

```rust
macro_rules! debug_log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}
```

### 4.2 ELF 정보 확인

```bash
riscv64-elf-readelf -h xv6-riscv/kernel/kernel  # 헤더
riscv64-elf-readelf -l xv6-riscv/kernel/kernel  # 프로그램 헤더
riscv64-elf-objdump -d xv6-riscv/kernel/kernel  # 디스어셈블
```

---

## 5. 문제 해결

### 5.1 "Not Supported Opcode: 0x5"

원인: RVC (압축 명령어)로 빌드됨

해결:
```bash
# xv6-riscv/Makefile에서
-march=rv64gc → -march=rv64g
```

### 5.2 "Not Implemented OP funct7=0x1"

원인: M Extension 미구현

해결: M Extension 구현 필요 (docs/M_EXTENSION_IMPLEMENTATION.md 참조)

### 5.3 "Not Implemented AMO"

원인: A Extension 미구현

해결: A Extension 구현 필요
