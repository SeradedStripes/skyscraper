#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(i64),
    Char(char),
    String(String),

    // Identifiers and keywords
    Instruction(String),
    Register(String),
    Directive(String),
    LabelDef(String),
    LabelRef(String),

    // Symbols
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    Bang,
    Colon,
    Assign,
    Dollar,
    Dot,

    // Special
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}
