# HQ Rendering Round 7 — True 3× Foreground Overlay Results

Status: **implementation + CI + artifact inspection complete**

Branch: `hq-rendering-phase7-overlays-3x`

Parent: `hq-rendering-phase6-hud-3x`

PR: **#10 — HQ Rendering Phase 7: true 3x tutorial, title and fades**

Validated implementation head:

`e0e6bf34ee36def8d11a7591cb9fea9a465b777a`

Successful CI:

**Run #71 / `35860790582`**

## Goal

Round 7 upgrades the remaining user-facing gameplay foreground timelines to
true 3× presentation assets while keeping the original 600×400 simulation,
25 Hz timing and exact legacy renderer behavior.

Covered timelines:

- tutorial overlay — symbol 737 — 149 frames.
- level title — symbol 746 — 100 frames.
- fade-in — symbol 747 — 43 frames.
- fade-out — symbol 34 — 41 frames.

## Final build strategy

All four HQ timelines now use the same final pipeline:

```text
canonical SWF
→ resolve display list / masks / embedded DefineText / CXFORMWITHALPHA at 6×
→ clip rasterization to the portion that can intersect the real 600×400 stage
→ premultiplied-alpha Lanczos downsample
→ final 3× timeline
→ BCBFRM02 with logical_pixel_scale = 3.0
```

The stage-clipping step is important for the fade symbols. Their native 1×
union canvas is approximately 3988×2660 even though only the 600×400 stage can
be visible. A naive 6× native-bounds bake attempts enormous temporary images.
Round 7 instead rasterizes only the visible stage intersection while preserving
the original symbol placement.

The new generic tool:

`reverse-engineering/tools/finalize_hq_timeline.py`

performs premultiplied-alpha Lanczos filtering only after Flash display-list,
text and color-transform semantics have been resolved.

## Runtime strategy

The four foreground timelines use `LazySpriteSet`.

Only the currently needed compressed PNG frames are decoded, with a two-frame
cache per timeline.

This is used for both:

- the original 1× overlay packs.
- the new true-3× overlay packs.

Making the legacy overlays lazy is pixel-neutral but important for memory:
the legacy fade canvases are approximately 3988×2660, so eagerly decoding all
84 fade frames would consume multiple gigabytes of RGBA memory.

HQ frames are copied 1:1 into the 1800×1200 presentation surface. The normal
Round-7 HQ foreground path no longer promotes those four 1× timelines with the
old bilinear helper.

## Timeline preservation

CI verified the exact source frame counts:

```text
tutorial 737 : 149
title    746 : 100
fade-in  747 : 43
fade-out  34 : 41
```

Legacy and HQ manifests also preserve:

- symbol IDs.
- source bounds.
- animation-label sequence.
- no unsupported fills.
- no missing bitmap IDs.
- logical timeline timing.

All HQ packs are `BCBFRM02` with embedded scale `3.0`.

## CXFORM / alpha proof

The title's known source alpha sequence is validated **before filtering**,
where the original Flash values remain exact:

```text
frame 44 = 204
frame 45 = 153
frame 46 = 102
frame 47 = 51
frame 50 = 51
```

Lanczos has negative lobes, so edge pixels can exceed the source maximum after
filtering without changing the intended timeline opacity. Run #71 therefore
validates exact CXFORM values on the 6× prefilter raster and verifies the final
filtered fade ordering separately.

Observed final maxima:

```text
44 → 240
45 → 180
46 → 119
47 → 60
50 → 60
```

The order remains correct and frames 47/50 remain equal.

## Asset sizes and decoded-memory budget

### Tutorial

```text
legacy canvas: 654×108
HQ canvas:    1800×318
legacy pack:  150,760 B
HQ pack:      5,196,597 B
```

### Level title

```text
legacy canvas: 310×50
HQ canvas:     924×144
legacy pack:   103,811 B
HQ pack:       3,051,434 B
```

### Fade-in

```text
legacy canvas: 3988×2660
HQ canvas:     1800×1200
legacy pack:   856,288 B
HQ pack:       1,069,084 B
```

### Fade-out

```text
legacy canvas: 3988×2660
HQ canvas:     1800×1200
legacy pack:   813,281 B
HQ pack:       1,012,572 B
```

Conservative two-frame-cache upper bounds from Run #71:

```text
legacy overlay caches: 170,418,336 B
HQ overlay caches:      40,203,648 B
all overlay caches:    210,621,984 B
Batman eager RGBA:     164,870,480 B
-----------------------------------
known combined upper:  375,492,464 B
```

That is approximately **358 MiB**, below the execution plan's **500 MiB**
stop condition.

This estimate is deliberately conservative because a normal HQ process does
not fill every legacy and HQ overlay cache simultaneously.

## Legacy regression

Converting the legacy overlay packs from eager to lazy decoding did not change
their pixels.

Run #71 result:

```text
15/15 historical legacy smoke screenshots match Run #45 by exact SHA-256.
```

The extra Round-5 walk checkpoint also remains present.

Therefore the memory change is a storage/decode-policy change only, not a
legacy rendering change.

## Performance

Run #71, 60 HQ iterations:

```json
{
  "mean_ms": 28.2634,
  "p50_ms": 27.2935,
  "p95_ms": 33.9755,
  "max_ms": 38.5938,
  "framebuffer_bytes": 9600000
}
```

Every reported sample in this run remained below the game's **40 ms / 25 Hz**
frame duration.

## Visual inspection

The Round-7 A/B artifacts show a clear improvement over legacy 1× → bilinear
3× promotion.

The most obvious gains are:

- cleaner `THE GOTHAM BANK` title letter edges.
- substantially cleaner tutorial lettering such as
  `PRESS THE SPACE BAR TO JUMP`.
- cleaner thin border geometry around tutorial panels.
- preserved title transparency/fade behavior.

Full-scene HQ smoke was also inspected. Title and tutorial overlays remain at
their expected stage locations and depth above gameplay/HUD.

Fade transitions continue to cover the stage correctly.

## Iteration notes

Round 7 exposed several useful failure modes before the final green run.

- The first 6× fade attempt used native union bounds and hit Pillow's
  decompression-bomb safety limit because the temporary canvas was enormous.
- A direct-3× fade workaround avoided that allocation but was superseded by the
  better stage-clipped 6× solution.
- Alpha validation was moved to the prefilter raster because Lanczos edge
  ringing can change postfilter maximum alpha while preserving the intended
  opacity sequence.
- The first lazy-pack unit test incorrectly assumed the existing 3× scale
  probe contained one frame; it is the five-frame Batarang probe. The test was
  corrected.
- Legacy overlays were then moved to lazy decoding as well so the total decoded
  working-set estimate satisfies the project's memory stop condition.

Run #71 is the first final pass containing all of those fixes together.

## CI result

Successful run:

**#71 / `35860790582`**

Passed:

1. canonical SWF SHA-256 verification.
2. complete reproducible asset rebuild.
3. recovered asset/control-geometry gates.
4. Round-1 HALFTONE contract.
5. Round-2 render-surface contract.
6. Round-3 1800×1200 HQ contract.
7. Round-4 BCBFRM02 compatibility contract.
8. Round-5 true-3× Batman contract.
9. Round-6 true-3× HUD/text contract.
10. Round-7 true-3× foreground timeline contract.
11. Rust sprite-pack tests — 4/4.
12. Windows x64 release compilation.
13. gameplay/visual smoke.
14. historical legacy SHA regression — 15/15 exact.
15. HQ 1800×1200 smoke.
16. render benchmark.
17. Batman A/B sheets.
18. HUD A/B sheets.
19. foreground overlay A/B sheets.
20. artifact upload.

## Windows artifact

Run #71 Windows artifact:

`Batman-CobbleBot-Level1A-Fidelity-Windows-x64`

Downloaded ZIP SHA-256:

`01c66af2412143346796aa87ceae729cb055f9db8173fe426513ea6f0687bfae`

Executable SHA-256:

`67de40d0a1f6a36725bf4020fa562a53dc51a1af36befeae5bb421c907eb955d`

Executable size:

`30,770,688 bytes`

## Round 7 verdict

Tutorial true 3×: **PASS**

Level title true 3×: **PASS**

Fade-in true 3×: **PASS**

Fade-out true 3×: **PASS**

Embedded DefineText before final filtering: **PASS**

CXFORM/alpha ordering: **PASS**

Original timeline frame counts/timing: **PASS**

BCBFRM02 scale metadata: **PASS**

1:1 HQ compositing: **PASS**

Legacy pixel regression: **PASS**

Known decoded-memory upper bound <500 MiB: **PASS**

60-frame HQ benchmark under 40 ms: **PASS**

Gameplay/camera/collision changes: **none**

## Next phase

Round 8 is the small moving/vector-derived world-object pass:

- pickup symbol 695.
- Batarang/projectile symbol 694.
- moon/small foreground symbols as justified by visual inspection.

The giant city panorama/tile pack remains intentionally deferred.
