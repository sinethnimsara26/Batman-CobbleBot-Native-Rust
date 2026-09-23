# HQ Rendering Plan

This folder contains the research and implementation blueprint for improving the visual resolution of the native CobbleBot port after the Level 1A renderer-repair pass.

## Documents

- [HQ_RENDERING_RESEARCH.md](HQ_RENDERING_RESEARCH.md) — diagnosis, external research, alternatives and decisions.
- [HQ_RENDERING_ARCHITECTURE.md](HQ_RENDERING_ARCHITECTURE.md) — selected 3× presentation architecture and asset strategy.
- [HQ_RENDERING_EXECUTION_PLAN.md](HQ_RENDERING_EXECUTION_PLAN.md) — phase-by-phase file changes, CI gates, budgets and rollback plan.
- [HQ_ROUND1_HALFTONE_RESULTS.md](HQ_ROUND1_HALFTONE_RESULTS.md) — executed Phase 1 result, CI proof and manual acceptance gate.
- [HQ_ROUND2_SURFACE_RESULTS.md](HQ_ROUND2_SURFACE_RESULTS.md) — executed Phase 2 render-surface refactor and exact legacy pixel-identity proof.
- [HQ_ROUND3_3X_SURFACE_RESULTS.md](HQ_ROUND3_3X_SURFACE_RESULTS.md) — executed Phase 3 1800×1200 HQ surface, alignment proof, and render benchmark.
- [HQ_ROUND4_PACK_V2_RESULTS.md](HQ_ROUND4_PACK_V2_RESULTS.md) — executed Phase 4 BCBFRM02 metadata, V1 compatibility, and mismatch-rejection proof.
- [HQ_ROUND5_BATMAN_3X_RESULTS.md](HQ_ROUND5_BATMAN_3X_RESULTS.md) — executed Phase 5 true 3× Batman rerasterization, runtime-memory proof, visual A/B gates, and Windows validation.
- [HQ_ROUND6_HUD_3X_RESULTS.md](HQ_ROUND6_HUD_3X_RESULTS.md) — executed Phase 6 true 3× HUD/device-font/digit reconstruction, exact field-registration proof, A/B inspection, and Windows validation.
- [HQ_ROUND7_OVERLAYS_3X_RESULTS.md](HQ_ROUND7_OVERLAYS_3X_RESULTS.md) — executed Phase 7 true 3× tutorial/title/fade reconstruction, stage-clipped supersampling, lazy memory control, and Windows validation.

## Decision summary

The project will **not** use AI upscaling as the canonical fix.

The source SWF still contains vector geometry, fonts, placements, transforms and real animation timelines, so the faithful solution is to preserve the 600×400 gameplay simulation while moving foreground/vector presentation to a higher-resolution supersampled pipeline.

Selected starting point:

- gameplay: 600×400, unchanged.
- base scenery: current 600×400 composition.
- HQ presentation: 1800×1200 (3×).
- important vector-derived assets: rebuild at 3× with build-time supersampling.
- final Win32 stretch: GDI HALFTONE.
- large world tile/background packs: keep current resolution initially and evaluate only after Batman/UI are fixed.
- Direct2D: fallback if software HQ misses performance/quality goals.
- AI super-resolution: optional bitmap-only experiment, never the default source of truth.

No runtime code is changed by this planning branch.
