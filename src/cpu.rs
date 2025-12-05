use crate::{DRAM_BASE, debug_log, decoder, memory::Memory};

const OP_I: u32 = 0x13;
const OP_R: u32 = 0x33;

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

    pub fn step(&mut self) {
        let inst = self.fetch();
        let op = decoder::opcode(inst);

        match op {
            OP_I => {
                debug_log!("IType");
                let funct3 = decoder::funct3(inst);
                let rd = decoder::rd(inst);
                let rs1 = decoder::rs1(inst);
                let rs1_val = self.read_reg(rs1);
                let imm = decoder::imm_i(inst);

                match funct3 {
                    0x0 => {
                        debug_log!(
                            "ADDI rd={}, rs1={}, rs1_val={}, imm={}",
                            rd,
                            rs1,
                            rs1_val,
                            imm
                        );
                        self.write_reg(rd, rs1_val.wrapping_add(imm as u32));
                    }
                    _ => debug_log!("Not Implemented"),
                }
            }
            OP_R => {
                debug_log!("RType");
                let funct3 = decoder::funct3(inst);
                let funct7 = decoder::funct7(inst);
                let rd = decoder::rd(inst);
                let rs1 = decoder::rs1(inst);
                let rs1_val = self.read_reg(rs1);
                let rs2 = decoder::rs2(inst);
                let rs2_val = self.read_reg(rs2);

                match (funct3, funct7) {
                    (0x0, 0x0) => {
                        debug_log!(
                            "ADD rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val
                        );
                        self.write_reg(rd, rs1_val.wrapping_add(rs2_val));
                    }
                    (0x0, 0x20) => {
                        debug_log!(
                            "SUB rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val
                        );
                        self.write_reg(rd, rs1_val.wrapping_sub(rs2_val));
                    }
                    _ => debug_log!("Not Implemented"),
                }
            }
            _ => panic!("Not Supported Opcode"),
        }
        self.pc += 4
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

    #[test]
    fn test_addi() {
        let mut cpu = Cpu::new();
        // ADDI x1, x0, 42 → 0x02A00093
        cpu.memory.write32(0x80000000, 0x02A00093);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 42);
        assert_eq!(cpu.pc, 0x80000004);
    }

    #[test]
    fn test_addi_negative() {
        let mut cpu = Cpu::new();
        // ADDI x1, x0, -1 → 0xFFF00093
        cpu.memory.write32(0x80000000, 0xFFF00093);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0xFFFFFFFF); // -1 as u32
    }

    #[test]
    fn test_add() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 10);
        cpu.write_reg(2, 20);
        // ADD x3, x1, x2 → 0x002081B3
        cpu.memory.write32(0x80000000, 0x002081B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 30);
    }

    #[test]
    fn test_sub() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 100);
        cpu.write_reg(2, 30);
        // SUB x3, x1, x2 → 0x402081B3
        cpu.memory.write32(0x80000000, 0x402081B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 70);
    }
}
