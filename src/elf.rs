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
    ParseError,
}

pub struct ElfFile {
    pub entry: u64,
    pub segments: Vec<Segment>,
}

impl ElfFile {
    pub fn load(bytes: &[u8]) -> Result<Self, ElfError> {
        let header = ElfHeader::parse(bytes)?;

        let phentsize = header.phentsize();
        let phoff = header.phoff();
        let phnum = header.phnum();

        let mut segments: Vec<Segment> = Vec::new();
        for i in 0..(phnum as usize) {
            let offset = phoff as usize + i * (phentsize as usize);
            let ph = ProgramHeader::parse(&bytes[offset..])?;

            if ph.p_type() == PT_LOAD {
                let data =
                    bytes[(ph.offset() as usize)..((ph.offset() + ph.filesz()) as usize)].to_vec();
                segments.push(Segment {
                    vaddr: ph.vaddr(),
                    data: data,
                    memsz: ph.memsz(),
                });
            }
        }

        Ok(ElfFile {
            entry: header.entry(),
            segments: segments,
        })
    }
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
            e_ident: bytes[0..16].try_into().map_err(|_| ElfError::ParseError)?,
            e_machine: u16::from_le_bytes(
                bytes[0x12..0x14]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            e_entry: u64::from_le_bytes(
                bytes[0x18..0x20]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            e_phoff: u64::from_le_bytes(
                bytes[0x20..0x28]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            e_phentsize: u16::from_le_bytes(
                bytes[0x36..0x38]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            e_phnum: u16::from_le_bytes(
                bytes[0x38..0x3A]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
        };

        header.validate()?;
        Ok(header)
    }

    pub fn validate(&self) -> Result<(), ElfError> {
        if self.e_ident[0..4] != ELF_MAGIC {
            return Err(ElfError::InvalidMagic);
        }

        if self.e_ident[4] != ELF_CLASS64 {
            return Err(ElfError::InvalidClass);
        }

        if self.e_ident[5] != ELF_DATA2LSB {
            return Err(ElfError::InvalidEndian);
        }

        if self.e_machine != EM_RISCV {
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

#[allow(dead_code)]
pub struct ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

impl ProgramHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self, ElfError> {
        Ok(ProgramHeader {
            p_type: u32::from_le_bytes(
                bytes[0x00..0x04]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            p_flags: u32::from_le_bytes(
                bytes[0x04..0x08]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            p_offset: u64::from_le_bytes(
                bytes[0x08..0x10]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            p_vaddr: u64::from_le_bytes(
                bytes[0x10..0x18]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            p_paddr: u64::from_le_bytes(
                bytes[0x18..0x20]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            p_filesz: u64::from_le_bytes(
                bytes[0x20..0x28]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            p_memsz: u64::from_le_bytes(
                bytes[0x28..0x30]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
            p_align: u64::from_le_bytes(
                bytes[0x30..0x38]
                    .try_into()
                    .map_err(|_| ElfError::ParseError)?,
            ),
        })
    }

    pub fn p_type(&self) -> u32 {
        self.p_type
    }

    pub fn vaddr(&self) -> u64 {
        self.p_vaddr
    }

    pub fn offset(&self) -> u64 {
        self.p_offset
    }

    pub fn filesz(&self) -> u64 {
        self.p_filesz
    }

    pub fn memsz(&self) -> u64 {
        self.p_memsz
    }
}

pub struct Segment {
    pub vaddr: u64,
    pub data: Vec<u8>,
    pub memsz: u64,
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

    fn create_program_header(
        p_type: u32,
        vaddr: u64,
        offset: u64,
        filesz: u64,
        memsz: u64,
    ) -> [u8; 56] {
        let mut ph = [0u8; 56];
        // p_type (0x00-0x03)
        ph[0x00..0x04].copy_from_slice(&p_type.to_le_bytes());
        // p_flags (0x04-0x07): R|W|X = 7
        ph[0x04..0x08].copy_from_slice(&7u32.to_le_bytes());
        // p_offset (0x08-0x0F)
        ph[0x08..0x10].copy_from_slice(&offset.to_le_bytes());
        // p_vaddr (0x10-0x17)
        ph[0x10..0x18].copy_from_slice(&vaddr.to_le_bytes());
        // p_paddr (0x18-0x1F)
        ph[0x18..0x20].copy_from_slice(&vaddr.to_le_bytes());
        // p_filesz (0x20-0x27)
        ph[0x20..0x28].copy_from_slice(&filesz.to_le_bytes());
        // p_memsz (0x28-0x2F)
        ph[0x28..0x30].copy_from_slice(&memsz.to_le_bytes());
        // p_align (0x30-0x37)
        ph[0x30..0x38].copy_from_slice(&0x1000u64.to_le_bytes());
        ph
    }

    #[test]
    fn test_program_header_parse() {
        let ph_bytes = create_program_header(PT_LOAD, 0x80000000, 0x1000, 0x2000, 0x3000);
        let ph = ProgramHeader::parse(&ph_bytes).unwrap();
        assert_eq!(ph.p_type(), PT_LOAD);
        assert_eq!(ph.vaddr(), 0x80000000);
        assert_eq!(ph.offset(), 0x1000);
        assert_eq!(ph.filesz(), 0x2000);
        assert_eq!(ph.memsz(), 0x3000);
    }

    #[test]
    fn test_program_header_pt_null() {
        let ph_bytes = create_program_header(0, 0, 0, 0, 0); // PT_NULL
        let ph = ProgramHeader::parse(&ph_bytes).unwrap();
        assert_eq!(ph.p_type(), 0);
    }

    #[test]
    fn test_program_header_bss_section() {
        // BSS: memsz > filesz (나머지는 0으로 채워야 함)
        let ph_bytes = create_program_header(PT_LOAD, 0x80010000, 0x2000, 0x1000, 0x5000);
        let ph = ProgramHeader::parse(&ph_bytes).unwrap();
        assert_eq!(ph.filesz(), 0x1000);
        assert_eq!(ph.memsz(), 0x5000);
        assert!(ph.memsz() > ph.filesz()); // BSS 영역 존재
    }

    // ElfLoader 테스트용 helper: 전체 ELF 파일 생성
    fn create_elf_file() -> Vec<u8> {
        let mut elf = Vec::new();

        // ELF Header (64 bytes)
        let mut header = create_valid_elf_header();
        header[0x38] = 0x02; // e_phnum = 2
        elf.extend_from_slice(&header);

        // Program Header 1: PT_LOAD (offset=176, filesz=8, memsz=16)
        let ph1 = create_program_header(PT_LOAD, 0x80000000, 176, 8, 16);
        elf.extend_from_slice(&ph1);

        // Program Header 2: PT_NULL (무시됨)
        let ph2 = create_program_header(0, 0, 0, 0, 0);
        elf.extend_from_slice(&ph2);

        // Segment data (8 bytes)
        elf.extend_from_slice(&[0x13, 0x00, 0x00, 0x00, 0x67, 0x80, 0x00, 0x00]); // nop; ret

        elf
    }

    #[test]
    fn test_elf_file_load() {
        let elf_bytes = create_elf_file();
        let elf = ElfFile::load(&elf_bytes).unwrap();

        assert_eq!(elf.entry, 0x80000000);
        assert_eq!(elf.segments.len(), 1); // PT_NULL은 무시됨
    }

    #[test]
    fn test_elf_file_segment_data() {
        let elf_bytes = create_elf_file();
        let elf = ElfFile::load(&elf_bytes).unwrap();

        let seg = &elf.segments[0];
        assert_eq!(seg.vaddr, 0x80000000);
        assert_eq!(seg.data.len(), 8);
        assert_eq!(seg.memsz, 16); // BSS 포함
        assert_eq!(
            seg.data,
            vec![0x13, 0x00, 0x00, 0x00, 0x67, 0x80, 0x00, 0x00]
        );
    }

    #[test]
    fn test_elf_file_invalid_magic() {
        let mut elf_bytes = create_elf_file();
        elf_bytes[0] = 0x00; // wrong magic
        assert!(matches!(
            ElfFile::load(&elf_bytes),
            Err(ElfError::InvalidMagic)
        ));
    }
}
