Each byte must:

Be loaded (1 cycle, L1),

Be classified (branch or LUT),

Potentially emitted (store),

Possibly participate in arithmetic (1–2 cycles).

Even a perfect hand-tuned loop needs at least ~4 cycles/byte scalar, or ~2 cpb with SIMD.
That’s the speed-of-light limit for parsing on modern superscalar CPUs.

So:

6–9 cpb → you’re near the scalar saturation ceiling.

4–5 cpb → only possible with SIMD scanning (like simdjson Stage 1).

2–3 cpb → theoretical maximum possible for text parsing before DRAM stalls dominate.
