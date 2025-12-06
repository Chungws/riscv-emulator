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
                    0x1 => {
                        let shamt = (imm as u32) & 0x1F;
                        debug_log!(
                            "SLLI rd={}, rs1={}, rs1_val={}, imm={}, shamt={}",
                            rd,
                            rs1,
                            rs1_val,
                            imm,
                            shamt
                        );
                        self.write_reg(rd, rs1_val << shamt);
                    }
                    0x2 => {
                        debug_log!(
                            "SLTI rd={}, rs1={}, rs1_val={}, imm={}",
                            rd,
                            rs1,
                            rs1_val,
                            imm
                        );
                        let result = if (rs1_val as i32) < imm { 1 } else { 0 };
                        self.write_reg(rd, result);
                    }
                    0x3 => {
                        debug_log!(
                            "SLTIU rd={}, rs1={}, rs1_val={}, imm={}",
                            rd,
                            rs1,
                            rs1_val,
                            imm
                        );
                        let result = if rs1_val < (imm as u32) { 1 } else { 0 };
                        self.write_reg(rd, result);
                    }
                    0x4 => {
                        debug_log!(
                            "XORI rd={}, rs1={}, rs1_val={}, imm={}",
                            rd,
                            rs1,
                            rs1_val,
                            imm
                        );
                        self.write_reg(rd, rs1_val ^ (imm as u32));
                    }
                    0x5 => {
                        let funct7 = ((imm as u32) >> 5) & 0x7F;
                        let shamt = (imm as u32) & 0x1F;
                        match funct7 {
                            0x00 => {
                                debug_log!(
                                    "SRLI rd={}, rs1={}, rs1_val={}, imm={}, shamt={}, funct7={}",
                                    rd,
                                    rs1,
                                    rs1_val,
                                    imm,
                                    shamt,
                                    funct7
                                );
                                self.write_reg(rd, rs1_val >> shamt);
                            }
                            0x20 => {
                                debug_log!(
                                    "SRAI rd={}, rs1={}, rs1_val={}, imm={}, shamt={}, funct7={}",
                                    rd,
                                    rs1,
                                    rs1_val,
                                    imm,
                                    shamt,
                                    funct7
                                );
                                self.write_reg(rd, ((rs1_val as i32) >> shamt) as u32);
                            }
                            _ => panic!("Not Implemented"),
                        }
                    }
                    0x6 => {
                        debug_log!(
                            "ORI rd={}, rs1={}, rs1_val={}, imm={}",
                            rd,
                            rs1,
                            rs1_val,
                            imm
                        );
                        self.write_reg(rd, rs1_val | (imm as u32));
                    }
                    0x7 => {
                        debug_log!(
                            "ANDI rd={}, rs1={}, rs1_val={}, imm={}",
                            rd,
                            rs1,
                            rs1_val,
                            imm
                        );
                        self.write_reg(rd, rs1_val & (imm as u32));
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
                    (0x1, 0x0) => {
                        let shamt = rs2_val & 0x1F;
                        debug_log!(
                            "SLL rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}, shamt={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val,
                            shamt
                        );
                        self.write_reg(rd, rs1_val << shamt);
                    }
                    (0x2, 0x0) => {
                        debug_log!(
                            "SLT rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val
                        );
                        let result = if (rs1_val as i32) < (rs2_val as i32) {
                            1
                        } else {
                            0
                        };
                        self.write_reg(rd, result);
                    }
                    (0x3, 0x0) => {
                        debug_log!(
                            "SLTU rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val
                        );
                        let result = if rs1_val < rs2_val { 1 } else { 0 };
                        self.write_reg(rd, result);
                    }
                    (0x4, 0x0) => {
                        debug_log!(
                            "XOR rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val
                        );
                        self.write_reg(rd, rs1_val ^ rs2_val);
                    }
                    (0x5, 0x0) => {
                        let shamt = rs2_val & 0x1F;
                        debug_log!(
                            "SRL rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}, shamt={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val,
                            shamt
                        );
                        self.write_reg(rd, rs1_val >> rs2_val);
                    }
                    (0x5, 0x20) => {
                        let shamt = rs2_val & 0x1F;
                        debug_log!(
                            "SRA rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}, shamt={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val,
                            shamt
                        );
                        self.write_reg(rd, ((rs1_val as i32) >> shamt) as u32);
                    }
                    (0x6, 0x0) => {
                        debug_log!(
                            "OR rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val
                        );
                        self.write_reg(rd, rs1_val | rs2_val);
                    }
                    (0x7, 0x0) => {
                        debug_log!(
                            "AND rd={}, rs1={}, rs1_val={}, rs2={}, rs2_val={}",
                            rd,
                            rs1,
                            rs1_val,
                            rs2,
                            rs2_val
                        );
                        self.write_reg(rd, rs1_val & rs2_val);
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

    // === R-type 논리 연산 ===
    #[test]
    fn test_and() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0b1100);
        cpu.write_reg(2, 0b1010);
        // AND x3, x1, x2 → 0x0020F1B3
        cpu.memory.write32(0x80000000, 0x0020F1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0b1000);
    }

    #[test]
    fn test_or() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0b1100);
        cpu.write_reg(2, 0b1010);
        // OR x3, x1, x2 → 0x0020E1B3
        cpu.memory.write32(0x80000000, 0x0020E1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0b1110);
    }

    #[test]
    fn test_or_with_zero() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x12345678);
        cpu.write_reg(2, 0);
        // OR x3, x1, x2
        cpu.memory.write32(0x80000000, 0x0020E1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0x12345678); // a | 0 = a
    }

    #[test]
    fn test_xor() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0b1100);
        cpu.write_reg(2, 0b1010);
        // XOR x3, x1, x2 → 0x0020C1B3
        cpu.memory.write32(0x80000000, 0x0020C1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0b0110);
    }

    // === I-type 논리 연산 ===
    #[test]
    fn test_andi() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFF);
        // ANDI x2, x1, 0x0F → 0x00F0F113
        cpu.memory.write32(0x80000000, 0x00F0F113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x0F);
    }

    #[test]
    fn test_andi_sign_extended() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFFFFFFFF);
        // ANDI x2, x1, -1 (0xFFF) → 0xFFF0F113
        cpu.memory.write32(0x80000000, 0xFFF0F113);
        cpu.step();
        // imm = -1 → 0xFFFFFFFF
        // 0xFFFFFFFF & 0xFFFFFFFF = 0xFFFFFFFF
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFF);
    }

    #[test]
    fn test_ori() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xF0);
        // ORI x2, x1, 0x0F → 0x00F0E113
        cpu.memory.write32(0x80000000, 0x00F0E113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFF);
    }

    #[test]
    fn test_xori() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFF);
        // XORI x2, x1, 0xFF → 0x0FF0C113
        cpu.memory.write32(0x80000000, 0x0FF0C113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0); // 0xFF ^ 0xFF = 0
    }

    #[test]
    fn test_xori_sign_extended() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFF);
        // XORI x2, x1, -1 (0xFFF) → 0xFFF0C113
        cpu.memory.write32(0x80000000, 0xFFF0C113);
        cpu.step();
        // imm = -1 → 부호확장 → 0xFFFFFFFF
        // 0xFF ^ 0xFFFFFFFF = 0xFFFFFF00
        assert_eq!(cpu.read_reg(2), 0xFFFFFF00);
    }
    // === R-type 시프트 ===
    #[test]
    fn test_sll() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 1);
        cpu.write_reg(2, 4);
        // SLL x3, x1, x2 → 0x002091B3
        // funct7=0000000, rs2=00010, rs1=00001, funct3=001, rd=00011, op=0110011
        cpu.memory.write32(0x80000000, 0x002091B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 16); // 1 << 4 = 16
    }

    #[test]
    fn test_srl() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        cpu.write_reg(2, 4);
        // SRL x3, x1, x2 → 0x0020D1B3
        // funct7=0000000, rs2=00010, rs1=00001, funct3=101, rd=00011, op=0110011
        cpu.memory.write32(0x80000000, 0x0020D1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0x08000000); // 논리 시프트, 0 채움
    }

    #[test]
    fn test_sra() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000); // 음수 (부호 비트 1)
        cpu.write_reg(2, 4);
        // SRA x3, x1, x2 → 0x4020D1B3
        // funct7=0100000, rs2=00010, rs1=00001, funct3=101, rd=00011, op=0110011
        cpu.memory.write32(0x80000000, 0x4020D1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0xF8000000); // 산술 시프트, 부호 채움
    }

    // === I-type 시프트 ===
    #[test]
    fn test_slli() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 1);
        // SLLI x2, x1, 4 → 0x00409113
        // imm=0000000_00100, rs1=00001, funct3=001, rd=00010, op=0010011
        cpu.memory.write32(0x80000000, 0x00409113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 16); // 1 << 4 = 16
    }

    #[test]
    fn test_srli() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        // SRLI x2, x1, 4 → 0x0040D113
        // imm=0000000_00100, rs1=00001, funct3=101, rd=00010, op=0010011
        cpu.memory.write32(0x80000000, 0x0040D113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x08000000); // 논리 시프트
    }

    #[test]
    fn test_srai() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        // SRAI x2, x1, 4 → 0x4040D113
        // imm=0100000_00100, rs1=00001, funct3=101, rd=00010, op=0010011
        cpu.memory.write32(0x80000000, 0x4040D113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xF8000000); // 산술 시프트
    }

    #[test]
    fn test_slt_signed() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, (-5_i32) as u32); // -5
        cpu.write_reg(2, 5);
        // SLT x3, x1, x2 → 0x0020A1B3
        cpu.memory.write32(0x80000000, 0x0020A1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 1); // -5 < 5 (signed)
    }

    #[test]
    fn test_sltu_unsigned() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, (-5_i32) as u32); // 0xFFFFFFFB
        cpu.write_reg(2, 5);
        // SLTU x3, x1, x2 → 0x0020B1B3
        cpu.memory.write32(0x80000000, 0x0020B1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0); // 0xFFFFFFFB > 5 (unsigned)
    }

    #[test]
    fn test_slti() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 5);
        // SLTI x2, x1, 10 → 0x00A0A113
        cpu.memory.write32(0x80000000, 0x00A0A113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 1); // 5 < 10
    }

    #[test]
    fn test_sltiu() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 5);
        // SLTIU x2, x1, -1 (0xFFF) → 0xFFF0B113
        cpu.memory.write32(0x80000000, 0xFFF0B113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 1); // 5 < 0xFFFFFFFF (unsigned)
    }
}
