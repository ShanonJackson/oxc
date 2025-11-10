use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::fs;

fn main() {
    let path = std::env::args().nth(1).expect("file path");
    let data = fs::read(&path).expect("read file");
    let text = String::from_utf8_lossy(&data);
    let alloc = Allocator::new();
    let st = SourceType::from_path(&path).unwrap_or_else(|_| SourceType::mjs());

    let cycles_before = rdtsc();
    let program = oxc_metal_parser::parse_program(&alloc, &text, st);
    let cycles_after = rdtsc();
    let cycles = cycles_after - cycles_before;

    let bytes = data.len() as u64;
    let cpb = if bytes > 0 { cycles as f64 / bytes as f64 } else { 0.0 };
    let hash = oxc_metal_parser::structural_hash(&program);
    println!("bytes={} cycles={} cycles/byte={:.3} nodes={} hash={:016x}", bytes, cycles, cpb, program.body.len(), hash);
}

#[inline]
fn rdtsc() -> u64 {
    #[cfg(all(target_arch = "x86_64"))]
    unsafe {
        core::arch::x86_64::_rdtsc() as u64
    }
    #[cfg(not(all(target_arch = "x86_64")))]
    {
        // Fallback to time
        use std::time::Instant;
        static mut START: Option<Instant> = None;
        unsafe {
            let now = Instant::now();
            let s = START.get_or_insert(now);
            now.duration_since(*s).as_nanos() as u64
        }
    }
}

