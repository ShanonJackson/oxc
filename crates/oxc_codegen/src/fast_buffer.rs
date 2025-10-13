#![expect(clippy::redundant_pub_crate)]

use std::{char, ptr, slice};

use oxc_data_structures::code_buffer::{DEFAULT_INDENT_WIDTH, IndentChar};

pub(crate) struct FastBuffer {
    buf: Vec<u8>,
    len: usize,
    indent_char: IndentChar,
    indent_width: usize,
}

#[inline(always)]
const fn utf8_lead_byte_width(byte: u8) -> usize {
    if byte & 0x80 == 0 {
        1
    } else if byte & 0xE0 == 0xC0 {
        2
    } else if byte & 0xF0 == 0xE0 {
        3
    } else {
        4
    }
}

#[inline(always)]
unsafe fn decode_two(lead: u8, cont: *const u8) -> char {
    let b2 = unsafe { *cont };
    debug_assert_eq!(lead & 0xE0, 0xC0);
    debug_assert_eq!(b2 & 0xC0, 0x80);
    let code = ((lead & 0x1F) as u32) << 6 | (b2 & 0x3F) as u32;
    debug_assert!(char::from_u32(code).is_some());
    unsafe { char::from_u32_unchecked(code) }
}

#[inline(always)]
unsafe fn decode_three(lead: u8, cont: *const u8) -> char {
    let b2 = unsafe { *cont };
    let cont1 = unsafe { cont.add(1) };
    let b3 = unsafe { *cont1 };
    debug_assert_eq!(lead & 0xF0, 0xE0);
    debug_assert_eq!(b2 & 0xC0, 0x80);
    debug_assert_eq!(b3 & 0xC0, 0x80);
    let code = ((lead & 0x0F) as u32) << 12 | ((b2 & 0x3F) as u32) << 6 | (b3 & 0x3F) as u32;
    debug_assert!(char::from_u32(code).is_some());
    unsafe { char::from_u32_unchecked(code) }
}

#[inline(always)]
unsafe fn decode_four(lead: u8, cont: *const u8) -> char {
    let b2 = unsafe { *cont };
    let cont1 = unsafe { cont.add(1) };
    let b3 = unsafe { *cont1 };
    let cont2 = unsafe { cont.add(2) };
    let b4 = unsafe { *cont2 };
    debug_assert_eq!(lead & 0xF8, 0xF0);
    debug_assert_eq!(b2 & 0xC0, 0x80);
    debug_assert_eq!(b3 & 0xC0, 0x80);
    debug_assert_eq!(b4 & 0xC0, 0x80);
    let code = ((lead & 0x07) as u32) << 18
        | ((b2 & 0x3F) as u32) << 12
        | ((b3 & 0x3F) as u32) << 6
        | (b4 & 0x3F) as u32;
    debug_assert!(char::from_u32(code).is_some());
    unsafe { char::from_u32_unchecked(code) }
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
        }
    }

    pub fn with_indent(indent_char: IndentChar, indent_width: usize) -> Self {
        Self { buf: Vec::new(), len: 0, indent_char, indent_width }
    }

    pub fn start_emission(
        &mut self,
        capacity: usize,
        indent_char: IndentChar,
        indent_width: usize,
    ) {
        let aligned_capacity =
            if capacity == 0 { 64 } else { capacity.next_power_of_two().max(64) };
        let current_capacity = self.buf.capacity();
        if current_capacity < aligned_capacity {
            self.buf.reserve(aligned_capacity);
        }
        self.buf.clear();
        self.len = 0;
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
            // SAFETY: `self.len > 0` so `self.len - 1` is in bounds.
            unsafe { Some(*self.buf.as_ptr().add(self.len - 1)) }
        }
    }

    #[inline]
    pub fn peek_nth_byte_back(&self, n: usize) -> Option<u8> {
        if n < self.len {
            // SAFETY: `n < self.len`, so pointer is in bounds.
            unsafe { Some(*self.buf.as_ptr().add(self.len - 1 - n)) }
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
            let ptr = self.buf.as_ptr();
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

            Some(match width {
                2 => decode_two(byte, slice_ptr.add(1)),
                3 => decode_three(byte, slice_ptr.add(1)),
                4 => decode_four(byte, slice_ptr.add(1)),
                _ => unreachable!("ASCII handled earlier"),
            })
        }
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
        let new_len = self.len.checked_add(bytes.len()).expect("buffer length overflow");
        if new_len > self.buf.capacity() {
            let target = new_len.next_power_of_two().max(64);
            self.buf.reserve(target);
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
    }

    #[inline(always)]
    fn write_single_byte(&mut self, byte: u8) {
        let new_len =
            self.len.checked_add(1).expect("fast buffer length overflow while writing byte");
        if new_len > self.buf.capacity() {
            let target = new_len.next_power_of_two().max(64);
            self.buf.reserve(target);
        }
        unsafe {
            let dst = self.buf.as_mut_ptr().add(self.len);
            ptr::write(dst, byte);
        }
        self.len = new_len;
    }

    #[inline(always)]
    fn write_repeat_byte(&mut self, byte: u8, count: usize) {
        debug_assert_ne!(count, 0);

        let new_len = self.len.checked_add(count).expect("buffer length overflow");
        if new_len > self.buf.capacity() {
            let target = new_len.next_power_of_two().max(64);
            self.buf.reserve(target);
        }

        unsafe {
            let dst = self.buf.as_mut_ptr().add(self.len);
            ptr::write_bytes(dst, byte, count);
        }
        self.len = new_len;
    }
}
