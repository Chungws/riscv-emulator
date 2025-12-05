pub fn opcode(inst: u32) -> u32 {
    inst & 0x7F
}

pub fn rd(inst: u32) -> u32 {
    (inst >> 7) & 0x1F
}

pub fn funct3(inst: u32) -> u32 {
    (inst >> 12) & 0x7
}

pub fn rs1(inst: u32) -> u32 {
    (inst >> 15) & 0x1F
}

pub fn rs2(inst: u32) -> u32 {
    (inst >> 20) & 0x1F
}

pub fn funct7(inst: u32) -> u32 {
    (inst >> 25) & 0x7F
}

pub fn imm_i(inst: u32) -> i32 {
    (inst as i32) >> 20
}

pub fn imm_s(inst: u32) -> i32 {
    let imm_4_0 = (inst >> 7) & 0x1F;
    let imm_11_5 = (inst >> 25) & 0x7F;
    let imm = (imm_11_5 << 5) | imm_4_0;
    ((imm as i32) << 20) >> 20
}

pub fn imm_b(inst: u32) -> i32 {
    let imm_12 = (inst >> 31) & 0x1;
    let imm_11 = (inst >> 7) & 0x1;
    let imm_10_5 = (inst >> 25) & 0x3F;
    let imm_4_1 = (inst >> 8) & 0xF;
    let imm = (imm_12 << 12) | (imm_11 << 11) | (imm_10_5 << 5) | (imm_4_1 << 1);
    ((imm as i32) << 19) >> 19
}

pub fn imm_u(inst: u32) -> i32 {
    (inst & 0xFFFFF000) as i32
}

pub fn imm_j(inst: u32) -> i32 {
    let imm_20 = (inst >> 31) & 0x1;
    let imm_19_12 = (inst >> 12) & 0xFF;
    let imm_11 = (inst >> 20) & 0x1;
    let imm_10_1 = (inst >> 21) & 0x3FF;
    let imm = (imm_20 << 20) | (imm_19_12 << 12) | (imm_11 << 11) | (imm_10_1 << 1);
    ((imm as i32) << 11) >> 11
}

#[cfg(test)]
mod tests {
    use super::*;

    // === 기본 필드 추출 ===
    #[test]
    fn test_decode_r_type() {
        // ADD x3, x1, x2 → 0x002081B3
        let inst = 0x002081B3;
        assert_eq!(opcode(inst), 0b0110011);
        assert_eq!(rd(inst), 3);
        assert_eq!(funct3(inst), 0);
        assert_eq!(rs1(inst), 1);
        assert_eq!(rs2(inst), 2);
        assert_eq!(funct7(inst), 0);
    }

    // === I-type immediate ===
    #[test]
    fn test_imm_i_positive() {
        // ADDI x1, x0, 42 → 0x02A00093
        let inst = 0x02A00093;
        assert_eq!(imm_i(inst), 42);
    }

    #[test]
    fn test_imm_i_negative() {
        // ADDI x1, x0, -1 → 0xFFF00093
        let inst = 0xFFF00093;
        assert_eq!(imm_i(inst), -1);
    }

    // === S-type immediate ===
    #[test]
    fn test_imm_s() {
        // SW x2, 8(x1) → 0x00212423
        let inst = 0x00212423;
        assert_eq!(imm_s(inst), 8);
    }

    #[test]
    fn test_imm_s_negative() {
        // SW x2, -4(x1) → 0xFE212E23
        let inst = 0xFE212E23;
        assert_eq!(imm_s(inst), -4);
    }

    // === B-type immediate ===
    #[test]
    fn test_imm_b() {
        // BEQ x1, x2, 16 → 0x00208863
        let inst = 0x00208863;
        assert_eq!(imm_b(inst), 16);
    }

    #[test]
    fn test_imm_b_negative() {
        // BEQ x1, x2, -8 → 0xFE208CE3
        let inst = 0xFE208CE3;
        assert_eq!(imm_b(inst), -8);
    }

    // === U-type immediate ===
    #[test]
    fn test_imm_u() {
        // LUI x1, 0x12345 → 0x123450B7
        let inst = 0x123450B7;
        assert_eq!(imm_u(inst), 0x12345000_u32 as i32);
    }

    // === J-type immediate ===
    #[test]
    fn test_imm_j() {
        // JAL x1, 16 → 0x010000EF
        let inst = 0x010000EF;
        assert_eq!(imm_j(inst), 16);
    }

    #[test]
    fn test_imm_j_negative() {
        // JAL x1, -4 → 0xFFDFF0EF
        let inst = 0xFFDFF0EF;
        assert_eq!(imm_j(inst), -4);
    }
}
