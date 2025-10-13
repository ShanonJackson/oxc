# oxc_codegen Bare-Metal Optimization Strategy

## 1. Current Architecture Snapshot

### 1.1 Pipeline entry and pass management
- `Codegen::build` now computes an estimated capacity, primes the emission buffer in `prepare_pass`, and prints the AST in a single pass while optionally wiring up the sourcemap builder.【F:crates/oxc_codegen/src/lib.rs†L226-L236】【F:crates/oxc_codegen/src/lib.rs†L442-L458】

- `FastBuffer` owns a single aligned `Vec<u8>` and caches the most recent byte and scalar so hot lookbacks avoid copying while writes memcpy straight into spare capacity.【F:crates/oxc_codegen/src/fast_buffer.rs†L16-L123】【F:crates/oxc_codegen/src/fast_buffer.rs†L216-L246】
- Specialized helpers (e.g., `print_str_escaping_script_close_tag`) perform SIMD-assisted scanning of literals for `</script` while delegating the final write back through the buffer abstraction.【F:crates/oxc_codegen/src/lib.rs†L289-L399】

### 1.3 Emission pipeline organization
- `fast_gen` exports many `emit_*` entry points that downcast the `Codegen` reference to the proper lifetime and hand off to implementation functions; top-level emission routes through `StatementEmitter` for statements and a large `match` for expressions prioritizing hot cases.【F:crates/oxc_codegen/src/fast_gen.rs†L1782-L1854】【F:crates/oxc_codegen/src/fast_gen.rs†L1863-L1982】
- Binary and logical expressions are handled iteratively via `BinaryExpressionVisitor`, which maintains its own stack inside the shared `Codegen` state to avoid recursion and tracks parentheses/wrapping decisions with context bitflags.【F:crates/oxc_codegen/src/binary_expr_visitor.rs†L1-L176】
- Context-dependent restrictions (forbid `in`, forbid calls, TypeScript extensions) are represented via the `Context` bitflag type passed down through emitters.【F:crates/oxc_codegen/src/context.rs†L8-L71】

### 1.4 Comment and annotation management
- Comments are pre-filtered into a `CommentsMap` that buckets by attachment key, maintains a contiguous backing store, and lazily returns pointer/length pairs when nodes request their leading/trailing trivia.【F:crates/oxc_codegen/src/comment.rs†L14-L172】
- Emission-time utilities pull comment blocks, handle indentation/newline rules, and extract legal/annotation comments for separate reporting when options request them.【F:crates/oxc_codegen/src/comment.rs†L360-L437】

### 1.5 Source map generation
- `SourcemapBuilder` mirrors esbuild’s design: it snapshots the original source, maintains per-line offset tables with optional column lookups, and records generated positions on every token addition while caching the last lookup for near-linear traversals.【F:crates/oxc_codegen/src/sourcemap_builder.rs†L9-L197】

## 2. Bottlenecks and Optimization Opportunities

1. **Heuristic capacity planning** – `estimate_output_capacity` still relies on coarse ratios of source length and comment counts; pathological mixes can overshoot and force late reallocations, undermining the single-allocation goal.【F:crates/oxc_codegen/src/lib.rs†L425-L439】
2. **Identifier spacing metadata** – `print_space_before_identifier` continues to query `FastBuffer::last_char` on non-ASCII tails, paying for UTF-8 decode instead of using planner-supplied lookback bits.【F:crates/oxc_codegen/src/lib.rs†L495-L521】【F:crates/oxc_codegen/src/fast_buffer.rs†L112-L123】
3. **Vector growth still amortized** – The aligned allocator still falls back to `reserve` when the estimator undershoots, so large literals or comment bursts can induce multiple reallocations in the hot path.【F:crates/oxc_codegen/src/fast_buffer.rs†L216-L246】【F:crates/oxc_codegen/src/comment.rs†L360-L383】
4. **Comment map slow paths** – The `CommentsMap` maintains sorted buckets and supports insertion into the middle of the backing array, incurring `Vec::insert` shifts for out-of-order keys and extra pointer chasing during lookups despite ultimately emitting comments in linear order per span.【F:crates/oxc_codegen/src/comment.rs†L132-L263】
5. **Large emitter match tables** – `emit_expression_impl` branches across dozens of variants at runtime; even though ordering favors hot nodes, each call still pays match dispatch and repeated `Context` adjustments instead of using precomputed dispatch data or flattened loops for the most common patterns.【F:crates/oxc_codegen/src/fast_gen.rs†L1863-L1982】
6. **Binary expression visitor bookkeeping** – Even with the inline stack, the visitor recomputes precedence and context bits on every turn; we can precompute wrapping decisions during planning to shrink the hot loop further.【F:crates/oxc_codegen/src/binary_expr_visitor.rs†L166-L228】
7. **Source map token updates per write** – Each mapping recomputes generated positions from the output slice and performs binary/linear searches through line tables; repeated small token writes create observable overhead when source maps are enabled.【F:crates/oxc_codegen/src/sourcemap_builder.rs†L104-L197】
8. **String escaping pipeline** – The hybrid scalar/SIMD search for `</script` reallocates via the general buffer path, meaning the optimized scan still ends with redundant tail updates and byte copying through `FastBuffer` instead of writing directly to the destination pointer.【F:crates/oxc_codegen/src/lib.rs†L289-L399】

## 3. Roadmap to Bare-Metal Throughput

### Stage 0 – Instrumentation and guardrails
- Wire up Criterion microbenchmarks and real-world macro benchmarks (bundler outputs, JSX-heavy React trees, TS declaration files) to capture allocations, CPU cycles, and branch miss rates for both single- and two-pass modes.【F:crates/oxc_codegen/src/lib.rs†L229-L247】
- Add optional counters inside `FastBuffer` and `SourcemapBuilder` to record reallocation counts, tail shifts, and mapping frequency, gated behind a debug flag to avoid production cost.【F:crates/oxc_codegen/src/fast_buffer.rs†L224-L244】【F:crates/oxc_codegen/src/sourcemap_builder.rs†L104-L197】

### Stage 1 – Buffer and capacity planning
- Replace the full-featured measurement pass with a lightweight planner that only walks the AST to tally literal lengths, static tokens, and comment bytes without executing formatting branches, producing an exact or tightly bounded capacity plan and per-node offsets.【F:crates/oxc_codegen/src/fast_gen.rs†L1799-L1854】【F:crates/oxc_codegen/src/comment.rs†L360-L437】
- Continue evolving the emitter toward a `FastWriter`: the tail cache has been eliminated in favor of cached byte/char metadata, but we still need planner-provided lookback flags so the runtime never decodes UTF-8 on the hot path.【F:crates/oxc_codegen/src/fast_buffer.rs†L16-L123】【F:crates/oxc_codegen/src/fast_buffer.rs†L216-L246】
- Align large allocations to cache-line multiples (e.g., 256-byte bumps) and keep a fallback to dynamic growth only when measurement was skipped (very small files) to guarantee single-allocation emission.

### Stage 2 – Traversal and dispatch tightening
- Collapse the `emit_expression_impl` mega `match` into tiered emitters: one specialized hot-path loop for identifiers/member chains/binary expressions and a secondary dispatch table for rare cases, reducing branch fan-out inside the hottest loop.【F:crates/oxc_codegen/src/fast_gen.rs†L1863-L1982】
- ✅ Replace `BinaryExpressionVisitor`’s heap-backed stack with an inline array plus spill buffer so common cases stay on the stack without giving up deep-expression correctness.【F:crates/oxc_codegen/src/binary_expr_visitor.rs†L16-L200】
- Precompute context flag transitions per node during planning so emission can rely on table lookups instead of recomputing bitwise combinations at runtime.【F:crates/oxc_codegen/src/context.rs†L32-L71】

### Stage 3 – Comment and annotation streaming
- Convert `CommentsMap` into a compact array of `(offset, len)` ranges sorted by start position, produced once during planning; emission can then linear-scan the array alongside AST traversal without binary searches or `Vec::insert` shifts.【F:crates/oxc_codegen/src/comment.rs†L132-L263】
- Materialize legal/annotation comment payloads into a side buffer during planning, so runtime emission only copies byte slices into the final result or the `legal_comments` return list without re-parsing content.【F:crates/oxc_codegen/src/comment.rs†L360-L437】

### Stage 4 – Source map batching
- Delay sourcemap token insertion by buffering `(generated_offset, original_pos, name_id)` tuples in a contiguous array during emission and feed them to `SourcemapBuilder` in a single pass, amortizing line/column searches.【F:crates/oxc_codegen/src/sourcemap_builder.rs†L104-L197】
- Memoize UTF-16 column translations for ASCII-only spans via the planning metadata to avoid redundant line-table lookups when multiple adjacent tokens map to the same line.

### Stage 5 – String and identifier emission
- Move script-close-tag scanning and general literal escaping into specialized SIMD kernels that operate on the destination pointer directly, writing escaped segments with `ptr::copy_nonoverlapping` and updating the output length manually to bypass the generic buffer path.【F:crates/oxc_codegen/src/lib.rs†L289-L399】
- Precompute identifier spacing requirements (does previous token end in identifier char?) during emission using per-node metadata or lookahead bits so that `print_space_before_identifier` no longer consults shared tail state.【F:crates/oxc_codegen/src/lib.rs†L495-L512】

### Stage 6 – Feature completeness and validation
- Mirror all public configuration knobs by threading them through the new planner/emitter, ensuring comment filtering, JSX detection, and TypeScript flags continue to match existing semantics before removing the legacy path.【F:crates/oxc_codegen/src/lib.rs†L88-L173】【F:crates/oxc_codegen/src/fast_gen.rs†L1799-L1854】
- Extend integration tests to byte-compare outputs between legacy and new engines and to validate sourcemap parity; keep a runtime flag to fall back to the existing emitter during rollout.

### Stage 7 – Rollout and continuous tuning
- Ship the new engine behind a feature gate or environment flag, collect telemetry on allocation counts and throughput across real workloads, and iterate on CPU-specific tuning (e.g., `target_feature` gated AVX2 for literal scanning) where benchmarks show measurable wins.
- Retire redundant code paths (old measurement logic, tail buffer maintenance) once confidence is high, reducing maintenance surface while keeping a debug-only compatibility mode for regression hunting.

This roadmap converts today’s flexible emitter into a tightly planned two-stage engine that allocates exactly once, minimizes branchy dispatch, and batches expensive work, unlocking the “bare metal” budget without sacrificing the correctness guarantees that downstream crates rely on.
