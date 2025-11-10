OXC Metal Parser — Build Guide (Windows, Ryzen 7 5800X)

Goal: reliable, repeatable build/run settings that avoid LTO issues on stable and surface the perf knobs we’re tuning (prefetch, WS SIMD).

Key Points

- Do NOT enable LTO on stable Rust for this workspace: proc‑macro crates cannot participate in LTO without nightly `-Zdylib-lto` and builds will fail.
- Use CPU‑tuned codegen flags and keep codegen units low for perf.
- Prefetch distance and whitespace SIMD default are auto‑tuned at runtime; you can override prefetch if you want to A/B.

Per‑Session (PowerShell)

- Disable LTO and set fast build flags:
  - `$env:CARGO_PROFILE_RELEASE_LTO = "false"`
  - `$env:RUSTFLAGS = "-C target-cpu=znver3 -C opt-level=3 -C codegen-units=1"`

- Optional: clear any stray METAL_* overrides:
  - `Remove-Item Env:METAL_* -ErrorAction SilentlyContinue`

- Optional: override prefetch distance (bytes) for A/B; default is 128:
  - `$env:METAL_PREFETCH = "96"`  (try 64, 96, 128, 192)

Persistent Cleanup (if you previously set RUSTFLAGS/LTO globally)

- Clear persistent RUSTFLAGS and restart terminal:
  - `setx RUSTFLAGS ""`
- Ensure Cargo profile doesn’t force LTO; if needed, override per session:
  - `$env:CARGO_PROFILE_RELEASE_LTO = "false"`

Single‑Command Feedback Loop

- Run consolidated correctness + perf check (prints settings line):
  - `cargo run --release -p oxc_metal_parser --example check -- crates/oxc_metal_parser/fixtures/moment.js`
  - You’ll see: `settings ws_simd=on|off prefetch=XXXB`
  - Output shows: shape/norm correctness vs `oxc_parser`, average cy/B, A/B scalar vs AVX2, and token A/B.

Other Useful Commands

- Scalar vs AVX2 selfcheck on a file:
  - `cargo run --release -p oxc_metal_parser --example selfcheck -- crates/oxc_metal_parser/fixtures/moment.js`

- Token stream diff (first mismatch):
  - `cargo run -p oxc_metal_parser --example tokdiff -- crates/oxc_metal_parser/fixtures/larger.js`

Runtime Tuning (auto + overrides)

- Whitespace SIMD: auto‑calibrated once per process. It’s enabled only if an AVX2 kernel is ≥5% faster on this CPU. Override with `METAL_SIMD_WS=1|0` if you need to force it.
- Prefetch distance: default 128 bytes. Override with `METAL_PREFETCH` (bytes). Check prints the effective value.

Why LTO Fails Here (reference)

- Stable Rust cannot apply LTO to `proc-macro` crates. If any dependency includes proc‑macro (e.g., `proc-macro2`, `seq-macro`), enabling `-C lto=*` causes build errors like:
  - `lto cannot be used for proc-macro crate type without -Zdylib-lto`
- Workaround: disable LTO (`$env:CARGO_PROFILE_RELEASE_LTO = "false"`) and keep other perf flags.

