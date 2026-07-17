//! x86-64 instruction encoding for the Skyscraper bootstrap compiler.

/// Map Skyscraper register name to x86-64 register index.
///
/// Skyscraper calling convention → System V AMD64 ABI:
///   ret0 → rax, ret1 → rdx
///   arg0-arg7 → rdi, rsi, rdx, rcx, r8, r9, r10, r11
///   temp0-temp5 → r12, r13, r14, r15, r8, r9
///   r0-r15 → rax, rcx, rdx, rbx, rsp, rbp, rsi, rdi, r8-r15
///   sp → rsp, fp → rbp
pub fn register_index(name: &str) -> Option<u8> {
    match name {
        "r0" => Some(0),
        "r1" => Some(1),
        "r2" => Some(2),
        "r3" => Some(3),
        "r4" => Some(4),
        "r5" => Some(5),
        "r6" => Some(6),
        "r7" => Some(7),
        "r8" => Some(8),
        "r9" => Some(9),
        "r10" => Some(10),
        "r11" => Some(11),
        "r12" => Some(12),
        "r13" => Some(13),
        "r14" => Some(14),
        "r15" => Some(15),
        "arg0" => Some(7),
        "arg1" => Some(6),
        "arg2" => Some(2),
        "arg3" => Some(1),
        "arg4" => Some(8),
        "arg5" => Some(9),
        "arg6" => Some(10),
        "arg7" => Some(11),
        "ret0" => Some(0),
        "ret1" => Some(2),
        "temp0" => Some(12),
        "temp1" => Some(13),
        "temp2" => Some(14),
        "temp3" => Some(15),
        "temp4" => Some(8),
        "temp5" => Some(9),
        "sp" => Some(4),
        "fp" => Some(5),
        _ => None,
    }
}

/// REX.W prefix for 64-bit operand size, with B bit for extended registers.
pub fn rex_w(reg: u8) -> u8 {
    0x48 | if reg >= 8 { 0x01 } else { 0 }
}

/// REX prefix for 64-bit MOV r/m64, r64 with extended src and dst.
fn rex_w_rr(dst: u8, src: u8) -> u8 {
    0x48 | if src >= 8 { 0x04 } else { 0 } | if dst >= 8 { 0x01 } else { 0 }
}

/// MOV r64, imm64 (10 bytes).
/// `mov rax, 0x1234567890ABCDEF`
pub fn emit_mov_reg_imm64(code: &mut Vec<u8>, reg: u8, imm: i64) {
    code.push(rex_w(reg));
    code.push(0xB8 | (reg & 0x7));
    code.extend_from_slice(&imm.to_le_bytes());
}

/// MOV r64, imm32 (7 bytes, sign-extended).
/// Use when imm fits in a signed i32.
pub fn emit_mov_reg_imm32(code: &mut Vec<u8>, reg: u8, imm: i32) {
    code.push(rex_w(reg));
    code.push(0xC7);
    code.push(0xC0 | (reg & 0x7));
    code.extend_from_slice(&imm.to_le_bytes());
}

/// MOV r64, r64 (3 bytes).
pub fn emit_mov_reg_reg(code: &mut Vec<u8>, dst: u8, src: u8) {
    if dst == src {
        return;
    }
    code.push(rex_w_rr(dst, src));
    code.push(0x89);
    code.push(0xC0 | ((src & 0x7) << 3) | (dst & 0x7));
}

/// OR r64, imm32 (7 bytes, sign-extended).
pub fn emit_or_reg_imm32(code: &mut Vec<u8>, reg: u8, imm: i32) {
    code.push(rex_w(reg));
    code.push(0x81);
    code.push(0xC8 | (reg & 0x7));
    code.extend_from_slice(&imm.to_le_bytes());
}

/// XOR r32, r32 (2 bytes, zero-extends to 64-bit).
#[allow(dead_code)]
pub fn emit_xor_reg_reg(code: &mut Vec<u8>, dst: u8, src: u8) {
    let need_rex = dst >= 8 || src >= 8;
    if need_rex {
        code.push(0x40 | if dst >= 8 { 0x04 } else { 0 } | if src >= 8 { 0x01 } else { 0 });
    }
    code.push(0x31);
    code.push(0xC0 | ((src & 0x7) << 3) | (dst & 0x7));
}

/// SYSCALL instruction (2 bytes).
pub fn emit_syscall(code: &mut Vec<u8>) {
    code.extend_from_slice(&[0x0F, 0x05]);
}

/// NOP instruction (1 byte).
pub fn emit_nop(code: &mut Vec<u8>) {
    code.push(0x90);
}

/// CMP r64, r64 (3 bytes). Sets flags.
#[allow(dead_code)]
pub fn emit_cmp_reg_reg(code: &mut Vec<u8>, reg1: u8, reg2: u8) {
    code.push(rex_w_rr(reg1, reg2));
    code.push(0x39); // CMP r/m64, r64
    code.push(0xC0 | ((reg2 & 0x7) << 3) | (reg1 & 0x7));
}

/// CMP r64, imm32 (7 bytes). Sets flags.
pub fn emit_cmp_reg_imm32(code: &mut Vec<u8>, reg: u8, imm: i32) {
    code.push(rex_w(reg));
    code.push(0x81);
    code.push(0xF8 | (reg & 0x7)); // CMP r/m64, imm32
    code.extend_from_slice(&imm.to_le_bytes());
}

/// TEST r64, r64 (3 bytes). Sets flags (AND without storing result).
pub fn emit_test_reg_reg(code: &mut Vec<u8>, reg1: u8, reg2: u8) {
    code.push(rex_w_rr(reg1, reg2));
    code.push(0x85); // TEST r/m64, r64
    code.push(0xC0 | ((reg2 & 0x7) << 3) | (reg1 & 0x7));
}

/// SUB r64, r64 (3 bytes). Sets flags.
pub fn emit_sub_reg_reg(code: &mut Vec<u8>, dst: u8, src: u8) {
    code.push(rex_w_rr(dst, src));
    code.push(0x29); // SUB r/m64, r64
    code.push(0xC0 | ((src & 0x7) << 3) | (dst & 0x7));
}

/// ADD r64, imm32 (7 bytes). Sets flags.
pub fn emit_add_reg_imm32(code: &mut Vec<u8>, reg: u8, imm: i32) {
    code.push(rex_w(reg));
    code.push(0x81);
    code.push(0xC0 | (reg & 0x7)); // ADD r/m64, imm32
    code.extend_from_slice(&imm.to_le_bytes());
}

/// JMP rel32 (5 bytes). Target = PC + rel32.
pub fn emit_jmp_rel32(code: &mut Vec<u8>, rel32: i32) {
    code.push(0xE9);
    code.extend_from_slice(&rel32.to_le_bytes());
}

/// Jcc rel32 (6 bytes). Conditional jump.
/// cc: 0=O, 1=NO, 2=B/C, 3=NB/NC, 4=Z/E, 5=NZ/NE, 6=BE, 7=NBE, 8=S, 9=NS, 10=P/PE, 11=NP/PO, 12=L/NGE, 13=NL/GE, 14=LE/NG, 15=NLE/G
pub fn emit_jcc_rel32(code: &mut Vec<u8>, cc: u8, rel32: i32) {
    code.push(0x0F);
    code.push(0x80 | cc);
    code.extend_from_slice(&rel32.to_le_bytes());
}

/// CALL rel32 (5 bytes). Push return addr, jump to PC + rel32.
pub fn emit_call_rel32(code: &mut Vec<u8>, rel32: i32) {
    code.push(0xE8);
    code.extend_from_slice(&rel32.to_le_bytes());
}

/// CALL r/m64 (2 bytes). Push return addr, jump to register.
pub fn emit_call_reg(code: &mut Vec<u8>, reg: u8) {
    code.push(0xFF);
    code.push(0xD0 | (reg & 0x7));
}

/// RET instruction (1 byte).
pub fn emit_ret(code: &mut Vec<u8>) {
    code.push(0xC3);
}

/// JMP r/m64 (2 bytes). Jump to register.
pub fn emit_jmp_reg(code: &mut Vec<u8>, reg: u8) {
    code.push(0xFF);
    code.push(0xE0 | (reg & 0x7));
}

/// MOV r64, [r64 + imm32] (load, 7 bytes with offset).
pub fn emit_load_reg_mem(code: &mut Vec<u8>, dst: u8, base: u8, offset: i32) {
    let need_rex = dst >= 8 || base >= 8;
    if need_rex {
        code.push(0x48 | if dst >= 8 { 0x04 } else { 0 } | if base >= 8 { 0x01 } else { 0 });
    } else {
        code.push(0x48);
    }
    code.push(0x8B);
    // When base is RSP (4) or R12 (12), r/m=100 triggers SIB encoding.
    // We must emit a SIB byte: scale=0, index=none(100), base=RSP.
    let use_sib = (base & 0x7) == 4;
    if use_sib {
        if (-128..=127).contains(&offset) {
            code.push(0x40 | ((dst & 0x7) << 3) | 0x04); // mod=01, reg=dst, r/m=100 (SIB)
            code.push(0x24); // SIB: scale=0, index=none, base=RSP
            code.push(offset as u8);
        } else {
            code.push(0x80 | ((dst & 0x7) << 3) | 0x04); // mod=10, reg=dst, r/m=100 (SIB)
            code.push(0x24);
            code.extend_from_slice(&offset.to_le_bytes());
        }
    } else {
        if (-128..=127).contains(&offset) {
            code.push(0x40 | ((dst & 0x7) << 3) | (base & 0x7));
            code.push(offset as u8);
        } else {
            code.push(0x80 | ((dst & 0x7) << 3) | (base & 0x7));
            code.extend_from_slice(&offset.to_le_bytes());
        }
    }
}

/// MOV [r64 + imm32], r64 (store, 7 bytes with offset).
pub fn emit_store_mem_reg(code: &mut Vec<u8>, src: u8, base: u8, offset: i32) {
    let need_rex = src >= 8 || base >= 8;
    if need_rex {
        code.push(0x48 | if src >= 8 { 0x04 } else { 0 } | if base >= 8 { 0x01 } else { 0 });
    } else {
        code.push(0x48);
    }
    code.push(0x89);
    // When base is RSP (4) or R12 (12), r/m=100 triggers SIB encoding.
    let use_sib = (base & 0x7) == 4;
    if use_sib {
        if (-128..=127).contains(&offset) {
            code.push(0x40 | ((src & 0x7) << 3) | 0x04);
            code.push(0x24);
            code.push(offset as u8);
        } else {
            code.push(0x80 | ((src & 0x7) << 3) | 0x04);
            code.push(0x24);
            code.extend_from_slice(&offset.to_le_bytes());
        }
    } else {
        if (-128..=127).contains(&offset) {
            code.push(0x40 | ((src & 0x7) << 3) | (base & 0x7));
            code.push(offset as u8);
        } else {
            code.push(0x80 | ((src & 0x7) << 3) | (base & 0x7));
            code.extend_from_slice(&offset.to_le_bytes());
        }
    }
}

/// Map a Skyscraper ABI syscall number to a Linux x86-64 syscall number.
pub fn translate_syscall(skyscraper_nr: i64) -> Option<i64> {
    match skyscraper_nr {
        0 => Some(60),   // exit
        1 => Some(0),    // read
        2 => Some(1),    // write
        3 => Some(2),    // open
        4 => Some(3),    // close
        5 => Some(8),    // seek (lseek)
        6 => Some(4),    // stat (x86-64 uses stat, but differently)
        7 => Some(9),    // mmap
        8 => Some(11),   // munmap
        9 => Some(12),   // brk
        10 => Some(228), // clock (clock_gettime)
        11 => Some(24),  // yield (sched_yield)
        12 => Some(39),  // getpid
        13 => Some(57),  // fork
        14 => Some(59),  // exec (execve)
        15 => Some(22),  // pipe
        16 => Some(32),  // dup
        17 => Some(33),  // dup2
        18 => Some(16),  // ioctl
        19 => Some(201), // time (gettimeofday)
        20 => Some(35),  // sleep (nanosleep)
        21 => Some(10),  // mprotect
        22 => Some(217), // getdents
        23 => Some(87),  // unlink
        24 => Some(82),  // rename
        25 => Some(83),  // mkdir
        26 => Some(84),  // rmdir
        27 => Some(80),  // chdir
        28 => Some(79),  // getcwd (getcwd)
        _ => None,
    }
}
