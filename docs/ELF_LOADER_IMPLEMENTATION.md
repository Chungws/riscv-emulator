# ELF Loader 구현 가이드

RISC-V 에뮬레이터에서 ELF 파일을 로드하기 위한 구현 가이드.

---

## 1. ELF 개요

### 1.1 ELF란?

ELF (Executable and Linkable Format)는 UNIX 계열 시스템의 표준 실행 파일 형식.
RISC-V 툴체인으로 빌드한 프로그램은 ELF 형식.

### 1.2 왜 필요한가?

- xv6-riscv 커널 로드
- 일반 RISC-V 바이너리 실행
- QEMU와 동일한 방식

### 1.3 ELF 구조

```
┌─────────────────────┐
│     ELF Header      │  ← 파일 시작, 전체 정보
├─────────────────────┤
│  Program Headers    │  ← 로딩에 필요한 정보
├─────────────────────┤
│                     │
│      Segments       │  ← 실제 코드/데이터
│                     │
├─────────────────────┤
│  Section Headers    │  ← 디버깅용 (무시 가능)
└─────────────────────┘
```

---

## 2. ELF Header (64-bit)

### 2.1 구조 (총 64바이트)

| 오프셋 | 크기 | 필드 | 설명 |
|--------|------|------|------|
| 0x00 | 4 | e_ident[0..4] | Magic: 0x7F, 'E', 'L', 'F' |
| 0x04 | 1 | e_ident[4] | Class: 1=32bit, 2=64bit |
| 0x05 | 1 | e_ident[5] | Endian: 1=little, 2=big |
| 0x06 | 1 | e_ident[6] | Version: 1 |
| 0x07 | 9 | e_ident[7..16] | Padding |
| 0x10 | 2 | e_type | Type: 2=EXEC, 3=DYN |
| 0x12 | 2 | e_machine | Machine: 0xF3=RISC-V |
| 0x14 | 4 | e_version | Version: 1 |
| 0x18 | 8 | e_entry | **Entry point 주소** |
| 0x20 | 8 | e_phoff | **Program header 오프셋** |
| 0x28 | 8 | e_shoff | Section header 오프셋 (무시) |
| 0x30 | 4 | e_flags | Flags |
| 0x34 | 2 | e_ehsize | ELF header 크기 (64) |
| 0x36 | 2 | e_phentsize | **Program header 엔트리 크기** |
| 0x38 | 2 | e_phnum | **Program header 개수** |
| 0x3A | 2 | e_shentsize | Section header 엔트리 크기 |
| 0x3C | 2 | e_shnum | Section header 개수 |
| 0x3E | 2 | e_shstrndx | Section name string table index |

### 2.2 중요 필드

- `e_entry`: 프로그램 시작 주소 (PC 초기값)
- `e_phoff`: Program Header 시작 위치
- `e_phentsize`: Program Header 하나의 크기
- `e_phnum`: Program Header 개수

---

## 3. Program Header (64-bit)

### 3.1 구조 (총 56바이트)

| 오프셋 | 크기 | 필드 | 설명 |
|--------|------|------|------|
| 0x00 | 4 | p_type | **세그먼트 타입** |
| 0x04 | 4 | p_flags | 권한 (R/W/X) |
| 0x08 | 8 | p_offset | **파일 내 오프셋** |
| 0x10 | 8 | p_vaddr | **가상 주소 (로드 위치)** |
| 0x18 | 8 | p_paddr | 물리 주소 (보통 vaddr와 동일) |
| 0x20 | 8 | p_filesz | **파일 내 크기** |
| 0x28 | 8 | p_memsz | **메모리 크기** |
| 0x30 | 8 | p_align | 정렬 |

### 3.2 세그먼트 타입 (p_type)

| 값 | 이름 | 설명 |
|----|------|------|
| 0 | PT_NULL | 무시 |
| 1 | PT_LOAD | **로드할 세그먼트** |
| 2+ | 기타 | 무시 |

### 3.3 PT_LOAD 처리

```
1. 파일에서 p_offset 위치부터 p_filesz 바이트 읽기
2. 메모리 p_vaddr 위치에 복사
3. p_memsz > p_filesz이면 나머지는 0으로 채움 (BSS)
```

---

## 4. 로딩 흐름

```
┌─────────────────────────────────────────────┐
│ 1. 파일 열기                                  │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│ 2. ELF Header 읽기 (64바이트)                 │
│    - Magic 검증 (0x7F, E, L, F)              │
│    - Class 확인 (64-bit)                     │
│    - Machine 확인 (RISC-V)                   │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│ 3. Program Headers 읽기                       │
│    - e_phoff 위치로 이동                      │
│    - e_phnum개 읽기                          │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│ 4. PT_LOAD 세그먼트 로드                      │
│    - p_offset에서 p_filesz 읽기              │
│    - p_vaddr에 복사                          │
│    - 나머지 0으로 채움                        │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│ 5. Entry Point 설정                          │
│    - PC = e_entry                           │
└─────────────────────────────────────────────┘
```

---

## 5. 구현 단계

### Step 1: ELF 상수 정의 ✅

**목표**: ELF 관련 상수 정의

- [x] Magic number 상수 (0x7F, 'E', 'L', 'F')
- [x] Class 상수 (ELFCLASS64 = 2)
- [x] Machine 상수 (EM_RISCV = 0xF3)
- [x] Type 상수 (PT_LOAD = 1)

**검증**: 상수 정의 확인 ✅

---

### Step 2: ELF Header 파싱

**목표**: ELF 헤더 읽기 및 검증

- [ ] ElfHeader 구조체 정의
- [ ] 파일에서 64바이트 읽기
- [ ] Magic number 검증
- [ ] Class (64-bit) 검증
- [ ] Machine (RISC-V) 검증
- [ ] entry, phoff, phentsize, phnum 추출

**검증**: 유효한 ELF 파일 헤더 파싱 테스트

---

### Step 3: Program Header 파싱

**목표**: Program Header 읽기

- [ ] ProgramHeader 구조체 정의
- [ ] e_phoff 위치에서 읽기
- [ ] e_phnum개의 Program Header 파싱
- [ ] p_type, p_offset, p_vaddr, p_filesz, p_memsz 추출

**검증**: Program Header 목록 출력 테스트

---

### Step 4: 세그먼트 로딩

**목표**: PT_LOAD 세그먼트를 메모리에 로드

- [ ] PT_LOAD 세그먼트 필터링
- [ ] 파일에서 데이터 읽기 (p_offset, p_filesz)
- [ ] 메모리에 쓰기 (p_vaddr)
- [ ] BSS 영역 0으로 초기화 (p_memsz - p_filesz)

**검증**: 세그먼트 로드 후 메모리 확인

---

### Step 5: Entry Point 설정

**목표**: PC를 entry point로 설정

- [ ] e_entry 값을 PC에 설정
- [ ] 또는 load_elf() 반환값으로 entry 전달

**검증**: 로드 후 PC 확인

---

### Step 6: main.rs 통합

**목표**: 커맨드라인에서 ELF 파일 로드

- [ ] 커맨드라인 인자 처리
- [ ] 파일 경로 받아서 load_elf() 호출
- [ ] 에러 처리

**검증**: `cargo run -- kernel` 실행

---

## 6. 파일 구조

```
src/
├── elf.rs          # ELF 파싱 모듈 (새로 생성)
├── cpu/
│   └── mod.rs      # load_elf() 메서드 추가
├── main.rs         # 커맨드라인 처리
└── ...
```

---

## 7. API 설계

### ElfLoader (elf.rs)

```rust
pub struct ElfLoader {
    // 파싱된 정보
}

impl ElfLoader {
    pub fn load(path: &str) -> Result<ElfLoader, ElfError>;
    pub fn entry(&self) -> u64;
    pub fn segments(&self) -> &[Segment];
}

pub struct Segment {
    pub vaddr: u64,
    pub data: Vec<u8>,
    pub memsz: u64,
}
```

### CPU 확장

```rust
impl Cpu {
    pub fn load_elf(&mut self, path: &str) -> Result<(), ElfError>;
}
```

---

## 8. 에러 처리

| 에러 | 설명 |
|------|------|
| InvalidMagic | Magic number 불일치 |
| InvalidClass | 32-bit ELF (64-bit만 지원) |
| InvalidMachine | RISC-V가 아님 |
| FileError | 파일 읽기 실패 |
| LoadError | 메모리 로드 실패 |

---

## 9. 테스트

### 단위 테스트

1. ELF 헤더 파싱
2. Program Header 파싱
3. 세그먼트 로드

### 통합 테스트

1. 간단한 RISC-V ELF 로드 및 실행
2. xv6 커널 로드

---

## 10. 참고 자료

- ELF Specification: https://refspecs.linuxfoundation.org/elf/elf.pdf
- RISC-V ELF: https://github.com/riscv-non-isa/riscv-elf-psabi-doc
- xv6-riscv: https://github.com/mit-pdos/xv6-riscv
