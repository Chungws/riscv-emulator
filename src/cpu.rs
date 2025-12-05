use crate::{DRAM_BASE, memory::Memory};

pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
    pub memory: Memory,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: DRAM_BASE,
            memory: Memory::new(),
        }
    }

    pub fn read_reg(&self, index: usize) -> u32 {
        self.regs[index]
    }

    pub fn write_reg(&mut self, index: usize, value: u32) {
        if index != 0 {
            self.regs[index] = value;
        }
    }

    pub fn fetch(&self) -> u32 {
        self.memory.read32(self.pc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_init() {
        let cpu = Cpu::new();
        for i in 0..32 {
            assert_eq!(cpu.regs[i], 0);
        }
        assert_eq!(cpu.pc, 0x80000000);
    }

    #[test]
    fn test_x0_always_zero() {
        let mut cpu = Cpu::new();
        cpu.write_reg(0, 100);
        assert_eq!(cpu.read_reg(0), 0);
    }

    #[test]
    fn test_fetch() {
        let mut cpu = Cpu::new();
        // ADDI x1, x0, 42를 메모리에 로드
        // addi x1, x0, 42 → 0x02A00093
        cpu.memory.write32(0x80000000, 0x02A00093);

        let instruction = cpu.fetch();
        assert_eq!(instruction, 0x02A00093);
    }
}
