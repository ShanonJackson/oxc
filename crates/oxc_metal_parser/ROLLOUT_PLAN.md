# OXC Metal Parser — Drop‑In Rollout Plan

Goal: ship `oxc_metal_parser` as a selectable, single‑pass, SPOB parser that produces a valid OXC AST, with strict correctness gates and a safe fallback to `oxc_parser`.

## Readiness Check

- Correctness gates (must all pass):
  - AST parity vs `oxc_parser` on representative corpora (fixtures + mixed repo).
  - Token A/B parity across backends (scalar vs AVX2).
- Perf baselines on Ryzen 7 5800X (Zen 3):
  - Non‑minified (`moment.js`): ~5.55–5.90 cy/B.
  - Minified (`three.js`): ~8.30–8.35 cy/B (with prefetched tuning).
- Stability: No hangs/panics on large inputs; examples show `shape_ok` and `norm_ok`.

## Scope for v0 Drop‑In

- JS subset first (no TS/JSX initially):
  - Expressions: literals, identifiers, binary/logical, parens, calls, member access.
  - Statements: expression/variable (let/const/var), blocks, if, for/while (dummy or minimal), return/break/continue.
  - Modules: import/export parse (shallow; defer deep semantics).
  - Strings, numbers, comments, hashbang.
- Single‑pass SPOB maintained; regex literals deferred to Phase 1 (division by default).

## Integration Plan

- Feature flag + runtime switch:
  - Cargo feature: `metal_parser` (off by default).
  - Runtime env/CLI: `OXC_PARSER=metal` or `--parser=metal`.
  - NAPI: optional flag; default remains `oxc_parser`.
- Adapter: thin wrapper mapping `metal_parser::parse_program` → `oxc_ast::Program<'a>`.
- Safety fallback: on error/panic, log and fall back to `oxc_parser` transparently.

## Correctness Gate (CI)

- Fast harness job (averaged timings):
  - Run `examples/correctness` with `METAL_CHECK_REPEAT=5` on:
    - `crates/oxc_metal_parser/fixtures/moment.js`
    - `crates/oxc_metal_parser/fixtures/three.js`
    - A small mixed directory (e.g., `tasks/coverage/babel/small`).
  - Gate on: `shape_ok=all`, `norm_ok=all`, token A/B ok. Record cpB.
- Conformance (longer): shadow run across a curated subset of test262/babel comparing AST kind histograms and structural hash (subset‑aware).

## Blocking Grammar Tasks (Phase 1)

- Regex vs division:
  - Parser tracks `REGEX_OK`; tokenizer decides `/.../` vs `/` with no look‑behind.
- Template literals:
  - Backtick scanning with `${` nesting; extend string scanner to backtick + nested braces.
- Minimal ASI:
  - Keep current “emit on `;`, `}`, EOF” until explicit rules; add restricted ASI where required (return/break/continue/throw lines).
- Identifier escapes:
  - Keep ASCII fast path; add slow path for `\uXXXX`; otherwise fall back to `Other` (flag for Phase 2).

## Perf Guardrails

- Defaults (Zen 3):
  - AVX2 kernels: STRING + DIGITS on; WS + IDENT off.
  - Prefetch default: 128B; allow `METAL_PREFETCH=96` for minified bundles.
- Tokenizer:
  - Punctuation LUT; block‑local scans for short numbers/idents within 32B; fallback to SIMD for long tokens.
  - Avoid fused local WS skip on minified inputs (did not consistently help).

## Rollout Steps

1. Add adapter + feature flag; default off.
2. Wire CLI/NAPI flag and `OXC_PARSER=metal` env guard.
3. Add CI job invoking `examples/correctness` (repeat=5) and failing on shape/norm mismatches.
4. Enable opt‑in in the oxc binary and NAPI; keep fallback to `oxc_parser`.
5. Incrementally extend grammar (regex, templates, ASI), each gated by the single‑command correctness harness.

## Fuzzing & Safety

- Add cargo‑fuzz target that compares `metal` vs `oxc_parser` AST shape/hash on random inputs.
- Periodically run sanitizer builds on correctness corpora.

## Developer Commands

- Single‑file correctness (averaged):
  - `METAL_CHECK_REPEAT=5 cargo run --release -p oxc_metal_parser --example correctness -- crates/oxc_metal_parser/fixtures/moment.js`
  - `METAL_CHECK_REPEAT=5 cargo run --release -p oxc_metal_parser --example correctness -- crates/oxc_metal_parser/fixtures/three.js`
- Minified A/B hint:
  - `METAL_CHECK_REPEAT=5 METAL_PREFETCH=96 cargo run --release -p oxc_metal_parser --example correctness -- crates/oxc_metal_parser/fixtures/three.js`

## Risks & Mitigations

- Grammar gaps (regex, templates, ASI): delay drop‑in for some repos.
  - Mitigation: keep feature off by default; fallback to `oxc_parser` on error.
- Unicode identifiers/escapes: ASCII fast path for perf; slow path added as needed.
- Maintenance: keep fast paths hot and isolated; rely on the harness and conformance tests for regression catching.

