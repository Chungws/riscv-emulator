use riscv_emulator::Cpu;

fn main() {
    let mut cpu = Cpu::new();

    // RV64I 테스트: UART로 "RV64!" 출력 + 64비트 연산 검증
    let program: Vec<u32> = vec![
        // x1 = 0x10000000 (UART 주소)
        0x100000B7, // lui x1, 0x10000

        // 'R' 출력
        0x05200113, // addi x2, x0, 82
        0x00208023, // sb x2, 0(x1)

        // 'V' 출력
        0x05600113, // addi x2, x0, 86
        0x00208023, // sb x2, 0(x1)

        // '6' 출력
        0x03600113, // addi x2, x0, 54
        0x00208023, // sb x2, 0(x1)

        // '4' 출력
        0x03400113, // addi x2, x0, 52
        0x00208023, // sb x2, 0(x1)

        // 64비트 연산 테스트: -1 + 2 = 1
        0xFFF00193, // addi x3, x0, -1        # x3 = 0xFFFFFFFFFFFFFFFF
        0x00218213, // addi x4, x3, 2         # x4 = 1

        // '!' 출력
        0x02100113, // addi x2, x0, 33
        0x00208023, // sb x2, 0(x1)

        // '\n' 출력
        0x00A00113, // addi x2, x0, 10
        0x00208023, // sb x2, 0(x1)

        // 종료
        0x00000073, // ecall
    ];

    cpu.load_program(&program);
    cpu.run();

    // 레지스터 상태 출력으로 64비트 동작 확인
    println!("\n=== RV64I Test Results ===");
    println!("x3 = {:#018x} (expected: 0xffffffffffffffff)", cpu.read_reg(3));
    println!("x4 = {:#018x} (expected: 0x0000000000000001)", cpu.read_reg(4));

    // 검증
    let pass = cpu.read_reg(3) == 0xFFFFFFFFFFFFFFFF && cpu.read_reg(4) == 1;
    println!("\nTest: {}", if pass { "PASSED" } else { "FAILED" });
}
