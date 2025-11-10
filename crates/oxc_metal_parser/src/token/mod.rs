use crate::hot::{is_ident_continue_ascii, is_ident_start_ascii, Scanner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokKind {
    Eof, Semi, LBrace, RBrace, LParen, RParen, Comma, Assign, Plus, Minus, Star, Slash,
    Ident, Num, Str, KwLet,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tok { pub kind: TokKind, pub start: u32, pub end: u32 }

pub struct Tokenizer<'s> {
    scan: &'s mut Scanner<'s>,
}

impl<'s> Tokenizer<'s> {
    pub fn new(scan: &'s mut Scanner<'s>) -> Self { Self { scan } }

    #[inline(always)]
    fn map_punct(b: u8) -> Option<TokKind> {
        match b {
            b';' => Some(TokKind::Semi),
            b'{' => Some(TokKind::LBrace),
            b'}' => Some(TokKind::RBrace),
            b'(' => Some(TokKind::LParen),
            b')' => Some(TokKind::RParen),
            b',' => Some(TokKind::Comma),
            b'+' => Some(TokKind::Plus),
            b'-' => Some(TokKind::Minus),
            b'*' => Some(TokKind::Star),
            b'/' => Some(TokKind::Slash),
            b'=' => Some(TokKind::Assign),
            _ => None,
        }
    }

    #[inline]
    pub fn next(&mut self) -> Tok {
        self.skip_ws_and_comments();
        let start = self.scan.idx as u32;
        if self.scan.is_eof() { return Tok { kind: TokKind::Eof, start, end: start } }
        let b = self.scan.peek();
        // fast-path single-byte punctuation
        if let Some(kind) = Self::map_punct(b) {
            self.scan.bump();
            return Tok { kind, start, end: self.scan.idx as u32 };
        }
        // multi-byte or class-based
        let tok = match b {
            b'"' | b'\'' => {
                let quote = b;
                self.scan.bump();
                self.scan.scan_string_simple(quote);
                TokKind::Str
            }
            b'0'..=b'9' => {
                self.scan.advance_digits_or_dot();
                TokKind::Num
            }
            _ if is_ident_start_ascii(b) => {
                self.scan.bump();
                self.scan.advance_ident_continue();
                let s = self.scan.slice_from(start as usize);
                if s == b"let" { TokKind::KwLet } else { TokKind::Ident }
            }
            _ => { self.scan.bump(); TokKind::Other }
        };
        Tok { kind: tok, start, end: self.scan.idx as u32 }
    }

    #[inline]
    fn skip_ws_and_comments(&mut self) {
        loop {
            self.scan.skip_ws();
            // line comment
            if self.scan.byte_at(self.scan.idx) == Some(b'/') && self.scan.byte_at(self.scan.idx + 1) == Some(b'/') {
                // consume until newline or EOF
                self.scan.advance_by(2);
                while let Some(c) = self.scan.byte_at(self.scan.idx) {
                    if c == b'\n' || c == b'\r' { break; }
                    self.scan.advance_by(1);
                }
                continue;
            }
            // block comment
            if self.scan.byte_at(self.scan.idx) == Some(b'/') && self.scan.byte_at(self.scan.idx + 1) == Some(b'*') {
                self.scan.advance_by(2);
                while let Some(c) = self.scan.byte_at(self.scan.idx) {
                    if c == b'*' && self.scan.byte_at(self.scan.idx + 1) == Some(b'/') {
                        self.scan.advance_by(2);
                        break;
                    }
                    self.scan.advance_by(1);
                }
                continue;
            }
            break;
        }
    }
}
