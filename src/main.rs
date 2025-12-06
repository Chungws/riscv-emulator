use riscv_emulator::Cpu;

fn main() {
    let mut cpu = Cpu::new();

    // UART로 "Hi!\n" 출력하는 프로그램
    // let program: Vec<u32> = vec![
    //     0x100000B7, // lui x1, 0x10000       # x1 = 0x10000000 (UART 주소)
    //     0x04800113, // addi x2, x0, 72       # x2 = 'H'
    //     0x00208023, // sb x2, 0(x1)          # UART에 'H' 출력
    //     0x06900113, // addi x2, x0, 105      # x2 = 'i'
    //     0x00208023, // sb x2, 0(x1)          # UART에 'i' 출력
    //     0x02100113, // addi x2, x0, 33       # x2 = '!'
    //     0x00208023, // sb x2, 0(x1)          # UART에 '!' 출력
    //     0x00A00113, // addi x2, x0, 10       # x2 = '\n'
    //     0x00208023, // sb x2, 0(x1)          # UART에 '\n' 출력
    //     0x00000073, // ecall                 # 종료
    // ];

    // cpu.load_program(&program);
    cpu.run();
}
