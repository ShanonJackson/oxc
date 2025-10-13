# CodegenFast Rewrite Plan

## Goals

- Replace the existing `Codegen` printing core with a near zero-allocation, single-threaded emitter that targets "bare metal" performance while remaining byte-for-byte compatible with the current output.
- Maintain feature parity with every path in `gen.rs`, including comment handling, JSX, TypeScript constructs, source maps, minification, annotation comments, and indentation semantics.
- Provide a drop-in API surface so `Codegen::build` can route to the new engine without observable behavior changes for downstream crates.

## Current Architecture Review

### Dispatch Model

`gen.rs` defines the `Gen` and `GenExpr` traits and implements them for every AST node. Each node recursively calls `print` helpers on the shared `Codegen` state, so emission interleaves tree traversal with output writes. 【F:crates/oxc_codegen/src/gen.rs†L21-L158】

### Codegen State and Buffering

`Codegen` owns mutable state for the traversal (indentation, comment queues, operator tracking) and writes into `CodeBuffer`, a Vec-backed UTF-8 builder that grows on demand. `build` reserves the original source length, builds comments, optionally initializes the source map builder, and emits the program in a single pass. 【F:crates/oxc_codegen/src/lib.rs†L80-L235】

`CodeBuffer` itself is a general-purpose string builder that resizes as needed, supports indentation utilities, and enforces UTF-8 correctness. It performs dynamic allocations whenever capacity is exceeded. 【F:crates/oxc_data_structures/src/code_buffer.rs†L1-L193】

### Output Semantics

The generator manages numerous behaviors besides raw token emission:

- Leading and trailing comments are interspersed by `print_comments_at` calls before or after statements. 【F:crates/oxc_codegen/src/gen.rs†L104-L160】
- Special treatment is required for directives, JSX, annotation comments (e.g., `/* @__PURE__ */`), and script closing tag escaping. 【F:crates/oxc_codegen/src/gen.rs†L69-L101】【F:crates/oxc_codegen/src/lib.rs†L265-L320】
- `Codegen` tracks operator precedence and prints parentheses through `BinaryExpressionVisitor` integration.
- Source map emission piggybacks on traversal, updating `SourcemapBuilder` as text is appended.

The architecture favors clarity but results in frequent virtual dispatch, shared mutable state, and repeated branching on options across every print site.

## Constraints for the Rewrite

1. **Bit-for-bit output parity:** Every formatting decision, including whitespace and comment ordering, must match the current printer to avoid test churn.
2. **Drop-in integration:** Existing public APIs (`Codegen::build`, option structs, sourcemap handling) must continue to function without new callers migrating immediately.
3. **Safety with selective `unsafe`:** We can use `unsafe` to eliminate bounds checks or tighten loops but must encapsulate it carefully and audit for UB.
4. **Performance focus:** Minimize allocations, branch mispredictions, and function call overhead. Accept platform specialization (SIMD) behind runtime detection when it meaningfully improves throughput.

## Proposed Architecture: `CodegenFast`

### High-Level Structure

- Introduce a new `codegen_fast` module containing `CodegenFast<'a>` and helper types.
- Retain the public `Codegen` type as façade. `Codegen::build` dispatches to `CodegenFast::build` when the aggressive mode flag is enabled (default once stabilized).
- Mirror the existing `CodegenOptions`, comments map, sourcemap builder, and state fields, but reorganize them into tightly packed structs to improve cache locality.

### Two-Pass Strategy

1. **Measurement Pass:** Traverse the AST without writing output to compute an upper bound for emitted bytes, gather offsets for sourcemap segments, and classify hot vs. cold emission paths. The pass records per-node metadata (e.g., directive literal lengths, comment lengths) in a compact side table indexed by node ids or span positions.
2. **Emission Pass:** Allocate a single `Vec<u8>` with `with_capacity_exact(total_bytes)` (rounding up to cache-line boundaries). Emit directly into the slice using unchecked writes guarded by debug assertions. Because lengths are known, we can avoid bounds checks and repeated option lookups.

If profiling shows measurement overhead outweighs benefits for small files, guard it with a heuristic (e.g., bypass for inputs < 4 KiB).

### Buffering and Writing

- Replace `CodeBuffer` with a specialized `FastBuffer` that exposes `push_ascii_unchecked`, `push_bytes_unchecked`, and `push_str_lossy` methods operating on a raw pointer into the reserved slice. The buffer tracks length via `usize` and only falls back to checked variants when debug assertions are enabled.
- Provide SIMD-accelerated routines for common escapes (`</script`, newline scanning, identifier validation). These can use `std::arch` intrinsics with `cfg` guards and runtime CPUID gating.
- Inline the majority of emission helpers (`#[inline(always)]`). Mark rare branches (error recovery, cold comment paths) with `#[cold]` to help branch prediction.

### Traversal Engine

- Convert the recursive `Gen` trait implementations into specialized functions grouped by node category (statements, expressions, literals). Instead of trait dispatch, use `match` directly inside `CodegenFast` with manual loop unrolling for hot constructs (binary expressions, call chains).
- Precompute operator precedence tables in const arrays and rely on iterative loops to avoid stack allocations (`BinaryExpressionVisitor` can become an array-based stack within `CodegenFast`).
- For constructs requiring lookahead (e.g., `for` headers, arrow function bodies), store minimal metadata from the measurement pass to skip repeated condition checks.

### Comments and Source Maps

- During measurement, compute the exact byte contribution of each comment (including indentation) so emission can copy slices with a single unchecked write.
- Source map spans can be accumulated while writing by consulting precomputed offsets. We can store them in a contiguous `Vec` and hand them to `SourcemapBuilder` in bulk to reduce per-node overhead.

### Integration Plan

1. **Scaffold Module:** Create `codegen_fast` module with a feature flag and mirror of existing options. Implement a no-op adapter that defers to current `Codegen` to keep tests green while scaffolding.
2. **Measurement Infrastructure:** Implement AST walker to compute byte budgets, comment metadata, and sourcemap checkpoints using existing traversal logic as reference.
3. **FastBuffer Implementation:** Write the unchecked buffer with debug assertions, SIMD feature detection, and fallbacks for non-supported architectures.
4. **Statement Emission:** Port high-frequency statement printers (blocks, expression statements, variable declarations, functions) to `CodegenFast`, validating byte-for-byte parity against current output using existing integration tests.
5. **Expression Emission:** Port expression printers with emphasis on binary precedence, call chains, literals, JSX, and TypeScript nodes. Reuse measurement metadata to avoid dynamic branching.
6. **Comments and Directives:** Ensure comment ordering, annotation comments, and directive quoting follow the same semantics using precomputed data.
7. **Source Map and Legal Comments:** Reimplement sourcemap hooks and legal comment extraction, validating with existing sourcemap tests.
8. **Performance Validation:** Benchmark against current codegen on representative workloads. Profile to confirm reductions in allocations, branch mispredictions, and CPU cycles. Tune SIMD thresholds and inline/cold annotations accordingly.
9. **Gradual Rollout:** Behind a cargo feature or environment flag, allow opt-in usage. Once stable, switch the default path and keep the legacy generator available for fallback/testing until confidence is high.

### Drop-In Compatibility Checklist

- Mirror public methods (`with_options`, `with_source_text`, `with_scoping`, `with_private_member_mappings`) on `CodegenFast`.
- Preserve mutation semantics for `needs_semicolon`, `print_next_indent_as_space`, and JSX tracking to keep external behavior identical.
- Keep string escaping helpers (`print_str_escaping_script_close_tag`) accessible from both engines to avoid duplication.
- Ensure tests comparing emitted strings pass without updates; add byte-level assertions for regression coverage during migration.

## Next Steps

- Prototype the measurement pass to validate estimated buffer sizes and confirm we can compute accurate byte counts for comments and directives.
- Draft benchmarks (e.g., bundler-sized JS files, TS with JSX) to quantify baseline vs. target performance.
- Identify hotspots in current `gen.rs` via profiling to prioritize rewrite order (likely expressions, loops, class bodies).
- Begin translating high-frequency printers into the new module while keeping the legacy generator as correctness reference.

This plan sets the stage for an aggressive rewrite that honors drop-in requirements and existing semantics while enabling low-level optimizations tailored to modern CPU architectures.
