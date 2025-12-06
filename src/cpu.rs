use core::panic;

use crate::{Bus, debug_log, decoder, devices::DRAM_BASE};

const OP_IMM: u32 = 0x13;
const OP_IMM_32: u32 = 0x1B;
const OP: u32 = 0x33;
const OP_32: u32 = 0x3B;
const LOAD: u32 = 0x03;
const STORE: u32 = 0x23;
const BRANCH: u32 = 0x63;
const JAL: u32 = 0x6F;
const JALR: u32 = 0x67;
const LUI: u32 = 0x37;
const AUIPC: u32 = 0x17;
const SYSTEM: u32 = 0x73;

pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: DRAM_BASE,
            bus: Bus::new(),
            halted: false,
        }
    }

    pub fn read_reg(&self, index: usize) -> u64 {
        self.regs[index]
    }

    pub fn write_reg(&mut self, index: usize, value: u64) {
        if index != 0 {
            self.regs[index] = value;
        }
    }

    pub fn fetch(&self) -> u32 {
        self.bus.read32(self.pc)
    }

    pub fn load_program(&mut self, program: &[u32]) {
        for (i, &inst) in program.iter().enumerate() {
            let addr = DRAM_BASE + (i as u64) * 4;
            self.bus.write32(addr, inst);
        }
    }

    pub fn run(&mut self) {
        while !self.halted {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let inst = self.fetch();
        let op = decoder::opcode(inst);

        match op {
            OP_IMM => self.execute_op_imm(inst),
            OP_IMM_32 => self.execute_op_imm_32(inst),
            OP => self.execute_op(inst),
            OP_32 => self.execute_op_32(inst),
            LOAD => self.execute_load(inst),
            STORE => self.execute_store(inst),
            BRANCH => {
                if self.execute_branch(inst) {
                    return; // 분기 성공 시 PC 증가 안함
                }
            }
            JAL => {
                self.execute_jal(inst);
                return; // PC 직접 설정
            }
            JALR => {
                self.execute_jalr(inst);
                return; // PC 직접 설정
            }
            LUI => self.execute_lui(inst),
            AUIPC => self.execute_auipc(inst),
            SYSTEM => self.execute_system(inst),
            _ => panic!("Not Supported Opcode: {:#x}", op),
        }
        self.pc += 4;
    }

    fn execute_op_imm(&mut self, inst: u32) {
        debug_log!("OP_IMM");
        let funct3 = decoder::funct3(inst);
        let rd = decoder::rd(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let imm = decoder::imm_i(inst);

        match funct3 {
            0x0 => {
                debug_log!("ADDI rd={}, rs1={}, rs1_val={}, imm={}", rd, rs1, rs1_val, imm);
                self.write_reg(rd, rs1_val.wrapping_add(imm as u64));
            }
            0x1 => {
                let shamt = (imm as u64) & 0x3F;
                debug_log!("SLLI rd={}, rs1={}, rs1_val={}, shamt={}", rd, rs1, rs1_val, shamt);
                self.write_reg(rd, rs1_val << shamt);
            }
            0x2 => {
                debug_log!("SLTI rd={}, rs1={}, rs1_val={}, imm={}", rd, rs1, rs1_val, imm);
                let result = if (rs1_val as i64) < (imm as i64) { 1 } else { 0 };
                self.write_reg(rd, result);
            }
            0x3 => {
                debug_log!("SLTIU rd={}, rs1={}, rs1_val={}, imm={}", rd, rs1, rs1_val, imm);
                let result = if rs1_val < (imm as u64) { 1 } else { 0 };
                self.write_reg(rd, result);
            }
            0x4 => {
                debug_log!("XORI rd={}, rs1={}, rs1_val={}, imm={}", rd, rs1, rs1_val, imm);
                self.write_reg(rd, rs1_val ^ (imm as u64));
            }
            0x5 => {
                let funct7 = ((imm as u64) >> 5) & 0x7F;
                let shamt = (imm as u64) & 0x3F;
                match funct7 {
                    0x00 => {
                        debug_log!("SRLI rd={}, rs1={}, rs1_val={}, shamt={}", rd, rs1, rs1_val, shamt);
                        self.write_reg(rd, rs1_val >> shamt);
                    }
                    0x20 => {
                        debug_log!("SRAI rd={}, rs1={}, rs1_val={}, shamt={}", rd, rs1, rs1_val, shamt);
                        self.write_reg(rd, ((rs1_val as i64) >> shamt) as u64);
                    }
                    _ => panic!("Not Implemented OP_IMM funct7: {:#x}", funct7),
                }
            }
            0x6 => {
                debug_log!("ORI rd={}, rs1={}, rs1_val={}, imm={}", rd, rs1, rs1_val, imm);
                self.write_reg(rd, rs1_val | (imm as u64));
            }
            0x7 => {
                debug_log!("ANDI rd={}, rs1={}, rs1_val={}, imm={}", rd, rs1, rs1_val, imm);
                self.write_reg(rd, rs1_val & (imm as u64));
            }
            _ => panic!("Not Implemented OP_IMM funct3: {:#x}", funct3),
        }
    }

    fn execute_op_imm_32(&mut self, inst: u32) {
        debug_log!("OP_IMM_32");
        let funct3 = decoder::funct3(inst);
        let rd = decoder::rd(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let imm = decoder::imm_i(inst);

        match funct3 {
            0x0 => {
                debug_log!("ADDIW rd={}, rs1={}, rs1_val={}, imm={}", rd, rs1, rs1_val, imm);
                let result = (rs1_val as i32).wrapping_add(imm as i32);
                self.write_reg(rd, result as i64 as u64);
            }
            0x1 => {
                let shamt = (imm as u64) & 0x3F;
                debug_log!("SLLIW rd={}, rs1={}, rs1_val={}, shamt={}", rd, rs1, rs1_val, shamt);
                self.write_reg(rd, ((rs1_val as u32) << shamt) as i32 as i64 as u64);
            }
            0x5 => {
                let funct7 = ((imm as u64) >> 5) & 0x7F;
                let shamt = (imm as u64) & 0x3F;
                match funct7 {
                    0x00 => {
                        debug_log!("SRLIW rd={}, rs1={}, rs1_val={}, shamt={}", rd, rs1, rs1_val, shamt);
                        self.write_reg(rd, ((rs1_val as u32) >> shamt) as u64);
                    }
                    0x20 => {
                        debug_log!("SRAIW rd={}, rs1={}, rs1_val={}, shamt={}", rd, rs1, rs1_val, shamt);
                        self.write_reg(rd, ((rs1_val as i32) >> shamt) as i64 as u64);
                    }
                    _ => panic!("Not Implemented OP_IMM_32 funct7: {:#x}", funct7),
                }
            }
            _ => panic!("Not Implemented OP_IMM_32 funct3: {:#x}", funct3),
        }
    }

    fn execute_op(&mut self, inst: u32) {
        debug_log!("OP");
        let funct3 = decoder::funct3(inst);
        let funct7 = decoder::funct7(inst);
        let rd = decoder::rd(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let rs2 = decoder::rs2(inst);
        let rs2_val = self.read_reg(rs2);

        match (funct3, funct7) {
            (0x0, 0x0) => {
                debug_log!("ADD rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                self.write_reg(rd, rs1_val.wrapping_add(rs2_val));
            }
            (0x0, 0x20) => {
                debug_log!("SUB rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                self.write_reg(rd, rs1_val.wrapping_sub(rs2_val));
            }
            (0x1, 0x0) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SLL rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, rs1_val << shamt);
            }
            (0x2, 0x0) => {
                debug_log!("SLT rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let result = if (rs1_val as i64) < (rs2_val as i64) { 1 } else { 0 };
                self.write_reg(rd, result);
            }
            (0x3, 0x0) => {
                debug_log!("SLTU rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let result = if rs1_val < rs2_val { 1 } else { 0 };
                self.write_reg(rd, result);
            }
            (0x4, 0x0) => {
                debug_log!("XOR rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                self.write_reg(rd, rs1_val ^ rs2_val);
            }
            (0x5, 0x0) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SRL rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, rs1_val >> shamt);
            }
            (0x5, 0x20) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SRA rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, ((rs1_val as i64) >> shamt) as u64);
            }
            (0x6, 0x0) => {
                debug_log!("OR rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                self.write_reg(rd, rs1_val | rs2_val);
            }
            (0x7, 0x0) => {
                debug_log!("AND rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                self.write_reg(rd, rs1_val & rs2_val);
            }
            _ => panic!("Not Implemented OP funct3={:#x}, funct7={:#x}", funct3, funct7),
        }
    }

    fn execute_op_32(&mut self, inst: u32) {
        debug_log!("OP_32");
        let funct3 = decoder::funct3(inst);
        let funct7 = decoder::funct7(inst);
        let rd = decoder::rd(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let rs2 = decoder::rs2(inst);
        let rs2_val = self.read_reg(rs2);

        match (funct3, funct7) {
            (0x0, 0x0) => {
                debug_log!("ADDW rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let result = (rs1_val as i32).wrapping_add(rs2_val as i32);
                self.write_reg(rd, result as i64 as u64);
            }
            (0x0, 0x20) => {
                debug_log!("SUBW rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let result = (rs1_val as i32).wrapping_sub(rs2_val as i32);
                self.write_reg(rd, result as i64 as u64);
            }
            (0x1, 0x0) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SLLW rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, ((rs1_val as u32) << shamt) as i32 as i64 as u64);
            }
            (0x5, 0x0) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SRLW rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, ((rs1_val as u32) >> shamt) as u64);
            }
            (0x5, 0x20) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SRAW rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, ((rs1_val as i32) >> shamt) as i64 as u64);
            }
            _ => panic!("Not Implemented OP_32 funct3={:#x}, funct7={:#x}", funct3, funct7),
        }
    }

    fn execute_load(&mut self, inst: u32) {
        debug_log!("LOAD");
        let funct3 = decoder::funct3(inst);
        let rd = decoder::rd(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let imm = decoder::imm_i(inst);
        let addr = (rs1_val as i64).wrapping_add(imm as i64) as u64;

        match funct3 {
            0x0 => {
                let val = self.bus.read8(addr) as i8 as i64 as u64;
                debug_log!("LB rd={}, addr={:#x}, val={:#x}", rd, addr, val);
                self.write_reg(rd, val);
            }
            0x1 => {
                let val = self.bus.read16(addr) as i16 as i64 as u64;
                debug_log!("LH rd={}, addr={:#x}, val={:#x}", rd, addr, val);
                self.write_reg(rd, val);
            }
            0x2 => {
                let val = self.bus.read32(addr) as i32 as i64 as u64;
                debug_log!("LW rd={}, addr={:#x}, val={:#x}", rd, addr, val);
                self.write_reg(rd, val);
            }
            0x3 => {
                let val = self.bus.read64(addr);
                debug_log!("LD rd={}, addr={:#x}, val={:#x}", rd, addr, val);
                self.write_reg(rd, val);
            }
            0x4 => {
                let val = self.bus.read8(addr) as u64;
                debug_log!("LBU rd={}, addr={:#x}, val={:#x}", rd, addr, val);
                self.write_reg(rd, val);
            }
            0x5 => {
                let val = self.bus.read16(addr) as u64;
                debug_log!("LHU rd={}, addr={:#x}, val={:#x}", rd, addr, val);
                self.write_reg(rd, val);
            }
            0x6 => {
                let val = self.bus.read32(addr) as u64;
                debug_log!("LWU rd={}, addr={:#x}, val={:#x}", rd, addr, val);
                self.write_reg(rd, val);
            }
            _ => panic!("Not Implemented LOAD funct3: {:#x}", funct3),
        }
    }

    fn execute_store(&mut self, inst: u32) {
        debug_log!("STORE");
        let funct3 = decoder::funct3(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let rs2 = decoder::rs2(inst);
        let rs2_val = self.read_reg(rs2);
        let imm = decoder::imm_s(inst);
        let addr = (rs1_val as i64).wrapping_add(imm as i64) as u64;

        match funct3 {
            0x0 => {
                debug_log!("SB addr={:#x}, val={:#x}", addr, rs2_val as u8);
                self.bus.write8(addr, rs2_val as u8);
            }
            0x1 => {
                debug_log!("SH addr={:#x}, val={:#x}", addr, rs2_val as u16);
                self.bus.write16(addr, rs2_val as u16);
            }
            0x2 => {
                debug_log!("SW addr={:#x}, val={:#x}", addr, rs2_val as u32);
                self.bus.write32(addr, rs2_val as u32);
            }
            0x3 => {
                debug_log!("SD addr={:#x}, val={:#x}", addr, rs2_val);
                self.bus.write64(addr, rs2_val);
            }
            _ => panic!("Not Implemented STORE funct3: {:#x}", funct3),
        }
    }

    /// Returns true if branch was taken
    fn execute_branch(&mut self, inst: u32) -> bool {
        debug_log!("BRANCH");
        let funct3 = decoder::funct3(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let rs2 = decoder::rs2(inst);
        let rs2_val = self.read_reg(rs2);
        let imm = decoder::imm_b(inst);

        let taken = match funct3 {
            0x0 => {
                debug_log!("BEQ rs1_val={}, rs2_val={}, imm={}", rs1_val, rs2_val, imm);
                rs1_val == rs2_val
            }
            0x1 => {
                debug_log!("BNE rs1_val={}, rs2_val={}, imm={}", rs1_val, rs2_val, imm);
                rs1_val != rs2_val
            }
            0x4 => {
                debug_log!("BLT rs1_val={}, rs2_val={}, imm={}", rs1_val, rs2_val, imm);
                (rs1_val as i64) < (rs2_val as i64)
            }
            0x5 => {
                debug_log!("BGE rs1_val={}, rs2_val={}, imm={}", rs1_val, rs2_val, imm);
                (rs1_val as i64) >= (rs2_val as i64)
            }
            0x6 => {
                debug_log!("BLTU rs1_val={}, rs2_val={}, imm={}", rs1_val, rs2_val, imm);
                rs1_val < rs2_val
            }
            0x7 => {
                debug_log!("BGEU rs1_val={}, rs2_val={}, imm={}", rs1_val, rs2_val, imm);
                rs1_val >= rs2_val
            }
            _ => panic!("Not Implemented BRANCH funct3: {:#x}", funct3),
        };

        if taken {
            self.pc = (self.pc as i64).wrapping_add(imm as i64) as u64;
        }
        taken
    }

    fn execute_jal(&mut self, inst: u32) {
        debug_log!("JAL");
        let rd = decoder::rd(inst);
        let imm = decoder::imm_j(inst);
        debug_log!("JAL rd={}, imm={}, pc={:#x}", rd, imm, self.pc);
        self.write_reg(rd, self.pc + 4);
        self.pc = (self.pc as i64).wrapping_add(imm as i64) as u64;
    }

    fn execute_jalr(&mut self, inst: u32) {
        debug_log!("JALR");
        let rd = decoder::rd(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let imm = decoder::imm_i(inst);
        debug_log!("JALR rd={}, rs1_val={:#x}, imm={}", rd, rs1_val, imm);
        self.write_reg(rd, self.pc + 4);
        self.pc = ((rs1_val as i64).wrapping_add(imm as i64) as u64) & !1u64;
    }

    fn execute_lui(&mut self, inst: u32) {
        debug_log!("LUI");
        let rd = decoder::rd(inst);
        let imm = decoder::imm_u(inst);
        debug_log!("LUI rd={}, imm={:#x}", rd, imm);
        self.write_reg(rd, imm as u64);
    }

    fn execute_auipc(&mut self, inst: u32) {
        debug_log!("AUIPC");
        let rd = decoder::rd(inst);
        let imm = decoder::imm_u(inst);
        debug_log!("AUIPC rd={}, imm={:#x}, pc={:#x}", rd, imm, self.pc);
        self.write_reg(rd, (self.pc as i64).wrapping_add(imm as i64) as u64);
    }

    fn execute_system(&mut self, inst: u32) {
        debug_log!("SYSTEM");
        let imm = decoder::imm_i(inst);
        match imm {
            0x000 => {
                debug_log!("ECALL");
                self.halted = true;
            }
            0x001 => {
                debug_log!("EBREAK");
                self.halted = true;
            }
            _ => panic!("Unknown SYSTEM imm: {:#x}", imm),
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
        assert_eq!(cpu.read_reg(0), 0);
    }

    #[test]
    fn test_fetch() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0x02A00093);
        let instruction = cpu.fetch();
        assert_eq!(instruction, 0x02A00093);
    }

    #[test]
    fn test_addi() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0x02A00093);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 42);
        assert_eq!(cpu.pc, 0x80000004);
    }

    #[test]
    fn test_addi_negative() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0xFFF00093);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn test_add() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 10);
        cpu.write_reg(2, 20);
        cpu.bus.write32(0x80000000, 0x002081B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 30);
    }

    #[test]
    fn test_sub() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 100);
        cpu.write_reg(2, 30);
        cpu.bus.write32(0x80000000, 0x402081B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 70);
    }

    #[test]
    fn test_and() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0b1100);
        cpu.write_reg(2, 0b1010);
        cpu.bus.write32(0x80000000, 0x0020F1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0b1000);
    }

    #[test]
    fn test_or() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0b1100);
        cpu.write_reg(2, 0b1010);
        cpu.bus.write32(0x80000000, 0x0020E1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0b1110);
    }

    #[test]
    fn test_or_with_zero() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x12345678);
        cpu.write_reg(2, 0);
        cpu.bus.write32(0x80000000, 0x0020E1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0x12345678);
    }

    #[test]
    fn test_xor() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0b1100);
        cpu.write_reg(2, 0b1010);
        cpu.bus.write32(0x80000000, 0x0020C1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0b0110);
    }

    #[test]
    fn test_andi() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFF);
        cpu.bus.write32(0x80000000, 0x00F0F113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x0F);
    }

    #[test]
    fn test_andi_sign_extended() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFFFFFFFF);
        cpu.bus.write32(0x80000000, 0xFFF0F113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFF);
    }

    #[test]
    fn test_ori() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xF0);
        cpu.bus.write32(0x80000000, 0x00F0E113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFF);
    }

    #[test]
    fn test_xori() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFF);
        cpu.bus.write32(0x80000000, 0x0FF0C113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0);
    }

    #[test]
    fn test_xori_sign_extended() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFF);
        cpu.bus.write32(0x80000000, 0xFFF0C113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFFFFFFFF00);
    }

    #[test]
    fn test_sll() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 1);
        cpu.write_reg(2, 4);
        cpu.bus.write32(0x80000000, 0x002091B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 16);
    }

    #[test]
    fn test_srl() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        cpu.write_reg(2, 4);
        cpu.bus.write32(0x80000000, 0x0020D1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0x08000000);
    }

    #[test]
    fn test_srl_shamt_wrap() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x8000000000000000);
        cpu.write_reg(2, 68);
        cpu.bus.write32(0x80000000, 0x0020D1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0x0800000000000000);
    }

    #[test]
    fn test_sra() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x8000000000000000);
        cpu.write_reg(2, 4);
        cpu.bus.write32(0x80000000, 0x4020D1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0xF800000000000000);
    }

    #[test]
    fn test_slli() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 1);
        cpu.bus.write32(0x80000000, 0x00409113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 16);
    }

    #[test]
    fn test_srli() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        cpu.bus.write32(0x80000000, 0x0040D113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x08000000);
    }

    #[test]
    fn test_srai() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x8000000000000000);
        cpu.bus.write32(0x80000000, 0x4040D113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xF800000000000000);
    }

    #[test]
    fn test_slt_signed() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, (-5_i32) as u64);
        cpu.write_reg(2, 5);
        cpu.bus.write32(0x80000000, 0x0020A1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 1);
    }

    #[test]
    fn test_sltu_unsigned() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, (-5_i32) as u64);
        cpu.write_reg(2, 5);
        cpu.bus.write32(0x80000000, 0x0020B1B3);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0);
    }

    #[test]
    fn test_slti() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 5);
        cpu.bus.write32(0x80000000, 0x00A0A113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 1);
    }

    #[test]
    fn test_sltiu() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 5);
        cpu.bus.write32(0x80000000, 0xFFF0B113);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 1);
    }

    #[test]
    fn test_sw_lw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.write_reg(2, 0xDEADBEEF);
        cpu.bus.write32(0x80000000, 0x0020A023);
        cpu.step();
        cpu.bus.write32(0x80000004, 0x0000A183);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0xFFFFFFFFDEADBEEF);
    }

    #[test]
    fn test_lb_sign_extend() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.bus.write8(0x80001000, 0x80);
        cpu.bus.write32(0x80000000, 0x00008103);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFFFFFFFF80);
    }

    #[test]
    fn test_lbu_zero_extend() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.bus.write8(0x80001000, 0x80);
        cpu.bus.write32(0x80000000, 0x0000C103);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x00000080);
    }

    #[test]
    fn test_lh_sign_extend() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.bus.write16(0x80001000, 0x8000);
        cpu.bus.write32(0x80000000, 0x00009103);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFFFFFF8000);
    }

    #[test]
    fn test_lhu_zero_extend() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.bus.write16(0x80001000, 0x8000);
        cpu.bus.write32(0x80000000, 0x0000D103);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x00008000);
    }

    #[test]
    fn test_ld() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.bus.write64(0x80001000, 0xDEADBEEFCAFEBABE);
        cpu.bus.write32(0x80000000, 0x0000B103);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xDEADBEEFCAFEBABE);
    }

    #[test]
    fn test_sd() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.write_reg(2, 0x123456789ABCDEF0);
        cpu.bus.write32(0x80000000, 0x0020B023);
        cpu.step();
        assert_eq!(cpu.bus.read64(0x80001000), 0x123456789ABCDEF0);
    }

    #[test]
    fn test_sd_ld() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.write_reg(2, 0xFEDCBA9876543210);
        cpu.bus.write32(0x80000000, 0x0020B023);
        cpu.step();
        cpu.bus.write32(0x80000004, 0x0000B183);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0xFEDCBA9876543210);
    }

    #[test]
    fn test_lwu() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.bus.write32(0x80001000, 0xDEADBEEF);
        cpu.bus.write32(0x80000000, 0x0000E103);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x00000000DEADBEEF);
    }

    #[test]
    fn test_sb() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.write_reg(2, 0xDEADBEEF);
        cpu.bus.write32(0x80000000, 0x00208023);
        cpu.step();
        assert_eq!(cpu.bus.read8(0x80001000), 0xEF);
    }

    #[test]
    fn test_sh() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80001000);
        cpu.write_reg(2, 0xDEADBEEF);
        cpu.bus.write32(0x80000000, 0x00209023);
        cpu.step();
        assert_eq!(cpu.bus.read16(0x80001000), 0xBEEF);
    }

    #[test]
    fn test_beq_taken() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 100);
        cpu.write_reg(2, 100);
        cpu.bus.write32(0x80000000, 0x00208463);
        cpu.step();
        assert_eq!(cpu.pc, 0x80000008);
    }

    #[test]
    fn test_beq_not_taken() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 100);
        cpu.write_reg(2, 200);
        cpu.bus.write32(0x80000000, 0x00208463);
        cpu.step();
        assert_eq!(cpu.pc, 0x80000004);
    }

    #[test]
    fn test_bne_taken() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 100);
        cpu.write_reg(2, 200);
        cpu.bus.write32(0x80000000, 0x00209463);
        cpu.step();
        assert_eq!(cpu.pc, 0x80000008);
    }

    #[test]
    fn test_blt_signed() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, (-5_i32) as u64);
        cpu.write_reg(2, 5);
        cpu.bus.write32(0x80000000, 0x0020C463);
        cpu.step();
        assert_eq!(cpu.pc, 0x80000008);
    }

    #[test]
    fn test_bge_signed() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 5);
        cpu.write_reg(2, (-5_i32) as u64);
        cpu.bus.write32(0x80000000, 0x0020D463);
        cpu.step();
        assert_eq!(cpu.pc, 0x80000008);
    }

    #[test]
    fn test_bltu_unsigned() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 5);
        cpu.write_reg(2, (-1_i32) as u64);
        cpu.bus.write32(0x80000000, 0x0020E463);
        cpu.step();
        assert_eq!(cpu.pc, 0x80000008);
    }

    #[test]
    fn test_bgeu_unsigned() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, (-1_i32) as u64);
        cpu.write_reg(2, 5);
        cpu.bus.write32(0x80000000, 0x0020F463);
        cpu.step();
        assert_eq!(cpu.pc, 0x80000008);
    }

    #[test]
    fn test_branch_backward() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x80000008;
        cpu.write_reg(1, 1);
        cpu.write_reg(2, 1);
        cpu.bus.write32(0x80000008, 0xFE208CE3);
        cpu.step();
        assert_eq!(cpu.pc, 0x80000000);
    }

    #[test]
    fn test_jal() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0x008000EF);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0x80000004);
        assert_eq!(cpu.pc, 0x80000008);
    }

    #[test]
    fn test_jal_backward() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x80000008;
        cpu.bus.write32(0x80000008, 0xFFDFF0EF);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0x8000000C);
        assert_eq!(cpu.pc, 0x80000004);
    }

    #[test]
    fn test_jalr() {
        let mut cpu = Cpu::new();
        cpu.write_reg(2, 0x80001000);
        cpu.bus.write32(0x80000000, 0x000100E7);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0x80000004);
        assert_eq!(cpu.pc, 0x80001000);
    }

    #[test]
    fn test_jalr_with_offset() {
        let mut cpu = Cpu::new();
        cpu.write_reg(2, 0x80001000);
        cpu.bus.write32(0x80000000, 0x004100E7);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0x80000004);
        assert_eq!(cpu.pc, 0x80001004);
    }

    #[test]
    fn test_jalr_clears_lsb() {
        let mut cpu = Cpu::new();
        cpu.write_reg(2, 0x80001001);
        cpu.bus.write32(0x80000000, 0x000100E7);
        cpu.step();
        assert_eq!(cpu.pc, 0x80001000);
    }

    #[test]
    fn test_lui() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0x123450B7);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0x12345000);
    }

    #[test]
    fn test_lui_high_bit() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0x800000B7);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0xFFFFFFFF80000000);
    }

    #[test]
    fn test_auipc() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0x12345097);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0x80000000 + 0x12345000);
    }

    #[test]
    fn test_auipc_different_pc() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x80001000;
        cpu.bus.write32(0x80001000, 0x00001097);
        cpu.step();
        assert_eq!(cpu.read_reg(1), 0x80001000 + 0x1000);
    }

    #[test]
    fn test_ecall() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0x00000073);
        cpu.step();
        assert!(cpu.halted);
    }

    #[test]
    fn test_ebreak() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x80000000, 0x00100073);
        cpu.step();
        assert!(cpu.halted);
    }

    #[test]
    fn test_ecall_check_a0() {
        let mut cpu = Cpu::new();
        cpu.write_reg(10, 0);
        cpu.bus.write32(0x80000000, 0x00000073);
        cpu.step();
        assert!(cpu.halted);
        assert_eq!(cpu.read_reg(10), 0);
    }

    // === RV64I W suffix operations ===
    #[test]
    fn test_addiw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 10);
        cpu.bus.write32(0x80000000, 0x0140811B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 30);
    }

    #[test]
    fn test_addiw_overflow() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x7FFFFFFF);
        cpu.bus.write32(0x80000000, 0x0010811B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFF80000000);
    }

    #[test]
    fn test_addiw_negative() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0);
        cpu.bus.write32(0x80000000, 0xFFF0811B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn test_addiw_ignores_upper_bits() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFFFFFFFF00000005);
        cpu.bus.write32(0x80000000, 0x0030811B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 8);
    }

    #[test]
    fn test_slliw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 1);
        cpu.bus.write32(0x80000000, 0x0040911B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 16);
    }

    #[test]
    fn test_slliw_sign_extend() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x40000000);
        cpu.bus.write32(0x80000000, 0x0010911B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFF80000000);
    }

    #[test]
    fn test_srliw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        cpu.bus.write32(0x80000000, 0x0040D11B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x08000000);
    }

    #[test]
    fn test_srliw_upper_ignored() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFFFFFFFF80000000);
        cpu.bus.write32(0x80000000, 0x0040D11B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x08000000);
    }

    #[test]
    fn test_sraiw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        cpu.bus.write32(0x80000000, 0x4040D11B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0xFFFFFFFFF8000000);
    }

    #[test]
    fn test_sraiw_positive() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x40000000);
        cpu.bus.write32(0x80000000, 0x4040D11B);
        cpu.step();
        assert_eq!(cpu.read_reg(2), 0x04000000);
    }

    #[test]
    fn test_addw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 10);
        cpu.write_reg(2, 20);
        cpu.bus.write32(0x80000000, 0x002081BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 30);
    }

    #[test]
    fn test_addw_overflow() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x7FFFFFFF);
        cpu.write_reg(2, 1);
        cpu.bus.write32(0x80000000, 0x002081BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0xFFFFFFFF80000000);
    }

    #[test]
    fn test_subw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 100);
        cpu.write_reg(2, 30);
        cpu.bus.write32(0x80000000, 0x402081BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 70);
    }

    #[test]
    fn test_subw_underflow() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0);
        cpu.write_reg(2, 1);
        cpu.bus.write32(0x80000000, 0x402081BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn test_sllw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 1);
        cpu.write_reg(2, 4);
        cpu.bus.write32(0x80000000, 0x002091BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 16);
    }

    #[test]
    fn test_sllw_sign_extend() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x40000000);
        cpu.write_reg(2, 1);
        cpu.bus.write32(0x80000000, 0x002091BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0xFFFFFFFF80000000);
    }

    #[test]
    fn test_srlw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        cpu.write_reg(2, 4);
        cpu.bus.write32(0x80000000, 0x0020D1BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0x08000000);
    }

    #[test]
    fn test_sraw() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x80000000);
        cpu.write_reg(2, 4);
        cpu.bus.write32(0x80000000, 0x4020D1BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0xFFFFFFFFF8000000);
    }

    #[test]
    fn test_sraw_positive() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x40000000);
        cpu.write_reg(2, 4);
        cpu.bus.write32(0x80000000, 0x4020D1BB);
        cpu.step();
        assert_eq!(cpu.read_reg(3), 0x04000000);
    }
}
