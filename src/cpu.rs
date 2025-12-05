use crate::DRAM_BASE;

pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: DRAM_BASE,
        }
    }

    pub fn read_arg(&self, index: usize) -> u32 {
        self.regs[index]
    }

    pub fn write_reg(&mut self, index: usize, value: u32) {
        if index != 0 {
            self.regs[index] = value;
        }
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
        assert_eq!(cpu.read_arg(0), 0);
    }
}
