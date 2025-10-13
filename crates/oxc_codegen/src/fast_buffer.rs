#![expect(clippy::redundant_pub_crate)]

use std::ptr;

use oxc_data_structures::code_buffer::{DEFAULT_INDENT_WIDTH, IndentChar};

use crate::utf8::{decode_utf8_from_lead, utf8_lead_byte_width};

pub(crate) struct FastBuffer {
    buf: Vec<u8>,
    base_ptr: *mut u8,
    write_ptr: *mut u8,
    capacity: usize,
    len_synced: bool,
    len: usize,
    indent_char: IndentChar,
    indent_width: usize,
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
            base_ptr: ptr::null_mut(),
            write_ptr: ptr::null_mut(),
            capacity: 0,
            len_synced: true,
            len: 0,
            indent_char: IndentChar::default(),
            indent_width: DEFAULT_INDENT_WIDTH,
        }
    }

    pub fn with_indent(indent_char: IndentChar, indent_width: usize) -> Self {
        Self {
            buf: Vec::new(),
            base_ptr: ptr::null_mut(),
            write_ptr: ptr::null_mut(),
            capacity: 0,
            len_synced: true,
            len: 0,
            indent_char,
            indent_width,
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
        self.sync_len();
        if self.capacity < aligned_capacity {
            let additional = aligned_capacity - self.len;
            self.buf.reserve(additional);
            self.refresh_pointers();
        }
        self.buf.clear();
        self.len = 0;
        self.len_synced = true;
        self.capacity = self.buf.capacity();
        self.base_ptr = self.buf.as_mut_ptr();
        self.write_ptr = self.base_ptr;
        self.indent_char = indent_char;
        self.indent_width = indent_width;
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
        if self.len == 0 {
            None
        } else {
            // SAFETY: `self.len > 0` so `self.write_ptr.offset(-1)` is in bounds.
            unsafe { Some(*self.write_ptr.sub(1)) }
        }
    }

    #[inline]
    pub fn peek_nth_byte_back(&self, n: usize) -> Option<u8> {
        if n < self.len {
            // SAFETY: `n < self.len`, so pointer is in bounds.
            unsafe { Some(*self.write_ptr.sub(n + 1)) }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn last_char(&self) -> Option<char> {
        if self.len == 0 {
            return None;
        }

        unsafe {
            let ptr = self.base_ptr as *const u8;
            let mut index = self.len - 1;
            let mut byte = *ptr.add(index);

            if byte & 0x80 == 0 {
                return Some(byte as char);
            }

            let mut continuation_bytes = 0usize;
            while byte & 0xC0 == 0x80 {
                continuation_bytes += 1;
                if index == 0 {
                    return None;
                }
                index -= 1;
                byte = *ptr.add(index);
            }

            let width = utf8_lead_byte_width(byte);
            debug_assert!(
                width > continuation_bytes,
                "utf-8 width {width} shorter than continuation count {continuation_bytes}",
            );
            if width - 1 != continuation_bytes {
                return None;
            }
            let slice_ptr = ptr.add(index);

            Some(decode_utf8_from_lead(byte, slice_ptr.add(1), width))
        }
    }

    pub fn into_string(mut self) -> String {
        self.sync_len();
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
            self.ensure_capacity(additional);
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
        self.sync_len();
        self.buf.clear();
        self.len = 0;
        self.capacity = self.buf.capacity();
        self.base_ptr = self.buf.as_mut_ptr();
        self.write_ptr = self.base_ptr;
        self.len_synced = true;
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
        let len = bytes.len();
        self.ensure_capacity(len);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), self.write_ptr, len);
            self.write_ptr = self.write_ptr.add(len);
        }
        self.len = self.len.checked_add(len).expect("buffer length overflow");
        self.len_synced = false;
    }

    #[inline(always)]
    fn write_single_byte(&mut self, byte: u8) {
        self.ensure_capacity(1);
        unsafe {
            ptr::write(self.write_ptr, byte);
            self.write_ptr = self.write_ptr.add(1);
        }
        self.len = self.len.checked_add(1).expect("fast buffer length overflow while writing byte");
        self.len_synced = false;
    }

    #[inline(always)]
    fn write_repeat_byte(&mut self, byte: u8, count: usize) {
        debug_assert_ne!(count, 0);

        self.ensure_capacity(count);
        unsafe {
            ptr::write_bytes(self.write_ptr, byte, count);
            self.write_ptr = self.write_ptr.add(count);
        }
        self.len = self.len.checked_add(count).expect("buffer length overflow");
        self.len_synced = false;
    }

    #[inline(always)]
    fn ensure_capacity(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }
        let required = self.len.checked_add(additional).expect("buffer length overflow");
        if required > self.capacity {
            let target = required.next_power_of_two().max(64);
            self.sync_len();
            let additional = target - self.len;
            self.buf.reserve(additional);
            self.refresh_pointers();
        }
    }

    #[inline(always)]
    fn sync_len(&mut self) {
        if !self.len_synced {
            unsafe { self.buf.set_len(self.len) };
            self.len_synced = true;
        }
    }

    #[inline(always)]
    fn refresh_pointers(&mut self) {
        self.base_ptr = self.buf.as_mut_ptr();
        self.capacity = self.buf.capacity();
        unsafe {
            self.write_ptr = self.base_ptr.add(self.len);
        }
        self.len_synced = true;
    }
}
