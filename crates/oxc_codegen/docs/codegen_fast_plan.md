# CodegenFast Rewrite Plan

## Goals

- Replace the existing `Codegen` printing core with a near zero-allocation, single-threaded emitter that targets "bare metal" performance while remaining byte-for-byte compatible with the current output.
- Maintain feature parity with every path in `gen.rs`, including comment handling, JSX, TypeScript constructs, source maps, minification, annotation comments, and indentation semantics.
- Provide a drop-in API surface so `Codegen::build` can route to the new engine without observable behavior changes for downstream crates.

## Current Architecture Review

### Dispatch Model

`gen.rs` historically defined the `Gen` and `GenExpr` traits and implemented them for every AST node. Each node recursively called `print` helpers on the shared `Codegen` state, so emission interleaved tree traversal with output writes. 【F:crates/oxc_codegen/src/gen.rs†L21-L158】

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

- Introduce a dedicated `fast_gen` module containing `CodegenFast<'a>` and helper types.
- Retain the public `Codegen` type as façade. `Codegen::build` dispatches unconditionally to `CodegenFast::build`, so callers continue using the legacy API surface while benefiting from the optimized backend.
- Mirror the existing `CodegenOptions`, comments map, sourcemap builder, and state fields, but reorganize them into tightly packed structs to improve cache locality.

### Single-Pass Strategy

1. **Capacity Estimate:** Before emission, compute a lightweight upper bound using the original source length, comment sizes, and constant overheads. The heuristic intentionally over-allocates slightly to avoid second passes without re-walking the AST.
2. **Emission Pass:** Allocate a `Vec<u8>` with the estimated capacity, emit directly into the slice using unchecked writes guarded by debug assertions, and shrink to fit once traversal completes. The single-pass approach keeps branch predictor state warm and avoids replaying the AST for measurement.

### Buffering and Writing

- Replace `CodeBuffer` with a specialized `FastBuffer` that exposes `push_ascii_unchecked`, `push_bytes_unchecked`, and `push_str` methods operating on a raw pointer into the reserved slice. The buffer tracks length via `usize` and only falls back to checked variants when debug assertions are enabled.
- Provide SIMD-ready hooks for future escapes (`</script`, newline scanning, identifier validation) using `std::arch` intrinsics guarded by runtime CPUID checks when profiling demonstrates a win.
- Inline the majority of emission helpers (`#[inline(always)]`). Mark rare branches (error recovery, cold comment paths) with `#[cold]` to help branch prediction.

### Traversal Engine

- Convert the recursive `Gen` trait implementations into specialized functions grouped by node category (statements, expressions, literals). Instead of trait dispatch, use `match` directly inside `CodegenFast` with manual loop unrolling for hot constructs (binary expressions, call chains).
- Precompute operator precedence tables in const arrays and rely on iterative loops to avoid stack allocations (`BinaryExpressionVisitor` becomes an array-based stack within `CodegenFast`).
- For constructs requiring lookahead (e.g., `for` headers, arrow function bodies), compute the necessary metadata on the fly and cache it in the fast state to skip repeated condition checks.

### Comments and Source Maps

- Store comments in a contiguous arena with cursor-tracked buckets so emission can iterate without hashing, clone-free.
- Source map spans piggyback on emission via inline hooks that forward location information to the existing `SourcemapBuilder`, preserving byte-for-byte mapping behavior.

### Integration Plan

1. **Scaffold Module:** Create the `fast_gen` module behind a façade that initially forwards to the legacy implementation so tests stay green during the migration.
2. **FastBuffer Implementation:** Write the unchecked buffer with debug assertions, and integrate it with the façade.
3. **Statement Emission:** Port high-frequency statement printers (blocks, expression statements, variable declarations, functions) to `CodegenFast`, validating byte-for-byte parity against current output using existing integration tests.
4. **Expression Emission:** Port expression printers with emphasis on binary precedence, call chains, literals, JSX, and TypeScript nodes.
5. **Comments and Directives:** Ensure comment ordering, annotation comments, and directive quoting follow the same semantics using the contiguous arena.
6. **Source Map and Legal Comments:** Reimplement sourcemap hooks and legal comment extraction, validating with existing sourcemap tests.
7. **Performance Validation:** Benchmark against current codegen on representative workloads. Profile to confirm reductions in allocations, branch mispredictions, and CPU cycles. Tune SIMD thresholds and inline/cold annotations accordingly.
8. **Gradual Rollout:** Once parity and performance are confirmed, switch the façade to route directly into the fast generator.

### Drop-In Compatibility Checklist

- Mirror public methods (`with_options`, `with_source_text`, `with_scoping`, `with_private_member_mappings`) on `CodegenFast`.
- Preserve mutation semantics for `needs_semicolon`, `print_next_indent_as_space`, and JSX tracking to keep external behavior identical.
- Keep string escaping helpers (`print_str_escaping_script_close_tag`) accessible from both engines to avoid duplication.
- Ensure tests comparing emitted strings pass without updates; add byte-level assertions for regression coverage during migration.

## Next Steps

- Continue profiling to identify remaining SIMD candidates and cold-path cleanups.
- Expand benchmark coverage (e.g., bundler-sized JS files, TS with JSX) to monitor regressions as follow-up optimizations land.
- Document the final architecture and its invariants so future contributors can reason about the unsafe code and buffer guarantees.

This plan captures the architecture implemented in the repository today, highlighting the single-pass emitter and contiguous comment storage while tracking areas for future tuning.
