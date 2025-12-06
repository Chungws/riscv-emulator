# RISC-V Emulator

학습용 RV32I 베어메탈 에뮬레이터 (Rust)

## 목표

- RV32I 기본 명령어셋 (37개 명령어) 구현
- 단계별 테스트 주도 개발
- 추후 OS 실행을 위한 확장 가능한 구조

## 문서

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - RV32I 구조, 명령어 목록, 메모리 레이아웃
- [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) - 14단계 구현 가이드

## 빌드 & 실행

```bash
cargo build
cargo run -- <binary>
```

## 테스트

```bash
cargo test
```

## 구현 현황

- [x] Step 1: CPU 구조체 (레지스터 + PC)
- [x] Step 2: 메모리 시스템
- [x] Step 3: Fetch
- [x] Step 4: Decode
- [x] Step 5: 산술 연산 (ADD, SUB, ADDI)
- [x] Step 6: 논리 연산 (AND, OR, XOR...)
- [x] Step 7: 시프트 연산 (SLL, SRL, SRA...)
- [x] Step 8: 비교 연산 (SLT, SLTU...)
- [ ] Step 9: 로드/스토어 (LW, SW...)
- [ ] Step 10: 분기 (BEQ, BNE...)
- [ ] Step 11: 점프 (JAL, JALR)
- [ ] Step 12: 상위 즉시값 (LUI, AUIPC)
- [ ] Step 13: 시스템 (ECALL, EBREAK)
- [ ] Step 14: 통합 테스트

## 참고 자료

- [RISC-V Specifications](https://riscv.org/technical/specifications/)
- [RISC-V Green Card](https://www.cl.cam.ac.uk/teaching/1617/ECAD+Arch/files/docs/RISCVGreenCardv8-20151013.pdf)
- [riscv-tests](https://github.com/riscv-software-src/riscv-tests)
