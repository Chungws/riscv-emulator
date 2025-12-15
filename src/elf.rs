pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

pub const EI_CLASS: usize = 4;
pub const EI_DATA: usize = 5;

pub const ELF_CLASS64: u8 = 2;
pub const ELF_DATA2LSB: u8 = 1;

pub const EM_RISCV: u16 = 0xF3;

pub const PT_LOAD: u32 = 1;
