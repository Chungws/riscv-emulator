use core::panic;

use crate::{Bus, Csr, csr, debug_log, decoder, devices, elf};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrivilegeMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

pub struct Cpu {
    pub regs: [u64; 32],
    pub csr: Csr,
    pub pc: u64,
    pub mode: PrivilegeMode,
    pub bus: Bus,
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut csr = Csr::new();
        // misa: RV64I + S + U 지원
        // 비트 63-62: MXL=2 (64비트)
        // 비트 8: I (기본 정수)
        // 비트 18: S (Supervisor)
        // 비트 20: U (User)
        csr.write(csr::MISA, 0x8000000000140100);

        // mhartid: single core = 0
        csr.write(csr::MHARTID, 0);

        Self {
            regs: [0; 32],
            csr: csr,
            pc: devices::memory::DRAM_BASE,
            mode: PrivilegeMode::Machine,
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

    pub fn fetch(&mut self) -> u32 {
        self.bus.read32(self.pc)
    }

    pub fn load_program(&mut self, program: &[u32]) {
        for (i, &inst) in program.iter().enumerate() {
            let addr = devices::memory::DRAM_BASE + (i as u64) * 4;
            self.bus.write32(addr, inst);
        }
    }

    pub fn load_segments(&mut self, segments: &[elf::Segment], entry: u64) {
        for segment in segments {
            for (i, byte) in segment.data.iter().enumerate() {
                self.bus.write8(segment.vaddr + i as u64, *byte);
            }
            for i in segment.data.len()..segment.memsz as usize {
                self.bus.write8(segment.vaddr + i as u64, 0);
            }
        }
        self.pc = entry;
    }

    pub fn trap(&mut self, cause: u64, tval: u64) {
        let is_interrupt = (cause & csr::INTERRUPT_BIT) > 0;
        self.csr.write(csr::MEPC, self.pc);
        self.csr.write(csr::MCAUSE, cause);
        self.csr.write(csr::MTVAL, tval);

        let mut mstatus = self.csr.read(csr::MSTATUS);
        let mie = (mstatus & csr::MSTATUS_MIE) != 0;
        if mie {
            mstatus |= csr::MSTATUS_MPIE;
        } else {
            mstatus &= !csr::MSTATUS_MPIE;
        }
        mstatus &= !csr::MSTATUS_MIE;

        mstatus &= !csr::MSTATUS_MPP;
        mstatus |= (self.mode as u64) << 11;

        self.csr.write(csr::MSTATUS, mstatus);

        self.mode = PrivilegeMode::Machine;

        let mtvec = self.csr.read(csr::MTVEC);
        let mode = mtvec & 0x3;
        let base = mtvec & !0x3;
        if mode == 0 {
            self.pc = base;
        } else {
            if is_interrupt {
                self.pc = base + 4 * (cause & 0x3FF);
            } else {
                self.pc = base;
            }
        }
    }

    pub fn run(&mut self) {
        while !self.halted {
            self.step();
        }
    }

    pub fn step(&mut self) {
        self.bus.receive_uart_input();
        self.bus.tick();
        if self.check_pending_interrupts() {
            return;
        }

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
            SYSTEM => {
                if self.execute_system(inst) {
                    return; // trap 시 PC 증가 안함
                }
            }
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
                debug_log!(
                    "ADDI rd={}, rs1={}, rs1_val={}, imm={}",
                    rd,
                    rs1,
                    rs1_val,
                    imm
                );
                self.write_reg(rd, rs1_val.wrapping_add(imm as u64));
            }
            0x1 => {
                let shamt = (imm as u64) & 0x3F;
                debug_log!(
                    "SLLI rd={}, rs1={}, rs1_val={}, shamt={}",
                    rd,
                    rs1,
                    rs1_val,
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
                let result = if (rs1_val as i64) < (imm as i64) {
                    1
                } else {
                    0
                };
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
                let result = if rs1_val < (imm as u64) { 1 } else { 0 };
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
                self.write_reg(rd, rs1_val ^ (imm as u64));
            }
            0x5 => {
                let funct7 = ((imm as u64) >> 5) & 0x7F;
                let shamt = (imm as u64) & 0x3F;
                match funct7 {
                    0x00 => {
                        debug_log!(
                            "SRLI rd={}, rs1={}, rs1_val={}, shamt={}",
                            rd,
                            rs1,
                            rs1_val,
                            shamt
                        );
                        self.write_reg(rd, rs1_val >> shamt);
                    }
                    0x20 => {
                        debug_log!(
                            "SRAI rd={}, rs1={}, rs1_val={}, shamt={}",
                            rd,
                            rs1,
                            rs1_val,
                            shamt
                        );
                        self.write_reg(rd, ((rs1_val as i64) >> shamt) as u64);
                    }
                    _ => panic!("Not Implemented OP_IMM funct7: {:#x}", funct7),
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
                self.write_reg(rd, rs1_val | (imm as u64));
            }
            0x7 => {
                debug_log!(
                    "ANDI rd={}, rs1={}, rs1_val={}, imm={}",
                    rd,
                    rs1,
                    rs1_val,
                    imm
                );
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
                debug_log!(
                    "ADDIW rd={}, rs1={}, rs1_val={}, imm={}",
                    rd,
                    rs1,
                    rs1_val,
                    imm
                );
                let result = (rs1_val as i32).wrapping_add(imm as i32);
                self.write_reg(rd, result as i64 as u64);
            }
            0x1 => {
                let shamt = (imm as u64) & 0x3F;
                debug_log!(
                    "SLLIW rd={}, rs1={}, rs1_val={}, shamt={}",
                    rd,
                    rs1,
                    rs1_val,
                    shamt
                );
                self.write_reg(rd, ((rs1_val as u32) << shamt) as i32 as i64 as u64);
            }
            0x5 => {
                let funct7 = ((imm as u64) >> 5) & 0x7F;
                let shamt = (imm as u64) & 0x3F;
                match funct7 {
                    0x00 => {
                        debug_log!(
                            "SRLIW rd={}, rs1={}, rs1_val={}, shamt={}",
                            rd,
                            rs1,
                            rs1_val,
                            shamt
                        );
                        self.write_reg(rd, ((rs1_val as u32) >> shamt) as u64);
                    }
                    0x20 => {
                        debug_log!(
                            "SRAIW rd={}, rs1={}, rs1_val={}, shamt={}",
                            rd,
                            rs1,
                            rs1_val,
                            shamt
                        );
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
            (0x0, 0x01) => {
                debug_log!("MUL rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                self.write_reg(rd, rs1_val.wrapping_mul(rs2_val));
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
            (0x1, 0x01) => {
                debug_log!("MULH rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let res = (rs1_val as i64 as i128) * (rs2_val as i64 as i128);
                self.write_reg(rd, (res >> 64) as i64 as u64);
            }
            (0x2, 0x0) => {
                debug_log!("SLT rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let result = if (rs1_val as i64) < (rs2_val as i64) {
                    1
                } else {
                    0
                };
                self.write_reg(rd, result);
            }
            (0x2, 0x01) => {
                debug_log!("MULHSU rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let res = (rs1_val as i64 as i128) * (rs2_val as u128 as i128);
                self.write_reg(rd, (res >> 64) as i64 as u64);
            }
            (0x3, 0x0) => {
                debug_log!("SLTU rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let result = if rs1_val < rs2_val { 1 } else { 0 };
                self.write_reg(rd, result);
            }
            (0x3, 0x01) => {
                debug_log!("MULHU rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let res = (rs1_val as u128) * (rs2_val as u128);
                self.write_reg(rd, (res >> 64) as u64);
            }
            (0x4, 0x0) => {
                debug_log!("XOR rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                self.write_reg(rd, rs1_val ^ rs2_val);
            }
            (0x4, 0x01) => {
                debug_log!("DIV rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                if rs2_val == 0 {
                    self.write_reg(rd, -1 as i64 as u64);
                } else {
                    let res = (rs1_val as i64).wrapping_div(rs2_val as i64);
                    self.write_reg(rd, res as u64);
                }
            }
            (0x5, 0x0) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SRL rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, rs1_val >> shamt);
            }
            (0x5, 0x01) => {
                debug_log!("DIVU rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                if rs2_val == 0 {
                    self.write_reg(rd, u64::MAX);
                } else {
                    let res = rs1_val.wrapping_div(rs2_val);
                    self.write_reg(rd, res);
                }
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
            (0x6, 0x01) => {
                debug_log!("REM rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                if rs2_val == 0 {
                    self.write_reg(rd, rs1_val as i64 as u64);
                } else {
                    let res = (rs1_val as i64).wrapping_rem(rs2_val as i64);
                    self.write_reg(rd, res as u64);
                }
            }
            (0x7, 0x0) => {
                debug_log!("AND rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                self.write_reg(rd, rs1_val & rs2_val);
            }
            (0x7, 0x01) => {
                debug_log!("REMU rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                if rs2_val == 0 {
                    self.write_reg(rd, rs1_val);
                } else {
                    let res = rs1_val.wrapping_rem(rs2_val);
                    self.write_reg(rd, res);
                }
            }
            _ => panic!(
                "Not Implemented OP funct3={:#x}, funct7={:#x}",
                funct3, funct7
            ),
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
            (0x0, 0x01) => {
                debug_log!("MULW rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                let result = (rs1_val as i32).wrapping_mul(rs2_val as i32);
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
            (0x4, 0x01) => {
                debug_log!("DIVW rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                if rs2_val == 0 {
                    self.write_reg(rd, -1 as i32 as i64 as u64);
                } else {
                    let res = (rs1_val as u32 as i32).wrapping_div(rs2_val as u32 as i32);
                    self.write_reg(rd, res as u64);
                }
            }
            (0x5, 0x0) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SRLW rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, ((rs1_val as u32) >> shamt) as u64);
            }
            (0x5, 0x01) => {
                debug_log!("DIVUW rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                if rs2_val == 0 {
                    self.write_reg(rd, -1 as i32 as u64);
                } else {
                    let res = (rs1_val as u32).wrapping_div(rs2_val as u32) as i32 as i64;
                    self.write_reg(rd, res as u64);
                }
            }
            (0x5, 0x20) => {
                let shamt = rs2_val & 0x3F;
                debug_log!("SRAW rd={}, rs1_val={}, shamt={}", rd, rs1_val, shamt);
                self.write_reg(rd, ((rs1_val as i32) >> shamt) as i64 as u64);
            }
            (0x6, 0x01) => {
                debug_log!("REMW rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                if rs2_val == 0 {
                    self.write_reg(rd, rs1_val as u32 as i32 as i64 as u64);
                } else {
                    let res = (rs1_val as i32).wrapping_rem(rs2_val as i32);
                    self.write_reg(rd, res as i64 as u64);
                }
            }
            (0x7, 0x01) => {
                debug_log!("REMUW rd={}, rs1_val={}, rs2_val={}", rd, rs1_val, rs2_val);
                if rs2_val == 0 {
                    self.write_reg(rd, rs1_val as u32 as i32 as i64 as u64);
                } else {
                    let res = (rs1_val as u32).wrapping_rem(rs2_val as u32) as i32 as i64;
                    self.write_reg(rd, res as u64);
                }
            }
            _ => panic!(
                "Not Implemented OP_32 funct3={:#x}, funct7={:#x}",
                funct3, funct7
            ),
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

    fn execute_system(&mut self, inst: u32) -> bool {
        debug_log!("SYSTEM");
        let funct3 = decoder::funct3(inst);
        let rd = decoder::rd(inst);
        let rs1 = decoder::rs1(inst);
        let rs1_val = self.read_reg(rs1);
        let csr_addr = decoder::csr_addr(inst);

        let taken = match funct3 {
            0x0 => {
                let funct7 = decoder::funct7(inst);
                let rs2 = decoder::rs2(inst);
                match (funct7, rs2) {
                    (0x00, 0x00) => {
                        debug_log!("ECALL");
                        match self.mode {
                            PrivilegeMode::Machine => {
                                debug_log!("ECALL Machine Mode");
                                self.trap(csr::ECALL_FROM_M, 0);
                            }
                            PrivilegeMode::Supervisor => {
                                debug_log!("ECALL Supervisor Mode");
                                self.trap(csr::ECALL_FROM_S, 0);
                            }
                            PrivilegeMode::User => {
                                debug_log!("ECALL User Mode");
                                self.trap(csr::ECALL_FROM_U, 0);
                            }
                        }
                        true
                    }
                    (0x00, 0x01) => {
                        debug_log!("EBREAK");
                        self.trap(csr::BREAKPOINT, 0);
                        true
                    }
                    (0x18, 0x02) => {
                        debug_log!("MRET");
                        self.pc = self.csr.read(csr::MEPC);

                        let mut mstatus = self.csr.read(csr::MSTATUS);
                        let mpie = (mstatus & csr::MSTATUS_MPIE) != 0;
                        if mpie {
                            mstatus |= csr::MSTATUS_MIE;
                        } else {
                            mstatus &= !csr::MSTATUS_MIE;
                        }
                        mstatus |= csr::MSTATUS_MPIE;

                        let mpp = (mstatus & csr::MSTATUS_MPP) >> 11;
                        self.mode = match mpp {
                            0 => PrivilegeMode::User,
                            1 => PrivilegeMode::Supervisor,
                            3 => PrivilegeMode::Machine,
                            _ => panic!("Not Avaliable PrivilegeMode"),
                        };
                        mstatus &= !csr::MSTATUS_MPP;
                        self.csr.write(csr::MSTATUS, mstatus);
                        true
                    }
                    (0x08, 0x02) => {
                        debug_log!("SRET");
                        self.pc = self.csr.read(csr::SEPC);

                        let mut sstatus = self.csr.read(csr::SSTATUS);
                        let spie = (sstatus & csr::SSTATUS_SPIE) != 0;
                        if spie {
                            sstatus |= csr::SSTATUS_SIE;
                        } else {
                            sstatus &= !csr::SSTATUS_SIE;
                        }
                        sstatus |= csr::SSTATUS_SPIE;

                        let spp = (sstatus & csr::SSTATUS_SPP) != 0;
                        self.mode = if spp {
                            PrivilegeMode::Supervisor
                        } else {
                            PrivilegeMode::User
                        };
                        sstatus &= !csr::SSTATUS_SPP;
                        self.csr.write(csr::SSTATUS, sstatus);
                        true
                    }
                    _ => panic!("Not Implemented!"),
                }
            }
            0x1 => {
                debug_log!(
                    "CSRRW rd={}, rs1={}, rs1_val={}, csr_addr={}",
                    rd,
                    rs1,
                    rs1_val,
                    csr_addr
                );
                let old = self.csr.read(csr_addr);
                self.csr.write(csr_addr, rs1_val);
                self.write_reg(rd, old);
                false
            }
            0x2 => {
                debug_log!(
                    "CSRRS rd={}, rs1={}, rs1_val={}, csr_addr={}",
                    rd,
                    rs1,
                    rs1_val,
                    csr_addr
                );
                let old = self.csr.read(csr_addr);
                if rs1_val != 0x0 {
                    self.csr.write(csr_addr, old | rs1_val);
                }
                self.write_reg(rd, old);
                false
            }
            0x3 => {
                debug_log!(
                    "CSRRC rd={}, rs1={}, rs1_val={}, csr_addr={}",
                    rd,
                    rs1,
                    rs1_val,
                    csr_addr
                );
                let old = self.csr.read(csr_addr);
                if rs1_val != 0x0 {
                    self.csr.write(csr_addr, old & !rs1_val);
                }
                self.write_reg(rd, old);
                false
            }
            0x5 => {
                debug_log!(
                    "CSRRWI rd={}, rs1={}, rs1_val={}, csr_addr={}",
                    rd,
                    rs1,
                    rs1_val,
                    csr_addr
                );
                let old = self.csr.read(csr_addr);
                self.csr.write(csr_addr, rs1 as u64);
                self.write_reg(rd, old);
                false
            }
            0x6 => {
                debug_log!(
                    "CSRRSI rd={}, rs1={}, rs1_val={}, csr_addr={}",
                    rd,
                    rs1,
                    rs1_val,
                    csr_addr
                );
                let old = self.csr.read(csr_addr);
                if rs1 != 0x0 {
                    self.csr.write(csr_addr, old | (rs1 as u64));
                }
                self.write_reg(rd, old);
                false
            }
            0x7 => {
                debug_log!(
                    "CSRRCI rd={}, rs1={}, rs1_val={}, csr_addr={}",
                    rd,
                    rs1,
                    rs1_val,
                    csr_addr
                );
                let old = self.csr.read(csr_addr);
                if rs1 != 0x0 {
                    self.csr.write(csr_addr, old & !(rs1 as u64));
                }
                self.write_reg(rd, old);
                false
            }
            _ => panic!("Unknown SYSTEM csr_addr: {:#x}", csr_addr),
        };
        taken
    }

    fn check_pending_interrupts(&mut self) -> bool {
        let mip = self.csr.read(csr::MIP);
        if self.bus.check_timer_interrupt() {
            self.csr.write(csr::MIP, mip | csr::MIP_MTIP);
        } else {
            self.csr.write(csr::MIP, mip & !csr::MIP_MTIP);
        }

        let mip = self.csr.read(csr::MIP);
        if self.bus.check_software_interrupt() {
            self.csr.write(csr::MIP, mip | csr::MIP_MSIP);
        } else {
            self.csr.write(csr::MIP, mip & !csr::MIP_MSIP);
        }

        let mip = self.csr.read(csr::MIP);
        if self.bus.check_uart_interrupt() {
            self.csr.write(csr::MIP, mip | csr::MIP_MEIP);
        } else {
            self.csr.write(csr::MIP, mip & !csr::MIP_MEIP);
        }

        let mstatus = self.csr.read(csr::MSTATUS);
        if mstatus & csr::MSTATUS_MIE == 0 {
            return false;
        }

        let mip = self.csr.read(csr::MIP);
        let mie = self.csr.read(csr::MIE);

        if (mip & csr::MIP_MSIP != 0) && (mie & csr::MIE_MSIE != 0) {
            self.trap(csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_SOFTWARE, 0);
            return true;
        }

        if (mip & csr::MIP_MTIP != 0) && (mie & csr::MIE_MTIE != 0) {
            self.trap(csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_TIMER, 0);
            return true;
        }

        if (mip & csr::MIP_MEIP != 0) && (mie & csr::MIE_MEIE != 0) {
            self.trap(csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_EXTERNAL, 0);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests;
