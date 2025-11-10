use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::fs;
use std::path::{PathBuf};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p oxc_metal_parser --example selfcheck -- <file.js>");
        std::process::exit(2)
    });
    let path = resolve_path(&arg);
    let data = fs::read(&path).expect("read file");
    let text = String::from_utf8_lossy(&data);
    let st = SourceType::from_path(path.as_path()).unwrap_or_else(|_| SourceType::mjs());

    // Force scalar
    let prev = std::env::var("METAL_BACKEND").ok();
    unsafe { std::env::set_var("METAL_BACKEND", "scalar"); }
    let alloc = Allocator::new();
    let t0 = rdtsc_serialized();
    let prog_scalar = oxc_metal_parser::parse_program(&alloc, &text, st);
    let t1 = rdtsc_serialized();
    let h_scalar = oxc_metal_parser::structural_hash(&prog_scalar);

    // Force AVX2
    unsafe { std::env::set_var("METAL_BACKEND", "avx2"); }
    let alloc = Allocator::new();
    let t2 = rdtsc_serialized();
    let prog_avx2 = oxc_metal_parser::parse_program(&alloc, &text, st);
    let t3 = rdtsc_serialized();
    let h_avx2 = oxc_metal_parser::structural_hash(&prog_avx2);

    // Restore env
    if let Some(v) = prev { unsafe { std::env::set_var("METAL_BACKEND", v); } } else { unsafe { std::env::remove_var("METAL_BACKEND"); } }

    let bytes = data.len() as u64;
    let cpb_scalar = (t1 - t0) as f64 / bytes as f64;
    let cpb_avx2 = (t3 - t2) as f64 / bytes as f64;
    let ok = h_scalar == h_avx2;
    println!(
        "selfcheck ok={} cpb_scalar={:.3} cpb_avx2={:.3} hash_scalar={:016x} hash_avx2={:016x}",
        ok, cpb_scalar, cpb_avx2, h_scalar, h_avx2
    );
    if !ok { std::process::exit(1); }
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
