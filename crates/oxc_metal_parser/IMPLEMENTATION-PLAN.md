# IMPLEMENTATION PLAN — OXC Single‑Pass‑Over‑Bytes (SPOB) Parser

Goal: Build a single‑pass, SIMD/assembly‑accelerated JavaScript/TypeScript parser that produces a valid OXC AST with strict SPOB invariants, smallest possible instruction footprint, and zero byte rescans. Performance and correctness are the only constraints.

This document defines deliverables, invariants, milestones, ISA policy, verification gates, and performance KPIs to enable fast feedback while staying on the maximal-performance path.

## Scope & Targets
- Output: Valid OXC AST identical in semantics to `oxc_parser` (script/module, JS/TS, Annex B where required).
- Platforms: Windows 11 x86_64 (MSVC ABI) first; Linux/macOS later.
- ISA tiers:
  - Tier 0: Scalar fallback (correctness safety net; not tuned).
  - Tier 1: AVX2 baseline (Haswell/Zen2+). Primary dev target.
  - Tier 2: AVX‑512 (Ice Lake/Zen4/5) with VBMI/VPOPCNTDQ when available.
  - Tier 3: ARM NEON/SVE2 (follow‑on).
- Hot path implementation policy: Hand‑rolled assembly where it buys measurable wins (micro‑kernels and final classifier); Rust `std::arch` intrinsics acceptable as scaffolding in the first thin slice to accelerate feedback. Promote to inline `asm!` or external `.asm` once a block >10% time.

## Non‑Negotiable Invariants
- SPOB: Each byte is fetched exactly once by the classifier/scan path. No rescans. Bounded token speculation only (ring buffer of fixed size; no byte reads on rewind).
- Zero heap on the hot path: Use arenas/bump allocators only. No per‑token allocations.
- Parser disambiguation: Regex vs division via parser‑owned `REGEX_OK` bit; ASI via carried line‑terminator flag.
- AST representation: Packed index graph with 32‑bit node IDs; spans as 32‑bit start + 32‑bit length (use 64‑bit mode for large files if needed).

## Early Feedback Strategy (Vertical Slice First)
Deliver a minimal end‑to‑end slice quickly to measure cycles/byte and validate SPOB semantics before broadening grammar coverage.

### M0 — Thin Slice (Days 1–3)
- 64‑byte SIMD classifier (AVX2) for ASCII classes, line terminators, backslashes; UTF‑8 fast path (reject on malformed).
- Tokens: identifiers (ASCII), decimal numbers (no exponents), punctuators `{}()[];,+-*/=`, strings `"'` (support `\\` only).
- Micro‑kernels (assembly‑first): string scanner (unescaped‑quote via mask + carry). Template/regex omitted in M0.
- Parser: Pratt for `+ - * / =` and precedence; statement dispatcher for blocks, variable declarations (`let/const/var` minimal), expression statements.
- AST: allocate with arenas; spans recorded; output deterministic AST hash + node count.
- CLI example: `cargo run -p oxc_metal_parser --example spob -- file.js` prints cycles/byte, AST hash.
- SPOB debug shadow: assert strictly monotonic byte advancement; log kernel entry [start,end) in debug.

M0 Exit Criteria:
- Correct AST for a curated micro‑suite (basic JS files) matches `oxc_parser` on the supported subset.
- Median cycles/byte measured on small corpus; SPOB assertions hold; no rescans.

### M1 — Strings/Templates/Regex + ASI (Week 1)
- Add micro‑kernels (assembly‑first):
  - String: finalize escapes (including `\u{}` minimal decode on scalar tail only).
  - Template: `${` boundaries, `TPL_DEPTH`, resume expression mode; unescaped backtick handling.
  - Regex: scan until `/`, suppress inside `[...]`, ignore `\/`; flags read via LUT.
- Classifier completeness for escape masks, CR/LF/LS/PS, and CRLF across block boundaries.
- Token micro‑tape (ring 256–512 entries) for bounded speculation; rewind parser state from tokens only.
- Parser wires `REGEX_OK`, ASI bit; directive prologue (strict) and hashbang; HTML comments per Annex B.
- Add perf dashboard (CSV): cycles/byte (P50/P95/P99), mispredicts/KiB (if available), AST bytes/byte.

M1 Exit Criteria:
- Pass curated tricky cases (templates in expressions/statements, regex/division ambiguity, ASI) vs `oxc_parser`.
- Perf regression tests stable; no SPOB violations.

### M2 — Numeric & Unicode Correctness (Week 2)
- Numeric literals: separators, bin/oct/hex, exponents, legacy octal (sloppy), BigInt suffix.
- Unicode identifiers (start/continue) including escapes; full line terminator semantics.
- Directive prologue full semantics (no escapes/comments interleaved).
- Finish UTF‑8 validator fast path; bail to error on malformed bytes without rescans.

M2 Exit Criteria:
- Substantial slice of test262 (lexing + parsing core) passes; differential runs on selected repos (React, Lodash) with AST equivalence on shared features.

### M3 — Full JS (Week 3)
- Complete remaining grammar coverage for script/module, Annex B behaviors, and TS goal toggles as required for AST parity.
- Parser computed‑goto layout and dispatch tables; finalize token kinds and flags.
- Promote classifier to hand assembly (AVX2) with tuned register allocation; begin AVX‑512 specialization.

M3 Exit Criteria:
- test262 full pass for JS goal (script/module), including Annex B. Snapshot only failures, if any.

### M4 — Optimization & AVX‑512 (Week 4)
- PGO/LTO builds; place kernels + dispatch in a 32–64 KiB region for uop cache residency.
- AVX‑512 path with VBMI/VPOPCNTDQ; runtime dispatch from a cold shim.
- Benchmark against esbuild/oxc on large repos; drive P95/P99 down (layout/mispredict work).

M4 Exit Criteria:
- Lock in cycles/byte budget; demonstrate consistent wins against baselines on macro workloads.

## ISA & Dispatch Policy
- Runtime feature detection: select function pointers once at startup; hot loops remain monomorphic.
- Inline `asm!` for Windows MSVC ABI kernels, or external `.asm` via `cc` crate with `build.rs` (MASM/clang‑cl integrated assembler); document clobbers and calling convention.
- Keep scalar fallback for correctness and non‑supported CPUs.

## State & Data Layout
- Mode/Depth bitfield (single GPR on hot path):
  - Bits: IN_STR, IN_TPL, IN_REGEX, IN_CMT_SL, IN_CMT_ML, REGEX_OK, STRICT, ASI_LT.
  - Small integers: `TPL_DEPTH`, depth counters for paren/brace/brack (modulo small ints for quick checks).
- Token ring (256–512): `{ kind, start, len, flags, num_payload/hash }` stored in a small cache‑resident buffer; wraparound with no allocations.
- AST arenas: contiguous regions per node category; node IDs are 32‑bit indices; spans are `{ start:u32, len:u32 }` (upgrade to 64‑bit mode for very large files via feature flag).

## Micro‑Kernels (Assembly‑First)
- String: vector search for unescaped terminator via precomputed masks; scalar tail only for escape decode; forward‑only pointer motion.
- Template: like string; toggle `TPL_DEPTH` on `${` and `}`; return to parser at expression boundaries without leaving template mode.
- Regex: maintain `IN_CLASS`; suppress matches inside `[]`; skip `\/`; read flags; tokenization semantics respect `u/v` where it affects escapes.
- Comments: `//` via next line terminator mask; `/*…*/` via `*` then `/` pairing; EOF error detection without rescans.

## Correctness Gates
- AST parity vs `oxc_parser` on the covered subset per milestone.
- test262: partial early (M2), full JS goal by M3; include Annex B.
- Fuzzers: grammar fuzzer + regex literal fuzzer; sanitize on cold debug builds (UBSan/ASan) only.
- SPOB shadow: assert monotonic byte advancement; optional logging of kernel [start,end) ranges.

## Performance Workflow & KPIs
- KPIs per commit:
  - cycles/byte (P50/P95/P99) per ISA tier
  - branch mispredicts / KiB (if measurable)
  - L1I/L1D MPKI snapshots on profiling runs
  - AST bytes per source byte; node count per KB
- Windows timing harness:
  - Pin thread (`SetThreadAffinityMask`), raise priority, warm caches.
  - Use `__rdtsc` or `QueryThreadCycleTime`; normalize by input bytes.
  - Disable DVFS/turbo for stable numbers when needed; otherwise track frequency.

## Testing & CI Flow
- Fast lane (on every run): curated micro fixtures covering tricky constructs; cycles/byte smoke test; SPOB assertions.
- Conformance lane (manual or nightly): test262 script/module; Babel fixtures for regex/templates/comments; differential AST on corpora (React, TS server, Lodash).
- Snapshots: record only failing test classes; review via `cargo insta review` where applicable.

### Differential AST Feedback Loop
- Tooling: add `examples/diff.rs` that parses a file with `oxc_parser` and `oxc_metal_parser`, then compares ASTs.
- Modes:
  - `shape`: node kinds, arity, child ordering only.
  - `normalized`: shape + semantic fields with canonicalization (numeric forms, regex body/flags, identifier escapes, CRLF/LF normalization for templates/strings).
  - `strict`: exact equality including spans and raw literal text.
- CLI: `cargo run -p oxc_metal_parser --example diff -- file.js --mode shape|normalized|strict --limit 10`.
- Milestone gates:
  - M0: shape equality for supported subset; emit structural hash and compare.
  - M1: normalized mode passes curated tricky suite; CI fast lane fails on regressions.
  - M2: corpus diffs (React/Lodash/TS server slices) produce CSV of mismatch classes and paths; trend must improve.
  - M3: strict mode passes test262 (script/module); strict mismatches block the milestone.

## Windows 11 Focus Points
- Toolchain: MSVC (clang‑cl acceptable). Ensure inline `asm!` uses the LLVM integrated assembler; validate clobbers.
- External assembly: if chosen, unify MASM/clang‑cl paths in `build.rs` and ship pre‑assembled objects where necessary.
- CPU dispatch: CPUID checks for AVX2/AVX‑512 sub‑features (`vbmi`, `vpopcntdq`, `bw`, `vl`, `dq`).

## AST Integration with OXC
- Phase A: emit existing `oxc_ast` node shapes directly using OXC allocators; avoid intermediate CST.
- Phase B: optionally compress internal node layout (index graph) without changing the public AST API; ensure both modes produce identical AST externally.
- Spans and comments: carry through exactly; ASI and directive prologue semantics must mirror `oxc_parser`.

## Staged ISA Rollout
1) AVX2 baseline + scalar fallback.
2) AVX‑512 specialization (VBMI/VPOPCNTDQ) with identical results and runtime dispatch.
3) NEON/SVE2 port with the same SPOB invariants.

## Deliverables per Milestone
- Code: example binary `spob` reporting AST hash, node count, bytes, cycles/byte; ISA features at startup.
- Artifacts: CSV perf logs per corpus and ISA; failing snapshot summaries.
- Docs: short milestone note in this file with current KPIs, open correctness gaps, next targets.

## Risk Controls
- Highest risk: regex literal semantics, numeric literal edges, directive prologue subtleties. Gated by tests; feature‑flag advanced cases until verified.
- Bounded speculation overflow: provide a slow but SPOB‑preserving path that extends the ring buffer without byte rescans (or hard error with diagnostic in debug builds).
- Parallelism: any intra‑file parallel mode that pre‑scans to place split points violates strict SPOB; keep off by default and document explicitly.

## Acceptance Criteria (Per Phase)
- M0: End‑to‑end AST on subset; SPOB assertions pass; cycles/byte reported.
- M1: Strings/templates/regex/ASI/directives correctness vs `oxc_parser` on curated suite; perf stable/improving.
- M2: Numeric/Unicode correctness; partial test262 pass; differential equivalence on selected repos.
- M3: Full test262 (JS goal); AST parity preserved; classifier in assembly.
- M4: PGO/LTO gains; AVX‑512 wins; macrobenchmarks show sustained improvement vs baseline.

## Checklists

### M0
- [ ] AVX2 classifier (ASCII classes, escapes, UTF‑8 fast path)
- [ ] String kernel (asm), `\\` escape only
- [ ] Minimal Pratt + statements
- [ ] Arena AST, spans
- [ ] CLI example + cycles/byte
- [ ] SPOB shadow assertions

### M1
- [ ] Template kernel (asm) with `${` boundaries
- [ ] Regex kernel (asm) with class handling and flags
- [ ] ASI bit + directive prologue + hashbang + HTML comments
- [ ] Token ring buffer (256–512)
- [ ] Perf CSVs + basic dashboard

### M2
- [ ] Numeric literals completeness
- [ ] Unicode identifiers + escapes
- [ ] Full line terminators (CR/LF/LS/PS) and CRLF boundaries
- [ ] UTF‑8 validator finalized

### M3
- [ ] Full grammar coverage (script/module, Annex B)
- [ ] Computed‑goto parser layout
- [ ] Classifier promoted to asm (AVX2); begin AVX‑512 specialization
- [ ] test262 full JS pass

### M4
- [ ] PGO/LTO + hot region layout (32–64 KiB)
- [ ] AVX‑512 path + dispatch
- [ ] Macrobenchmarks + P95/P99 analysis

---

This plan prioritizes early, measurable progress (M0) while hard‑locking SPOB and correctness invariants. Hand‑rolled assembly is introduced where it yields concrete wins, starting with micro‑kernels and graduating the classifier once validated and profiled.
