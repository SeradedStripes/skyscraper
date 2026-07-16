pub mod elf;
pub mod x86_64;

use crate::parser::{BinOp, Expr, Operand, Program, Statement};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CodegenError {
    UnknownInstruction(String),
    UnknownRegister(String),
    MissingLabel(String),
    MissingEntryPoint,
    InvalidOperand(String),
    IoError(std::io::Error),
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
}

struct LabelPos {
    section: Section,
    offset: usize,
}

struct Fixup {
    offset: usize,
    label: String,
}

struct Codegen {
    code: Vec<u8>,
    data: Vec<u8>,
    constants: HashMap<String, i64>,
    label_positions: HashMap<String, LabelPos>,
    labels: HashMap<String, usize>,
    fixups: Vec<Fixup>,
    entry_point: Option<String>,
    current_section: Section,
}

impl Codegen {
    fn new() -> Self {
        Self {
            code: Vec::new(),
            data: Vec::new(),
            constants: HashMap::new(),
            label_positions: HashMap::new(),
            labels: HashMap::new(),
            fixups: Vec::new(),
            entry_point: None,
            current_section: Section::Code,
        }
    }

    fn compile(&mut self, program: &Program) -> Result<(), CodegenError> {
        // Pass 1: emit code and data, collect label positions and fixups
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        // Compute final label addresses
        let code_size = self.code.len();
        for (name, pos) in &self.label_positions {
            let addr = match pos.section {
                Section::Code => (BASE_ADDR as usize) + elf::HDR_SIZE as usize + pos.offset,
                Section::Data => {
                    (BASE_ADDR as usize) + elf::HDR_SIZE as usize + code_size + pos.offset
                }
            };
            self.labels.insert(name.clone(), addr);
        }

        // Re-evaluate constants with correct label addresses
        for stmt in &program.statements {
            if let Statement::ConstantDef { name, value } = stmt {
                let val = self.eval_expr(value, code_size)?;
                self.constants.insert(name.clone(), val);
            }
        }

        // Resolve fixups (label references in immediate operands)
        self.resolve_fixups()?;
        Ok(())
    }

    fn emit_statement(&mut self, stmt: &Statement) -> Result<(), CodegenError> {
        match stmt {
            Statement::LabelDef(name) => {
                self.label_positions.insert(
                    name.clone(),
                    LabelPos {
                        section: self.current_section,
                        offset: self.section_offset(),
                    },
                );
            }
            Statement::Instruction { name, operands } => {
                self.emit_instruction(name, operands)?;
            }
            Statement::Directive { name, args } => {
                self.emit_directive(name, args)?;
            }
            Statement::ConstantDef { name, value } => {
                // Evaluate with partial info; will be re-evaluated later
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
        }
    }

    fn emit_instruction(&mut self, name: &str, operands: &[Operand]) -> Result<(), CodegenError> {
        match name {
            "ldi" => self.emit_ldi(operands),
            "lui" => self.emit_lui(operands),
            "ori" => self.emit_ori(operands),
            "mv" => self.emit_mv(operands),
            "syscall" => {
                if let Some(Operand::Immediate(Expr::Number(nr))) = operands.first() {
                    let linux_nr = x86_64::translate_syscall(*nr).ok_or_else(|| {
                        CodegenError::InvalidOperand(format!(
                            "unknown Skyscraper syscall number: {}",
                            nr
                        ))
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
            _ => Err(CodegenError::UnknownInstruction(name.to_string())),
        }
    }

    fn emit_ldi(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand(
                "ldi requires two operands".into(),
            ));
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
            _ => Err(CodegenError::InvalidOperand(
                "ldi second operand must be immediate".into(),
            )),
        }
    }

    fn emit_lui(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand(
                "lui requires two operands".into(),
            ));
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
            _ => Err(CodegenError::InvalidOperand(
                "lui second operand must be immediate".into(),
            )),
        }
    }

    fn emit_ori(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 3 {
            return Err(CodegenError::InvalidOperand(
                "ori requires three operands".into(),
            ));
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
            _ => Err(CodegenError::InvalidOperand(
                "ori third operand must be immediate".into(),
            )),
        }
    }

    fn emit_mv(&mut self, operands: &[Operand]) -> Result<(), CodegenError> {
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand(
                "mv requires two operands".into(),
            ));
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
            ".bss" => {}
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
            let bytes = addr.to_le_bytes();
            self.code[fixup.offset..fixup.offset + 8].copy_from_slice(&bytes);
        }
        Ok(())
    }

    fn get_reg(&self, operand: &Operand) -> Result<u8, CodegenError> {
        match operand {
            Operand::Register(name) => x86_64::register_index(name)
                .ok_or_else(|| CodegenError::UnknownRegister(name.clone())),
            _ => Err(CodegenError::InvalidOperand(
                "expected register operand".into(),
            )),
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

    elf::write_elf(&codegen.code, &codegen.data, entry_offset, output)?;
    Ok(())
}
