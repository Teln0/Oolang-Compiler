use crate::lexer::BinOpTokenKind::*;
use crate::lexer::DelimTokenKind::*;
use crate::lexer::KeywordTokenKind::*;
use crate::lexer::TokenKind::*;
use crate::reporting::CharSpan;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BinOpTokenKind {
    Plus,
    Minus,
    Star,
    Slash,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DelimTokenKind {
    Paren,
    SBracket,
    CBracket,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum KeywordTokenKind {
    Pub,
    Priv,
    Abstract,
    Static,
    Native,

    Class,
    Inter,
    Enum,
    Impl,

    Where,
    Fn,

    Mod,
    Use,

    If,
    Else,
    While,
    Match,
    Loop,
    For,

    Let,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TokenKind {
    Eq,
    EqEq,
    Not,
    NotEq,
    Gt,
    GtEq,
    Ls,
    LsEq,
    OrOr,
    AndAnd,

    BinOp(BinOpTokenKind),
    BinOpAssign(BinOpTokenKind),

    OpeningDelim(DelimTokenKind),
    ClosingDelim(DelimTokenKind),

    Semicolon,
    Colon,
    Inheritance,
    ColonColon,
    Comma,
    Dot,
    Arrow,

    PlusPlus,
    MinusMinus,

    Num,
    Float,
    Ident,
    True,
    False,
    Null,
    StringLiteral,

    Keyword(KeywordTokenKind),

    EndOfFile,
}

pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: CharSpan,
    pub string: &'a str,
}

pub struct Lexer<'a> {
    current_byte: usize,
    source: &'a str,
    tokens: Vec<Token<'a>>,
    bytes: &'a [u8],
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            current_byte: 0,
            source,
            tokens: vec![],
            bytes: source.as_bytes(),
        }
    }

    fn scan_token(&mut self) -> Result<(), LexerError> {
        while !self.is_at_end() && char::from(self.peek()).is_whitespace() {
            self.advance();
        }
        if self.is_at_end() {
            return Ok(());
        }

        let base = self.current_byte;

        match self.advance() {
            b'+' => Ok(self.pick_3(base, b'=', b'+', BinOpAssign(Plus), PlusPlus, BinOp(Plus))),
            b'-' => {
                if self.peek() == b'>' {
                    self.advance();
                    Ok(self.add(base, Arrow))
                } else {
                    Ok(self.pick_3(
                        base,
                        b'=',
                        b'-',
                        BinOpAssign(Minus),
                        MinusMinus,
                        BinOp(Minus),
                    ))
                }
            }
            b'*' => Ok(self.pick_2(base, b'=', BinOpAssign(Star), BinOp(Star))),
            b'/' => Ok(self.pick_2(base, b'=', BinOpAssign(Slash), BinOp(Slash))),
            b'=' => Ok(self.pick_2(base, b'=', EqEq, Eq)),
            b'!' => Ok(self.pick_2(base, b'=', NotEq, Not)),
            b'>' => Ok(self.pick_2(base, b'=', GtEq, Gt)),
            b'<' => Ok(self.pick_2(base, b'=', LsEq, Ls)),
            b'|' => self.advance_if(b'|', base, OrOr),
            b'&' => self.advance_if(b'&', base, AndAnd),
            b'(' => Ok(self.add(base, OpeningDelim(Paren))),
            b')' => Ok(self.add(base, ClosingDelim(Paren))),
            b'[' => Ok(self.add(base, OpeningDelim(SBracket))),
            b']' => Ok(self.add(base, ClosingDelim(SBracket))),
            b'{' => Ok(self.add(base, OpeningDelim(CBracket))),
            b'}' => Ok(self.add(base, ClosingDelim(CBracket))),
            b';' => Ok(self.add(base, Semicolon)),
            b':' => Ok(self.pick_2(base, b':', ColonColon, Colon)),
            b',' => Ok(self.add(base, Comma)),
            b'.' => Ok(self.add(base, Dot)),
            b'"' => {
                macro_rules! err_if_at_end {
                    () => {
                        if self.is_at_end() {
                            return Err(LexerError {
                                kind: LexerErrorKind::UnterminatedString,
                                pos: base,
                            });
                        }
                    };
                }
                err_if_at_end!();
                while self.peek() != b'"' {
                    self.advance();
                    err_if_at_end!();
                }
                self.advance();
                Ok(self.add(base, StringLiteral))
            }
            c if char::from(c).is_ascii_digit() => {
                while !self.is_at_end() && char::from(self.peek()).is_ascii_digit() {
                    self.advance();
                }

                if self.is_at_end() {
                    self.add(base, Num);
                    return Ok(());
                }
                if self.peek() != b'.' {
                    self.add(base, Num);
                    return Ok(());
                }
                self.advance();
                while !self.is_at_end() && char::from(self.peek()).is_ascii_digit() {
                    self.advance();
                }
                self.add(base, Float);
                Ok(())
            }
            c if Lexer::is_valid_ident(c, true) => {
                while !self.is_at_end() && Lexer::is_valid_ident(self.peek(), false) {
                    self.advance();
                }

                let result = &self.source[base..self.current_byte];
                match result {
                    "pub" => self.add(base, Keyword(Pub)),
                    "priv" => self.add(base, Keyword(Priv)),
                    "abstract" => self.add(base, Keyword(Abstract)),
                    "static" => self.add(base, Keyword(Static)),
                    "native" => self.add(base, Keyword(Native)),
                    "class" => self.add(base, Keyword(Class)),
                    "inter" => self.add(base, Keyword(Inter)),
                    "enum" => self.add(base, Keyword(Enum)),
                    "impl" => self.add(base, Keyword(Impl)),
                    "fn" => self.add(base, Keyword(Fn)),
                    "mod" => self.add(base, Keyword(Mod)),
                    "use" => self.add(base, Keyword(Use)),
                    "if" => self.add(base, Keyword(If)),
                    "else" => self.add(base, Keyword(Else)),
                    "while" => self.add(base, Keyword(While)),
                    "match" => self.add(base, Keyword(Match)),
                    "loop" => self.add(base, Keyword(Loop)),
                    "for" => self.add(base, Keyword(For)),
                    "let" => self.add(base, Keyword(Let)),

                    "true" => self.add(base, True),
                    "false" => self.add(base, False),
                    "null" => self.add(base, Null),
                    _ => self.add(base, Ident),
                }

                Ok(())
            }
            _ => Err(LexerError {
                kind: LexerErrorKind::UnrecognizedCharacter,
                pos: base,
            }),
        }
    }

    fn is_valid_ident(c: u8, first: bool) -> bool {
        c == b'_'
            || if first {
                char::from(c).is_ascii_alphabetic()
            } else {
                char::from(c).is_ascii_alphanumeric()
            }
    }

    fn peek(&self) -> u8 {
        self.bytes[self.current_byte]
    }

    fn advance(&mut self) -> u8 {
        let r = self.peek();
        self.current_byte += 1;
        r
    }

    fn advance_if(
        &mut self,
        condition: u8,
        base: usize,
        kind: TokenKind,
    ) -> Result<(), LexerError> {
        if self.peek() == condition {
            self.advance();
            self.add(base, kind);
            Ok(())
        } else {
            Err(LexerError {
                kind: LexerErrorKind::UnrecognizedCharacter,
                pos: self.current_byte,
            })
        }
    }

    fn add(&mut self, base: usize, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            span: CharSpan {
                base,
                len: self.current_byte - base,
            },
            string: &self.source[base..self.current_byte],
        })
    }

    fn pick_2(&mut self, base: usize, condition: u8, ok: TokenKind, not_ok: TokenKind) {
        let kind = if self.peek() == condition {
            self.advance();
            ok
        } else {
            not_ok
        };
        self.tokens.push(Token {
            kind,
            span: CharSpan {
                base,
                len: self.current_byte - base,
            },
            string: &self.source[base..self.current_byte],
        })
    }

    fn pick_3(
        &mut self,
        base: usize,
        condition1: u8,
        condition2: u8,
        op1: TokenKind,
        op2: TokenKind,
        op3: TokenKind,
    ) {
        let kind = if self.peek() == condition1 {
            self.advance();
            op1
        } else {
            if self.peek() == condition2 {
                self.advance();
                op2
            } else {
                op3
            }
        };
        self.tokens.push(Token {
            kind,
            span: CharSpan {
                base,
                len: self.current_byte - base,
            },
            string: &self.source[base..self.current_byte],
        })
    }

    fn is_at_end(&self) -> bool {
        self.current_byte >= self.bytes.len()
    }

    pub fn lex(mut self) -> Result<Vec<Token<'a>>, LexerError> {
        while !self.is_at_end() {
            self.scan_token()?;
        }
        self.tokens.push(Token {
            kind: EndOfFile,
            span: CharSpan {
                base: self.current_byte,
                len: 0,
            },
            string: "",
        });
        Ok(self.tokens)
    }
}

#[derive(Debug)]
pub enum LexerErrorKind {
    UnrecognizedCharacter,
    UnterminatedString,
}

#[derive(Debug)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub pos: usize,
}
