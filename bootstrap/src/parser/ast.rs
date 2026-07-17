#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    LabelDef {
        name: String,
        line: usize,
    },
    Instruction {
        name: String,
        operands: Vec<Operand>,
        line: usize,
    },
    Directive {
        name: String,
        args: Vec<Expr>,
        line: usize,
    },
    ConstantDef {
        name: String,
        value: Expr,
        line: usize,
    },
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(String),
    Immediate(Expr),
    MemoryRef { base: String, offset: Option<Expr> },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    String(String),
    LabelRef(String),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    CurrentAddr,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
