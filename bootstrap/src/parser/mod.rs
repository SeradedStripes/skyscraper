mod ast;

pub use ast::*;

use crate::lexer::{Lexer, LexerError, Token, TokenKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    file: String,
    base_dir: PathBuf,
    included: HashSet<PathBuf>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ParseError {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl Parser {
    pub fn new(source: &str, file: &str) -> Result<Self, LexerError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let path = Path::new(file);
        let base_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        Ok(Parser {
            tokens,
            pos: 0,
            file: file.to_string(),
            base_dir,
            included: HashSet::new(),
        })
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if matches!(&self.peek().kind, TokenKind::Directive(n) if n == ".include") {
                let included = self.parse_include()?;
                statements.extend(included);
            } else {
                let stmt = self.parse_statement()?;
                statements.push(stmt);
            }
        }

        Ok(Program { statements })
    }

    fn parse_error(&self, line: usize, col: usize, message: String) -> ParseError {
        ParseError {
            file: self.file.clone(),
            line,
            col,
            message,
        }
    }

    fn parse_include(&mut self) -> Result<Vec<Statement>, ParseError> {
        let token = self.advance();
        let line = token.line;

        let path_token = self.peek().clone();
        let rel_path = match &path_token.kind {
            TokenKind::String(s) => s.clone(),
            _ => {
                return Err(self.parse_error(
                    line,
                    path_token.col,
                    "include requires a string path".to_string(),
                ));
            }
        };
        self.advance();

        let abs_path = self.base_dir.join(&rel_path);
        let canonical = abs_path
            .canonicalize()
            .map_err(|e| self.parse_error(line, 0, format!("cannot read '{}': {}", rel_path, e)))?;

        if self.included.contains(&canonical) {
            return Ok(Vec::new());
        }
        self.included.insert(canonical.clone());

        let source = std::fs::read_to_string(&canonical)
            .map_err(|e| self.parse_error(line, 0, format!("cannot read '{}': {}", rel_path, e)))?;

        let included_base = canonical.parent().unwrap_or(Path::new(".")).to_path_buf();

        let mut sub = Parser {
            tokens: Vec::new(),
            pos: 0,
            file: rel_path.clone(),
            base_dir: included_base,
            included: self.included.clone(),
        };
        let mut lexer = Lexer::new(&source);
        sub.tokens = lexer.tokenize().map_err(|e| {
            self.parse_error(
                line,
                0,
                format!("lexer error in '{}': {}", rel_path, e.message),
            )
        })?;

        let program = sub.parse()?;
        self.included = sub.included;

        Ok(program.statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.peek().clone();

        match &token.kind {
            TokenKind::LabelDef(name) => {
                let name = name.clone();
                let line = token.line;
                self.advance();
                Ok(Statement::LabelDef { name, line })
            }
            TokenKind::Instruction(_) => self.parse_instruction(),
            TokenKind::Directive(_) => self.parse_directive(),
            TokenKind::LabelRef(name) => {
                // Could be a constant definition: name = expr
                if self.peek_next_is(&TokenKind::Assign) {
                    self.parse_constant_def()
                } else {
                    Err(self.parse_error(
                        token.line,
                        token.col,
                        format!("unexpected identifier: {}", name),
                    ))
                }
            }
            _ => {
                self.advance();
                Err(self.parse_error(
                    token.line,
                    token.col,
                    format!("unexpected token: {:?}", token.kind),
                ))
            }
        }
    }

    fn parse_instruction(&mut self) -> Result<Statement, ParseError> {
        let token = self.advance();
        let line = token.line;
        let name = match &token.kind {
            TokenKind::Instruction(n) => n.clone(),
            _ => unreachable!(),
        };

        let mut operands = Vec::new();

        if !self.is_at_end() && !self.is_newline_or_eof() {
            operands.push(self.parse_operand()?);

            while self.peek_is(&TokenKind::Comma) {
                self.advance();
                operands.push(self.parse_operand()?);
            }
        }

        Ok(Statement::Instruction {
            name,
            operands,
            line,
        })
    }

    fn parse_operand(&mut self) -> Result<Operand, ParseError> {
        let token = self.peek().clone();

        match &token.kind {
            TokenKind::Register(name) => {
                let name = name.clone();
                self.advance();
                Ok(Operand::Register(name))
            }
            TokenKind::LBracket => self.parse_memory_ref(),
            TokenKind::Number(_)
            | TokenKind::Minus
            | TokenKind::LabelRef(_)
            | TokenKind::Dollar => Ok(Operand::Immediate(self.parse_expr()?)),
            TokenKind::Char(c) => {
                let c = *c;
                self.advance();
                Ok(Operand::Immediate(Expr::Number(c as i64)))
            }
            _ => Err(self.parse_error(
                token.line,
                token.col,
                format!("unexpected token in operand: {:?}", token.kind),
            )),
        }
    }

    fn parse_memory_ref(&mut self) -> Result<Operand, ParseError> {
        self.advance(); // skip '['

        let base = match &self.peek().kind {
            TokenKind::Register(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            TokenKind::LabelRef(name) => {
                let name = name.clone();
                self.advance();
                if self.peek_is(&TokenKind::RBracket) {
                    self.advance();
                    return Ok(Operand::MemoryRef {
                        base: name,
                        offset: None,
                    });
                }
                return Err(self.parse_error(
                    self.peek().line,
                    self.peek().col,
                    "expected ']' after label in memory reference".to_string(),
                ));
            }
            _ => {
                return Err(self.parse_error(
                    self.peek().line,
                    self.peek().col,
                    "expected register or label after '['".to_string(),
                ));
            }
        };

        if !self.peek_is(&TokenKind::Plus) {
            self.expect(&TokenKind::RBracket)?;
            return Ok(Operand::MemoryRef { base, offset: None });
        }

        self.advance(); // skip '+'
        let offset = self.parse_expr()?;
        self.expect(&TokenKind::RBracket)?;

        Ok(Operand::MemoryRef {
            base,
            offset: Some(offset),
        })
    }

    fn parse_directive(&mut self) -> Result<Statement, ParseError> {
        let token = self.advance();
        let line = token.line;
        let name = match &token.kind {
            TokenKind::Directive(n) => n.clone(),
            _ => unreachable!(),
        };

        let mut args = Vec::new();

        if self.is_expr_start() {
            args.push(self.parse_expr()?);

            while self.peek_is(&TokenKind::Comma) {
                self.advance();
                args.push(self.parse_expr()?);
            }
        }

        Ok(Statement::Directive { name, args, line })
    }

    fn is_expr_start(&self) -> bool {
        !self.is_at_end()
            && matches!(
                self.peek().kind,
                TokenKind::Number(_)
                    | TokenKind::String(_)
                    | TokenKind::Char(_)
                    | TokenKind::Minus
                    | TokenKind::LabelRef(_)
                    | TokenKind::Dollar
                    | TokenKind::Dot
                    | TokenKind::LParen
                    | TokenKind::LBracket
                    | TokenKind::Ampersand
            )
    }

    fn parse_constant_def(&mut self) -> Result<Statement, ParseError> {
        let line = self.peek().line;
        let name = match &self.peek().kind {
            TokenKind::LabelRef(n) => n.clone(),
            _ => unreachable!(),
        };
        self.advance();
        self.expect(&TokenKind::Assign)?;
        let value = self.parse_expr()?;
        Ok(Statement::ConstantDef { name, value, line })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;

        while self.peek_is(&TokenKind::Plus)
            || self.peek_is(&TokenKind::Minus)
            || self.peek_is(&TokenKind::Star)
            || self.peek_is(&TokenKind::Slash)
            || self.peek_is(&TokenKind::Percent)
        {
            let op = match &self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_primary()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();

        match &token.kind {
            TokenKind::Number(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Number(n))
            }
            TokenKind::Minus => {
                self.advance();
                let n = match &self.peek().kind {
                    TokenKind::Number(n) => -*n,
                    _ => {
                        return Err(self.parse_error(
                            self.peek().line,
                            self.peek().col,
                            "expected number after '-'".to_string(),
                        ));
                    }
                };
                self.advance();
                Ok(Expr::Number(n))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::String(s))
            }
            TokenKind::LabelRef(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::LabelRef(name))
            }
            TokenKind::Dollar | TokenKind::Dot => {
                self.advance();
                Ok(Expr::CurrentAddr)
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(self.parse_error(
                token.line,
                token.col,
                format!("expected expression, got {:?}", token.kind),
            )),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
            col: 0,
        })
    }

    fn peek_next_is(&self, kind: &TokenKind) -> bool {
        self.tokens
            .get(self.pos + 1)
            .is_some_and(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(kind))
    }

    fn peek_is(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        if self.peek_is(kind) {
            Ok(self.advance())
        } else {
            Err(self.parse_error(
                self.peek().line,
                self.peek().col,
                format!("expected {:?}, got {:?}", kind, self.peek().kind),
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.peek_is(&TokenKind::Eof)
    }

    fn is_newline_or_eof(&self) -> bool {
        self.peek_is(&TokenKind::Newline) || self.peek_is(&TokenKind::Eof)
    }
}
