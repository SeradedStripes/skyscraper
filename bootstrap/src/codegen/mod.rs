pub mod elf;
pub mod x86_64;

use crate::parser::{BinOp, Expr, Operand, Program, Statement};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CodegenError {
    UnknownInstruction { name: String, line: usize },
    UnknownRegister { name: String, line: usize },
    MissingLabel(String),
    MissingEntryPoint,
    InvalidOperand { message: String, line: usize },
    IoError(std::io::Error),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UnknownInstruction { name, line } => {
                write!(f, "line {}: unknown instruction: {}", line, name)
            }
            CodegenError::UnknownRegister { name, line } => {
                write!(f, "line {}: unknown register: {}", line, name)
            }
            CodegenError::MissingLabel(name) => {
                write!(f, "missing label: {}", name)
            }
            CodegenError::MissingEntryPoint => {
                write!(f, "no _start label found")
            }
            CodegenError::InvalidOperand { message, line } => {
                write!(f, "line {}: {}", line, message)
            }
            CodegenError::IoError(e) => {
                write!(f, "IO error: {}", e)
            }
        }
    }
}

impl From<std::io::Error> for CodegenError {
    fn from(e: std::io::Error) -> Self {
        CodegenError::IoError(e)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Code,
    Data,
    Bss,
}

struct LabelPos {
    section: Section,
    offset: usize,
}

struct Fixup {
    offset: usize,
    label: String,
    kind: FixupKind,
}

#[derive(Clone, Copy)]
enum FixupKind {
    /// Absolute address (for ldi/lui)
    Absolute,
    /// Relative offset from instruction start (for branches/jumps)
    Relative {
        /// Offset from instruction start to the displacement field
        disp_offset: i64,
        /// Total instruction size in bytes
        instr_size: i32,
    },
}

struct Codegen {
    code: Vec<u8>,
    data: Vec<u8>,
    bss_size: usize,
    constants: HashMap<String, i64>,
    label_positions: HashMap<String, LabelPos>,
    labels: HashMap<String, usize>,
    fixups: Vec<Fixup>,
    entry_point: Option<String>,
    current_section: Section,
    current_line: usize,
}

impl Codegen {
    fn new() -> Self {
        Self {
            code: Vec::new(),
            data: Vec::new(),
            bss_size: 0,
            constants: HashMap::new(),
            label_positions: HashMap::new(),
            labels: HashMap::new(),
            fixups: Vec::new(),
            entry_point: None,
            current_section: Section::Code,
            current_line: 0,
        }
    }

    fn compile(&mut self, program: &Program) -> Result<(), CodegenError> {
        // Pass 1: emit code and data, collect label positions and fixups
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        // Compute final label addresses
        let code_size = self.code.len();
        let data_size = self.data.len();
        for (name, pos) in &self.label_positions {
            let addr = match pos.section {
                Section::Code => (BASE_ADDR as usize) + elf::HDR_SIZE as usize + pos.offset,
                Section::Data => {
                    (BASE_ADDR as usize) + elf::HDR_SIZE as usize + code_size + pos.offset
                }
                Section::Bss => {
                    (BASE_ADDR as usize)
                        + elf::HDR_SIZE as usize
                        + code_size
                        + data_size
                        + pos.offset
                }
            };
            self.labels.insert(name.clone(), addr);
        }

        // Re-evaluate constants with correct label addresses
        for stmt in &program.statements {
            if let Statement::ConstantDef { name, value, .. } = stmt {
                let val = self.eval_expr(value, code_size)?;
                self.constants.insert(name.clone(), val);
            }
        }

        // Resolve fixups (label references in immediate operands)
        self.resolve_fixups()?;
        Ok(())
    }

    fn line_err(&self, message: String) -> CodegenError {
        CodegenError::InvalidOperand {
            message,
            line: self.current_line,
        }
    }

    fn emit_statement(&mut self, stmt: &Statement) -> Result<(), CodegenError> {
        match stmt {
            Statement::LabelDef { name, line } => {
                self.current_line = *line;
                self.label_positions.insert(
                    name.clone(),
                    LabelPos {
                        section: self.current_section,
                        offset: self.section_offset(),
                    },
                );
            }
            Statement::Instruction {
                name,
                operands,
                line,
            } => {
                self.current_line = *line;
                self.emit_instruction(name, operands)?;
            }
            Statement::Directive { name, args, line } => {
                self.current_line = *line;
                self.emit_directive(name, args)?;
            }
            Statement::ConstantDef { name, value, line } => {
                self.current_line = *line;
                let val = self.eval_expr(value, 0)?;
                self.constants.insert(name.clone(), val);
            }
        }
        Ok(())
    }

    fn section_offset(&self) -> usize {
        match self.current_section {
            Section::Code => self.code.len(),
            Section::Data => self.data.len(),
            Section::Bss => self.bss_size,
        }
    }

    fn emit_instruction(&mut self, name: &str, operands: &[Operand]) -> Result<(), CodegenError> {
        match name {
            "ldi" => self.emit_ldi(operands),
            "lui" => self.emit_lui(operands),
            "ori" => self.emit_ori(operands),
            "mv" => self.emit_mv(operands),
            "cmp" => self.emit_cmp(operands),
            "cmpi" => self.emit_cmpi(operands),
            "add" => self.emit_alu_rr(operands, 0x00),
            "addi" => self.emit_alu_ri(operands, 0x00),
            "sub" => self.emit_alu_rr(operands, 0x05),
            "subi" => self.emit_alu_ri(operands, 0x05),
            "and" => self.emit_alu_rr(operands, 0x04),
            "andi" => self.emit_alu_ri(operands, 0x04),
            "or" => self.emit_alu_rr(operands, 0x06),
            "xor" => self.emit_alu_rr(operands, 0x07),
            "not" => self.emit_alu_rr(operands, 0x10),
            "neg" => self.emit_alu_rr(operands, 0x0D),
            "shl" => self.emit_alu_rr(operands, 0x08),
            "shr" => self.emit_alu_rr(operands, 0x09),
            "mul" => self.emit_mul(operands),
            "muli" => self.emit_muli(operands),
            "div" => self.emit_div(operands),
            "mod" => self.emit_mod(operands),
            "ld" => self.emit_ld(operands),
            "sd" => self.emit_sd(operands),
            "beq" => self.emit_branch(operands, 0x04),
            "bne" => self.emit_branch(operands, 0x05),
            "blt" => self.emit_branch(operands, 0x0C),
            "bge" => self.emit_branch(operands, 0x0D),
            "ble" => self.emit_branch(operands, 0x0E),
            "bgt" => self.emit_branch(operands, 0x0F),
            "bltu" => self.emit_branch(operands, 0x02),
            "bgeu" => self.emit_branch(operands, 0x03),
            "bzs" => self.emit_bzs(operands),
            "bnz" => self.emit_bnz(operands),
            "j" => self.emit_j(operands),
            "jal" => self.emit_jal(operands),
            "jr" => self.emit_jr(operands),
            "jalr" => self.emit_jalr(operands),
            "ret" => {
                x86_64::emit_ret(&mut self.code);
                Ok(())
            }
            "syscall" => {
                if let Some(Operand::Immediate(Expr::Number(nr))) = operands.first() {
                    let linux_nr = x86_64::translate_syscall(*nr).ok_or_else(|| {
                        self.line_err(format!("unknown Skyscraper syscall number: {}", nr))
                    })?;
                    x86_64::emit_mov_reg_imm32(&mut self.code, 0, linux_nr as i32);
                }
                x86_64::emit_syscall(&mut self.code);
                Ok(())
            }
            "halt" | "nop" => {
                x86_64::emit_nop(&mut self.code);
                Ok(())
            }
            _ => Err(CodegenError::UnknownInstruction {
                name: name.to_string(),
                line: self.current_line,
            }),
        }
    }

    fn emit_cmp(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("cmp requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs1 = self.get_reg(&operands[1])?;
        let rs2 = self.get_reg(&operands[2])?;
        // Rd = Rs1 - Rs2, set flags
        x86_64::emit_mov_reg_reg(&mut self.code, rd, rs1);
        x86_64::emit_sub_reg_reg(&mut self.code, rd, rs2);
        Ok(())
    }

    fn emit_cmpi(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("cmpi requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs1 = self.get_reg(&operands[1])?;
        let imm = self.get_imm(operands, 2)?;
        // Rd = Rs1 - imm, set flags
        x86_64::emit_mov_reg_reg(&mut self.code, rd, rs1);
        x86_64::emit_cmp_reg_imm32(&mut self.code, rd, imm as i32);
        Ok(())
    }

    fn emit_alu_rr(&mut self, operands: &[Operand], op: u8) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("ALU RR requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs1 = self.get_reg(&operands[1])?;
        let rs2 = self.get_reg(&operands[2])?;

        x86_64::emit_mov_reg_reg(&mut self.code, rd, rs1);

        match op {
            0x00 => {
                // ADD
                let need_rex = rd >= 8 || rs2 >= 8;
                if need_rex {
                    self.code.push(
                        0x48 | if rd >= 8 { 0x04 } else { 0 } | if rs2 >= 8 { 0x01 } else { 0 },
                    );
                } else {
                    self.code.push(0x48);
                }
                self.code.push(0x01); // ADD r/m64, r64
                self.code.push(0xC0 | ((rs2 & 0x7) << 3) | (rd & 0x7));
            }
            0x05 => {
                // SUB
                x86_64::emit_sub_reg_reg(&mut self.code, rd, rs2);
            }
            0x04 => {
                // AND
                let need_rex = rd >= 8 || rs2 >= 8;
                if need_rex {
                    self.code.push(
                        0x48 | if rd >= 8 { 0x04 } else { 0 } | if rs2 >= 8 { 0x01 } else { 0 },
                    );
                } else {
                    self.code.push(0x48);
                }
                self.code.push(0x21); // AND r/m64, r64
                self.code.push(0xC0 | ((rs2 & 0x7) << 3) | (rd & 0x7));
            }
            0x06 => {
                // OR
                let need_rex = rd >= 8 || rs2 >= 8;
                if need_rex {
                    self.code.push(
                        0x48 | if rd >= 8 { 0x04 } else { 0 } | if rs2 >= 8 { 0x01 } else { 0 },
                    );
                } else {
                    self.code.push(0x48);
                }
                self.code.push(0x09); // OR r/m64, r64
                self.code.push(0xC0 | ((rs2 & 0x7) << 3) | (rd & 0x7));
            }
            0x07 => {
                // XOR
                x86_64::emit_xor_reg_reg(&mut self.code, rd, rs2);
            }
            0x10 => {
                // NOT (XOR with -1 = all bits set)
                x86_64::emit_mov_reg_imm32(&mut self.code, rd, -1);
                let need_rex = rd >= 8;
                if need_rex {
                    self.code.push(0x48 | 0x04);
                } else {
                    self.code.push(0x48);
                }
                self.code.push(0x31); // XOR r/m64, r64
                self.code.push(0xC0 | ((rd & 0x7) << 3) | (rd & 0x7));
            }
            0x0D => {
                // NEG rd = 0 - rs1
                x86_64::emit_sub_reg_reg(&mut self.code, 0, rd); // sub rax, rd
                x86_64::emit_mov_reg_reg(&mut self.code, rd, 0); // mov rd, rax
            }
            0x08 => {
                // SHL
                x86_64::emit_mov_reg_reg(&mut self.code, 1, rs2); // mov rcx, rs2 (shift count)
                self.code.push(0x48);
                self.code.push(0xD3); // SHL r/m64, cl
                self.code.push(0xE0 | (rd & 0x7));
            }
            0x09 => {
                // SHR
                x86_64::emit_mov_reg_reg(&mut self.code, 1, rs2);
                self.code.push(0x48);
                self.code.push(0xD3); // SHR r/m64, cl
                self.code.push(0xE8 | (rd & 0x7));
            }
            _ => {
                return Err(self.line_err(format!("unsupported ALU op: {}", op)));
            }
        }
        Ok(())
    }

    fn emit_alu_ri(&mut self, operands: &[Operand], op: u8) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("ALU RI requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs1 = self.get_reg(&operands[1])?;
        let imm = self.get_imm(operands, 2)?;

        x86_64::emit_mov_reg_reg(&mut self.code, rd, rs1);

        match op {
            0x00 => x86_64::emit_add_reg_imm32(&mut self.code, rd, imm as i32),
            0x05 => {
                // SUB rd, imm = rd + (-imm)
                x86_64::emit_add_reg_imm32(&mut self.code, rd, -(imm as i32));
            }
            0x04 => {
                // AND rd, imm
                self.code.push(x86_64::rex_w(rd));
                self.code.push(0x81);
                self.code.push(0xE0 | (rd & 0x7)); // AND r/m64, imm32
                self.code.extend_from_slice(&(imm as i32).to_le_bytes());
            }
            0x06 => {
                // OR rd, imm
                x86_64::emit_or_reg_imm32(&mut self.code, rd, imm as i32);
            }
            _ => {
                return Err(self.line_err(format!("unsupported ALU RI op: {}", op)));
            }
        }
        Ok(())
    }

    fn emit_mul(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("mul requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs1 = self.get_reg(&operands[1])?;
        let rs2 = self.get_reg(&operands[2])?;
        // IMUL r64, r/m64: rd = rd * rs2
        // First, move rs1 to rd
        x86_64::emit_mov_reg_reg(&mut self.code, rd, rs1);
        // Then IMUL rd, rs2
        let need_rex = rd >= 8 || rs2 >= 8;
        if need_rex {
            self.code
                .push(0x48 | if rd >= 8 { 0x04 } else { 0 } | if rs2 >= 8 { 0x01 } else { 0 });
        } else {
            self.code.push(0x48);
        }
        self.code.push(0x0F);
        self.code.push(0xAF); // IMUL r64, r/m64
        self.code.push(0xC0 | ((rd & 0x7) << 3) | (rs2 & 0x7));
        Ok(())
    }

    fn emit_muli(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("muli requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs1 = self.get_reg(&operands[1])?;
        let imm = self.get_imm(operands, 2)?;
        x86_64::emit_mov_reg_reg(&mut self.code, rd, rs1);
        // IMUL r64, r/m64, imm32
        let need_rex = rd >= 8;
        if need_rex {
            self.code.push(0x49);
        } else {
            self.code.push(0x48);
        }
        self.code.push(0x69); // IMUL r64, r/m64, imm32
        self.code.push(0xC0 | ((rd & 0x7) << 3) | (rd & 0x7));
        self.code.extend_from_slice(&(imm as i32).to_le_bytes());
        Ok(())
    }

    fn emit_div(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("div requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs1 = self.get_reg(&operands[1])?;
        let rs2 = self.get_reg(&operands[2])?;
        // unsigned div: rax = rs1, rdx = 0, div rs2 -> quotient in rax
        // Save ret0/ret1 if they're the destination
        x86_64::emit_mov_reg_reg(&mut self.code, 0, rs1); // rax = rs1
        x86_64::emit_xor_reg_reg(&mut self.code, 2, 2); // rdx = 0
        // DIV r/m64: rdx:rax / rs2 -> quotient in rax, remainder in rdx
        let need_rex = rs2 >= 8;
        if need_rex {
            self.code.push(0x49);
        } else {
            self.code.push(0x48);
        }
        self.code.push(0xF7);
        self.code.push(0xF0 | (rs2 & 0x7)); // DIV r/m64
        x86_64::emit_mov_reg_reg(&mut self.code, rd, 0); // rd = rax (quotient)
        Ok(())
    }

    fn emit_mod(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("mod requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs1 = self.get_reg(&operands[1])?;
        let rs2 = self.get_reg(&operands[2])?;
        x86_64::emit_mov_reg_reg(&mut self.code, 0, rs1); // rax = rs1
        x86_64::emit_xor_reg_reg(&mut self.code, 2, 2); // rdx = 0
        let need_rex = rs2 >= 8;
        if need_rex {
            self.code.push(0x49);
        } else {
            self.code.push(0x48);
        }
        self.code.push(0xF7);
        self.code.push(0xF0 | (rs2 & 0x7)); // DIV r/m64
        x86_64::emit_mov_reg_reg(&mut self.code, rd, 2); // rd = rdx (remainder)
        Ok(())
    }

    fn emit_ld(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(self.line_err("ld requires two operands".to_string()));
        }
        let dst = self.get_reg(&operands[0])?;
        match &operands[1] {
            Operand::MemoryRef { base, offset } => {
                let base_reg = self.get_reg(&Operand::Register(base.clone()))?;
                let offset_val = match offset {
                    Some(e) => self.eval_expr(e, self.code.len())? as i32,
                    None => 0,
                };
                x86_64::emit_load_reg_mem(&mut self.code, dst, base_reg, offset_val);
                Ok(())
            }
            _ => Err(self.line_err("ld requires memory reference".to_string())),
        }
    }

    fn emit_sd(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(self.line_err("sd requires two operands".to_string()));
        }
        // ISA syntax: sd Rs2, [Rs1 + imm9]
        // operands[0] = Rs2 (source register), operands[1] = [Rs1 + imm9]
        let src = self.get_reg(&operands[0])?;
        match &operands[1] {
            Operand::MemoryRef { base, offset } => {
                let base_reg = self.get_reg(&Operand::Register(base.clone()))?;
                let offset_val = match offset {
                    Some(e) => self.eval_expr(e, self.code.len())? as i32,
                    None => 0,
                };
                x86_64::emit_store_mem_reg(&mut self.code, src, base_reg, offset_val);
                Ok(())
            }
            _ => Err(self.line_err("sd requires memory reference as second operand".to_string())),
        }
    }

    fn emit_branch(&mut self, operands: &[Operand], cc: u8) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(self.line_err("branch requires two operands".to_string()));
        }
        // Jcc rel32: 0F 8x xx xx xx xx (6 bytes)
        // disp32 is at offset +2 from instruction start
        let instr_start = self.code.len();
        x86_64::emit_jcc_rel32(&mut self.code, cc, 0);
        match &operands[1] {
            Operand::Immediate(Expr::LabelRef(name)) => {
                self.fixups.push(Fixup {
                    offset: instr_start + 2,
                    label: name.clone(),
                    kind: FixupKind::Relative {
                        disp_offset: instr_start as i64,
                        instr_size: 6,
                    },
                });
            }
            Operand::Immediate(expr) => {
                let target = self.eval_expr(expr, self.code.len())?;
                let rel = target - instr_start as i64 - 6;
                let bytes = (rel as i32).to_le_bytes();
                self.code[instr_start + 2..instr_start + 6].copy_from_slice(&bytes);
            }
            _ => {
                return Err(self.line_err("branch target must be immediate".to_string()));
            }
        }
        Ok(())
    }

    fn emit_bzs(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(self.line_err("bzs requires two operands".to_string()));
        }
        let reg = self.get_reg(&operands[0])?;
        // TEST reg, reg (3 bytes) + JZ rel32 (6 bytes) = 9 bytes
        let instr_start = self.code.len();
        x86_64::emit_test_reg_reg(&mut self.code, reg, reg);
        x86_64::emit_jcc_rel32(&mut self.code, 0x04, 0); // JZ
        match &operands[1] {
            Operand::Immediate(Expr::LabelRef(name)) => {
                self.fixups.push(Fixup {
                    offset: instr_start + 5,
                    label: name.clone(),
                    kind: FixupKind::Relative {
                        disp_offset: instr_start as i64 + 3,
                        instr_size: 6,
                    },
                });
            }
            Operand::Immediate(expr) => {
                let target = self.eval_expr(expr, self.code.len())?;
                let rel = target - instr_start as i64 - 9;
                let bytes = (rel as i32).to_le_bytes();
                self.code[instr_start + 5..instr_start + 9].copy_from_slice(&bytes);
            }
            _ => {
                return Err(self.line_err("bzs target must be immediate".to_string()));
            }
        }
        Ok(())
    }

    fn emit_bnz(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(self.line_err("bnz requires two operands".to_string()));
        }
        let reg = self.get_reg(&operands[0])?;
        let instr_start = self.code.len();
        x86_64::emit_test_reg_reg(&mut self.code, reg, reg);
        x86_64::emit_jcc_rel32(&mut self.code, 0x05, 0); // JNZ
        match &operands[1] {
            Operand::Immediate(Expr::LabelRef(name)) => {
                self.fixups.push(Fixup {
                    offset: instr_start + 5,
                    label: name.clone(),
                    kind: FixupKind::Relative {
                        disp_offset: instr_start as i64 + 3,
                        instr_size: 6,
                    },
                });
            }
            Operand::Immediate(expr) => {
                let target = self.eval_expr(expr, self.code.len())?;
                let rel = target - instr_start as i64 - 9;
                let bytes = (rel as i32).to_le_bytes();
                self.code[instr_start + 5..instr_start + 9].copy_from_slice(&bytes);
            }
            _ => {
                return Err(self.line_err("bnz target must be immediate".to_string()));
            }
        }
        Ok(())
    }

    fn emit_j(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.is_empty() {
            return Err(self.line_err("j requires an operand".into()));
        }
        let instr_start = self.code.len();
        x86_64::emit_jmp_rel32(&mut self.code, 0);
        match &operands[0] {
            Operand::Immediate(Expr::LabelRef(name)) => {
                self.fixups.push(Fixup {
                    offset: instr_start + 1,
                    label: name.clone(),
                    kind: FixupKind::Relative {
                        disp_offset: instr_start as i64,
                        instr_size: 5,
                    },
                });
            }
            Operand::Immediate(expr) => {
                let target = self.eval_expr(expr, self.code.len())?;
                let rel = target - instr_start as i64 - 5;
                let bytes = (rel as i32).to_le_bytes();
                self.code[instr_start + 1..instr_start + 5].copy_from_slice(&bytes);
            }
            _ => {
                return Err(self.line_err("j target must be immediate".to_string()));
            }
        }
        Ok(())
    }

    fn emit_jal(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.is_empty() {
            return Err(self.line_err("jal requires an operand".to_string()));
        }
        let instr_start = self.code.len();
        x86_64::emit_call_rel32(&mut self.code, 0);
        match &operands[0] {
            Operand::Immediate(Expr::LabelRef(name)) => {
                self.fixups.push(Fixup {
                    offset: instr_start + 1,
                    label: name.clone(),
                    kind: FixupKind::Relative {
                        disp_offset: instr_start as i64,
                        instr_size: 5,
                    },
                });
            }
            Operand::Immediate(expr) => {
                let target = self.eval_expr(expr, self.code.len())?;
                let rel = target - instr_start as i64 - 5;
                let bytes = (rel as i32).to_le_bytes();
                self.code[instr_start + 1..instr_start + 5].copy_from_slice(&bytes);
            }
            _ => {
                return Err(self.line_err("jal target must be immediate".to_string()));
            }
        }
        Ok(())
    }

    fn emit_jr(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.is_empty() {
            return Err(self.line_err("jr requires an operand".to_string()));
        }
        let reg = self.get_reg(&operands[0])?;
        x86_64::emit_jmp_reg(&mut self.code, reg);
        Ok(())
    }

    fn emit_jalr(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.is_empty() {
            return Err(self.line_err("jalr requires an operand".to_string()));
        }
        let reg = self.get_reg(&operands[0])?;
        x86_64::emit_call_reg(&mut self.code, reg);
        Ok(())
    }

    fn eval_operand(&self, operands: &[Operand], idx: usize) -> Result<i64, CodegenError> {
        match &operands[idx] {
            Operand::Immediate(e) => self.eval_expr(e, self.code.len()),
            _ => Err(self.line_err("expected immediate operand".to_string())),
        }
    }

    fn get_imm(&self, operands: &[Operand], idx: usize) -> Result<i64, CodegenError> {
        self.eval_operand(operands, idx)
    }

    fn emit_ldi(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(self.line_err("ldi requires two operands".to_string()));
        }
        let reg = self.get_reg(&operands[0])?;
        match &operands[1] {
            Operand::Immediate(Expr::LabelRef(name)) => {
                // Check if it's a constant first
                if let Some(val) = self.constants.get(name) {
                    let val = *val;
                    if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
                        x86_64::emit_mov_reg_imm32(&mut self.code, reg, val as i32);
                    } else {
                        x86_64::emit_mov_reg_imm64(&mut self.code, reg, val);
                    }
                    return Ok(());
                }
                // Otherwise, it's a label - emit with fixup
                let fixup_offset = self.code.len() + 2; // skip REX + opcode
                x86_64::emit_mov_reg_imm64(&mut self.code, reg, 0);
                self.fixups.push(Fixup {
                    offset: fixup_offset,
                    label: name.clone(),
                    kind: FixupKind::Absolute,
                });
                Ok(())
            }
            Operand::Immediate(expr) => {
                let code_size = self.code.len();
                let val = self.eval_expr(expr, code_size)?;
                if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
                    x86_64::emit_mov_reg_imm32(&mut self.code, reg, val as i32);
                } else {
                    x86_64::emit_mov_reg_imm64(&mut self.code, reg, val);
                }
                Ok(())
            }
            _ => Err(self.line_err("ldi second operand must be immediate".to_string())),
        }
    }

    fn emit_lui(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(self.line_err("lui requires two operands".to_string()));
        }
        let reg = self.get_reg(&operands[0])?;
        match &operands[1] {
            Operand::Immediate(Expr::LabelRef(name)) => {
                // Check if it's a constant first
                if let Some(val) = self.constants.get(name) {
                    let val = *val;
                    if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
                        x86_64::emit_mov_reg_imm32(&mut self.code, reg, val as i32);
                    } else {
                        x86_64::emit_mov_reg_imm64(&mut self.code, reg, val);
                    }
                    return Ok(());
                }
                // Otherwise, it's a label - emit with fixup
                let fixup_offset = self.code.len() + 2;
                x86_64::emit_mov_reg_imm64(&mut self.code, reg, 0);
                self.fixups.push(Fixup {
                    offset: fixup_offset,
                    label: name.clone(),
                    kind: FixupKind::Absolute,
                });
                Ok(())
            }
            Operand::Immediate(expr) => {
                let code_size = self.code.len();
                let val = self.eval_expr(expr, code_size)?;
                if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
                    x86_64::emit_mov_reg_imm32(&mut self.code, reg, val as i32);
                } else {
                    x86_64::emit_mov_reg_imm64(&mut self.code, reg, val);
                }
                Ok(())
            }
            _ => Err(self.line_err("lui second operand must be immediate".to_string())),
        }
    }

    fn emit_ori(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(self.line_err("ori requires three operands".to_string()));
        }
        let rd = self.get_reg(&operands[0])?;
        let rs = self.get_reg(&operands[1])?;
        match &operands[2] {
            Operand::Immediate(Expr::LabelRef(name)) => {
                // Check if it's a constant first
                if let Some(val) = self.constants.get(name) {
                    let val = *val;
                    if rd != rs {
                        x86_64::emit_mov_reg_reg(&mut self.code, rd, rs);
                    }
                    if val >= 0 && val <= i32::MAX as i64 {
                        x86_64::emit_or_reg_imm32(&mut self.code, rd, val as i32);
                    } else {
                        x86_64::emit_mov_reg_imm64(&mut self.code, rd, val);
                    }
                    return Ok(());
                }
                // ori rd, rs, label is typically the second half of a lui+ori pair.
                // On x86-64, lui already loaded the full address, so ori with same label
                // is redundant. Just skip the ori.
                Ok(())
            }
            Operand::Immediate(expr) => {
                let code_size = self.code.len();
                let val = self.eval_expr(expr, code_size)?;
                if rd != rs {
                    x86_64::emit_mov_reg_reg(&mut self.code, rd, rs);
                }
                if val >= 0 && val <= i32::MAX as i64 {
                    x86_64::emit_or_reg_imm32(&mut self.code, rd, val as i32);
                } else {
                    x86_64::emit_mov_reg_imm64(&mut self.code, rd, val);
                }
                Ok(())
            }
            _ => Err(self.line_err("ori third operand must be immediate".to_string())),
        }
    }

    fn emit_mv(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(self.line_err("mv requires two operands".to_string()));
        }
        let dst = self.get_reg(&operands[0])?;
        let src = self.get_reg(&operands[1])?;
        x86_64::emit_mov_reg_reg(&mut self.code, dst, src);
        Ok(())
    }

    fn emit_directive(&mut self, name: &str, args: &[Expr]) -> Result<(), CodegenError> {
        match name {
            ".text" => self.current_section = Section::Code,
            ".data" => self.current_section = Section::Data,
            ".bss" => self.current_section = Section::Bss,
            ".global" => {
                if let Some(Expr::LabelRef(label)) = args.first() {
                    self.entry_point = Some(label.clone());
                }
            }
            ".string" => {
                if let Some(Expr::String(s)) = args.first() {
                    self.data.extend_from_slice(s.as_bytes());
                }
            }
            ".byte" => {
                let val = self.eval_expr(
                    args.first()
                        .ok_or_else(|| self.line_err(".byte requires an argument".to_string()))?,
                    0,
                )?;
                self.data.push(val as u8);
            }
            ".word" => {
                let val = self.eval_expr(
                    args.first()
                        .ok_or_else(|| self.line_err(".word requires an argument".to_string()))?,
                    0,
                )?;
                self.data.extend_from_slice(&(val as u16).to_le_bytes());
            }
            ".long" => {
                let val = self.eval_expr(
                    args.first()
                        .ok_or_else(|| self.line_err(".long requires an argument".to_string()))?,
                    0,
                )?;
                self.data.extend_from_slice(&(val as u32).to_le_bytes());
            }
            ".quad" => {
                let val = self.eval_expr(
                    args.first()
                        .ok_or_else(|| self.line_err(".quad requires an argument".to_string()))?,
                    0,
                )?;
                self.data.extend_from_slice(&val.to_le_bytes());
            }
            ".space" => {
                let size = self.eval_expr(
                    args.first().ok_or_else(|| {
                        self.line_err(".space requires a size argument".to_string())
                    })?,
                    0,
                )? as usize;
                match self.current_section {
                    Section::Data => self.data.extend(std::iter::repeat_n(0, size)),
                    Section::Bss => self.bss_size += size,
                    Section::Code => self.code.extend(std::iter::repeat_n(0, size)),
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn eval_expr(&self, expr: &Expr, code_size: usize) -> Result<i64, CodegenError> {
        match expr {
            Expr::Number(n) => Ok(*n),
            Expr::String(s) => Ok(s.len() as i64),
            Expr::CurrentAddr => {
                let offset = elf::HDR_SIZE as usize + code_size + self.data.len();
                Ok(BASE_ADDR + offset as i64)
            }
            Expr::LabelRef(name) => {
                if let Some(pos) = self.label_positions.get(name) {
                    let addr = match pos.section {
                        Section::Code => (BASE_ADDR as usize) + elf::HDR_SIZE as usize + pos.offset,
                        Section::Data => {
                            (BASE_ADDR as usize) + elf::HDR_SIZE as usize + code_size + pos.offset
                        }
                        Section::Bss => {
                            (BASE_ADDR as usize)
                                + elf::HDR_SIZE as usize
                                + code_size
                                + self.data.len()
                                + pos.offset
                        }
                    };
                    Ok(addr as i64)
                } else if let Some(val) = self.constants.get(name) {
                    Ok(*val)
                } else {
                    Err(CodegenError::MissingLabel(name.clone()))
                }
            }
            Expr::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left, code_size)?;
                let r = self.eval_expr(right, code_size)?;
                match op {
                    BinOp::Add => Ok(l.wrapping_add(r)),
                    BinOp::Sub => Ok(l.wrapping_sub(r)),
                    BinOp::Mul => Ok(l.wrapping_mul(r)),
                    BinOp::Div => Ok(l.wrapping_div(r)),
                    BinOp::Mod => Ok(l.wrapping_rem(r)),
                }
            }
        }
    }

    fn resolve_fixups(&mut self) -> Result<(), CodegenError> {
        for fixup in &self.fixups {
            let addr = self
                .labels
                .get(&fixup.label)
                .ok_or_else(|| CodegenError::MissingLabel(fixup.label.clone()))?;
            match fixup.kind {
                FixupKind::Absolute => {
                    let bytes = addr.to_le_bytes();
                    self.code[fixup.offset..fixup.offset + 8].copy_from_slice(&bytes);
                }
                FixupKind::Relative {
                    disp_offset,
                    instr_size,
                } => {
                    let base_offset = (BASE_ADDR as usize) + elf::HDR_SIZE as usize;
                    let next_ip = base_offset + disp_offset as usize + instr_size as usize;
                    let rel = *addr as i64 - next_ip as i64;
                    let bytes = (rel as i32).to_le_bytes();
                    self.code[fixup.offset..fixup.offset + 4].copy_from_slice(&bytes);
                }
            }
        }
        Ok(())
    }

    fn get_reg(&self, operand: &Operand) -> Result<u8, CodegenError> {
        match operand {
            Operand::Register(name) => {
                x86_64::register_index(name).ok_or_else(|| CodegenError::UnknownRegister {
                    name: name.clone(),
                    line: self.current_line,
                })
            }
            _ => Err(self.line_err("expected register operand".to_string())),
        }
    }
}

const BASE_ADDR: i64 = 0x400000;

pub fn compile(program: &Program, output: &Path) -> Result<(), CodegenError> {
    let mut codegen = Codegen::new();
    codegen.compile(program)?;

    let entry_offset = if let Some(name) = &codegen.entry_point {
        let addr = codegen
            .labels
            .get(name)
            .ok_or_else(|| CodegenError::MissingLabel(name.clone()))?;
        addr - BASE_ADDR as usize - elf::HDR_SIZE as usize
    } else {
        return Err(CodegenError::MissingEntryPoint);
    };

    elf::write_elf(
        &codegen.code,
        &codegen.data,
        codegen.bss_size,
        entry_offset,
        output,
    )?;
    Ok(())
}
