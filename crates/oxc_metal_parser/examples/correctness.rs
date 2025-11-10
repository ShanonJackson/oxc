use oxc_allocator::Allocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};

// One-file harness you can freely edit between runs.
// Edit SCENARIOS and FILES below, then run:
//   cargo run --release -p oxc_metal_parser --example correctness -- [optional files/dirs]
// Uses METAL_CHECK_REPEAT (default 5).

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    prefetch: Option<usize>,
    ident_simd: Option<bool>,
    ws_simd: Option<bool>,
    string_simd: Option<bool>,
}

// Edit these to A/B quickly.
const SCENARIOS: &[Scenario] = &[
    Scenario { name: "baseline", prefetch: None, ident_simd: None, ws_simd: None, string_simd: None },
    Scenario { name: "minified_auto", prefetch: None, ident_simd: None, ws_simd: None, string_simd: None },
    Scenario { name: "prefetch96", prefetch: Some(96), ident_simd: None, ws_simd: None, string_simd: None },
    Scenario { name: "prefetch64", prefetch: Some(64), ident_simd: None, ws_simd: None, string_simd: None },
    Scenario { name: "string_off", prefetch: None, ident_simd: None, ws_simd: None, string_simd: Some(false) },
    Scenario { name: "ident_on", prefetch: None, ident_simd: Some(true), ws_simd: None, string_simd: None },
];

fn main() {
    set_thread_perf_hints();
    warmup();

    let repeats: usize = std::env::var("METAL_CHECK_REPEAT").ok().and_then(|s| s.parse().ok()).unwrap_or(5).max(1).min(20);

    let mut files = Vec::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        // Default to fixtures if no args; resolve from crate dir
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let cands = [
            base.join("fixtures").join("moment.js"),
            base.join("fixtures").join("three.js"),
            base.join("..").join("..").join("crates").join("oxc_metal_parser").join("fixtures").join("moment.js"),
            base.join("..").join("..").join("crates").join("oxc_metal_parser").join("fixtures").join("three.js"),
        ];
        for pb in cands { if pb.exists() { files.push(pb); } }
    } else {
        for a in args { collect_js(&resolve_path(&a), &mut files); }
    }
    if files.is_empty() {
        eprintln!("no .js files found");
        std::process::exit(2);
    }

    println!("correctness scenarios={} repeat={} files={}:", SCENARIOS.len(), repeats, files.len());
    for sc in SCENARIOS {
        let guard = EnvGuard::new(*sc);
        println!("scenario name={} prefetch={} ident_simd={:?} ws_simd={:?} string_simd={:?}",
            sc.name,
            sc.prefetch.map(|v| v.to_string()).unwrap_or_else(|| "(auto)".into()),
            sc.ident_simd, sc.ws_simd, sc.string_simd
        );
        for p in &files {
            run_one(p, repeats);
        }
        drop(guard);
    }
}

fn run_one(p: &Path, repeats: usize) {
    let Ok(data) = fs::read(p) else { println!("ERR read {}", p.display()); return; };
    let text = String::from_utf8_lossy(&data);
    let st = SourceType::from_path(p).unwrap_or_else(|_| SourceType::mjs());
    let bytes = data.len() as u64;

    // metal averaged
    let mut cyc_m: u128 = 0;
    let alloc_m_final = Allocator::new();
    let metal = {
        let mut last = None;
        for i in 0..repeats {
            let t0 = rdtsc_serialized();
            if i + 1 == repeats {
                let prog = oxc_metal_parser::parse_program(&alloc_m_final, &text, st);
                let t1 = rdtsc_serialized();
                cyc_m += (t1 - t0) as u128;
                last = Some(prog);
            } else {
                let alloc_tmp = Allocator::new();
                let _ = oxc_metal_parser::parse_program(&alloc_tmp, &text, st);
                let t1 = rdtsc_serialized();
                cyc_m += (t1 - t0) as u128;
            }
        }
        last.unwrap()
    };

    // oxc averaged
    let mut cyc_o: u128 = 0;
    let alloc_o_final = Allocator::new();
    let oxc = {
        let mut last = None;
        for i in 0..repeats {
            let t0 = rdtsc_serialized();
            if i + 1 == repeats {
                let prog = OxcParser::new(&alloc_o_final, &text, st).parse().program;
                let t1 = rdtsc_serialized();
                cyc_o += (t1 - t0) as u128;
                last = Some(prog);
            } else {
                let alloc_tmp = Allocator::new();
                let _ = OxcParser::new(&alloc_tmp, &text, st).parse().program;
                let t1 = rdtsc_serialized();
                cyc_o += (t1 - t0) as u128;
            }
        }
        last.unwrap()
    };

    let cpb_m = (cyc_m as f64 / repeats as f64) / bytes as f64;
    let cpb_o = (cyc_o as f64 / repeats as f64) / bytes as f64;
    let shape_equal = oxc.body.len() == metal.body.len();
    let norm_equal = oxc_metal_parser::structural_hash(&oxc) == oxc_metal_parser::structural_hash(&metal);
    println!(
        "file={} bytes={} cpb_metal={:.3} cpb_oxc={:.3} shape_equal={} norm_equal={}",
        p.display(), bytes, cpb_m, cpb_o, shape_equal, norm_equal
    );

    // A/B metal
    let saved = std::env::var("METAL_BACKEND").ok();
    unsafe { std::env::set_var("METAL_BACKEND", "scalar"); }
    let mut cyc_s: u128 = 0;
    let alloc_s_final = Allocator::new();
    let pr_s = {
        let mut last = None;
        for i in 0..repeats {
            let t0 = rdtsc_serialized();
            if i + 1 == repeats {
                let prog = oxc_metal_parser::parse_program(&alloc_s_final, &text, st);
                let t1 = rdtsc_serialized();
                cyc_s += (t1 - t0) as u128;
                last = Some(prog);
            } else {
                let alloc_tmp = Allocator::new();
                let _ = oxc_metal_parser::parse_program(&alloc_tmp, &text, st);
                let t1 = rdtsc_serialized();
                cyc_s += (t1 - t0) as u128;
            }
        }
        last.unwrap()
    };
    let h_s = oxc_metal_parser::structural_hash(&pr_s);

    unsafe { std::env::set_var("METAL_BACKEND", "avx2"); }
    let mut cyc_v: u128 = 0;
    let alloc_v_final = Allocator::new();
    let pr_v = {
        let mut last = None;
        for i in 0..repeats {
            let t0 = rdtsc_serialized();
            if i + 1 == repeats {
                let prog = oxc_metal_parser::parse_program(&alloc_v_final, &text, st);
                let t1 = rdtsc_serialized();
                cyc_v += (t1 - t0) as u128;
                last = Some(prog);
            } else {
                let alloc_tmp = Allocator::new();
                let _ = oxc_metal_parser::parse_program(&alloc_tmp, &text, st);
                let t1 = rdtsc_serialized();
                cyc_v += (t1 - t0) as u128;
            }
        }
        last.unwrap()
    };
    let h_v = oxc_metal_parser::structural_hash(&pr_v);
    if let Some(s) = saved { unsafe { std::env::set_var("METAL_BACKEND", s); } } else { unsafe { std::env::remove_var("METAL_BACKEND"); } }
    let cpb_s = (cyc_s as f64 / repeats as f64) / bytes as f64;
    let cpb_v = (cyc_v as f64 / repeats as f64) / bytes as f64;
    println!(
        "ab file={} size={} cpb_scalar={:.3} cpb_avx2={:.3} ab_ok={}",
        p.display(), bytes, cpb_s, cpb_v, h_s == h_v
    );

    // Token A/B
    unsafe { std::env::set_var("METAL_BACKEND", "scalar"); }
    let toks_scalar = lex(&text);
    unsafe { std::env::set_var("METAL_BACKEND", "avx2"); }
    let toks_avx2 = lex(&text);
    if let Some((idx, a, b)) = first_tok_diff(&toks_scalar, &toks_avx2) {
        println!("tok_ab diff at {}: scalar={:?} avx2={:?}", idx, a, b);
    } else {
        println!("tok_ab ok tokens={}", toks_scalar.len());
    }
    unsafe { std::env::remove_var("METAL_BACKEND"); }
}

fn collect_js(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() { if is_js(path) { out.push(path.to_path_buf()); } return; }
    let Ok(rd) = fs::read_dir(path) else { return; };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() { collect_js(&p, out); } else if is_js(&p) { out.push(p); }
    }
}
fn is_js(p: &Path) -> bool {
    matches!(p.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase().as_str(),
        "js" | "mjs" | "cjs" | "jsx")
}
fn resolve_path(arg: &str) -> PathBuf {
    let p = PathBuf::from(arg);
    if p.is_absolute() || p.exists() { return p; }
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for c in [base.join(arg), base.join("..").join(arg), base.join("..").join("..").join(arg)] {
        if c.exists() { return c; }
    }
    p
}

#[inline]
fn rdtsc_serialized() -> u64 {
    #[cfg(all(target_arch = "x86_64"))]
    unsafe {
        core::arch::x86_64::_mm_lfence();
        let t = core::arch::x86_64::_rdtsc();
        core::arch::x86_64::_mm_lfence();
        t as u64
    }
    #[cfg(not(all(target_arch = "x86_64")))]
    {
        use std::time::Instant;
        static mut START: Option<Instant> = None;
        unsafe {
            let now = Instant::now();
            let s = START.get_or_insert(now);
            now.duration_since(*s).as_nanos() as u64
        }
    }
}

// Inline tokenization helpers (scalar vs avx2)
use oxc_metal_parser::hot::Scanner;
use oxc_metal_parser::token::{Tok, TokKind, Tokenizer};

fn lex(text: &str) -> Vec<Tok> {
    let mut scan = Scanner::new(text.as_bytes());
    let mut tz = Tokenizer::new(&mut scan);
    let mut out = Vec::new();
    loop {
        let t = tz.next();
        out.push(t);
        if matches!(t.kind, TokKind::Eof) { break; }
    }
    out
}

fn first_tok_diff(a: &[Tok], b: &[Tok]) -> Option<(usize, Tok, Tok)> {
    let n = a.len().min(b.len());
    for i in 0..n { if a[i] != b[i] { return Some((i, a[i], b[i])); } }
    if a.len() != b.len() {
        let i = n;
        let aa = *a.get(i.saturating_sub(1)).unwrap_or_else(|| a.last().unwrap());
        let bb = *b.get(i.saturating_sub(1)).unwrap_or_else(|| b.last().unwrap());
        return Some((i, aa, bb));
    }
    None
}

struct EnvGuard { saved: Vec<(String, Option<String>)> }
impl EnvGuard {
    fn new(sc: Scenario) -> Self {
        let mut saved = Vec::new();
        for k in ["METAL_PREFETCH", "METAL_SIMD_IDENT", "METAL_SIMD_WS", "METAL_SIMD_STRING"] {
            saved.push((k.to_string(), std::env::var(k).ok()));
        }
        unsafe {
            if let Some(v) = sc.prefetch { std::env::set_var("METAL_PREFETCH", v.to_string()); } else { std::env::remove_var("METAL_PREFETCH"); }
            if let Some(v) = sc.ident_simd { std::env::set_var("METAL_SIMD_IDENT", if v { "1" } else { "0" }); } else { std::env::remove_var("METAL_SIMD_IDENT"); }
            if let Some(v) = sc.ws_simd { std::env::set_var("METAL_SIMD_WS", if v { "1" } else { "0" }); } else { std::env::remove_var("METAL_SIMD_WS"); }
            if let Some(v) = sc.string_simd { std::env::set_var("METAL_SIMD_STRING", if v { "1" } else { "0" }); } else { std::env::remove_var("METAL_SIMD_STRING"); }
        }
        Self { saved }
    }
}
impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (k, v) in self.saved.drain(..) {
            unsafe { if let Some(val) = v { std::env::set_var(&k, val); } else { std::env::remove_var(&k); } }
        }
    }
}

#[cfg(windows)]
fn set_thread_perf_hints() {
    use std::ffi::c_void;
    unsafe extern "system" {
        fn GetCurrentThread() -> *mut c_void;
        fn GetCurrentProcess() -> *mut c_void;
        fn SetThreadPriority(hThread: *mut c_void, nPriority: i32) -> i32;
        fn SetThreadAffinityMask(hThread: *mut c_void, mask: usize) -> usize;
        fn SetPriorityClass(hProcess: *mut c_void, dwPriorityClass: u32) -> i32;
    }
    unsafe {
        let thread = GetCurrentThread();
        let process = GetCurrentProcess();
        let _ = SetPriorityClass(process, 0x00000080); // HIGH_PRIORITY_CLASS
        let _ = SetThreadPriority(thread, 2); // THREAD_PRIORITY_HIGHEST
        let _ = SetThreadAffinityMask(thread, 1usize << 2);
    }
}

#[cfg(not(windows))]
fn set_thread_perf_hints() {}

fn warmup() {
    let text = "  12345 'a' \"b\" 6789 (x)(y) // c\n/* d */";
    let st = SourceType::mjs();
    let alloc = Allocator::new();
    let _ = oxc_metal_parser::parse_program(&alloc, text, st);
}
