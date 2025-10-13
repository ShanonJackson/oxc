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
    tail_head: usize,
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
            tail_head: 0,
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
            tail_head: 0,
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
            let additional = aligned_capacity.saturating_sub(self.buf.len());
            self.buf.reserve(additional);
        }
        unsafe {
            self.buf.set_len(0);
        }
        self.len = 0;
        self.indent_char = indent_char;
        self.indent_width = indent_width;
        self.tail_len = 0;
        self.tail_head = 0;
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
        if self.tail_len == 0 { None } else { Some(self.tail_byte_from_back(0)) }
    }

    #[inline]
    pub fn peek_nth_byte_back(&self, n: usize) -> Option<u8> {
        if n < self.tail_len {
            Some(self.tail_byte_from_back(n))
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

        let mut buf = [0u8; 4];
        let mut filled = 0;
        for offset in 0..self.tail_len.min(4) {
            let byte = self.tail_byte_from_back(offset);
            buf[3 - offset] = byte;
            filled += 1;
            if byte & 0b1100_0000 != 0b1000_0000 {
                let start = 4 - filled;
                return str::from_utf8(&buf[start..]).ok().and_then(|s| s.chars().next());
            }
        }

        if self.len <= self.tail_len {
            return None;
        }

        let mut offset = self.tail_len;
        while filled < 4 && offset < self.len {
            let byte = unsafe { *self.buf.as_ptr().add(self.len - 1 - offset) };
            buf[3 - filled] = byte;
            filled += 1;
            if byte & 0b1100_0000 != 0b1000_0000 {
                let start = 4 - filled;
                return str::from_utf8(&buf[start..]).ok().and_then(|s| s.chars().next());
            }
            offset += 1;
        }

        None
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
        debug_assert!(byte.is_ascii(), "byte {byte} is not ASCII");
        self.write_single_byte(byte);
    }

    #[inline(always)]
    pub unsafe fn print_byte_unchecked(&mut self, byte: u8) {
        self.write_single_byte(byte);
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
        self.write_repeat_byte(byte, count);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
        self.tail_len = 0;
        self.tail_head = 0;
        unsafe {
            self.buf.set_len(0);
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        match bytes.len() {
            0 => return,
            1 => {
                self.write_single_byte(bytes[0]);
                return;
            }
            _ => {}
        }
        debug_assert_eq!(self.buf.len(), self.len);
        self.update_tail(bytes);
        let new_len = self.len.checked_add(bytes.len()).expect("buffer length overflow");
        if new_len > self.buf.capacity() {
            let target = new_len.next_power_of_two().max(64);
            let additional = target - self.buf.len();
            self.buf.reserve_exact(additional);
        }
        debug_assert!(
            self.buf.capacity() >= new_len,
            "fast buffer capacity {} shorter than required {} (len {} bytes)",
            self.buf.capacity(),
            new_len,
            bytes.len()
        );
        unsafe {
            let dst = self.buf.as_mut_ptr().add(self.len);
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
        }
        self.len = new_len;
        unsafe {
            self.buf.set_len(new_len);
        }
    }

    #[inline(always)]
    fn write_single_byte(&mut self, byte: u8) {
        debug_assert_eq!(self.buf.len(), self.len);
        self.push_tail_byte(byte);
        let new_len =
            self.len.checked_add(1).expect("fast buffer length overflow while writing byte");
        if new_len > self.buf.capacity() {
            let target = new_len.next_power_of_two().max(64);
            let additional = target - self.buf.len();
            self.buf.reserve_exact(additional);
        }
        unsafe {
            let dst = self.buf.as_mut_ptr().add(self.len);
            ptr::write(dst, byte);
            self.len = new_len;
            self.buf.set_len(new_len);
        }
    }

    #[inline(always)]
    fn write_repeat_byte(&mut self, byte: u8, count: usize) {
        debug_assert_ne!(count, 0);
        debug_assert_eq!(self.buf.len(), self.len);

        self.update_tail_repeat(byte, count);

        let new_len = self.len.checked_add(count).expect("buffer length overflow");
        if new_len > self.buf.capacity() {
            let target = new_len.next_power_of_two().max(64);
            let additional = target - self.buf.len();
            self.buf.reserve_exact(additional);
        }

        unsafe {
            let dst = self.buf.as_mut_ptr().add(self.len);
            ptr::write_bytes(dst, byte, count);
            self.len = new_len;
            self.buf.set_len(new_len);
        }
    }

    #[inline(always)]
    fn push_tail_byte(&mut self, byte: u8) {
        let idx = self.tail_head;
        self.tail[idx] = byte;
        self.tail_head = (idx + 1) % TAIL_CAPACITY;
        if self.tail_len < TAIL_CAPACITY {
            self.tail_len += 1;
        }
    }

    fn update_tail(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        if bytes.len() >= TAIL_CAPACITY {
            let start = bytes.len() - TAIL_CAPACITY;
            self.tail.copy_from_slice(&bytes[start..]);
            self.tail_len = TAIL_CAPACITY;
            self.tail_head = 0;
            return;
        }

        for &byte in bytes {
            self.push_tail_byte(byte);
        }
    }

    fn update_tail_repeat(&mut self, byte: u8, mut count: usize) {
        if count >= TAIL_CAPACITY {
            self.tail.fill(byte);
            self.tail_len = TAIL_CAPACITY;
            self.tail_head = 0;
            count %= TAIL_CAPACITY;
            if count == 0 {
                return;
            }
        }

        for _ in 0..count {
            self.push_tail_byte(byte);
        }
    }

    #[inline]
    fn tail_byte_from_back(&self, n: usize) -> u8 {
        debug_assert!(n < self.tail_len);
        let base = self.tail_head + TAIL_CAPACITY - 1;
        let idx = base.wrapping_sub(n) % TAIL_CAPACITY;
        unsafe { *self.tail.get_unchecked(idx) }
    }
}
