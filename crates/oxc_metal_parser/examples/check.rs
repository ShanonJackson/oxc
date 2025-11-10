use oxc_allocator::Allocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};
use oxc_ast::ast::Statement;

fn main() {
    let be = oxc_metal_parser::hot::backend::name(oxc_metal_parser::hot::backend::detect());
    let arg = std::env::args().nth(1);
    let root = arg.map(|a| resolve_path(&a)).unwrap_or_else(default_fixtures_root);
    let mut files = Vec::new();
    collect_js(&root, &mut files);
    if files.is_empty() {
        eprintln!("no .js files found under {}", root.display());
        std::process::exit(2);
    }

    println!("backend={} files={} root={}", be, files.len(), root.display());

    let mut shape_ok = 0usize;
    let mut norm_ok = 0usize;
    let mut shape_bad = 0usize;
    let mut norm_bad = 0usize;
    let mut cpb_metal_sum = 0f64;
    let mut cpb_oxc_sum = 0f64;
    let mut bytes_sum = 0u64;
    let mut largest: Option<(PathBuf, usize)> = None;

    for p in files {
        let data = match fs::read(&p) { Ok(d) => d, Err(e) => { println!("ERR read {}: {}", p.display(), e); continue; } };
        if largest.as_ref().map(|(_, sz)| data.len() > *sz).unwrap_or(true) {
            largest = Some((p.clone(), data.len()));
        }
        let text = String::from_utf8_lossy(&data);
        let st = SourceType::from_path(p.as_path()).unwrap_or_else(|_| SourceType::mjs());

        // Perf single run per file
        let t0 = rdtsc_serialized();
        let alloc_m = Allocator::new();
        let metal = oxc_metal_parser::parse_program(&alloc_m, &text, st);
        let t1 = rdtsc_serialized();

        let t2 = rdtsc_serialized();
        let alloc_o = Allocator::new();
        let oxc = OxcParser::new(&alloc_o, &text, st).parse().program;
        let t3 = rdtsc_serialized();

        let bytes = data.len() as u64;
        let cpb_metal = (t1 - t0) as f64 / bytes as f64;
        let cpb_oxc = (t3 - t2) as f64 / bytes as f64;
        cpb_metal_sum += cpb_metal;
        cpb_oxc_sum += cpb_oxc;
        bytes_sum += bytes;

        let shape_equal = oxc.body.len() == metal.body.len();
        if shape_equal { shape_ok += 1; } else { shape_bad += 1; }

        let norm_equal = oxc_metal_parser::structural_hash(&oxc) == oxc_metal_parser::structural_hash(&metal);
        if norm_equal { norm_ok += 1; } else { norm_bad += 1; }

        if !(shape_equal && norm_equal) {
            println!(
                "MISMATCH file={} shape_equal={} norm_equal={} oxc_stmts={} metal_stmts={}",
                p.display(), shape_equal, norm_equal, oxc.body.len(), metal.body.len()
            );
            // Print per-kind histograms and first differing index by kind
            let (ho, hm) = (histogram(&oxc.body), histogram(&metal.body));
            println!("oxc_hist={:?}", ho);
            println!("metal_hist={:?}", hm);
            if let Some((i, ko, km)) = first_stmt_kind_diff(&oxc.body, &metal.body) {
                println!("first_stmt_kind_diff idx={} oxc_kind={} metal_kind={}", i, ko, km);
            }
            print_first_spans(&metal, &text, "metal");
            print_first_spans(&oxc, &text, "oxc");
        }
    }

    let n = (shape_ok + shape_bad).max(1) as f64;
    let avg_metal = cpb_metal_sum / n;
    let avg_oxc = cpb_oxc_sum / n;
    let (t_lo, t_hi) = theoretical_window(be);
    let ratio_vs_hi = if t_hi > 0.0 { avg_metal / t_hi } else { f64::NAN };
    println!(
        "summary shape_ok={} shape_bad={} norm_ok={} norm_bad={} avg_metal_cpb={:.3} avg_oxc_cpb={:.3} target_window_cpb=[{:.1},{:.1}] ratio_vs_hi={:.3}",
        shape_ok, shape_bad, norm_ok, norm_bad, avg_metal, avg_oxc, t_lo, t_hi, ratio_vs_hi
    );

    // Optional A/B within metal: scalar vs avx2 on the largest file (if present)
    if let Some((lp, sz)) = largest {
        if has_avx2() {
            if let Ok(data) = fs::read(&lp) {
                let text = String::from_utf8_lossy(&data);
                let st = SourceType::from_path(lp.as_path()).unwrap_or_else(|_| SourceType::mjs());
                let mut saved = std::env::var("METAL_BACKEND").ok();

                unsafe { std::env::set_var("METAL_BACKEND", "scalar"); }
                let t0 = rdtsc_serialized();
                let alloc = Allocator::new();
                let pr_scalar = oxc_metal_parser::parse_program(&alloc, &text, st);
                let t1 = rdtsc_serialized();
                let h_scalar = oxc_metal_parser::structural_hash(&pr_scalar);

                unsafe { std::env::set_var("METAL_BACKEND", "avx2"); }
                let t2 = rdtsc_serialized();
                let alloc = Allocator::new();
                let pr_avx2 = oxc_metal_parser::parse_program(&alloc, &text, st);
                let t3 = rdtsc_serialized();
                let h_avx2 = oxc_metal_parser::structural_hash(&pr_avx2);

                if let Some(v) = saved.take() { unsafe { std::env::set_var("METAL_BACKEND", v); } } else { unsafe { std::env::remove_var("METAL_BACKEND"); } }

                let bytes = data.len() as u64;
                let cpb_scalar = (t1 - t0) as f64 / bytes as f64;
                let cpb_avx2 = (t3 - t2) as f64 / bytes as f64;
                let ok = h_scalar == h_avx2;
                println!(
                    "ab file={} size={} cpb_scalar={:.3} cpb_avx2={:.3} ab_ok={}",
                    lp.display(), sz, cpb_scalar, cpb_avx2, ok
                );

                // Token stream diff (scalar vs avx2) on largest file
                unsafe { std::env::set_var("METAL_BACKEND", "scalar"); }
                let toks_scalar = lex(&text);
                unsafe { std::env::set_var("METAL_BACKEND", "avx2"); }
                let toks_avx2 = lex(&text);
                if let Some((idx, a, b)) = first_tok_diff(&toks_scalar, &toks_avx2) {
                    println!("tok_ab diff at {}: scalar={:?} avx2={:?}", idx, a, b);
                } else {
                    println!("tok_ab ok tokens={}", toks_scalar.len());
                }
                if let Some(v) = saved { unsafe { std::env::set_var("METAL_BACKEND", v); } } else { unsafe { std::env::remove_var("METAL_BACKEND"); } }
            }
        }
    }

    let no_fail = std::env::var("METAL_NO_FAIL").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);
    if norm_bad > 0 && !no_fail { std::process::exit(1); }
}

fn kind_code(s: &Statement<'_>) -> &'static str {
    match s {
        Statement::ExpressionStatement(_) => "Expr",
        Statement::VariableDeclaration(_) => "VarDecl",
        Statement::EmptyStatement(_) => "Empty",
        Statement::FunctionDeclaration(_) => "FnDecl",
        Statement::ForStatement(_) | Statement::ForInStatement(_) | Statement::ForOfStatement(_) => "For",
        Statement::IfStatement(_) => "If",
        Statement::WhileStatement(_) | Statement::DoWhileStatement(_) => "While",
        Statement::ReturnStatement(_) => "Return",
        Statement::BlockStatement(_) => "Block",
        _ => "Other",
    }
}

fn histogram(body: &oxc_allocator::Vec<'_, Statement<'_>>) -> std::collections::BTreeMap<&'static str, usize> {
    let mut m = std::collections::BTreeMap::new();
    for s in body.iter() { *m.entry(kind_code(s)).or_insert(0) += 1; }
    m
}

fn first_stmt_kind_diff<'a>(a: &oxc_allocator::Vec<'a, Statement<'a>>, b: &oxc_allocator::Vec<'a, Statement<'a>>)
    -> Option<(usize, &'static str, &'static str)>
{
    let n = a.len().min(b.len());
    for i in 0..n { let (ka, kb) = (kind_code(&a[i]), kind_code(&b[i])); if ka != kb { return Some((i, ka, kb)); } }
    if a.len() != b.len() {
        let i = n.saturating_sub(1);
        return Some((n, kind_code(&a[i]), kind_code(&b[i])));
    }
    None
}

fn collect_js(dir: &Path, out: &mut Vec<PathBuf>) {
    if dir.is_file() {
        if is_js(dir) { out.push(dir.to_path_buf()); }
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return; };
    for e in rd.flatten() {
        let path = e.path();
        if path.is_dir() { collect_js(&path, out); } else if is_js(&path) { out.push(path); }
    }
}

fn is_js(p: &Path) -> bool {
    matches!(p.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase().as_str(),
        "js" | "mjs" | "cjs" | "jsx")
}

fn default_fixtures_root() -> PathBuf {
    // crates/oxc_metal_parser/fixtures
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        base.join("fixtures"),
        base.join("..").join("..").join("crates").join("oxc_metal_parser").join("fixtures"),
    ];
    for c in candidates { if c.exists() { return c; } }
    base
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

fn theoretical_window(backend: &str) -> (f64, f64) {
    match backend {
        // End-to-end lex+parse+AST targets; aggressive but achievable with SPOB and tight AST emission
        "avx512" => (4.0, 6.0),
        "avx2" => (6.0, 9.0),
        // Scalar isn’t a goal; set a loose window for reference only
        _ => (12.0, 20.0),
    }
}

#[inline]
fn has_avx2() -> bool {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        return std::arch::is_x86_feature_detected!("avx2");
    }
    #[allow(unreachable_code)]
    { false }
}

// Inline tokenization helpers (scalar vs avx2)
use oxc_metal_parser::hot::Scanner;
use oxc_metal_parser::token::{Tokenizer, Tok, TokKind};

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
    for i in 0..n {
        if a[i] != b[i] { return Some((i, a[i], b[i])); }
    }
    if a.len() != b.len() {
        let i = n;
        let aa = *a.get(i.saturating_sub(1)).unwrap_or_else(|| a.last().unwrap());
        let bb = *b.get(i.saturating_sub(1)).unwrap_or_else(|| b.last().unwrap());
        return Some((i, aa, bb));
    }
    None
}

fn print_first_spans(program: &oxc_ast::ast::Program<'_>, text: &str, label: &str) {
    use oxc_ast::ast::Statement;
    let mut shown = 0usize;
    for s in program.body.iter() {
        let span = match s {
            Statement::ExpressionStatement(b) => b.span,
            Statement::VariableDeclaration(b) => b.span,
            Statement::BlockStatement(b) => b.span,
            _ => continue,
        };
        let (a, b) = (span.start as usize, span.end as usize);
        let a = a.min(text.len());
        let b = b.min(text.len());
        let slice = &text[a..b];
        let preview = slice.chars().take(40).collect::<String>().replace('\n', "\\n").replace('\r', "");
        println!("{}[{}] span=[{},{}] len={} preview=\"{}\"", label, shown, a, b, b.saturating_sub(a), preview);
        shown += 1;
        if shown >= 3 { break; }
    }
}
