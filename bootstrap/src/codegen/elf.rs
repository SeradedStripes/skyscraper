/// Minimal ELF64 binary writer for Linux x86-64.
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub const BASE_ADDR: u64 = 0x400000;
pub const HDR_SIZE: u64 = 120; // 64 (ELF header) + 56 (1 program header)

/// Write a minimal statically-linked ELF64 executable.
///
/// Memory layout: [ELF hdr][PHDR] [code] [data]
/// Everything is in one PT_LOAD segment mapped at BASE_ADDR.
pub fn write_elf(
    code: &[u8],
    data: &[u8],
    bss_size: usize,
    entry_offset: usize,
    path: &Path,
) -> io::Result<()> {
    let file_size = HDR_SIZE as usize + code.len() + data.len();
    let mem_size = file_size + bss_size;
    let entry_addr = BASE_ADDR + HDR_SIZE + entry_offset as u64;

    let mut f = fs::File::create(path)?;

    // -- ELF header (64 bytes) --
    let mut hdr = [0u8; 64];
    // e_ident
    hdr[0..4].copy_from_slice(b"\x7fELF");
    hdr[4] = 2; // EI_CLASS: ELFCLASS64
    hdr[5] = 1; // EI_DATA: ELFDATA2LSB (little-endian)
    hdr[6] = 1; // EI_VERSION: EV_CURRENT
    hdr[7] = 0; // EI_OSABI: ELFOSABI_NONE (Linux)

    write_u16(&mut hdr[16..18], 2); // e_type: ET_EXEC
    write_u16(&mut hdr[18..20], 0x3E); // e_machine: EM_X86_64
    write_u32(&mut hdr[20..24], 1); // e_version
    write_u64(&mut hdr[24..32], entry_addr); // e_entry
    write_u64(&mut hdr[32..40], 64); // e_phoff
    write_u64(&mut hdr[40..48], 0); // e_shoff (none)
    write_u32(&mut hdr[48..52], 0); // e_flags
    write_u16(&mut hdr[52..54], 64); // e_ehsize
    write_u16(&mut hdr[54..56], 56); // e_phentsize
    write_u16(&mut hdr[56..58], 1); // e_phnum
    write_u16(&mut hdr[58..60], 0); // e_shentsize
    write_u16(&mut hdr[60..62], 0); // e_shnum
    write_u16(&mut hdr[62..64], 0); // e_shstrndx

    f.write_all(&hdr)?;

    // -- Program header (56 bytes) --
    let mut phdr = [0u8; 56];
    write_u32(&mut phdr[0..4], 1); // p_type: PT_LOAD
    write_u32(&mut phdr[4..8], 5); // p_flags: PF_R | PF_X
    write_u64(&mut phdr[8..16], 0); // p_offset (whole file)
    write_u64(&mut phdr[16..24], BASE_ADDR); // p_vaddr
    write_u64(&mut phdr[24..32], BASE_ADDR); // p_paddr
    write_u64(&mut phdr[32..40], file_size as u64); // p_filesz
    write_u64(&mut phdr[40..48], mem_size as u64); // p_memsz
    write_u64(&mut phdr[48..56], 0x200000); // p_align

    f.write_all(&phdr)?;

    // -- Code --
    f.write_all(code)?;

    // -- Data --
    f.write_all(data)?;

    Ok(())
}

fn write_u16(buf: &mut [u8], val: u16) {
    buf.copy_from_slice(&val.to_le_bytes());
}

fn write_u32(buf: &mut [u8], val: u32) {
    buf.copy_from_slice(&val.to_le_bytes());
}

fn write_u64(buf: &mut [u8], val: u64) {
    buf.copy_from_slice(&val.to_le_bytes());
}
