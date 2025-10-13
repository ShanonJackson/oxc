use std::{ptr, slice, str};

use oxc_data_structures::code_buffer::{DEFAULT_INDENT_WIDTH, IndentChar};

const MIN_CAPACITY: usize = 64;
const ALLOCATION_ALIGNMENT: usize = 256;

#[cfg(debug_assertions)]
#[derive(Default, Clone, Copy)]
struct BufferMetrics {
    reallocations: usize,
    writes: usize,
    written_bytes: usize,
}

pub(crate) struct FastBuffer {
    buf: Vec<u8>,
    len: usize,
    indent_char: IndentChar,
    indent_width: usize,
    last_byte: Option<u8>,
    last_char: Option<char>,
    #[cfg(debug_assertions)]
    metrics: BufferMetrics,
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
            last_byte: None,
            last_char: None,
            #[cfg(debug_assertions)]
            metrics: BufferMetrics::default(),
        }
    }

    pub fn with_indent(indent_char: IndentChar, indent_width: usize) -> Self {
        Self {
            buf: Vec::new(),
            len: 0,
            indent_char,
            indent_width,
            last_byte: None,
            last_char: None,
            #[cfg(debug_assertions)]
            metrics: BufferMetrics::default(),
        }
    }

    pub fn start_emission(
        &mut self,
        capacity: usize,
        indent_char: IndentChar,
        indent_width: usize,
    ) {
        let aligned_capacity = Self::align_capacity(capacity);
        self.buf = Vec::with_capacity(aligned_capacity);
        self.len = 0;
        self.indent_char = indent_char;
        self.indent_width = indent_width;
        self.last_byte = None;
        self.last_char = None;
        #[cfg(debug_assertions)]
        {
            self.metrics = BufferMetrics::default();
        }
    }

    #[inline]
    pub(crate) fn align_capacity(capacity: usize) -> usize {
        let requested = capacity.max(MIN_CAPACITY);
        let rem = requested % ALLOCATION_ALIGNMENT;
        if rem == 0 { requested } else { requested + (ALLOCATION_ALIGNMENT - rem) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    #[inline]
    pub fn last_byte(&self) -> Option<u8> {
        self.last_byte
    }

    #[inline]
    pub fn peek_nth_byte_back(&self, n: usize) -> Option<u8> {
        if n < self.len {
            // SAFETY: `n < len`, so pointer is in bounds
            unsafe { Some(*self.buf.as_ptr().add(self.len - 1 - n)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn last_char(&self) -> Option<char> {
        if let Some(ch) = self.last_char {
            return Some(ch);
        }
        if self.len == 0 {
            return None;
        }
        // SAFETY: `len` bytes were written.
        let slice = unsafe { slice::from_raw_parts(self.buf.as_ptr(), self.len) };
        decode_last_char(slice)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
                let target = Self::align_capacity(required);
                self.buf.reserve(target - self.buf.capacity());
                #[cfg(debug_assertions)]
                {
                    self.metrics.reallocations += 1;
                }
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
        self.last_byte = None;
        self.last_char = None;
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.last_byte = bytes.last().copied();
        #[cfg(debug_assertions)]
        {
            self.metrics.writes += 1;
            self.metrics.written_bytes += bytes.len();
        }
        let new_len = self.len.checked_add(bytes.len()).expect("buffer length overflow");
        if new_len > self.buf.capacity() {
            let target = Self::align_capacity(new_len);
            self.buf.reserve(target - self.buf.capacity());
            #[cfg(debug_assertions)]
            {
                self.metrics.reallocations += 1;
            }
        }
        unsafe {
            let dst = self.buf.as_mut_ptr().add(self.len);
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
        }
        self.len = new_len;
        let mut last_char = decode_last_char(bytes);
        if last_char.is_none() {
            // SAFETY: `new_len` bytes were written
            let slice = unsafe { slice::from_raw_parts(self.buf.as_ptr(), new_len) };
            last_char = decode_last_char(slice);
        }
        self.last_char = last_char;
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
