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
fn test_misa_init() {
    let cpu = Cpu::new();
    let misa = cpu.csr.read(csr::MISA);

    // MXL = 2 (64-bit)
    assert_eq!(misa >> 62, 2);

    // I extension (bit 8)
    assert_ne!(misa & (1 << 8), 0);

    // S extension (bit 18)
    assert_ne!(misa & (1 << 18), 0);

    // U extension (bit 20)
    assert_ne!(misa & (1 << 20), 0);
}

#[test]
fn test_mhartid_init() {
    let cpu = Cpu::new();
    assert_eq!(cpu.csr.read(csr::MHARTID), 0); // single core
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

// === Trap Tests ===

#[test]
fn test_ecall_from_m_mode() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001000);
    cpu.bus.write32(0x80000000, 0x00000073); // ecall
    cpu.step();

    assert_eq!(cpu.pc, 0x80001000); // jumped to mtvec
    assert_eq!(cpu.csr.read(csr::MEPC), 0x80000000); // saved old PC
    assert_eq!(cpu.csr.read(csr::MCAUSE), csr::ECALL_FROM_M); // cause = 11
    assert_eq!(cpu.mode, PrivilegeMode::Machine);
}

#[test]
fn test_ecall_from_s_mode() {
    let mut cpu = Cpu::new();
    cpu.mode = PrivilegeMode::Supervisor;
    cpu.csr.write(csr::MTVEC, 0x80001000);
    cpu.bus.write32(0x80000000, 0x00000073); // ecall
    cpu.step();

    assert_eq!(cpu.pc, 0x80001000);
    assert_eq!(cpu.csr.read(csr::MCAUSE), csr::ECALL_FROM_S); // cause = 9
    assert_eq!(cpu.mode, PrivilegeMode::Machine); // switched to M
}

#[test]
fn test_ebreak() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001000);
    cpu.bus.write32(0x80000000, 0x00100073); // ebreak
    cpu.step();

    assert_eq!(cpu.pc, 0x80001000);
    assert_eq!(cpu.csr.read(csr::MEPC), 0x80000000);
    assert_eq!(cpu.csr.read(csr::MCAUSE), csr::BREAKPOINT); // cause = 3
}

#[test]
fn test_trap_saves_mstatus() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001000);
    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE); // MIE = 1
    cpu.bus.write32(0x80000000, 0x00000073); // ecall
    cpu.step();

    let mstatus = cpu.csr.read(csr::MSTATUS);
    assert_eq!(mstatus & csr::MSTATUS_MPIE, csr::MSTATUS_MPIE); // MPIE = old MIE
    assert_eq!(mstatus & csr::MSTATUS_MIE, 0); // MIE = 0
    assert_eq!(mstatus & csr::MSTATUS_MPP, csr::MSTATUS_MPP); // MPP = Machine (3)
}

#[test]
fn test_trap_mpp_stores_previous_mode() {
    let mut cpu = Cpu::new();
    cpu.mode = PrivilegeMode::Supervisor;
    cpu.csr.write(csr::MTVEC, 0x80001000);
    cpu.bus.write32(0x80000000, 0x00000073); // ecall
    cpu.step();

    let mstatus = cpu.csr.read(csr::MSTATUS);
    // MPP should be 1 (Supervisor)
    assert_eq!((mstatus & csr::MSTATUS_MPP) >> 11, 1);
}

#[test]
fn test_ecall_no_pc_increment() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001000);
    cpu.bus.write32(0x80000000, 0x00000073); // ecall
    cpu.step();

    // PC should be mtvec, not mtvec + 4
    assert_eq!(cpu.pc, 0x80001000);
    // mepc should be the ecall instruction address
    assert_eq!(cpu.csr.read(csr::MEPC), 0x80000000);
}

#[test]
fn test_trap_mtvec_direct_mode() {
    // mtvec mode = 0 (Direct): 모든 트랩이 base로
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001000); // mode = 0
    cpu.bus.write32(0x80000000, 0x00000073); // ecall
    cpu.step();

    assert_eq!(cpu.pc, 0x80001000);
}

#[test]
fn test_trap_mtvec_direct_mode_strips_mode_bits() {
    // mtvec에 mode 비트가 있어도 base만 사용
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001000 | 0x0); // 명시적 Direct mode
    cpu.bus.write32(0x80000000, 0x00000073); // ecall
    cpu.step();

    assert_eq!(cpu.pc, 0x80001000);
}

#[test]
fn test_trap_mtvec_vectored_mode_exception() {
    // mtvec mode = 1 (Vectored): 예외는 여전히 base로
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001000 | 0x1); // mode = 1 (Vectored)
    cpu.bus.write32(0x80000000, 0x00000073); // ecall (예외)
    cpu.step();

    // 예외는 Vectored 모드에서도 base로 점프
    assert_eq!(cpu.pc, 0x80001000);
}

#[test]
fn test_trap_mtvec_vectored_mode_extracts_base() {
    // Vectored 모드에서 하위 2비트 제거 확인
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001001); // base=0x80001000, mode=1
    cpu.bus.write32(0x80000000, 0x00100073); // ebreak
    cpu.step();

    // base = mtvec & !0x3 = 0x80001000
    assert_eq!(cpu.pc, 0x80001000);
}

// === MRET/SRET Tests ===

#[test]
fn test_mret_restores_pc() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MEPC, 0x80002000);
    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MPP); // MPP = Machine (3)
    cpu.bus.write32(0x80000000, 0x30200073); // mret
    cpu.step();

    assert_eq!(cpu.pc, 0x80002000);
}

#[test]
fn test_mret_restores_mode_from_mpp() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MEPC, 0x80002000);
    // MPP = Supervisor (1 << 11)
    cpu.csr.write(csr::MSTATUS, 1 << 11);
    cpu.bus.write32(0x80000000, 0x30200073); // mret
    cpu.step();

    assert_eq!(cpu.mode, PrivilegeMode::Supervisor);
}

#[test]
fn test_mret_restores_mie_from_mpie() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MEPC, 0x80002000);
    cpu.csr
        .write(csr::MSTATUS, csr::MSTATUS_MPIE | csr::MSTATUS_MPP); // MPIE=1
    cpu.bus.write32(0x80000000, 0x30200073); // mret
    cpu.step();

    let mstatus = cpu.csr.read(csr::MSTATUS);
    assert_eq!(mstatus & csr::MSTATUS_MIE, csr::MSTATUS_MIE); // MIE = 1
    assert_eq!(mstatus & csr::MSTATUS_MPIE, csr::MSTATUS_MPIE); // MPIE = 1
}

#[test]
fn test_mret_clears_mpp() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MEPC, 0x80002000);
    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MPP); // MPP = Machine
    cpu.bus.write32(0x80000000, 0x30200073); // mret
    cpu.step();

    let mstatus = cpu.csr.read(csr::MSTATUS);
    assert_eq!(mstatus & csr::MSTATUS_MPP, 0); // MPP cleared
}

#[test]
fn test_sret_restores_pc() {
    let mut cpu = Cpu::new();
    cpu.mode = PrivilegeMode::Supervisor;
    cpu.csr.write(csr::SEPC, 0x80003000);
    cpu.csr.write(csr::SSTATUS, csr::SSTATUS_SPP); // SPP = Supervisor
    cpu.bus.write32(0x80000000, 0x10200073); // sret
    cpu.step();

    assert_eq!(cpu.pc, 0x80003000);
}

#[test]
fn test_sret_restores_mode_from_spp() {
    let mut cpu = Cpu::new();
    cpu.mode = PrivilegeMode::Supervisor;
    cpu.csr.write(csr::SEPC, 0x80003000);
    cpu.csr.write(csr::SSTATUS, 0); // SPP = 0 (User)
    cpu.bus.write32(0x80000000, 0x10200073); // sret
    cpu.step();

    assert_eq!(cpu.mode, PrivilegeMode::User);
}

#[test]
fn test_sret_restores_sie_from_spie() {
    let mut cpu = Cpu::new();
    cpu.mode = PrivilegeMode::Supervisor;
    cpu.csr.write(csr::SEPC, 0x80003000);
    cpu.csr
        .write(csr::SSTATUS, csr::SSTATUS_SPIE | csr::SSTATUS_SPP); // SPIE=1, SPP=S
    cpu.bus.write32(0x80000000, 0x10200073); // sret
    cpu.step();

    let sstatus = cpu.csr.read(csr::SSTATUS);
    assert_eq!(sstatus & csr::SSTATUS_SIE, csr::SSTATUS_SIE); // SIE = 1 (from SPIE)
    assert_eq!(sstatus & csr::SSTATUS_SPIE, csr::SSTATUS_SPIE); // SPIE = 1
}

#[test]
fn test_sret_clears_spp() {
    let mut cpu = Cpu::new();
    cpu.mode = PrivilegeMode::Supervisor;
    cpu.csr.write(csr::SEPC, 0x80003000);
    cpu.csr.write(csr::SSTATUS, csr::SSTATUS_SPP); // SPP = Supervisor
    cpu.bus.write32(0x80000000, 0x10200073); // sret
    cpu.step();

    let sstatus = cpu.csr.read(csr::SSTATUS);
    assert_eq!(sstatus & csr::SSTATUS_SPP, 0); // SPP cleared
}

#[test]
fn test_mret_no_pc_increment() {
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MEPC, 0x80002000);
    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MPP);
    cpu.bus.write32(0x80000000, 0x30200073); // mret
    cpu.step();

    // PC should be mepc, not mepc + 4
    assert_eq!(cpu.pc, 0x80002000);
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

// === CSR Instructions ===

#[test]
fn test_csrrw() {
    // CSRRW x1, 0x300, x2
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0xAAAA);
    cpu.write_reg(2, 0xBBBB);
    cpu.bus.write32(0x80000000, 0x300110F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0xAAAA); // rd = old CSR
    assert_eq!(cpu.csr.read(0x300), 0xBBBB); // CSR = rs1
}

#[test]
fn test_csrrw_rd_x0() {
    // CSRRW x0, 0x300, x2 (rd=x0, just write)
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0xAAAA);
    cpu.write_reg(2, 0xBBBB);
    cpu.bus.write32(0x80000000, 0x30011073);
    cpu.step();
    assert_eq!(cpu.read_reg(0), 0); // x0 always 0
    assert_eq!(cpu.csr.read(0x300), 0xBBBB); // CSR = rs1
}

#[test]
fn test_csrrs() {
    // CSRRS x1, 0x300, x2
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0b1100);
    cpu.write_reg(2, 0b0011);
    cpu.bus.write32(0x80000000, 0x300120F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0b1100); // rd = old CSR
    assert_eq!(cpu.csr.read(0x300), 0b1111); // CSR = CSR | rs1
}

#[test]
fn test_csrrs_rs1_x0() {
    // CSRRS x1, 0x300, x0 (read only, no modify)
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0xAAAA);
    cpu.bus.write32(0x80000000, 0x300020F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0xAAAA); // rd = CSR
    assert_eq!(cpu.csr.read(0x300), 0xAAAA); // CSR unchanged
}

#[test]
fn test_csrrc() {
    // CSRRC x1, 0x300, x2
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0b1111);
    cpu.write_reg(2, 0b0011);
    cpu.bus.write32(0x80000000, 0x300130F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0b1111); // rd = old CSR
    assert_eq!(cpu.csr.read(0x300), 0b1100); // CSR = CSR & ~rs1
}

#[test]
fn test_csrrc_rs1_x0() {
    // CSRRC x1, 0x300, x0 (read only, no modify)
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0xAAAA);
    cpu.bus.write32(0x80000000, 0x300030F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0xAAAA); // rd = CSR
    assert_eq!(cpu.csr.read(0x300), 0xAAAA); // CSR unchanged
}

#[test]
fn test_csrrwi() {
    // CSRRWI x1, 0x300, 0x1F (zimm=31)
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0xAAAA);
    cpu.bus.write32(0x80000000, 0x300FD0F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0xAAAA); // rd = old CSR
    assert_eq!(cpu.csr.read(0x300), 0x1F); // CSR = zimm
}

#[test]
fn test_csrrsi() {
    // CSRRSI x1, 0x300, 0x03 (zimm=3)
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0b1100);
    cpu.bus.write32(0x80000000, 0x3001E0F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0b1100); // rd = old CSR
    assert_eq!(cpu.csr.read(0x300), 0b1111); // CSR = CSR | zimm
}

#[test]
fn test_csrrsi_zimm_0() {
    // CSRRSI x1, 0x300, 0 (read only)
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0xAAAA);
    cpu.bus.write32(0x80000000, 0x300060F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0xAAAA); // rd = CSR
    assert_eq!(cpu.csr.read(0x300), 0xAAAA); // CSR unchanged
}

#[test]
fn test_csrrci() {
    // CSRRCI x1, 0x300, 0x03 (zimm=3)
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0b1111);
    cpu.bus.write32(0x80000000, 0x3001F0F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0b1111); // rd = old CSR
    assert_eq!(cpu.csr.read(0x300), 0b1100); // CSR = CSR & ~zimm
}

#[test]
fn test_csrrci_zimm_0() {
    // CSRRCI x1, 0x300, 0 (read only)
    let mut cpu = Cpu::new();
    cpu.csr.write(0x300, 0xAAAA);
    cpu.bus.write32(0x80000000, 0x300070F3);
    cpu.step();
    assert_eq!(cpu.read_reg(1), 0xAAAA); // rd = CSR
    assert_eq!(cpu.csr.read(0x300), 0xAAAA); // CSR unchanged
}

// === Integration Tests ===

#[test]
fn test_ecall_mret_roundtrip() {
    // ecall로 trap → mret으로 복귀하는 전체 흐름 테스트
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80001000); // trap handler at 0x80001000
    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE); // MIE = 1

    // Main code at 0x80000000
    cpu.bus.write32(0x80000000, 0x00000073); // ecall

    // Handler at 0x80001000: just mret
    cpu.bus.write32(0x80001000, 0x30200073); // mret

    // Step 1: ecall
    cpu.step();
    assert_eq!(cpu.pc, 0x80001000); // jumped to handler
    assert_eq!(cpu.csr.read(csr::MEPC), 0x80000000); // saved PC
    assert_eq!(cpu.csr.read(csr::MCAUSE), csr::ECALL_FROM_M);
    let mstatus = cpu.csr.read(csr::MSTATUS);
    assert_eq!(mstatus & csr::MSTATUS_MPIE, csr::MSTATUS_MPIE); // MPIE = old MIE
    assert_eq!(mstatus & csr::MSTATUS_MIE, 0); // MIE = 0

    // Step 2: mret
    cpu.step();
    assert_eq!(cpu.pc, 0x80000000); // returned to ecall
    let mstatus = cpu.csr.read(csr::MSTATUS);
    assert_eq!(mstatus & csr::MSTATUS_MIE, csr::MSTATUS_MIE); // MIE restored
}

#[test]
fn test_ecall_mret_roundtrip_from_supervisor() {
    // S-mode에서 ecall → M-mode handler → mret으로 S-mode 복귀
    let mut cpu = Cpu::new();
    cpu.mode = PrivilegeMode::Supervisor;
    cpu.csr.write(csr::MTVEC, 0x80001000);

    cpu.bus.write32(0x80000000, 0x00000073); // ecall
    cpu.bus.write32(0x80001000, 0x30200073); // mret

    // Step 1: ecall from S-mode
    cpu.step();
    assert_eq!(cpu.mode, PrivilegeMode::Machine);
    assert_eq!(cpu.csr.read(csr::MCAUSE), csr::ECALL_FROM_S);
    let mpp = (cpu.csr.read(csr::MSTATUS) & csr::MSTATUS_MPP) >> 11;
    assert_eq!(mpp, 1); // MPP = Supervisor

    // Step 2: mret
    cpu.step();
    assert_eq!(cpu.mode, PrivilegeMode::Supervisor); // restored to S
    assert_eq!(cpu.pc, 0x80000000);
}

#[test]
fn test_uart_output_rv64() {
    // UART로 "RV64!" 출력 + 64비트 연산 검증
    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80002000); // ecall handler

    let program: Vec<u32> = vec![
        // x1 = 0x10000000 (UART address)
        0x100000B7, // lui x1, 0x10000
        // 'R' output
        0x05200113, // addi x2, x0, 82
        0x00208023, // sb x2, 0(x1)
        // 'V' output
        0x05600113, // addi x2, x0, 86
        0x00208023, // sb x2, 0(x1)
        // '6' output
        0x03600113, // addi x2, x0, 54
        0x00208023, // sb x2, 0(x1)
        // '4' output
        0x03400113, // addi x2, x0, 52
        0x00208023, // sb x2, 0(x1)
        // 64-bit arithmetic test: -1 + 2 = 1
        0xFFF00193, // addi x3, x0, -1        # x3 = 0xFFFFFFFFFFFFFFFF
        0x00218213, // addi x4, x3, 2         # x4 = 1
        // '!' output
        0x02100113, // addi x2, x0, 33
        0x00208023, // sb x2, 0(x1)
        // '\n' output
        0x00A00113, // addi x2, x0, 10
        0x00208023, // sb x2, 0(x1)
        // halt via ecall
        0x00000073, // ecall
    ];

    cpu.load_program(&program);

    // Run until ecall (16 instructions)
    for _ in 0..16 {
        cpu.step();
    }

    // Verify 64-bit arithmetic
    assert_eq!(cpu.read_reg(3), 0xFFFFFFFFFFFFFFFF);
    assert_eq!(cpu.read_reg(4), 1);

    // Verify we hit ecall (PC jumped to mtvec)
    assert_eq!(cpu.pc, 0x80002000);
}

#[test]
fn test_sum_1_to_10_loop() {
    // 1부터 10까지 더하는 루프 테스트
    // sum = 0; i = 1
    // while (i < 11) { sum += i; i++; }
    // result: sum = 55

    let mut cpu = Cpu::new();
    cpu.csr.write(csr::MTVEC, 0x80002000);

    // x1 = sum, x2 = i, x3 = limit
    let program: Vec<u32> = vec![
        // Initialize
        0x00000093, // addi x1, x0, 0      # sum = 0
        0x00100113, // addi x2, x0, 1      # i = 1
        0x00B00193, // addi x3, x0, 11     # limit = 11
        // Loop:
        0x002080B3, // add x1, x1, x2      # sum += i
        0x00110113, // addi x2, x2, 1      # i++
        0xFE314CE3, // blt x2, x3, -8      # if i < 11, loop
        // Done
        0x00000073, // ecall
    ];

    cpu.load_program(&program);

    // Run until trap
    let mut count = 0;
    while cpu.pc != 0x80002000 && count < 100 {
        cpu.step();
        count += 1;
    }

    assert_eq!(cpu.read_reg(1), 55); // sum = 1+2+...+10 = 55
    assert_eq!(cpu.read_reg(2), 11); // i = 11 (loop ended)
    assert_eq!(cpu.pc, 0x80002000); // hit ecall → trap
}

// === CLINT mtime tick 테스트 ===

#[test]
fn test_mtime_increments_on_step() {
    let mut cpu = Cpu::new();
    // NOP: addi x0, x0, 0
    cpu.bus.write32(0x80000000, 0x00000013);

    let mtime_before = cpu.bus.read64(0x200BFF8);
    cpu.step();
    let mtime_after = cpu.bus.read64(0x200BFF8);

    assert_eq!(mtime_after, mtime_before + 1);
}

#[test]
fn test_mtime_increments_multiple_steps() {
    let mut cpu = Cpu::new();
    // NOP: addi x0, x0, 0
    for i in 0..10 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    for _ in 0..10 {
        cpu.step();
    }

    assert_eq!(cpu.bus.read64(0x200BFF8), 10);
}

// === 타이머 인터럽트 테스트 ===

#[test]
fn test_timer_interrupt_triggers() {
    let mut cpu = Cpu::new();

    // 인터럽트 활성화
    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, csr::MIE_MTIE);
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // mtimecmp = 5 설정
    cpu.bus.write64(0x2004000, 5);

    // NOP 명령어 여러 개
    for i in 0..10 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    // tick() → check_pending_interrupts() 순서이므로
    // step 5에서 mtime = 5가 되고 즉시 인터럽트 발생
    for _ in 0..4 {
        cpu.step();
    }
    // 4번 step 후: mtime = 4, PC 진행 중

    // 5번째 step에서 인터럽트 발생 (mtime = 5 >= mtimecmp)
    cpu.step();

    assert_eq!(cpu.pc, 0x80001000); // mtvec으로 점프
    assert_eq!(cpu.csr.read(csr::MCAUSE), csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_TIMER);
}

#[test]
fn test_timer_interrupt_disabled_mie() {
    let mut cpu = Cpu::new();

    // 전역 인터럽트 비활성화 (MSTATUS_MIE = 0)
    cpu.csr.write(csr::MSTATUS, 0);
    cpu.csr.write(csr::MIE, csr::MIE_MTIE);
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // mtimecmp = 3
    cpu.bus.write64(0x2004000, 3);

    // NOP
    for i in 0..10 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    // 10번 step해도 인터럽트 발생 안 함
    for _ in 0..10 {
        cpu.step();
    }

    // PC는 계속 진행
    assert_eq!(cpu.pc, 0x80000000 + 10 * 4);
}

#[test]
fn test_timer_interrupt_disabled_mtie() {
    let mut cpu = Cpu::new();

    // 전역 인터럽트 활성화, 타이머 인터럽트 비활성화
    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, 0); // MTIE = 0
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // mtimecmp = 3
    cpu.bus.write64(0x2004000, 3);

    // NOP
    for i in 0..10 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    // 10번 step해도 인터럽트 발생 안 함
    for _ in 0..10 {
        cpu.step();
    }

    assert_eq!(cpu.pc, 0x80000000 + 10 * 4);
}

#[test]
fn test_timer_interrupt_saves_state() {
    let mut cpu = Cpu::new();

    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, csr::MIE_MTIE);
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // mtimecmp = 1 (즉시 인터럽트)
    cpu.bus.write64(0x2004000, 1);

    cpu.bus.write32(0x80000000, 0x00000013); // NOP
    // tick() → check_pending_interrupts() 순서이므로
    // step 1: tick() → mtime = 1 → 즉시 인터럽트 발생
    cpu.step();

    // mepc에 원래 PC 저장됨 (인터럽트 시점의 PC)
    assert_eq!(cpu.csr.read(csr::MEPC), 0x80000000);

    // mstatus 업데이트 (MIE -> MPIE, MIE = 0)
    let mstatus = cpu.csr.read(csr::MSTATUS);
    assert_eq!(mstatus & csr::MSTATUS_MPIE, csr::MSTATUS_MPIE);
    assert_eq!(mstatus & csr::MSTATUS_MIE, 0);
}

#[test]
fn test_timer_interrupt_clear_by_mtimecmp_update() {
    // mtimecmp를 더 큰 값으로 업데이트하면 인터럽트가 클리어됨
    let mut cpu = Cpu::new();

    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, csr::MIE_MTIE);
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // mtimecmp = 3
    cpu.bus.write64(0x2004000, 3);

    // NOP 명령어들
    for i in 0..20 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    // 인터럽트 발생 전까지 실행
    for _ in 0..3 {
        cpu.step();
    }
    assert!(cpu.bus.check_timer_interrupt()); // mtime >= mtimecmp

    // mtimecmp를 더 큰 값으로 업데이트 (인터럽트 클리어)
    cpu.bus.write64(0x2004000, 100);

    // 인터럽트 조건 해제됨
    assert!(!cpu.bus.check_timer_interrupt());
}

#[test]
fn test_timer_interrupt_full_cycle() {
    // 전체 사이클: 인터럽트 발생 → 핸들러 → mret → 정상 실행 재개
    let mut cpu = Cpu::new();

    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, csr::MIE_MTIE);
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // mtimecmp = 2 (빠른 인터럽트)
    cpu.bus.write64(0x2004000, 2);

    // 메인 코드: NOP들
    for i in 0..20 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    // 핸들러 코드 (0x80001000):
    // 간단히 mret만 실행 (실제로는 mtimecmp 업데이트 필요하지만
    // 테스트에서는 수동으로 처리)
    cpu.bus.write32(0x80001000, 0x30200073); // mret

    // tick() → check_pending_interrupts() 순서
    // Step 1: mtime = 1 < mtimecmp (2), NOP 실행
    cpu.step();
    assert_eq!(cpu.pc, 0x80000004);

    // Step 2: mtime = 2 >= mtimecmp, 인터럽트 발생!
    cpu.step();
    assert_eq!(cpu.pc, 0x80001000); // 핸들러로 점프
    assert_eq!(cpu.csr.read(csr::MEPC), 0x80000004); // 원래 PC 저장

    // mtimecmp 업데이트 (핸들러가 하는 일을 수동으로)
    cpu.bus.write64(0x2004000, 100);

    // Step 3: mret 실행
    cpu.step();
    assert_eq!(cpu.pc, 0x80000004); // 원래 위치로 복귀

    // MIE 복원 확인
    let mstatus = cpu.csr.read(csr::MSTATUS);
    assert_eq!(mstatus & csr::MSTATUS_MIE, csr::MSTATUS_MIE);

    // Step 4+: 정상 실행 재개
    cpu.step();
    assert_eq!(cpu.pc, 0x80000008); // 다음 명령어로 진행
}

#[test]
fn test_timer_interrupt_periodic() {
    // 주기적 인터럽트: 첫 번째 인터럽트 → 클리어 → 두 번째 인터럽트
    let mut cpu = Cpu::new();

    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, csr::MIE_MTIE);
    cpu.csr.write(csr::MTVEC, 0x80001000);

    let interval: u64 = 5;
    cpu.bus.write64(0x2004000, interval); // 첫 인터럽트: mtime = 5

    // 메인 코드: NOP들
    for i in 0..50 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    // 핸들러: mret
    cpu.bus.write32(0x80001000, 0x30200073);

    let mut interrupt_count = 0;

    for _ in 0..20 {
        cpu.step();

        // 핸들러로 점프했으면 인터럽트 발생
        if cpu.pc == 0x80001000 {
            interrupt_count += 1;

            // mtimecmp를 다음 interval로 업데이트
            let current_mtime = cpu.bus.read64(0x200BFF8);
            cpu.bus.write64(0x2004000, current_mtime + interval);

            // mret 실행
            cpu.step();
        }
    }

    // 여러 번의 인터럽트가 발생해야 함
    assert!(interrupt_count >= 2, "Expected at least 2 interrupts, got {}", interrupt_count);
}

// === MIP 레지스터 반영 테스트 ===

#[test]
fn test_mip_mtip_reflects_timer_condition() {
    let mut cpu = Cpu::new();

    // mtimecmp = 3
    cpu.bus.write64(0x2004000, 3);

    // 인터럽트 비활성화 상태에서 MIP만 확인
    cpu.csr.write(csr::MSTATUS, 0); // MIE = 0

    // NOP들
    for i in 0..10 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    // tick() → check_pending_interrupts() 순서이므로
    // step N 완료 후: mtime = N, MIP도 mtime = N 기준으로 평가됨
    cpu.step(); // mtime = 1
    cpu.step(); // mtime = 2
    let mip = cpu.csr.read(csr::MIP);
    assert_eq!(mip & csr::MIP_MTIP, 0, "MTIP should be 0 when mtime < mtimecmp");

    // step 3: mtime = 3 (>= mtimecmp), MTIP = 1
    cpu.step();
    let mip = cpu.csr.read(csr::MIP);
    assert_ne!(mip & csr::MIP_MTIP, 0, "MTIP should be 1 when mtime >= mtimecmp");

    // mtimecmp 업데이트 후: MTIP = 0
    cpu.bus.write64(0x2004000, 100);
    cpu.step();
    let mip = cpu.csr.read(csr::MIP);
    assert_eq!(mip & csr::MIP_MTIP, 0, "MTIP should be 0 after mtimecmp update");
}

#[test]
fn test_mip_msip_reflects_clint_msip() {
    let mut cpu = Cpu::new();

    // 인터럽트 비활성화 상태에서 MIP만 확인
    cpu.csr.write(csr::MSTATUS, 0); // MIE = 0

    // NOP들
    for i in 0..10 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    // CLINT msip = 0: MIP.MSIP = 0
    cpu.step();
    let mip = cpu.csr.read(csr::MIP);
    assert_eq!(mip & csr::MIP_MSIP, 0, "MSIP should be 0 when CLINT msip = 0");

    // CLINT msip = 1: MIP.MSIP = 1
    cpu.bus.write32(0x2000000, 1);
    cpu.step();
    let mip = cpu.csr.read(csr::MIP);
    assert_ne!(mip & csr::MIP_MSIP, 0, "MSIP should be 1 when CLINT msip = 1");

    // CLINT msip = 0: MIP.MSIP = 0
    cpu.bus.write32(0x2000000, 0);
    cpu.step();
    let mip = cpu.csr.read(csr::MIP);
    assert_eq!(mip & csr::MIP_MSIP, 0, "MSIP should be 0 after CLINT msip cleared");
}

// === 소프트웨어 인터럽트 테스트 ===

#[test]
fn test_software_interrupt_triggers() {
    let mut cpu = Cpu::new();

    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, csr::MIE_MSIE);
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // CLINT msip 설정 (소프트웨어 인터럽트 트리거)
    cpu.bus.write32(0x2000000, 1);

    // NOP
    cpu.bus.write32(0x80000000, 0x00000013);

    cpu.step();

    assert_eq!(cpu.pc, 0x80001000); // 핸들러로 점프
    assert_eq!(
        cpu.csr.read(csr::MCAUSE),
        csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_SOFTWARE
    );
}

#[test]
fn test_software_interrupt_disabled_msie() {
    let mut cpu = Cpu::new();

    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, 0); // MSIE = 0
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // CLINT msip 설정
    cpu.bus.write32(0x2000000, 1);

    for i in 0..5 {
        cpu.bus.write32(0x80000000 + i * 4, 0x00000013);
    }

    for _ in 0..5 {
        cpu.step();
    }

    // 인터럽트 발생 안 함
    assert_eq!(cpu.pc, 0x80000000 + 5 * 4);
}

#[test]
fn test_software_interrupt_priority_over_timer() {
    // 소프트웨어 인터럽트가 타이머보다 우선순위 높음
    let mut cpu = Cpu::new();

    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, csr::MIE_MSIE | csr::MIE_MTIE); // 둘 다 활성화
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // 타이머 인터럽트 조건 만족
    cpu.bus.write64(0x2004000, 1); // mtimecmp = 1
    cpu.bus.write64(0x200BFF8, 10); // mtime = 10 (>= mtimecmp)

    // 소프트웨어 인터럽트도 펜딩 (CLINT msip)
    cpu.bus.write32(0x2000000, 1);

    cpu.bus.write32(0x80000000, 0x00000013); // NOP

    cpu.step();

    // 소프트웨어 인터럽트가 먼저 처리됨
    assert_eq!(
        cpu.csr.read(csr::MCAUSE),
        csr::INTERRUPT_BIT | csr::INTERRUPT_FROM_SOFTWARE,
        "Software interrupt should have higher priority than timer"
    );
}

#[test]
fn test_software_interrupt_clear() {
    let mut cpu = Cpu::new();

    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE);
    cpu.csr.write(csr::MIE, csr::MIE_MSIE);
    cpu.csr.write(csr::MTVEC, 0x80001000);

    // 소프트웨어 인터럽트 발생 (CLINT msip)
    cpu.bus.write32(0x2000000, 1);
    cpu.bus.write32(0x80000000, 0x00000013);
    cpu.bus.write32(0x80001000, 0x30200073); // mret

    cpu.step(); // 인터럽트 발생
    assert_eq!(cpu.pc, 0x80001000);

    // CLINT msip 클리어
    cpu.bus.write32(0x2000000, 0);

    cpu.step(); // mret
    assert_eq!(cpu.pc, 0x80000000); // 복귀

    // 더 이상 인터럽트 발생 안 함
    cpu.csr.write(csr::MSTATUS, csr::MSTATUS_MIE); // mret이 MIE 복원함
    cpu.step();
    assert_eq!(cpu.pc, 0x80000004); // 정상 진행
}
