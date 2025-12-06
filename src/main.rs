use riscv_emulator::Cpu;

fn main() {
    println!("RISC-V Emulator");
    let mut cpu = Cpu::new();
    let program: Vec<u32> = vec![
        0x00000513, // li x10, 0
        0x00100593, // li x11, 1
        0x00B00613, // li x12, 11
        0x00B50533, // add x10, x10, x11
        0x00158593, // addi x11, x11, 1
        0xFEC5CCE3, // blt x11, x12, -8
        0x00000073, // ecall
    ];
    cpu.load_program(&program);
    cpu.run();
    println!("Answer : {}", cpu.read_reg(10));
}
