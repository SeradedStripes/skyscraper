#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    LabelDef(String),
    Instruction {
        name: String,
        operands: Vec<Operand>,
    },
    Directive {
        name: String,
        args: Vec<Expr>,
    },
    ConstantDef {
        name: String,
        value: Expr,
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
