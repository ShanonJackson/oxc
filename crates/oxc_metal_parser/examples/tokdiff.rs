use oxc_metal_parser::token::{Tok, TokKind, Tokenizer};
use oxc_metal_parser::hot::Scanner;
use std::path::{Path, PathBuf};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p oxc_metal_parser --example tokdiff -- <file.js>");
        std::process::exit(2)
    });
    let path = resolve_path(&arg);
    let data = std::fs::read(&path).expect("read file");
    let text = String::from_utf8_lossy(&data);

    // Scalar tokens
    unsafe { std::env::set_var("METAL_BACKEND", "scalar"); }
    let toks_scalar = lex(&text);

    // AVX2 tokens
    unsafe { std::env::set_var("METAL_BACKEND", "avx2"); }
    let toks_avx2 = lex(&text);

    // Restore
    unsafe { std::env::remove_var("METAL_BACKEND"); }

    let n = toks_scalar.len().min(toks_avx2.len());
    for i in 0..n {
        if toks_scalar[i] != toks_avx2[i] {
            println!(
                "DIFF at {}: scalar={:?} avx2={:?}",
                i, toks_scalar[i], toks_avx2[i]
            );
            std::process::exit(1);
        }
    }
    if toks_scalar.len() != toks_avx2.len() {
        println!("LEN DIFF: scalar_len={} avx2_len={}", toks_scalar.len(), toks_avx2.len());
        std::process::exit(1);
    }
    println!("tokdiff ok size={} tokens={}", data.len(), toks_scalar.len());
}

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

