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

- SPOB discipline:
  - All SIMD loops maintain forward-only advancement and use tail scalar for remainder; SPOB shadow asserts monotonic byte advancement.

- Parser robustness for real-world files:
  - Minimal Pratt parsing ( + - * / and parens ) captures many arithmetic-heavy patterns.
  - Do not assert strict forms (e.g., let without ident) during perf work; fallback to treating keywords as identifiers in expressions to avoid panics and to keep advancing.
  - Swallow top-level IIFEs `(function(){...})(...)` as a single expression statement; emit EmptyStatement for leading `;` to align with OXC top-level shape on UMD patterns (e.g., moment.js).

- Structural hashing strategy (bootstrapping phase):
  - Treat ExpressionStatement as kind-only in the structural hash to keep diff signal useful while the expression AST is incomplete; token A/B remains a strict correctness gate.

- Windows toolchain notes:
  - LTO conflicts with proc‑macro crates on stable; avoid `-C lto=*` unless using nightly `-Zdylib-lto`.
  - `-C target-cpu=znver3 -C opt-level=3 -C codegen-units=1` works well on Ryzen 7 5800X; PGO can be added without LTO.
