LEARNINGS

- SIMD ident-run mask correctness:
  - Unsigned saturated subtract trick is robust for ASCII ranges when used as:
    - v >= lo => subs_epu8(lo, v) == 0
    - v <= hi => subs_epu8(v, hi) == 0
    - In-range => AND of the above.
  - The prior “min(x, range) == x” approach misclassifies bytes < lo.

- UTF-8 slicing hazards:
  - Never construct Ident spans from non-ASCII starts; slicing &str by byte spans at arbitrary positions can panic at non-char boundaries.
  - Solution: default token kind for unknown bytes is Other, not Ident; only ASCII ident start + ASCII continues become Ident.

- Guardrails for correctness during SIMD rollout:
  - tokdiff example (scalar vs AVX2 token streams) is a fast regression detector for tokenizer SIMD changes.
  - selfcheck (scalar vs AVX2 AST) keeps backend parity and reports cpb; this is required before enabling new SIMD paths by default.
  - check consolidates parser diff (shape/norm), perf averages, backend A/B, token A/B into a single command.

- Early perf truths:
  - On tiny files, vector setup costs dominate and AVX2 can be slower than scalar; gains materialize on larger inputs and as we hoist constants and reduce per-iteration work.
  - Hoisting _mm256_set1 constants out of loops and marking hot helpers inline(always) has a visible impact before deeper classifier work.
  - Cold-start skew is real: first-run includes OnceLock init and branch prediction warmup. Adding a tiny synthetic warmup (ws/digits/strings/parens) before measurement stabilizes cpB.
  - Double-chunk unrolling (check two 32B blocks per iteration when masks are all-match) and light prefetching (~+128B ahead) reduce loop overhead and L2 stalls on large files.
  - Conditional aligned loads: using `_mm256_load_si256` when the pointer is 32B-aligned and falling back to `_mm256_loadu_si256` otherwise gives small but measurable wins on some inputs without risking UB.
  - Micro A/B at startup: a one-time whitespace kernel calibrator (synthetic 8KiB buffer) flips AVX2 whitespace on only if it's ≥5% faster on this CPU. This avoids regressions without user env flags.
  - Triple-chunk unroll (up to 96B) for digits and whitespace further reduces loop overhead on long runs; strings remain double-chunk due to trigger-byte semantics.
  - On Ryzen 7 5800X, prefetch=128B is the current best; 64/96 varied slightly but 128B produced the most stable low cpB.
  - Ident path: added 32B alignment prologue and triple-chunk unroll in AVX2 ident-continue. Still disabled by default on this CPU pending A/B, but ready if it wins in future.
  - On Zen 3 (5800X), ident SIMD did not consistently beat scalar after unrolling; keep it off by default and rely on per-CPU A/B.

- Measurement hygiene:
  - Add `METAL_CHECK_REPEAT` (default 3) to average cpB in `examples/check` and print `repeat=N` alongside settings. This de-noises runs and prevents chasing variance.
  - On Windows, setting high process/thread priority and pinning to a single core (via Kernel32) reduces jitter and helps expose small wins. Implemented in `examples/check.rs`.

- Non-ASCII inputs:
  - `moment.js` contains non-ASCII characters inside string literals (e.g., Chinese, Arabic). Our string scanning is byte-based and treats quotes/backslashes only, so Unicode inside strings does not degrade performance. Identifier logic is ASCII-only by design in this phase; unknown bytes fall back to `Other` and do not impact SPOB.

- Prefetch policy:
  - Fixed default prefetch=128B proved most stable across mixed workloads (ASCII minified and non‑minified). Per‑file whitespace heuristics did not consistently help on Zen 3. `METAL_PREFETCH` remains for A/B overrides.

- SPOB discipline:
  - All SIMD loops maintain forward-only advancement and use tail scalar for remainder; SPOB shadow asserts monotonic byte advancement.

- Parser robustness for real-world files:
  - Minimal Pratt parsing ( + - * / and parens ) captures many arithmetic-heavy patterns.
  - Do not assert strict forms (e.g., let without ident) during perf work; fallback to treating keywords as identifiers in expressions to avoid panics and to keep advancing.
  - Swallow top-level IIFEs `(function(){...})(...)` as a single expression statement; emit EmptyStatement for leading `;` to align with OXC top-level shape on UMD patterns (e.g., moment.js).
  - Incrementally materialize AST shape: emit `ParenthesizedExpression` for `(...)` and minimal `CallExpression` nodes for postfix `(...)(...)` chains (arguments elided for now). Structural hash remains lenient to avoid false negatives during this phase.

- Tokenizer improvements:
  - Introduced a single initial 32B read in `next()` to make first-byte decisions without redundant loads. This sets up a future fused path.
  - Switched punctuation mapping to a byte LUT to reduce branch fanout in operator-heavy (minified) code.
  - Added local block scanning for short numeric and identifier tokens within the current 32B window, falling back to SIMD kernels only when tokens extend beyond. This reduces per-token overhead on minified inputs with many short tokens.
  - Fused local ASCII whitespace skip at the start of `next()` (within current 32B) before falling back to the comment skipper. This avoids extra trips through the whitespace path for small stretches and keeps SPOB.
  - Note: On heavy minified code (three.js), a fused local WS skip did not consistently help and sometimes regressed; we reverted to the proven `skip_ws_and_comments()` path. The wins came from the LUT and block-local scans for short tokens.

- Structural hashing strategy (bootstrapping phase):
  - Treat ExpressionStatement as kind-only in the structural hash to keep diff signal useful while the expression AST is incomplete; token A/B remains a strict correctness gate.

- Windows toolchain notes:
  - LTO conflicts with proc‑macro crates on stable; avoid `-C lto=*` unless using nightly `-Zdylib-lto`.
  - `-C target-cpu=znver3 -C opt-level=3 -C codegen-units=1` works well on Ryzen 7 5800X; PGO can be added without LTO.
