pub mod backend;
use core::mem::MaybeUninit;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
#[cfg(target_arch = "x86_64")]
use std::sync::OnceLock;

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn load256(ptr: *const __m256i) -> __m256i {
    if (ptr as usize) & 31 == 0 { unsafe { _mm256_load_si256(ptr) } } else { unsafe { _mm256_loadu_si256(ptr) } }
}

// Prefetch distance (bytes ahead). Overridable via METAL_PREFETCH, defaults to 128.
// Legacy: keep a global reader for printing-only contexts (examples), but kernels use per-Scanner prefetch.
static PREFETCH_PRINT: OnceLock<usize> = OnceLock::new();

#[inline]
pub fn prefetch_distance() -> usize {
    *PREFETCH_PRINT.get_or_init(|| {
        if let Ok(s) = std::env::var("METAL_PREFETCH") { s.parse::<usize>().unwrap_or(128).min(512) } else { 128 }
    })
}

#[inline]
fn heuristic_prefetch(src: &[u8]) -> usize {
    let n = src.len().min(8192);
    if n == 0 { return 128; }
    let mut ws = 0usize;
    for &b in &src[..n] {
        match b { b' ' | b'\t' | b'\n' | b'\r' => ws += 1, _ => {} }
    }
    let ratio = (ws as f64) / (n as f64);
    if ratio < 0.10 { 96 } else { 128 }
}

// One-time, very light calibrator to decide if AVX2 whitespace kernel wins on this CPU.
// Uses a synthetic whitespace buffer and compares skip_ws scalar vs avx2.
static WS_AVX2_DEFAULT: OnceLock<bool> = OnceLock::new();

#[inline]
pub fn ws_default_enabled() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        *WS_AVX2_DEFAULT.get_or_init(|| {
            // synthetic buffer ~8KiB of mixed whitespace
            let mut v = Vec::with_capacity(8192);
            for _ in 0..2048 { v.extend_from_slice(b" \t\r\n"); }
            // bench helper
            let bench = |use_avx2_ws: bool| -> std::time::Duration {
                let mut total = std::time::Duration::ZERO;
                for _ in 0..64 { // repeat to smooth noise
                    let mut s = Scanner {
                        src: &v,
                        idx: 0,
                        shadow: SpobShadow::default(),
                        backend: backend::Backend::Avx2,
                        use_avx2_ws,
                        use_avx2_digits: true,
                        use_avx2_ident: false,
                        use_avx2_string: true,
                        prefetch_ahead: 128,
                    };
                    let t0 = std::time::Instant::now();
                    s.skip_ws();
                    total += t0.elapsed();
                }
                total
            };
            let t_scalar = bench(false);
            let t_avx2 = bench(true);
            // prefer AVX2 if >= 5% faster
            t_avx2 < t_scalar.saturating_mul(95) / 100
        })
    }
    #[cfg(not(target_arch = "x86_64"))]
    { false }
}

/// SPOB debug shadow: asserts monotonic forward-only advancement.
#[derive(Default, Debug, Clone, Copy)]
pub struct SpobShadow {
    last_advanced: usize,
}

impl SpobShadow {
    #[inline]
    pub fn advance_to(&mut self, idx: usize) {
        debug_assert!(idx >= self.last_advanced, "SPOB violated: went from {} to {}", self.last_advanced, idx);
        self.last_advanced = idx;
    }
}

/// Minimal streaming scanner over a byte slice.
/// Current implementation is scalar; provides placeholders for SIMD/ASM later.
pub struct Scanner<'s> {
    src: &'s [u8],
    pub idx: usize,
    shadow: SpobShadow,
    pub backend: backend::Backend,
    use_avx2_ws: bool,
    use_avx2_digits: bool,
    use_avx2_ident: bool,
    use_avx2_string: bool,
    prefetch_ahead: usize,
}

impl<'s> Scanner<'s> {
    pub fn new(src: &'s [u8]) -> Self {
        let be = backend::detect();
        // Default enable AVX2 paths if backend supports
        let mut s = Self {
            src,
            idx: 0,
            shadow: SpobShadow::default(),
            backend: be,
            // Default mix based on A/B data: digits+string SIMD on, ws+ident SIMD off
            use_avx2_ws: false,
            use_avx2_digits: matches!(be, backend::Backend::Avx2 | backend::Backend::Avx512),
            use_avx2_ident: false,
            use_avx2_string: matches!(be, backend::Backend::Avx2 | backend::Backend::Avx512),
            prefetch_ahead: 128,
        };
        // Calibrate whitespace SIMD once per process on x86_64 with AVX2 backend
        if matches!(be, backend::Backend::Avx2 | backend::Backend::Avx512) {
            s.use_avx2_ws = ws_default_enabled();
        }
        // Prefetch: env override; otherwise fixed default 128 for stability across workloads
        s.prefetch_ahead = if let Ok(sv) = std::env::var("METAL_PREFETCH") {
            sv.parse::<usize>().unwrap_or(128).min(512)
        } else {
            128
        };
        // Env overrides to quickly A/B individual kernels: 1 enable, 0 disable
        if let Ok(v) = std::env::var("METAL_SIMD_WS") { s.use_avx2_ws = v == "1"; }
        if let Ok(v) = std::env::var("METAL_SIMD_DIGITS") { s.use_avx2_digits = v == "1"; }
        if let Ok(v) = std::env::var("METAL_SIMD_IDENT") { s.use_avx2_ident = v == "1"; }
        if let Ok(v) = std::env::var("METAL_SIMD_STRING") { s.use_avx2_string = v == "1"; }
        s
    }

    #[inline(always)]
    pub fn is_eof(&self) -> bool { self.idx >= self.src.len() }

    #[inline(always)]
    pub fn peek(&self) -> u8 { self.src.get(self.idx).copied().unwrap_or(0) }

    #[inline(always)]
    pub fn bump(&mut self) -> Option<u8> {
        let b = self.src.get(self.idx).copied();
        if b.is_some() {
            self.idx += 1;
            self.shadow.advance_to(self.idx);
        }
        b
    }

    #[inline(always)]
    pub fn slice_from(&self, start: usize) -> &'s [u8] { &self.src[start..self.idx] }

    #[inline(always)]
    pub fn byte_at(&self, i: usize) -> Option<u8> { self.src.get(i).copied() }

    #[inline(always)]
    pub fn raw(&self) -> &'s [u8] { self.src }

    /// Skip ASCII whitespace and line terminators (CR, LF, TAB, SPACE).
    #[inline(always)]
    pub fn skip_ws(&mut self) {
        if self.use_avx2_ws { unsafe { self.skip_ws_avx2() } } else { self.skip_ws_scalar() }
    }

    /// Try match an ASCII keyword at current position.
    #[inline(always)]
    pub fn eat_kw(&mut self, kw: &[u8]) -> bool {
        if self.src.get(self.idx..self.idx + kw.len()) == Some(kw) {
            // ensure not followed by ident char
            let next = self.src.get(self.idx + kw.len()).copied();
            if next.map(|c| is_ident_continue_ascii(c)).unwrap_or(false) {
                return false;
            }
            self.idx += kw.len();
            self.shadow.advance_to(self.idx);
            true
        } else { false }
    }

    #[inline(always)]
    pub fn advance_by(&mut self, n: usize) {
        self.idx += n;
        self.shadow.advance_to(self.idx);
    }

    /// Advance while bytes are '0'..'9' or '.'
    #[inline(always)]
    pub fn advance_digits_or_dot(&mut self) {
        if self.use_avx2_digits { unsafe { self.advance_digits_or_dot_avx2() } } else { self.advance_digits_or_dot_scalar() }
    }

    #[inline(always)]
    fn advance_digits_or_dot_scalar(&mut self) {
        while let Some(c) = self.byte_at(self.idx) {
            match c { b'0'..=b'9' | b'.' => self.advance_by(1), _ => break }
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn advance_digits_or_dot_avx2(&mut self) {
        #[cfg(target_arch = "x86_64")]
        {
            let len = self.src.len();
            // Hoisted constants for '.' and digit ranges
            let (dot, d0, d9) = {
                static DIGITS: OnceLock<(__m256i, __m256i, __m256i)> = OnceLock::new();
                *DIGITS.get_or_init(|| unsafe {
                    (
                        _mm256_set1_epi8(b'.' as i8),
                        _mm256_set1_epi8(b'0' as i8),
                        _mm256_set1_epi8(b'9' as i8),
                    )
                })
            };
            let z   = _mm256_setzero_si256();
            // Align to 32B boundary with a tiny scalar prologue
            if self.idx + 32 <= len {
                let base = self.src.as_ptr() as usize;
                let mut steps = 0u8;
                while ((base + self.idx) & 31) != 0 && self.idx < len && steps < 32 {
                    match self.byte_at(self.idx) { Some(b'0'..=b'9' | b'.') => self.advance_by(1), _ => return }
                    steps += 1;
                }
            }
            while self.idx + 32 <= len {
                let pf = self.prefetch_ahead;
                if self.idx + (pf + 64) < len {
                    unsafe { _mm_prefetch(self.src.as_ptr().add(self.idx + pf) as *const i8, 3); }
                }
                let ptr = unsafe { self.src.as_ptr().add(self.idx) as *const __m256i };
                let v = unsafe { load256(ptr) };
                // digits: v in ['0'..'9'] => (v >= '0') & (v <= '9')
                let ge_0 = _mm256_cmpeq_epi8(_mm256_subs_epu8(d0, v), z);
                let le_9 = _mm256_cmpeq_epi8(_mm256_subs_epu8(v, d9), z);
                let m_d  = _mm256_and_si256(ge_0, le_9);
                let m_dot = _mm256_cmpeq_epi8(v, dot);
                let m = _mm256_or_si256(m_d, m_dot);
                let mask = _mm256_movemask_epi8(m) as u32;
                if mask == 0xFFFF_FFFF {
                    if self.idx + 64 <= len {
                        let ptr2 = unsafe { self.src.as_ptr().add(self.idx + 32) as *const __m256i };
                        let v2 = unsafe { load256(ptr2) };
                        let ge_0b = _mm256_cmpeq_epi8(_mm256_subs_epu8(d0, v2), z);
                        let le_9b = _mm256_cmpeq_epi8(_mm256_subs_epu8(v2, d9), z);
                        let m_db  = _mm256_and_si256(ge_0b, le_9b);
                        let m_dotb = _mm256_cmpeq_epi8(v2, dot);
                        let mb = _mm256_or_si256(m_db, m_dotb);
                        let mask2 = _mm256_movemask_epi8(mb) as u32;
                        if mask2 == 0xFFFF_FFFF {
                            if self.idx + 96 <= len {
                                let ptr3 = unsafe { self.src.as_ptr().add(self.idx + 64) as *const __m256i };
                                let v3 = unsafe { load256(ptr3) };
                                let ge_0c = _mm256_cmpeq_epi8(_mm256_subs_epu8(d0, v3), z);
                                let le_9c = _mm256_cmpeq_epi8(_mm256_subs_epu8(v3, d9), z);
                                let m_dc  = _mm256_and_si256(ge_0c, le_9c);
                                let m_dotc = _mm256_cmpeq_epi8(v3, dot);
                                let mc = _mm256_or_si256(m_dc, m_dotc);
                                let mask3 = _mm256_movemask_epi8(mc) as u32;
                                if mask3 == 0xFFFF_FFFF { self.advance_by(96); continue; }
                                let not3 = (!mask3) & 0xFFFF_FFFF;
                                let tz3 = not3.trailing_zeros() as usize;
                                self.advance_by(64 + tz3);
                                return;
                            } else { self.advance_by(64); continue; }
                        }
                        let not2 = (!mask2) & 0xFFFF_FFFF;
                        let tz2 = not2.trailing_zeros() as usize;
                        self.advance_by(32 + tz2);
                        return;
                    } else {
                        self.advance_by(32);
                        continue;
                    }
                }
                let not_mask = (!mask) & 0xFFFF_FFFF;
                let tz = not_mask.trailing_zeros() as usize;
                self.advance_by(tz);
                return;
            }
            self.advance_digits_or_dot_scalar();
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.advance_digits_or_dot_scalar();
        }
    }

    /// Advance while bytes are ASCII identifier-continue: [A-Za-z_$0-9]
    #[inline]
    pub fn advance_ident_continue(&mut self) {
        if self.use_avx2_ident { unsafe { self.advance_ident_continue_avx2() } } else { self.advance_ident_continue_scalar() }
    }

    #[inline(always)]
    fn advance_ident_continue_scalar(&mut self) {
        while let Some(c) = self.byte_at(self.idx) {
            if is_ident_continue_ascii(c) { self.advance_by(1); } else { break; }
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn advance_ident_continue_avx2(&mut self) {
        #[cfg(target_arch = "x86_64")]
        {
            let len = self.src.len();
            let consts = AVX2_CLASS_CONSTS.get_or_init(Avx2ClassConsts::new);

            // Align to 32B boundary with a tiny scalar prologue for long runs
            if self.idx + 32 <= len {
                let base = self.src.as_ptr() as usize;
                let mut steps = 0u8;
                while ((base + self.idx) & 31) != 0 && self.idx < len && steps < 32 {
                    if let Some(c) = self.byte_at(self.idx) { if is_ident_continue_ascii(c) { self.advance_by(1); steps += 1; continue; } }
                    return;
                }
            }

            while self.idx + 32 <= len {
                let pf = self.prefetch_ahead;
                if self.idx + (pf + 64) < len { unsafe { _mm_prefetch(self.src.as_ptr().add(self.idx + pf) as *const i8, 3); } }
                let ptr = unsafe { self.src.as_ptr().add(self.idx) as *const __m256i };
                let v = unsafe { load256(ptr) };
                let cm = unsafe { classify32_avx2(v, consts) };
                let mask = cm.ident;
                if mask == 0xFFFF_FFFF {
                    if self.idx + 64 <= len {
                        let ptr2 = unsafe { self.src.as_ptr().add(self.idx + 32) as *const __m256i };
                        let v2 = unsafe { load256(ptr2) };
                        let cm2 = unsafe { classify32_avx2(v2, consts) };
                        let mask2 = cm2.ident;
                        if mask2 == 0xFFFF_FFFF {
                            if self.idx + 96 <= len {
                                let ptr3 = unsafe { self.src.as_ptr().add(self.idx + 64) as *const __m256i };
                                let v3 = unsafe { load256(ptr3) };
                                let cm3 = unsafe { classify32_avx2(v3, consts) };
                                let mask3 = cm3.ident;
                                if mask3 == 0xFFFF_FFFF { self.advance_by(96); continue; }
                                let not3 = (!mask3) & 0xFFFF_FFFF;
                                let tz3 = not3.trailing_zeros() as usize;
                                self.advance_by(64 + tz3);
                                return;
                            } else { self.advance_by(64); continue; }
                        }
                        let not2 = (!mask2) & 0xFFFF_FFFF;
                        let tz2 = not2.trailing_zeros() as usize;
                        self.advance_by(32 + tz2);
                        return;
                    } else { self.advance_by(32); continue; }
                }
                let not_mask = (!mask) & 0xFFFF_FFFF;
                let tz = not_mask.trailing_zeros() as usize;
                self.advance_by(tz);
                return;
            }
            self.advance_ident_continue_scalar();
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.advance_ident_continue_scalar();
        }
    }

    #[inline(always)]
    fn skip_ws_scalar(&mut self) {
        while let Some(&b) = self.src.get(self.idx) {
            match b { b' ' | b'\t' | b'\n' | b'\r' => self.advance_by(1), _ => break }
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn skip_ws_avx2(&mut self) {
        #[cfg(target_arch = "x86_64")]
        {
            let len = self.src.len();
            // Specialized whitespace mask without full classify: eq to ' ', '\t', '\r', '\n'
            let sp = _mm256_set1_epi8(b' ' as i8);
            let tb = _mm256_set1_epi8(b'\t' as i8);
            let cr = _mm256_set1_epi8(b'\r' as i8);
            let lf = _mm256_set1_epi8(b'\n' as i8);
            // Align to 32B boundary with tiny scalar prologue for ws
            if self.idx + 32 <= len {
                let base = self.src.as_ptr() as usize;
                let mut steps = 0u8;
                while ((base + self.idx) & 31) != 0 && self.idx < len && steps < 32 {
                    match self.byte_at(self.idx) { Some(b' ' | b'\t' | b'\n' | b'\r') => self.advance_by(1), _ => return }
                    steps += 1;
                }
            }
            while self.idx + 32 <= len {
                let pf = self.prefetch_ahead;
                if self.idx + (pf + 64) < len { unsafe { _mm_prefetch(self.src.as_ptr().add(self.idx + pf) as *const i8, 3); } }
                let ptr = unsafe { self.src.as_ptr().add(self.idx) as *const __m256i };
                let v = unsafe { load256(ptr) };
                let m_sp = _mm256_cmpeq_epi8(v, sp);
                let m_tb = _mm256_cmpeq_epi8(v, tb);
                let m_cr = _mm256_cmpeq_epi8(v, cr);
                let m_lf = _mm256_cmpeq_epi8(v, lf);
                let m_ws = _mm256_or_si256(_mm256_or_si256(m_sp, m_tb), _mm256_or_si256(m_cr, m_lf));
                let mask = _mm256_movemask_epi8(m_ws) as u32;
                if mask == 0xFFFF_FFFF { // all whitespace
                    if self.idx + 64 <= len {
                        let ptr2 = unsafe { self.src.as_ptr().add(self.idx + 32) as *const __m256i };
                        let v2 = unsafe { load256(ptr2) };
                        let m_sp2 = _mm256_cmpeq_epi8(v2, sp);
                        let m_tb2 = _mm256_cmpeq_epi8(v2, tb);
                        let m_cr2 = _mm256_cmpeq_epi8(v2, cr);
                        let m_lf2 = _mm256_cmpeq_epi8(v2, lf);
                        let m_ws2 = _mm256_or_si256(_mm256_or_si256(m_sp2, m_tb2), _mm256_or_si256(m_cr2, m_lf2));
                        let mask2 = _mm256_movemask_epi8(m_ws2) as u32;
                        if mask2 == 0xFFFF_FFFF {
                            if self.idx + 96 <= len {
                                let ptr3 = unsafe { self.src.as_ptr().add(self.idx + 64) as *const __m256i };
                                let v3 = unsafe { load256(ptr3) };
                                let m_sp3 = _mm256_cmpeq_epi8(v3, sp);
                                let m_tb3 = _mm256_cmpeq_epi8(v3, tb);
                                let m_cr3 = _mm256_cmpeq_epi8(v3, cr);
                                let m_lf3 = _mm256_cmpeq_epi8(v3, lf);
                                let m_ws3 = _mm256_or_si256(_mm256_or_si256(m_sp3, m_tb3), _mm256_or_si256(m_cr3, m_lf3));
                                let mask3 = _mm256_movemask_epi8(m_ws3) as u32;
                                if mask3 == 0xFFFF_FFFF { self.advance_by(96); continue; }
                                let not3 = (!mask3) & 0xFFFF_FFFF;
                                let tz3 = not3.trailing_zeros() as usize;
                                self.advance_by(64 + tz3);
                                return;
                            } else { self.advance_by(64); continue; }
                        }
                        let not2 = (!mask2) & 0xFFFF_FFFF;
                        let tz2 = not2.trailing_zeros() as usize;
                        self.advance_by(32 + tz2);
                        return;
                    } else { self.advance_by(32); continue; }
                }
                // Find first non-whitespace byte
                let not_mask = (!mask) & 0xFFFF_FFFF;
                let tz = not_mask.trailing_zeros() as usize;
                self.advance_by(tz);
                return;
            }
            // Tail
            self.skip_ws_scalar();
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.skip_ws_scalar();
        }
    }

    /// Scan forward inside a quoted string until the matching unescaped quote.
    /// Handles simple escapes by skipping the next byte after a backslash.
    #[inline]
    pub fn scan_string_simple(&mut self, quote: u8) {
        if self.use_avx2_string { unsafe { self.scan_string_simple_avx2(quote) } } else { self.scan_string_simple_scalar(quote) }
    }

    #[inline(always)]
    fn scan_string_simple_scalar(&mut self, quote: u8) {
        while !self.is_eof() {
            let c = self.peek();
            self.advance_by(1);
            if c == quote { break; }
            if c == b'\\' {
                // skip escaped byte if any
                if !self.is_eof() { self.advance_by(1); }
            }
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn scan_string_simple_avx2(&mut self, quote: u8) {
        #[cfg(target_arch = "x86_64")]
        {
            let len = self.src.len();
            let qv = _mm256_set1_epi8(quote as i8);
            let bsv = {
                static BS: OnceLock<__m256i> = OnceLock::new();
                *BS.get_or_init(|| unsafe { _mm256_set1_epi8(b'\\' as i8) })
            };
            while self.idx + 32 <= len {
                let pf = self.prefetch_ahead;
                if self.idx + (pf + 64) < len { unsafe { _mm_prefetch(self.src.as_ptr().add(self.idx + pf) as *const i8, 3); } }
                let ptr = unsafe { self.src.as_ptr().add(self.idx) as *const __m256i };
                let v = unsafe { load256(ptr) };
                // quotes or backslash mask using prebuilt vectors
                let m_q = _mm256_cmpeq_epi8(v, qv);
                let m_bs = _mm256_cmpeq_epi8(v, bsv);
                let m = _mm256_or_si256(m_q, m_bs);
                let mask = _mm256_movemask_epi8(m) as u32;
                if mask == 0 {
                    if self.idx + 64 <= len {
                        let ptr2 = unsafe { self.src.as_ptr().add(self.idx + 32) as *const __m256i };
                        let v2 = unsafe { load256(ptr2) };
                        let m_q2 = _mm256_cmpeq_epi8(v2, qv);
                        let m_bs2 = _mm256_cmpeq_epi8(v2, bsv);
                        let m2 = _mm256_or_si256(m_q2, m_bs2);
                        let mask2 = _mm256_movemask_epi8(m2) as u32;
                        if mask2 == 0 { self.advance_by(64); continue; }
                        let tz2 = mask2.trailing_zeros() as usize;
                        self.advance_by(32 + tz2);
                        // handle the triggering byte from the second chunk
                        if !self.is_eof() {
                            let c = self.peek();
                            self.advance_by(1);
                            if c == b'\\' {
                                if !self.is_eof() { self.advance_by(1); }
                                continue;
                            }
                            if c == quote { return; }
                        }
                        continue;
                    } else { self.advance_by(32); continue; }
                }
                let tz = mask.trailing_zeros() as usize;
                self.advance_by(tz);
                // handle the triggering byte
                if !self.is_eof() {
                    let c = self.peek();
                    self.advance_by(1);
                    if c == b'\\' {
                        if !self.is_eof() { self.advance_by(1); }
                        continue;
                    }
                    if c == quote { return; }
                }
            }
            // tail
            self.scan_string_simple_scalar(quote);
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.scan_string_simple_scalar(quote);
        }
    }
}

#[cfg(target_arch = "x86_64")]
struct Avx2ClassConsts {
    sp: __m256i, tb: __m256i, cr: __m256i, lf: __m256i,
    a_up: __m256i, z_up: __m256i, a_lo: __m256i, z_lo: __m256i,
    d_0: __m256i, d_9: __m256i, us: __m256i, dl: __m256i,
    dq: __m256i, sq: __m256i, bs: __m256i, dot: __m256i,
}

#[cfg(target_arch = "x86_64")]
impl Avx2ClassConsts {
    #[inline]
    fn new() -> Self { unsafe {
        Self {
            sp: _mm256_set1_epi8(b' ' as i8),
            tb: _mm256_set1_epi8(b'\t' as i8),
            cr: _mm256_set1_epi8(b'\r' as i8),
            lf: _mm256_set1_epi8(b'\n' as i8),
            a_up: _mm256_set1_epi8(b'A' as i8),
            z_up: _mm256_set1_epi8(b'Z' as i8),
            a_lo: _mm256_set1_epi8(b'a' as i8),
            z_lo: _mm256_set1_epi8(b'z' as i8),
            d_0: _mm256_set1_epi8(b'0' as i8),
            d_9: _mm256_set1_epi8(b'9' as i8),
            us: _mm256_set1_epi8(b'_' as i8),
            dl: _mm256_set1_epi8(b'$' as i8),
            dq: _mm256_set1_epi8(b'"' as i8),
            sq: _mm256_set1_epi8(b'\'' as i8),
            bs: _mm256_set1_epi8(b'\\' as i8),
            dot: _mm256_set1_epi8(b'.' as i8),
        }
    }}
}

#[cfg(target_arch = "x86_64")]
static AVX2_CLASS_CONSTS: OnceLock<Avx2ClassConsts> = OnceLock::new();

#[cfg(target_arch = "x86_64")]
struct ClassMasks { ws: u32, digits: u32, ident: u32, quotes: u32, backslash: u32, dot: u32 }

#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn classify32_avx2(v: __m256i, c: &Avx2ClassConsts) -> ClassMasks {
    unsafe {
        let m_sp = _mm256_cmpeq_epi8(v, c.sp);
        let m_tb = _mm256_cmpeq_epi8(v, c.tb);
        let m_cr = _mm256_cmpeq_epi8(v, c.cr);
        let m_lf = _mm256_cmpeq_epi8(v, c.lf);
        let ws1 = _mm256_or_si256(m_sp, m_tb);
        let ws2 = _mm256_or_si256(m_cr, m_lf);
        let m_ws = _mm256_or_si256(ws1, ws2);

        let z = _mm256_setzero_si256();
        let ge_0 = _mm256_cmpeq_epi8(_mm256_subs_epu8(c.d_0, v), z);
        let le_9 = _mm256_cmpeq_epi8(_mm256_subs_epu8(v, c.d_9), z);
        let m_d  = _mm256_and_si256(ge_0, le_9);

        let ge_aa = _mm256_cmpeq_epi8(_mm256_subs_epu8(c.a_up, v), z);
        let le_zz = _mm256_cmpeq_epi8(_mm256_subs_epu8(v, c.z_up), z);
        let m_up = _mm256_and_si256(ge_aa, le_zz);
        let ge_a = _mm256_cmpeq_epi8(_mm256_subs_epu8(c.a_lo, v), z);
        let le_z = _mm256_cmpeq_epi8(_mm256_subs_epu8(v, c.z_lo), z);
        let m_lo = _mm256_and_si256(ge_a, le_z);
        let m_us = _mm256_cmpeq_epi8(v, c.us);
        let m_dl = _mm256_cmpeq_epi8(v, c.dl);
        let id1 = _mm256_or_si256(m_up, m_lo);
        let id2 = _mm256_or_si256(m_d, _mm256_or_si256(m_us, m_dl));
        let m_ident = _mm256_or_si256(id1, id2);

        let m_dq = _mm256_cmpeq_epi8(v, c.dq);
        let m_sq = _mm256_cmpeq_epi8(v, c.sq);
        let m_quotes = _mm256_or_si256(m_dq, m_sq);
        let m_bs = _mm256_cmpeq_epi8(v, c.bs);
        let m_dot = _mm256_cmpeq_epi8(v, c.dot);

        ClassMasks {
            ws: _mm256_movemask_epi8(m_ws) as u32,
            digits: _mm256_movemask_epi8(m_d) as u32,
            ident: _mm256_movemask_epi8(m_ident) as u32,
            quotes: _mm256_movemask_epi8(m_quotes) as u32,
            backslash: _mm256_movemask_epi8(m_bs) as u32,
            dot: _mm256_movemask_epi8(m_dot) as u32,
        }
    }
}

#[inline]
pub fn is_ident_start_ascii(b: u8) -> bool { (b'A'..=b'Z').contains(&b) || (b'a'..=b'z').contains(&b) || b == b'_' || b == b'$' }

#[inline]
pub fn is_ident_continue_ascii(b: u8) -> bool { is_ident_start_ascii(b) || (b'0'..=b'9').contains(&b) }
