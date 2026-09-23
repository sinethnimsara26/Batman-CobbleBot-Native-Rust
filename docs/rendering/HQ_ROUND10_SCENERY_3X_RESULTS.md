# HQ Rendering Round 10 — Depth-Preserved Selective Scenery

Status: **implementation + CI complete**

Branch: `hq-rendering-phase10-scenery-eval`

Parent: `hq-rendering-phase9-dpi-awareness`

PR: **#13 — HQ Rendering Phase 10: selective true-3× scenery**

Validated implementation head:

`875039b694f109e94035c9bf9b706ab0f0332a19`

Successful CI:

**Run #98 / `35881469882`**

## Why this phase exists

After Batman, HUD, tutorial/title/fades and moving world objects were rebuilt at
true 3×, the remaining Level-1A building/window geometry became the most
obvious low-resolution foreground element.

Round 10 therefore followed the execution plan's evidence-first rule: inspect
the level display list, probe the repeated source symbols, model cache cost,
then upgrade only the scenery group that materially benefits.

## Recovered Level-1A scenery structure

Level symbol **152** contains **41** visible frame-0 placements.

The repeated visible scenery symbols are:

- **144** — 6 placements.
- **145** — 6 placements.
- **146** — 14 placements.
- **147** — 12 placements.

Initial 1× versus true-3× probe sheets showed clear edge/detail improvement on
the window/facade geometry.

An important depth discovery changed the first split design: symbol **146** and
symbol **147** interleave in the original Flash display list. Pulling 146 above
a retained 147 layer changed **5,351** legacy pixels in the exact reconstruction
test.

The final design therefore keeps **144–147 together in one selected HQ layer**
so their original internal Flash depth order is preserved. Symbol **14** remains
the low-cost base scenery layer.

## Exact split proof

Before replacing selected scenery with HQ pixels, CI reconstructs the original
1× level using:

```text
retained symbol 14 base
+
selected symbols 144–147 in original internal depth order
```

and compares that reconstruction against the historical full Level-1A tile
layer.

Result:

- changed tiles: **0**
- changed pixels: **0**

This proves the scene decomposition itself does not alter legacy ordering or
appearance.

## HQ asset pipeline

Selected scenery is rebuilt from the verified SWF using:

```text
symbol 152
→ retain children 144–147
→ 6× raster with two-logical-pixel tile overscan
→ premultiplied-alpha Lanczos
→ final 3× pixels
→ transparent crop
→ BCLVT002 pack
```

The overscan prevents independent tile filtering from creating edge seams.

The new **BCLVT002** level-tile format carries:

- explicit logical pixel scale.
- logical tile key.
- transparent crop X/Y offsets.
- PNG payload.

The runtime loader remains backward-compatible with **BCLVT001**.

The packer also verifies that its requested logical scale matches the bake
manifest, preventing incorrectly labelled HQ packs.

## Runtime composition

HQ gameplay now composes scenery as:

```text
legacy backdrop
→ retained 1× scenery base
→ promote opaque base to 1800×1200
→ true-3× selected scenery layer copied 1:1
→ true-3× pickup/Batarang
→ true-3× Batman
→ true-3× HUD/tutorial/title/fades
```

No gameplay, collision, camera, physics, animation timing or audio behavior is
changed.

## Tile/archive results

Legacy full scenery:

- tiles: **168**
- pack: **313,634 B**
- runtime cache: **12 tiles**
- conservative decoded cache upper bound: **11,520,000 B**

HQ retained base:

- tiles: **99**
- pack: **150,420 B**
- runtime cache: **4 tiles**
- decoded cache upper bound: **3,840,000 B**

True-3× selected scenery:

- tiles: **82**
- pack: **1,510,785 B**
- runtime cache: **4 tiles**
- decoded cache upper bound: **34,560,000 B**

The renderer skips zero-coverage edge tile requests so moving across tile
boundaries does not decode invisible neighbors unnecessarily.

## Memory

Known sprite upper bound entering Round 10:

**375,648,100 B**

Conservative combined upper bound including the legacy scenery oracle, HQ base
cache and HQ detail cache:

**425,568,100 B (~405.9 MiB)**

Headroom below the project's 500 MiB stop condition:

**98,719,900 B (~94.1 MiB)**

Memory gate: **PASS**

## Normal HQ performance

Run #98, 60 warmed gameplay renders:

```text
mean  = 26.9488 ms
p50   = 26.7093 ms
p95   = 27.4676 ms
max   = 32.7571 ms
```

The CI requirement is p95 < **40 ms**.

Result: **PASS**

## Scrolling/cache-churn performance

A separate benchmark moves the camera across Level 1A and back in 600-pixel
steps to force real tile-cache turnover instead of measuring only a warmed
stationary frame.

Run #98, **41** scrolling samples:

```text
mean  = 25.2453 ms
p50   = 23.8473 ms
p95   = 30.7005 ms
max   = 33.0491 ms
```

Worst observed sample occurred near world-left **7800**, where denser scenery
caused additional tile work, but it still remained below the 40 ms game tick.

Scrolling p95 <40 ms: **PASS**

## Regression and CI gates

Run #98 passed:

- canonical SWF download + SHA-256 verification.
- resilient retry path for transient source-host failures.
- complete reproducible Level-1A asset reconstruction.
- Round 1–9 contracts.
- exact 1× scenery split/depth reconstruction.
- true-3× BCLVT002 metadata/crop bounds.
- missing-ID and unsupported-fill rejection.
- sprite/tile pack compatibility tests.
- Windows x64 release build.
- historical legacy pixel SHA-256 regression.
- 17 legacy + 17 HQ smoke screenshots.
- normal HQ p95 <40 ms.
- full-level scenery-scroll p95 <40 ms.
- scenery A/B comparison generation.
- all artifact uploads.

## Visual result

The A/B sheets show visibly cleaner window arches, mullions, frame edges and
facade contours than the old 1× layer promoted to 3×.

This is a source-faithful rerasterization from the original SWF geometry, not
AI upscaling.

## Result

Selective scenery source identification: **PASS**

Original 144–147 internal depth preserved: **PASS**

Exact 1× split reconstruction: **PASS — 0 changed pixels**

True 3× scenery pipeline: **PASS**

BCLVT001 compatibility: **PASS**

BCLVT002 scale/crop validation: **PASS**

Decoded-memory ceiling <500 MiB: **PASS**

Stationary HQ p95 <40 ms: **PASS**

Scrolling/cache-churn p95 <40 ms: **PASS**

Legacy renderer pixels unchanged: **PASS**

Gameplay/camera/collision changes: **none**

## Remaining scenery work

The large root/background layers are intentionally still not blindly promoted.

The execution plan's remaining order is:

1. moon / sky edge assets if their softness is still visible.
2. city panorama only if it remains objectionable after that.
3. full-level 3× tiles only as a last resort.

Round 10 demonstrates that selective source-faithful rerasterization gives a
visible improvement while preserving both depth and performance, so future
background work should follow the same measured approach.
