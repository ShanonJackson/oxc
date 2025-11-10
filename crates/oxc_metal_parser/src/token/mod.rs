use crate::hot::{is_ident_continue_ascii, is_ident_start_ascii, Scanner};
use std::sync::OnceLock;

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
    #[inline(always)]
    pub fn new(scan: &'s mut Scanner<'s>) -> Self { Self { scan } }

    #[inline(always)]
    fn punct_lut() -> &'static [TokKind; 256] {
        static LUT: OnceLock<[TokKind; 256]> = OnceLock::new();
        LUT.get_or_init(|| {
            let mut t = [TokKind::Other; 256];
            t[b';' as usize] = TokKind::Semi;
            t[b'{' as usize] = TokKind::LBrace;
            t[b'}' as usize] = TokKind::RBrace;
            t[b'(' as usize] = TokKind::LParen;
            t[b')' as usize] = TokKind::RParen;
            t[b',' as usize] = TokKind::Comma;
            t[b'+' as usize] = TokKind::Plus;
            t[b'-' as usize] = TokKind::Minus;
            t[b'*' as usize] = TokKind::Star;
            t[b'/' as usize] = TokKind::Slash;
            t[b'=' as usize] = TokKind::Assign;
            // Treat remaining single-byte punctuation as Other by default
            t
        })
    }

    #[inline(always)]
    pub fn next(&mut self) -> Tok {
        // Proven path: skip whitespace & comments via scanner helper
        self.skip_ws_and_comments();
        let start = self.scan.idx as u32;
        if self.scan.is_eof() { return Tok { kind: TokKind::Eof, start, end: start } }

        // Single-load 32B window (scalar view) for first-byte decisions
        let raw = self.scan.raw();
        let i = self.scan.idx;
        let b0 = unsafe { *raw.get_unchecked(i) };

        // Fast-path single-byte punctuation
        let pk = unsafe { *Self::punct_lut().get_unchecked(b0 as usize) };
        if !matches!(pk, TokKind::Other) {
            // SPOB: consume one byte
            self.scan.advance_by(1);
            return Tok { kind: pk, start, end: self.scan.idx as u32 };
        }

        // Multi-byte or class-based
        let kind = match b0 {
            b'"' | b'\'' => {
                self.scan.advance_by(1);
                self.scan.scan_string_simple(b0);
                TokKind::Str
            }
            b'0'..=b'9' => {
                // Consume first digit
                self.scan.advance_by(1);
                // Fast local scan within this 32B neighborhood; fall back to kernel if needed
                let raw = self.scan.raw();
                let mut j = self.scan.idx;
                // Scan up to remaining bytes in this 32B window
                let limit = (j + 31).min(raw.len());
                while j < limit {
                    let c = unsafe { *raw.get_unchecked(j) };
                    if (b'0'..=b'9').contains(&c) || c == b'.' { j += 1; } else { break; }
                }
                // Advance locally
                if j > self.scan.idx { self.scan.advance_by(j - self.scan.idx); }
                // If token likely extends beyond local window, finish with kernel
                if j == limit && j < raw.len() {
                    self.scan.advance_digits_or_dot();
                }
                TokKind::Num
            }
            _ if is_ident_start_ascii(b0) => {
                // Consume start
                self.scan.advance_by(1);
                // Fast local ident-continue within this 32B neighborhood
                let raw = self.scan.raw();
                let mut j = self.scan.idx;
                let limit = (j + 31).min(raw.len()); // we already advanced 1
                while j < limit {
                    let c = unsafe { *raw.get_unchecked(j) };
                    if is_ident_continue_ascii(c) { j += 1; } else { break; }
                }
                if j > self.scan.idx { self.scan.advance_by(j - self.scan.idx); }
                if j == limit && j < raw.len() {
                    self.scan.advance_ident_continue();
                }
                let s = self.scan.slice_from(start as usize);
                if s == b"let" { TokKind::KwLet } else { TokKind::Ident }
            }
            _ => { self.scan.advance_by(1); TokKind::Other }
        };

        Tok { kind, start, end: self.scan.idx as u32 }
    }

    #[inline(always)]
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
