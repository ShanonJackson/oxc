#![expect(clippy::redundant_pub_crate)]

use std::{ptr, slice, str};

use oxc_data_structures::code_buffer::{DEFAULT_INDENT_WIDTH, IndentChar};

const TAIL_CAPACITY: usize = 64;

pub(crate) struct FastBuffer {
    buf: Vec<u8>,
    len: usize,
    indent_char: IndentChar,
    indent_width: usize,
    tail: [u8; TAIL_CAPACITY],
    tail_len: usize,
}

impl Default for FastBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl FastBuffer {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            len: 0,
            indent_char: IndentChar::default(),
            indent_width: DEFAULT_INDENT_WIDTH,
            tail: [0; TAIL_CAPACITY],
            tail_len: 0,
        }
    }

    pub fn with_indent(indent_char: IndentChar, indent_width: usize) -> Self {
        Self {
            buf: Vec::new(),
            len: 0,
            indent_char,
            indent_width,
            tail: [0; TAIL_CAPACITY],
            tail_len: 0,
        }
    }

    pub fn start_emission(
        &mut self,
        capacity: usize,
        indent_char: IndentChar,
        indent_width: usize,
    ) {
        let aligned_capacity =
            if capacity == 0 { 64 } else { capacity.next_power_of_two().max(64) };
        if self.buf.capacity() < aligned_capacity {
            self.buf = Vec::with_capacity(aligned_capacity);
        }
        self.len = 0;
        self.indent_char = indent_char;
        self.indent_width = indent_width;
        self.tail_len = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    #[expect(dead_code)]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    #[inline]
    pub fn last_byte(&self) -> Option<u8> {
        if self.tail_len == 0 { None } else { Some(self.tail[self.tail_len - 1]) }
    }

    #[inline]
    pub fn peek_nth_byte_back(&self, n: usize) -> Option<u8> {
        if n < self.tail_len {
            Some(self.tail[self.tail_len - 1 - n])
        } else if n < self.len {
            // SAFETY: `n < self.len`, so pointer is in bounds
            unsafe { Some(*self.buf.as_ptr().add(self.len - 1 - n)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn last_char(&self) -> Option<char> {
        if self.tail_len == 0 {
            return None;
        }
        decode_last_char(&self.tail[..self.tail_len]).or_else(|| {
            if self.len == 0 {
                None
            } else {
                // SAFETY: `self.len` bytes were written.
                let slice = unsafe { slice::from_raw_parts(self.buf.as_ptr(), self.len) };
                decode_last_char(slice)
            }
        })
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: `self.len` bytes were initialized via `write_bytes`.
        unsafe { slice::from_raw_parts(self.buf.as_ptr(), self.len) }
    }

    pub fn into_string(mut self) -> String {
        unsafe { self.buf.set_len(self.len) };
        // SAFETY: all writes ensure UTF-8 correctness.
        unsafe { String::from_utf8_unchecked(std::mem::take(&mut self.buf)) }
    }

    #[inline(always)]
    pub fn print_ascii_byte(&mut self, byte: u8) {
        assert!(byte.is_ascii(), "byte {byte} is not ASCII");
        unsafe { self.print_byte_unchecked(byte) };
    }

    #[inline(always)]
    pub unsafe fn print_byte_unchecked(&mut self, byte: u8) {
        self.write_bytes(slice::from_ref(&byte));
    }

    #[inline]
    #[expect(dead_code)]
    pub fn print_char(&mut self, ch: char) {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        self.write_bytes(s.as_bytes());
    }

    #[inline(always)]
    pub fn print_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    #[inline]
    #[expect(dead_code)]
    pub fn print_ascii_bytes<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        let iter = bytes.into_iter();
        let hint = iter.size_hint();
        let additional = hint.1.unwrap_or(hint.0);
        if additional > 0 {
            let required = self.len.checked_add(additional).expect("buffer length overflow");
            if required > self.buf.capacity() {
                let target = required.next_power_of_two().max(64);
                self.buf.reserve(target);
            }
        }
        for byte in iter {
            self.print_ascii_byte(byte);
        }
    }

    pub unsafe fn print_bytes_unchecked(&mut self, bytes: &[u8]) {
        self.write_bytes(bytes);
    }

    pub fn print_indent(&mut self, depth: usize) {
        if depth == 0 {
            return;
        }
        let count = self.indent_width * depth;
        if count == 0 {
            return;
        }
        let byte = self.indent_char as u8;
        let chunk = [byte; 32];
        let mut remaining = count;
        while remaining >= chunk.len() {
            self.write_bytes(&chunk);
            remaining -= chunk.len();
        }
        if remaining > 0 {
            self.write_bytes(&chunk[..remaining]);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
        self.tail_len = 0;
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.update_tail(bytes);
        let new_len = self.len.checked_add(bytes.len()).expect("buffer length overflow");
        if new_len > self.buf.capacity() {
            let target = new_len.next_power_of_two().max(64);
            let additional = target - self.buf.len();
            self.buf.reserve(additional);
        }
        unsafe {
            let dst = self.buf.as_mut_ptr().add(self.len);
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
        }
        self.len = new_len;
    }

    fn update_tail(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        if bytes.len() >= TAIL_CAPACITY {
            let start = bytes.len() - TAIL_CAPACITY;
            self.tail.copy_from_slice(&bytes[start..]);
            self.tail_len = TAIL_CAPACITY;
            return;
        }

        let required = self.tail_len + bytes.len();
        if required > TAIL_CAPACITY {
            let drop = required - TAIL_CAPACITY;
            if drop >= self.tail_len {
                self.tail_len = 0;
            } else {
                self.tail.copy_within(drop..self.tail_len, 0);
                self.tail_len -= drop;
            }
        }
        let start = self.tail_len;
        let end = start + bytes.len();
        self.tail[start..end].copy_from_slice(bytes);
        self.tail_len = end;
    }
}

fn decode_last_char(bytes: &[u8]) -> Option<char> {
    if bytes.is_empty() {
        return None;
    }
    let mut index = bytes.len();
    while index > 0 {
        index -= 1;
        let byte = bytes[index];
        if byte & 0b1100_0000 != 0b1000_0000 {
            // SAFETY: Slice is guaranteed UTF-8 by construction.
            let slice = unsafe { bytes.get_unchecked(index..) };
            return str::from_utf8(slice).ok().and_then(|s| s.chars().next());
        }
    }
    None
}
