pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

pub const EI_CLASS: usize = 4;
pub const EI_DATA: usize = 5;

pub const ELF_CLASS64: u8 = 2;
pub const ELF_DATA2LSB: u8 = 1;

pub const EM_RISCV: u16 = 0xF3;

pub const PT_LOAD: u32 = 1;

#[derive(Debug)]
pub enum ElfError {
    InvalidMagic,
    InvalidClass,
    InvalidMachine,
    InvalidEndian,
}

pub struct ElfHeader {
    e_ident: [u8; 16],
    e_machine: u16,
    e_entry: u64,
    e_phoff: u64,
    e_phentsize: u16,
    e_phnum: u16,
}

impl ElfHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self, ElfError> {
        let header = ElfHeader {
            e_ident: bytes[0..16].try_into().unwrap(),
            e_machine: u16::from_le_bytes(bytes[0x12..0x14].try_into().unwrap()),
            e_entry: u64::from_le_bytes(bytes[0x18..0x20].try_into().unwrap()),
            e_phoff: u64::from_le_bytes(bytes[0x20..0x28].try_into().unwrap()),
            e_phentsize: u16::from_le_bytes(bytes[0x36..0x38].try_into().unwrap()),
            e_phnum: u16::from_le_bytes(bytes[0x38..0x3A].try_into().unwrap()),
        };

        match header.validate() {
            Err(err) => Err(err),
            _ => Ok(header),
        }
    }

    pub fn validate(&self) -> Result<(), ElfError> {
        if self.e_ident[0..4] != ELF_MAGIC {
            return Err(ElfError::InvalidMagic);
        }

        if self.e_ident[4] != 2 {
            return Err(ElfError::InvalidClass);
        }

        if self.e_ident[5] != 1 {
            return Err(ElfError::InvalidEndian);
        }

        if self.e_machine != 0xF3 {
            return Err(ElfError::InvalidMachine);
        }

        Ok(())
    }

    pub fn entry(&self) -> u64 {
        self.e_entry
    }

    pub fn phoff(&self) -> u64 {
        self.e_phoff
    }

    pub fn phentsize(&self) -> u16 {
        self.e_phentsize
    }

    pub fn phnum(&self) -> u16 {
        self.e_phnum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_elf_header() -> [u8; 64] {
        let mut header = [0u8; 64];
        // e_ident
        header[0] = 0x7F;
        header[1] = b'E';
        header[2] = b'L';
        header[3] = b'F';
        header[4] = 2; // 64-bit
        header[5] = 1; // little-endian
        header[6] = 1; // version
        // e_type (0x10-0x11): 2 = EXEC
        header[0x10] = 0x02;
        header[0x11] = 0x00;
        // e_machine (0x12-0x13): 0xF3 = RISC-V
        header[0x12] = 0xF3;
        header[0x13] = 0x00;
        // e_entry (0x18-0x1F): 0x80000000
        header[0x18] = 0x00;
        header[0x19] = 0x00;
        header[0x1A] = 0x00;
        header[0x1B] = 0x80;
        // e_phoff (0x20-0x27): 64 (right after header)
        header[0x20] = 0x40;
        // e_phentsize (0x36-0x37): 56
        header[0x36] = 0x38;
        // e_phnum (0x38-0x39): 2
        header[0x38] = 0x02;
        header
    }

    #[test]
    fn test_elf_header_parse_valid() {
        let bytes = create_valid_elf_header();
        let header = ElfHeader::parse(&bytes).unwrap();
        assert_eq!(header.entry(), 0x80000000);
        assert_eq!(header.phoff(), 64);
        assert_eq!(header.phentsize(), 56);
        assert_eq!(header.phnum(), 2);
    }

    #[test]
    fn test_elf_header_invalid_magic() {
        let mut bytes = create_valid_elf_header();
        bytes[0] = 0x00; // wrong magic
        assert!(matches!(
            ElfHeader::parse(&bytes),
            Err(ElfError::InvalidMagic)
        ));
    }

    #[test]
    fn test_elf_header_invalid_class() {
        let mut bytes = create_valid_elf_header();
        bytes[4] = 1; // 32-bit instead of 64-bit
        assert!(matches!(
            ElfHeader::parse(&bytes),
            Err(ElfError::InvalidClass)
        ));
    }

    #[test]
    fn test_elf_header_invalid_endian() {
        let mut bytes = create_valid_elf_header();
        bytes[5] = 2; // big-endian instead of little-endian
        assert!(matches!(
            ElfHeader::parse(&bytes),
            Err(ElfError::InvalidEndian)
        ));
    }

    #[test]
    fn test_elf_header_invalid_machine() {
        let mut bytes = create_valid_elf_header();
        bytes[0x12] = 0x00; // not RISC-V
        assert!(matches!(
            ElfHeader::parse(&bytes),
            Err(ElfError::InvalidMachine)
        ));
    }
}
