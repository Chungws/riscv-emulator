use std::{env, fs};

use riscv_emulator::{Cpu, elf::ElfFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <elf-file>", args[0]);
        return Ok(());
    }
    let elf_path = &args[1];
    let bytes = fs::read(elf_path)?;
    let elf_file = ElfFile::load(&bytes)?;

    let mut cpu = Cpu::new();

    cpu.load_segments(&elf_file.segments, elf_file.entry);
    cpu.run();

    Ok(())
}
