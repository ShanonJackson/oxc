use oxc_allocator::Allocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p oxc_metal_parser --example perf -- <file.js> [iters]");
        std::process::exit(2)
    });
    let path = resolve_path(&arg);
    let iters: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(1000);
    let data = fs::read(&path).expect("read file");
    let text = String::from_utf8_lossy(&data);
    let st = SourceType::from_path(path.as_path()).unwrap_or_else(|_| SourceType::mjs());

    // Warm-up
    for _ in 0..10 {
        let alloc = Allocator::new();
        let _ = oxc_metal_parser::parse_program(&alloc, &text, st);
        let alloc = Allocator::new();
        let _ = OxcParser::new(&alloc, &text, st).parse().program;
    }

    let bytes = data.len() as u64;

    // Measure metal
    let mut cycles_metal: u64 = 0;
    for _ in 0..iters {
        let alloc = Allocator::new();
        let t0 = rdtsc_serialized();
        let program = oxc_metal_parser::parse_program(&alloc, &text, st);
        let t1 = rdtsc_serialized();
        // quick structural hash to keep the compiler from eliding work
        let _h = oxc_metal_parser::structural_hash(&program);
        cycles_metal = cycles_metal.saturating_add(t1 - t0);
    }

    // Measure oxc_parser
    let mut cycles_oxc: u64 = 0;
    for _ in 0..iters {
        let alloc = Allocator::new();
        let t0 = rdtsc_serialized();
        let program = OxcParser::new(&alloc, &text, st).parse().program;
        let t1 = rdtsc_serialized();
        let _h = oxc_metal_parser::structural_hash(&program);
        cycles_oxc = cycles_oxc.saturating_add(t1 - t0);
    }

    let cpb_metal = (cycles_metal as f64 / iters as f64) / (bytes as f64);
    let cpb_oxc = (cycles_oxc as f64 / iters as f64) / (bytes as f64);

    // Single-run correctness check (shape/hash) on the provided file
    let alloc1 = Allocator::new();
    let alloc2 = Allocator::new();
    let pr_oxc = OxcParser::new(&alloc1, &text, st).parse().program;
    let pr_metal = oxc_metal_parser::parse_program(&alloc2, &text, st);
    let shape_equal = pr_oxc.body.len() == pr_metal.body.len();
    let hash_equal = oxc_metal_parser::structural_hash(&pr_oxc) == oxc_metal_parser::structural_hash(&pr_metal);

    // Report backend
    let be = oxc_metal_parser::hot::backend::name(oxc_metal_parser::hot::backend::detect());
    println!(
        "bytes={} iters={} backend={} metal_cpb={:.3} oxc_cpb={:.3} shape_equal={} hash_equal={}",
        bytes, iters, be, cpb_metal, cpb_oxc, shape_equal, hash_equal
    );
}

fn resolve_path(arg: &str) -> PathBuf {
    let p = PathBuf::from(arg);
    if p.is_absolute() || p.exists() { return p; }
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        base.join(arg),
        base.join("..").join(arg),
        base.join("..").join("..").join(arg),
    ];
    for c in candidates { if c.exists() { return c; } }
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
