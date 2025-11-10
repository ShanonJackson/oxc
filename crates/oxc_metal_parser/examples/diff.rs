use oxc_allocator::Allocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use std::path::{Path, PathBuf};

fn main() {
    let arg_path = std::env::args().nth(1).expect("file path");
    let path = resolve_path(&arg_path);
    let mode = std::env::args().nth(2).unwrap_or_else(|| "shape".into());
    let text = std::fs::read_to_string(&path).expect("read file");
    let st = SourceType::from_path(path.as_path()).unwrap_or_else(|_| SourceType::mjs());

    let alloc1 = Allocator::new();
    let alloc2 = Allocator::new();

    let oxc = OxcParser::new(&alloc1, &text, st).parse().program;
    let metal = oxc_metal_parser::parse_program(&alloc2, &text, st);
    let be = oxc_metal_parser::hot::backend::name(oxc_metal_parser::hot::backend::detect());

    match mode.as_str() {
        "shape" => {
            let eq = oxc.body.len() == metal.body.len();
            println!("backend={} shape_equal={} oxc_stmts={} metal_stmts={}", be, eq, oxc.body.len(), metal.body.len());
            if !eq { std::process::exit(1); }
        }
        _ => {
            let h1 = oxc_metal_parser::structural_hash(&metal);
            let h2 = oxc_metal_parser::structural_hash(&oxc);
            let eq = h1 == h2;
            println!("backend={} normalized_equal={} metal_hash={:016x} oxc_hash={:016x}", be, eq, h1, h2);
            if !eq { std::process::exit(1); }
        }
    }
}

fn resolve_path(arg: &str) -> PathBuf {
    let p = PathBuf::from(arg);
    if p.is_absolute() || p.exists() {
        return p;
    }
    // Resolve relative to this crate's manifest dir, also try going up directories
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        base.join(arg),
        base.join("..").join(arg),
        base.join("..").join("..").join(arg),
    ];
    for c in candidates {
        if c.exists() { return c; }
    }
    p
}
