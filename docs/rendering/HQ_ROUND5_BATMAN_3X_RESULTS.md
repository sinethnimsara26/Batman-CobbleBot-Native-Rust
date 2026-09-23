# HQ Rendering Round 5 — True 3× Batman Results

Status: **implementation + CI + artifact inspection complete; manual live-window gate pending**

Branch: `hq-rendering-phase5-batman-3x`

Parent implementation branch: `hq-rendering-phase4-pack-v2`

Draft PR: #8 — **HQ Rendering Phase 5: true 3x Batman rerasterization**

Validated implementation head:

`5439b26f1a97f7e1a9bedab1d8ae1a99462e1901`

Successful CI:

- Run #60
- Run ID: `35835569502`
- Result: **PASS**

## Goal

Round 5 fixes the most visible moving gameplay asset first: Batman.

The 600×400 gameplay coordinate system, camera, collision, input order, animation timing,
checkpoint logic and legacy renderer remain unchanged. Only the HQ presentation path gets
a separately rerasterized player.

## Build pipeline

The recovered Batman state baker now exposes the planned controls explicitly:

```text
--output-scale 3
--supersample 2
--downsample lanczos
```

The HQ build therefore renders the same recovered Flash geometry at a temporary 6× raster,
then downsamples in premultiplied-alpha `RGBa` space with Lanczos to the final 3× raster.
Filtering premultiplied color avoids dark/colored fringes from transparent pixels.

The legacy 1× pack is still built independently and remains the regression oracle.

## Frame/timeline contract

Both packs contain the same true child-animation timelines:

- frames: **279**
- state ranges/counts: unchanged
- missing bitmap IDs: **0**
- unsupported fills: **0**

The HQ manifest preserves the same recovered base placement scale and records:

```text
output_scale = 3
supersample = 2
downsample = lanczos
logical_pixel_scale = 3.0
```

HQ anchors are exactly proportional to the legacy registration anchor before per-frame
alpha cropping.

## Packed runtime assets

The frame packer alpha-crops every frame and stores the corrected per-frame anchor in the
BCBFRM record. This is important because the common bake canvas is intentionally much
larger than the visible player.

Measured Run #60 sizes:

| Metric | Legacy 1× | HQ 3× |
| --- | ---: | ---: |
| Packed PNG container | 757,404 B (~0.72 MiB) | 16,033,618 B (~15.29 MiB) |
| Runtime decoded RGBA | 16,268,932 B (~15.52 MiB) | 148,601,548 B (~141.72 MiB) |
| Common bake canvas | 324×253 | 970×759 |

Combined legacy + HQ decoded player memory:

```text
164,870,480 bytes ≈ 157.23 MiB
```

This is safely below the plan's 500 MiB stop condition.

### Run #59 accounting failure

Run #59 (`35835102565`) successfully completed the full 1× + 6×→3× asset reconstruction,
but the first memory gate incorrectly summed the un-cropped temporary bake canvases and
reported ~913 MiB.

That was a validation bug, not a runtime asset problem. The existing packer already cropped
each frame. Run #60 corrected the gate to budget the actual cropped records Rust decodes,
while keeping the 500 MiB limit unchanged.

## Runtime composition

The HQ playing path is now split at Batman's original depth:

```text
600×400 world pass (without Batman/UI)
    ↓
pixel-center bilinear promotion to 1800×1200
    ↓
true 3× Batman, copied 1:1
    ↓
current 1× foreground HUD/tutorial/title/fades promoted above Batman
    ↓
HALFTONE Win32 presentation
```

This avoids the incorrect alternative:

```text
blurry Batman baked into base
+ sharp Batman drawn over it
```

so there is no second low-resolution Batman underneath the HQ sprite.

Title and instructions screens retain the exact Round-3 full-frame path.

## Legacy regression proof

The new smoke harness adds one explicit `04_walk.png` checkpoint, producing 16 frames total.

All **15 historical legacy screenshots** still match the renderer-repair Run #45 SHA-256
baselines byte-for-byte.

Result:

```text
15 / 15 historical screenshots exact
0 historical mismatches
1 new walk checkpoint
```

Therefore the Round-5 refactor changed no legacy gameplay/render-state pixels.

## HQ smoke

Run #60 produced:

- 16/16 HQ screenshots
- exact dimensions: **1800×1200**
- aspect ratio: **3:2**
- successful stand/landed/walk/run/jump/glide/kick/held-punch/tutorial/exit coverage

## Batman A/B visual gates

CI generated seven dedicated comparisons:

- stand
- walk
- run
- jump
- glide
- punch
- kick

Each sheet compares:

```text
legacy 1× → bilinear 3×
vs.
true SWF reraster 3×
```

Manual artifact inspection after Run #60 found the HQ side materially cleaner in the
cowl/face, chest emblem, belt, gloves/boots, cape contours and diagonal animation edges.

Full-scene smoke inspection also found:

- no visible duplicate/blurry Batman underneath the HQ player
- no obvious alpha halo around Batman
- no obvious anchor drift in inspected run/jump/punch frames
- tutorial/HUD layers remain above Batman at the expected depth

The HUD/tutorial remain visibly softer than Batman by design; those are Phase 6/7 targets.

## Performance

Run #60, 60 HQ iterations:

```json
{
  "mean_ms": 28.8978,
  "p50_ms": 27.3637,
  "p95_ms": 35.3268,
  "max_ms": 42.2266,
  "framebuffer_bytes": 9600000
}
```

Compared with Round 4, true-HQ player composition plus separate foreground promotion adds
render cost. Mean and p95 remain below the game's 40 ms / 25 Hz tick duration. One maximum
sample exceeded 40 ms, so this remains worth watching in later rounds, but the run does not
show a consistent 40 ms budget miss.

## Tests

Run #60 passed:

1. canonical SWF SHA verification
2. full asset reconstruction
3. original collision/control-geometry gates
4. 279-frame player recovery gate
5. CXFORM/alpha gates
6. Round-1 HALFTONE contract
7. Round-2 render-surface contract
8. Round-3 HQ-surface contract
9. Round-4 BCBFRM01/02 compatibility contract
10. Round-5 true-3× Batman contract
11. Rust sprite-pack tests — **3 passed / 0 failed**
12. Windows x64 release compilation
13. 16-frame legacy smoke
14. 16-frame HQ smoke
15. historical legacy SHA gate — **15/15 exact**
16. HQ dimension/timing gate
17. seven Batman A/B comparison sheets
18. all artifact uploads

## Windows artifact

GitHub artifact:

`Batman-CobbleBot-Level1A-Fidelity-Windows-x64`

Downloaded Round-5 artifact ZIP SHA-256:

`77d3abe35eb8525298a6e171af5d3af2345bb74c5f244163980dc9847df7b61c`

Executable:

`batman_cobblebot_native.exe`

EXE size:

`20,335,104 bytes`

EXE SHA-256:

`e16764a4870b88ca1fac958f3914db16cff2c8d462850c04ba9fc99e0a4f4c4b`

## Round 5 verdict

True 3× SWF Batman rerasterization: **PASS**

279-frame/state preservation: **PASS**

Scale metadata / proportional registration: **PASS**

Runtime memory budget: **PASS**

Legacy 1× regression: **PASS (15/15 historical hashes)**

Windows build: **PASS**

HQ smoke: **PASS (16/16)**

A/B visual inspection: **material improvement observed**

Performance stop condition: **not triggered** (mean/p95 < 40 ms; one max sample > 40 ms)

## Manual gate

Per the execution plan, Round 5 stops here for the user's live-window test at the same large
viewport that exposed the original pixelation.

Do not begin Phase 6 until Batman's live-window appearance is accepted.

## Next phase after approval

Phase 6 rebuilds the HUD/text at true 3×:

- HUD shell
- BATARANG
- HEALTH
- SCORE
- lives x
- score/life/Batarang numeric glyphs

The goal is to remove the obvious remaining quality mismatch between the now-sharp Batman
and the still-soft HUD/text.
