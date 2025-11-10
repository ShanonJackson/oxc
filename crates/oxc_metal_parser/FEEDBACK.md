Those numbers tell you something is very wrong with the AVX2 path, not that the scalar code is magically 6× faster:

cpb_scalar = 23.35     // ~23 cycles per byte → reasonable for your scalar parser
cpb_avx2    = 144.97    // ~145 cycles per byte → 6× slower (!)


but both hashes match, so correctness is fine.

Here’s what that usually means in this style of code:

🔍 1. Runtime dispatch isn’t actually hitting the AVX2 code

The #[target_feature(enable = "avx2")] functions can only be inlined if the caller also has that feature.
→ If you call them from a generic function, the CPU executes a slow dynamic fallback through a stub.

Add #[cfg_attr(target_feature="avx2", inline(always))] or explicitly mark the caller as #[target_feature(enable="avx2")].

Or, compile with:

RUSTFLAGS="-C target-feature=+avx2"


for the whole crate, just to sanity-check.

🧩 2. AVX2 constants are rebuilt in every call

Avx2ClassConsts::new() is allocating and filling 16 vectors per call.
That easily adds hundreds of µops per block.

✅ Fix:

static CONSTS: OnceLock<Avx2ClassConsts> = OnceLock::new();
...
let consts = CONSTS.get_or_init(Avx2ClassConsts::new);


Build it once and reuse.

🧮 3. Misalignment or mis-detected backend

Make sure detect() is returning Backend::Avx2.
Print at startup:

eprintln!("backend = {}", hot::backend::name(scan.backend));


If you see “scalar,” then you’re benchmarking scalar code called through the AVX2 entry point.

⚙️ 4. AVX2 loads not unrolled enough

In advance_digits_or_dot_avx2 and friends, the loop condition hits scalar tails constantly, and advance_by incurs bounds checks.
Inline the hot loop, use unchecked pointer math:

#[inline(always)]
unsafe fn advance_by_unchecked(&mut self, n: usize) {
self.idx = self.idx.unchecked_add(n);
self.shadow.advance_to(self.idx);
}


(keep unsafe_op_in_unsafe_fn on.)

🧠 5. Measure right section

Make sure your timing excludes allocator and AST building; otherwise SIMD setup time dwarfs the lexing savings on small inputs.
Benchmark on ≥1 MB of JS code.

✅ 6. Sanity expectation

On a typical AVX2 CPU (Zen3, Alder/Raptor Lake):

Build	Expected cycles/byte	Typical throughput
Scalar	20–25	4–5 GB/s
AVX2 (32 B)	6–9	12–20 GB/s
AVX-512 (64 B)	4–6	18–25 GB/s

So your 144 cy/B means the vector path isn’t actually executing hot, or it’s thrashing on constant setup.

🧰 Next steps

Print backend at runtime → confirm AVX2.

Make Avx2ClassConsts static.

Compile with -C target-cpu=native -C target-feature=+avx2.

Wrap your benchmark in:

core::hint::black_box(scanner.advance_digits_or_dot());


to prevent dead-code elimination.

Re-run; cpb_avx2 should drop into the single-digit range.

Once it does, you’ll be in the right ballpark — that’s the performance tier this design should hit.
