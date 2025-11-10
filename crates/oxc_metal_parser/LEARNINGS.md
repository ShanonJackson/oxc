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

- SPOB discipline:
  - All SIMD loops maintain forward-only advancement and use tail scalar for remainder; SPOB shadow asserts monotonic byte advancement.

- Parser robustness for real-world files:
  - Minimal Pratt parsing ( + - * / and parens ) captures many arithmetic-heavy patterns.
  - Do not assert strict forms (e.g., let without ident) during perf work; fallback to treating keywords as identifiers in expressions to avoid panics and to keep advancing.

