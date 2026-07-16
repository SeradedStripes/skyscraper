mod token;

pub use token::{Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct LexerError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            if token.kind != TokenKind::Newline {
                tokens.push(token);
            }
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            line: self.line,
            col: self.col,
        });

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        let line = self.line;
        let col = self.col;

        let ch = self.peek();

        if ch == '\n' {
            self.advance();
            self.line += 1;
            self.col = 1;
            return Ok(Token {
                kind: TokenKind::Newline,
                line,
                col,
            });
        }

        if ch == ';' {
            self.skip_comment();
            return self.next_token();
        }

        if ch == '"' {
            return self.read_string().map(|s| Token {
                kind: TokenKind::String(s),
                line,
                col,
            });
        }

        if ch == '\'' {
            return self.read_char().map(|c| Token {
                kind: TokenKind::Char(c),
                line,
                col,
            });
        }

        if ch.is_ascii_digit()
            || (ch == '-' && self.peek_next().is_some_and(|c| c.is_ascii_digit()))
        {
            return self.read_number().map(|n| Token {
                kind: TokenKind::Number(n),
                line,
                col,
            });
        }

        if ch == '0' && self.peek_next() == Some('x') {
            return self.read_hex().map(|n| Token {
                kind: TokenKind::Number(n),
                line,
                col,
            });
        }

        if ch == '0' && self.peek_next() == Some('b') {
            return self.read_binary().map(|n| Token {
                kind: TokenKind::Number(n),
                line,
                col,
            });
        }

        if ch == '0' && self.peek_next() == Some('o') {
            return self.read_octal().map(|n| Token {
                kind: TokenKind::Number(n),
                line,
                col,
            });
        }

        if ch == '.' && self.peek_next().is_some_and(|c| c.is_ascii_alphabetic()) {
            return self.read_directive();
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            return self.read_identifier_or_keyword();
        }

        self.read_symbol()
    }

    fn peek(&self) -> char {
        self.source[self.pos]
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.pos];
        self.pos += 1;
        self.col += 1;
        ch
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn read_string(&mut self) -> Result<String, LexerError> {
        self.advance(); // skip opening quote
        let mut s = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\\' {
                self.advance();
                let escaped = self.read_escape_sequence()?;
                s.push(escaped);
            } else {
                s.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(LexerError {
                line: self.line,
                col: self.col,
                message: "unterminated string literal".to_string(),
            });
        }

        self.advance(); // skip closing quote
        Ok(s)
    }

    fn read_char(&mut self) -> Result<char, LexerError> {
        self.advance(); // skip opening quote

        let ch = if self.peek() == '\\' {
            self.advance();
            self.read_escape_sequence()?
        } else {
            self.advance()
        };

        if self.peek() != '\'' {
            return Err(LexerError {
                line: self.line,
                col: self.col,
                message: "unterminated character literal".to_string(),
            });
        }

        self.advance(); // skip closing quote
        Ok(ch)
    }

    fn read_escape_sequence(&mut self) -> Result<char, LexerError> {
        let ch = self.advance();
        match ch {
            'n' => Ok('\n'),
            't' => Ok('\t'),
            'r' => Ok('\r'),
            '0' => Ok('\0'),
            '\\' => Ok('\\'),
            '\'' => Ok('\''),
            '"' => Ok('"'),
            'x' => {
                let hi = self.advance();
                let lo = self.advance();
                let hex = format!("{}{}", hi, lo);
                u8::from_str_radix(&hex, 16)
                    .map(|b| b as char)
                    .map_err(|_| LexerError {
                        line: self.line,
                        col: self.col,
                        message: format!("invalid hex escape: \\x{}", hex),
                    })
            }
            _ => Err(LexerError {
                line: self.line,
                col: self.col,
                message: format!("invalid escape sequence: \\{}", ch),
            }),
        }
    }

    fn read_number(&mut self) -> Result<i64, LexerError> {
        let mut num_str = String::new();

        if self.peek() == '-' {
            num_str.push(self.advance());
        }

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num_str.push(self.advance());
        }

        num_str.parse::<i64>().map_err(|_| LexerError {
            line: self.line,
            col: self.col,
            message: format!("invalid number: {}", num_str),
        })
    }

    fn read_hex(&mut self) -> Result<i64, LexerError> {
        self.advance(); // skip '0'
        self.advance(); // skip 'x'

        let mut num_str = String::new();
        while !self.is_at_end() && (self.peek().is_ascii_hexdigit()) {
            num_str.push(self.advance());
        }

        i64::from_str_radix(&num_str, 16).map_err(|_| LexerError {
            line: self.line,
            col: self.col,
            message: format!("invalid hex number: 0x{}", num_str),
        })
    }

    fn read_binary(&mut self) -> Result<i64, LexerError> {
        self.advance(); // skip '0'
        self.advance(); // skip 'b'

        let mut num_str = String::new();
        while !self.is_at_end() && (self.peek() == '0' || self.peek() == '1') {
            num_str.push(self.advance());
        }

        i64::from_str_radix(&num_str, 2).map_err(|_| LexerError {
            line: self.line,
            col: self.col,
            message: format!("invalid binary number: 0b{}", num_str),
        })
    }

    fn read_octal(&mut self) -> Result<i64, LexerError> {
        self.advance(); // skip '0'
        self.advance(); // skip 'o'

        let mut num_str = String::new();
        while !self.is_at_end() && self.peek().is_ascii_digit() && self.peek() <= '7' {
            num_str.push(self.advance());
        }

        i64::from_str_radix(&num_str, 8).map_err(|_| LexerError {
            line: self.line,
            col: self.col,
            message: format!("invalid octal number: 0o{}", num_str),
        })
    }

    fn read_directive(&mut self) -> Result<Token, LexerError> {
        let line = self.line;
        let col = self.col;
        let mut ident = String::new();

        ident.push(self.advance()); // skip '.'

        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
            ident.push(self.advance());
        }

        Ok(Token {
            kind: TokenKind::Directive(ident),
            line,
            col,
        })
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token, LexerError> {
        let line = self.line;
        let col = self.col;
        let mut ident = String::new();

        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
            ident.push(self.advance());
        }

        // Check for label definition (ends with ':')
        if self.peek() == ':' {
            self.advance();
            return Ok(Token {
                kind: TokenKind::LabelDef(ident),
                line,
                col,
            });
        }

        // Determine token kind
        let kind = if ident.starts_with('.') {
            TokenKind::Directive(ident)
        } else if is_instruction(&ident) {
            TokenKind::Instruction(ident)
        } else if is_register(&ident) {
            TokenKind::Register(ident)
        } else {
            TokenKind::LabelRef(ident)
        };

        Ok(Token { kind, line, col })
    }

    fn read_symbol(&mut self) -> Result<Token, LexerError> {
        let line = self.line;
        let col = self.col;
        let ch = self.advance();

        let kind = match ch {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '&' => TokenKind::Ampersand,
            '|' => TokenKind::Pipe,
            '^' => TokenKind::Caret,
            '~' => TokenKind::Tilde,
            '!' => TokenKind::Bang,
            ':' => TokenKind::Colon,
            '=' => TokenKind::Assign,
            '$' => TokenKind::Dollar,
            '.' => TokenKind::Dot,
            _ => {
                return Err(LexerError {
                    line,
                    col,
                    message: format!("unexpected character: '{}'", ch),
                });
            }
        };

        Ok(Token { kind, line, col })
    }
}

fn is_instruction(s: &str) -> bool {
    matches!(
        s,
        "add"
            | "sub"
            | "mul"
            | "div"
            | "mod"
            | "and"
            | "or"
            | "xor"
            | "shl"
            | "shr"
            | "sar"
            | "neg"
            | "not"
            | "addi"
            | "subi"
            | "muli"
            | "divi"
            | "modi"
            | "andi"
            | "ori"
            | "xori"
            | "shli"
            | "shri"
            | "sari"
            | "lb"
            | "lh"
            | "lw"
            | "ld"
            | "lbs"
            | "lhs"
            | "lws"
            | "lfs"
            | "lfd"
            | "sb"
            | "sh"
            | "sw"
            | "sd"
            | "sfs"
            | "sfd"
            | "beq"
            | "bne"
            | "blt"
            | "bge"
            | "bltu"
            | "bgeu"
            | "ble"
            | "bgt"
            | "bzs"
            | "bnz"
            | "j"
            | "jal"
            | "jr"
            | "jalr"
            | "ret"
            | "lui"
            | "ldi"
            | "ldiu"
            | "cmp"
            | "tst"
            | "cmpi"
            | "tsti"
            | "mv"
            | "seb"
            | "seh"
            | "sew"
            | "zlb"
            | "zlh"
            | "zlw"
            | "mfpc"
            | "mtpc"
            | "mflr"
            | "mtlr"
            | "mfsp"
            | "mtsp"
            | "mffp"
            | "mtfp"
            | "vadd"
            | "vsub"
            | "vmul"
            | "vdiv"
            | "vand"
            | "vor"
            | "vxor"
            | "vnot"
            | "vld"
            | "vst"
            | "vdup"
            | "mset"
            | "mand"
            | "mor"
            | "mxor"
            | "mnot"
            | "mrd"
            | "mwr"
            | "mbs"
            | "mbc"
            | "mbt"
            | "syscall"
            | "halt"
            | "nop"
    )
}

fn is_register(s: &str) -> bool {
    if s.starts_with("r") && s.len() > 1 && s[1..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if s.starts_with("arg") && s.len() > 3 && s[3..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if s.starts_with("ret") && s.len() > 3 && s[3..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if s.starts_with("temp") && s.len() > 4 && s[4..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if s.starts_with("vec") && s.len() > 3 && s[3..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if s.starts_with("mask") && s.len() > 4 && s[4..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    matches!(s, "sp" | "fp" | "lr" | "pc" | "flags" | "zero" | "one")
}
