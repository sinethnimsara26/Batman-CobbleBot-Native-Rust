# HQ Rendering Round 3 — 3× Presentation Surface Results

Status: **implementation + CI complete**

Branch: `hq-rendering-phase3-3x-surface`

Parent implementation branch: `hq-rendering-phase2-surfaces`

## Goal

Round 3 creates the first real high-resolution presentation path while keeping
all gameplay, collision, camera and asset timelines in the original 600×400
logical coordinate system.

The runtime now renders through two surfaces:

- logical/base scratch: **600×400**
- HQ presentation: **1800×1200**

The 1800×1200 surface is the bitmap presented to Windows.

## Architecture implemented

### Logical renderer

The known-good renderer still produces the exact same 600×400 frame used by
Rounds 1 and 2.

No gameplay coordinate is multiplied by three.

### HQ promotion

The complete logical frame is promoted to 1800×1200 with a pixel-center
bilinear kernel.

For exact 3× scaling, the mapping is intentionally constructed so:

```text
HQ(3*x + 1, 3*y + 1) == logical(x, y)
```

for every logical pixel.

That gives CI a strict no-drift invariant while the two samples between logical
pixel centers are interpolated smoothly.

### Windows runtime

`App` owns both:

```text
base_fb = 600×400
fb      = 1800×1200
```

The normal playable executable now uses the HQ path by default.

The 1800×1200 DIB continues through Round 1's GDI HALFTONE presentation path.

## Headless support added

New command:

```
--render-smoke-hq <directory>
```

This executes the same Level 1A gameplay-smoke sequence as the legacy path but
writes 1800×1200 screenshots.

Legacy `--render-smoke` remains 600×400 and is still SHA-gated against the
renderer-repair baseline.

## Performance telemetry added

New command:

```
--benchmark-render <iterations> <output.json>
```

It measures a real Level 1A gameplay frame after asset loading/warm-up and
records:

- mean render time
- p50
- p95
- maximum
- logical/HQ dimensions
- framebuffer memory

The benchmark measures the 600×400 render plus 3× promotion. It does not include
asset loading.

## CI validation

### Run #54

Run ID: `35756682509`

Head:

`e1536b6637be63ec95f09ba456bf6b99e122a8d0`

Result: **PASS**

Validated:

- full original asset/fidelity pipeline
- Round 1 HALFTONE contract
- Round 2 surface abstraction
- Round 3 HQ architecture
- release compilation
- legacy smoke
- HQ smoke
- legacy pixel identity
- 1800×1200 HQ dimensions
- complete logical-center alignment

Initial benchmark:

```json
{
  "mean_ms": 29.9884,
  "p50_ms": 29.7506,
  "p95_ms": 31.2549,
  "max_ms": 33.6474,
  "framebuffer_bytes": 9600000
}
```

This was below the hard 40 ms / 25 Hz tick budget, but above the planning
target of <25 ms mean.

### Bilinear optimization

The proof kernel initially recomputed integer division/clamping inside all
2.16 million HQ pixel iterations.

It was optimized without changing the sampling equation:

- horizontal source indices and weights are precomputed once per frame
- row offsets and vertical weights are computed once per output row
- the hot inner loop performs only indexed reads + weighted RGB arithmetic

### Run #55

Run ID: `35757314806`

Head:

`9756bf80b6782859975fc6430569071ce72e4c7b`

Result: **PASS**

Optimized benchmark:

```json
{
  "mode": "hq3x",
  "iterations": 60,
  "logical_width": 600,
  "logical_height": 400,
  "hq_width": 1800,
  "hq_height": 1200,
  "mean_ms": 23.9818,
  "p50_ms": 23.7194,
  "p95_ms": 24.0813,
  "max_ms": 28.7709,
  "framebuffer_bytes": 9600000
}
```

Mean render time improved by approximately **20.0%**.

The optimized kernel therefore meets the original non-hard targets:

- mean < 25 ms: **PASS**
- p95 < 35 ms: **PASS**
- max < 40 ms: **PASS on this runner**

## Memory

Framebuffers only:

- 600×400 logical: 960,000 bytes
- 1800×1200 HQ: 8,640,000 bytes
- combined: **9,600,000 bytes (~9.16 MiB)**

No HQ Batman/UI asset memory has been added yet.

## Visual / alignment gates

Round #55 CI verifies all 15 HQ smoke frames:

- exact size: **1800×1200**
- aspect ratio: **3:2**
- matching filename/state set with legacy smoke
- every one of the 600×400 logical pixel centers survives exactly at
  `(3*x+1,3*y+1)`

Result:

**15 / 15 HQ frames passed complete logical-center alignment.**

The legacy 600×400 smoke set also still passes the Round 2 exact SHA-256 gate
against the renderer-repair baseline.

## Optimization pixel identity

The Run #54 and Run #55 HQ visual-smoke archives were compared locally.

Result:

- same 15 filenames: yes
- identical SHA-256 screenshots: **15 / 15**
- mismatches: **0**

The performance optimization changed no HQ pixels.

## Manual visual inspection

The 15-frame HQ smoke contact sheet was inspected after Run #54.

Observed:

- no layer displacement
- no camera drift
- no HUD displacement
- no reappearance of renderer-repair artifacts
- tutorial/title/fade sequencing remains coherent
- Batman and world remain registered correctly

At this stage all source assets are still 1×. The improvement comes from
filtered 3× presentation, not from newly rerasterized Batman/text assets.

Therefore Round 3 is a foundation round, not the final sharpness round.

## Windows artifact

Validated Run #55 artifact:

`Batman-CobbleBot-Level1A-Round3-HQ3x-Windows-x64-Optimized.zip`

ZIP SHA-256:

`4a66cc6cb0b4bb535ed49cdc0d9f97f30e08f8aaa49eaf9276b1365c8b9d532f`

Executable:

`batman_cobblebot_native.exe`

EXE size:

`4,290,560 bytes`

EXE SHA-256:

`e8925deb41b0b00005895a489c3617ef674fc091cc537bf7b00f7b46e4bbd0cc`

## Round 3 verdict

3× presentation architecture: **PASS**

Legacy rendering regression gate: **PASS**

HQ dimensions: **PASS**

Logical-coordinate alignment: **PASS**

25 Hz render-time budget on CI runner: **PASS**

Planned aspirational timing targets after optimization: **PASS**

Gameplay/collision/camera changes: **none**

## Next phase

Round 4 / Phase 4 is the sprite-pack format upgrade:

- introduce `BCBFRM02`
- store explicit logical pixel scale
- preserve `BCBFRM01` compatibility
- make the asset layer capable of distinguishing 1× and 3× frame packs

That format gate comes before the first true 3× Batman rerasterization.
