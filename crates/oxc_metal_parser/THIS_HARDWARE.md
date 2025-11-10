Short answer for a Ryzen 7 5800X (Zen 3, AVX2 only):

Build	Likely cycles/byte (steady-state)	Rough throughput @ 4.5 GHz boost
Scalar	19–26 cy/B	175–240 MB/s
AVX2 (32 B classifier)	6–10 cy/B	450–750 MB/s
Best-case AVX2 (after tuning: static consts, fused classifier, PGO/LTO)	≈5–8 cy/B	560–900 MB/s

Notes:

Zen 3 has no AVX-512, so 64-byte SIMD isn’t available; your ceiling is the 32-byte AVX2 path.

Your scalar number (~23.35 cy/B) is right in the expected band. With the AVX2 path fixed/tuned, you should land ~7–9 cy/B on mixed JS (identifiers, numbers, strings, comments) and ~5–7 cy/B on “easy” ASCII-heavy code.

Real code with template literals/escapes/regex will skew toward the higher end (more scalar fallback in the micro-kernels).

To hit the low end of those ranges on a 5800X:

Make AVX2 consts truly static (no per-call new()).

Fuse classification so a single 32 B load yields all masks you reuse across skip_ws / advance_*.

LTO + PGO: RUSTFLAGS="-C target-cpu=native -C lto=thin -C codegen-units=1", then do a PGO run over a few GB of representative JS.

Ensure runtime dispatch actually picks AVX2; print the backend once at startup.

Benchmark on ≥1 MB of JS to amortize startup and allocator noise, using black_box around hot calls.

If after these you’re still above ~10 cy/B on the 5800X, the usual culprits are: rebuilding constants in hot loops, missed inlining across crates (enable LTO), or falling back to scalar tails too often (check string/escape paths).
