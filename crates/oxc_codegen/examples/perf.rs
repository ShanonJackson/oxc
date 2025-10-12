use std::{error::Error, fs, path::PathBuf, time::Instant};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use pico_args::Arguments;

fn main() -> Result<(), Box<dyn Error>> {
    let mut pargs = Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        print_help();
        return Ok(());
    }

    let file: PathBuf =
        pargs.value_from_str(["-f", "--file"]).map_err(|_| "--file <path> is required")?;
    let iterations: usize = pargs.value_from_str(["-n", "--iterations"]).unwrap_or(100);
    let minify = pargs.contains("--minify");
    let extra = pargs.finish();
    if !extra.is_empty() {
        return Err(format!("unexpected arguments: {:?}", extra).into());
    }

    if iterations == 0 {
        return Err("--iterations must be greater than zero".into());
    }

    let source = fs::read_to_string(&file)?;
    let source_type = SourceType::from_path(&file).unwrap_or_else(|_| SourceType::mjs());

    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, &source, source_type).parse();
    if !parser_return.errors.is_empty() {
        return Err(format!("parser produced {} errors", parser_return.errors.len()).into());
    }
    let program = parser_return.program;

    // Warm up the JIT and ensure all code paths are compiled before timing.
    let options = if minify { CodegenOptions::minify() } else { CodegenOptions::default() };
    let warmup_output = Codegen::new().with_options(options.clone()).build(&program);
    let warmup_size = warmup_output.code.len();

    let start = Instant::now();
    let mut total_bytes: usize = 0;
    for _ in 0..iterations {
        let output = Codegen::new().with_options(options.clone()).build(&program);
        total_bytes += output.code.len();
    }
    let elapsed = start.elapsed();

    let avg_ns = elapsed.as_nanos() as f64 / iterations as f64;
    let throughput = (total_bytes as f64 / 1_048_576.0) / elapsed.as_secs_f64();

    println!(
        "file: {}\niterations: {}\nminify: {}\nwarmup_bytes: {}\navg_ns: {:.0}\nthroughput_mib_s: {:.2}\n",
        file.display(),
        iterations,
        minify,
        warmup_size,
        avg_ns,
        throughput,
    );

    Ok(())
}

fn print_help() {
    eprintln!(
        "Usage: cargo run -p oxc_codegen --example perf -- --file <path> [--iterations <n>] [--minify]\n\
         \nOptions:\n  -f, --file <path>        Input file to parse and print\n  -n, --iterations <n>     Number of codegen iterations to measure (default: 100)\n      --minify             Enable codegen minify option\n  -h, --help               Show this message"
    );
}
