# HQ Rendering Round 4 — BCBFRM02 Sprite-Pack Results

Status: **implementation + CI complete**

Branch: `hq-rendering-phase4-sprite-pack-v2`

Parent: `hq-rendering-phase3-3x-surface`

## Goal

Round 4 makes sprite resolution explicit before any true HQ sprite is introduced.

Previously, every sprite pack implicitly meant "one packed image pixel equals one logical presentation pixel." That assumption is unsafe once Batman can exist as both 1× and 3× source-derived frames.

Round 4 introduces `BCBFRM02`, adds runtime scale metadata, keeps `BCBFRM01` backward compatibility, and adds explicit asset-slot scale validation.

No image content is intentionally changed in this round.

## Format

### BCBFRM01

Legacy header:

```text
magic[8] = BCBFRM01
frame_count u32
frame records...
```

Runtime interpretation:

```text
logical_pixel_scale = 1.0
```

### BCBFRM02

New header:

```text
magic[8] = BCBFRM02
frame_count u32
logical_pixel_scale f32
frame records...
```

Per-frame payload is unchanged:

```text
width u16
height u16
cropped_anchor_x f32
cropped_anchor_y f32
png_length u32
PNG bytes
```

## Packer changes

`reverse-engineering/tools/pack_sprite_frames.py` now supports:

```text
--logical-pixel-scale <float>
--legacy-v1
```

Default output format is `BCBFRM02`.

The scale must be finite and greater than zero.

`--legacy-v1` is limited to scale 1.0 because BCBFRM01 has no field capable of expressing any other scale.

The JSON sidecar now records:

- format
- frame count
- logical pixel scale
- header byte count
- animation labels
- frame records

## Runtime changes

`src/assets.rs` introduces:

```rust
pub struct SpriteSet {
    pub frames: Vec<SpriteFrame>,
    pub logical_pixel_scale: f32,
}
```

`SpriteSet` dereferences to `[SpriteFrame]`, which preserves the existing indexing and iteration behavior throughout the renderer.

The decoder now accepts:

- `BCBFRM01` → implicit scale 1.0
- `BCBFRM02` → scale read from header

It rejects:

- unknown magic
- truncated v2 header
- non-finite scale
- scale <= 0
- truncated frame record
- frame PNG beyond pack length
- PNG dimensions disagreeing with frame header
- trailing unparsed bytes
- empty frame packs

## Asset-slot scale gate

`Assets::load()` now validates the expected logical scale for every current sprite set.

Current Round 4 expectations are all:

```text
1.0
```

This includes:

- Batman
- pickup
- thrown Batarang
- level title
- fade in
- fade out
- tutorial overlay

A mismatch panics with a clear error:

```text
<asset> logical pixel scale mismatch: expected X, got Y
```

Round 5 can therefore deliberately change only Batman's expected scale from 1.0 to 3.0. Accidentally feeding a 3× pack through old 1× placement math will fail immediately instead of silently rendering at the wrong size.

## Runtime pack inventory from CI

Run #56 generated:

| Pack | Frames | Scale | Packed bytes |
|---|---:|---:|---:|
| batman_frames | 279 | 1.0 | 757,404 |
| pickup_frames | 1 | 1.0 | 484 |
| batarang_frames | 5 | 1.0 | 1,466 |
| level_title_frames | 100 | 1.0 | 103,811 |
| fadein_frames | 43 | 1.0 | 856,288 |
| fadeout_frames | 41 | 1.0 | 813,281 |
| tutorial_overlay_frames | 149 | 1.0 | 150,760 |

Every runtime pack was verified from both:

- raw binary header
- JSON sidecar metadata

## Rust compatibility tests

Three native Windows Rust tests were added and executed in CI.

### Test 1 — BCBFRM01

Builds a complete one-frame BCBFRM01 pack around a real generated PNG and decodes it.

Expected:

- 1 frame
- scale 1.0
- anchors preserved

Result: **PASS**

### Test 2 — BCBFRM02

Builds a complete one-frame BCBFRM02 pack with scale 3.0 and decodes it.

Expected:

- 1 frame
- scale 3.0

Result: **PASS**

### Test 3 — expected-scale rejection

Decodes the same scale-3.0 v2 pack, then sends it to an asset slot requiring 1.0.

Expected:

- clear scale-mismatch panic

Result: **PASS**

CI test summary:

```text
3 passed
0 failed
```

## CI

GitHub Actions Run:

- run number: **#56**
- run ID: `35765517817`
- validated head: `e09b8d645a17577b265e9ace0e677a22197d18f5`
- result: **SUCCESS**

Passed:

1. canonical SWF hash verification
2. all asset reconstruction
3. renderer fidelity gates
4. Round 1 HALFTONE contract
5. Round 2 render-surface contract
6. Round 3 3× surface contract
7. Round 4 BCBFRM02 metadata contract
8. native BCBFRM01/v2 Rust tests
9. Windows release compilation
10. legacy smoke
11. HQ smoke
12. legacy exact-pixel baseline gate
13. HQ logical-center alignment gate
14. performance telemetry
15. artifact upload

## Visual regression proof

### Legacy

Round 4 legacy smoke still passes:

**15 / 15 exact SHA-256 matches against the renderer-repair baseline.**

### HQ

The complete Round 4 HQ smoke set was compared locally against validated Round 3 Run #55.

Result:

- filenames equal: yes
- screenshots compared: 15
- byte/SHA-256 identical: **15 / 15**
- mismatches: **0**

Therefore moving all current sprite assets from BCBFRM01 semantics to BCBFRM02 scale-1.0 metadata changed no output pixels.

## Performance

Run #56 telemetry:

```json
{
  "mode": "hq3x",
  "iterations": 60,
  "logical_width": 600,
  "logical_height": 400,
  "hq_width": 1800,
  "hq_height": 1200,
  "mean_ms": 21.0881,
  "p50_ms": 20.9752,
  "p95_ms": 21.7905,
  "max_ms": 23.5495,
  "framebuffer_bytes": 9600000
}
```

This remains comfortably inside the current renderer budget.

The lower number versus Round 3 should be treated as normal hosted-runner variation; Round 4 is a metadata/decoder change, not a rendering-speed optimization.

## Windows artifact

Artifact:

`Batman-CobbleBot-Level1A-Round4-SpritePackV2-Windows-x64.zip`

ZIP SHA-256:

`84e43db7e29b64f2af8051e1afdad16d60aca2c6ca00209c9259d3f4ea90d3e9`

Executable:

`batman_cobblebot_native.exe`

EXE SHA-256:

`b0a2343a97523c0a96d6e079e93500c839dffd74d034d72b37367267fa8a573d`

## Round 4 verdict

BCBFRM02 writer: **PASS**

BCBFRM01 runtime compatibility: **PASS**

BCBFRM02 runtime compatibility: **PASS**

Explicit scale metadata: **PASS**

Wrong-scale rejection: **PASS**

Legacy visual identity: **PASS**

HQ visual identity versus Round 3: **PASS**

Gameplay/camera/collision changes: **none**

## Next phase

Round 5 is the first true source-resolution upgrade:

**High-resolution Batman**

Planned work:

- extend the Batman baker with explicit HQ output controls
- preserve all 279 recovered child-animation frames
- render source geometry above final resolution
- downsample cleanly to a true 3× Batman pack
- pack Batman as `BCBFRM02 logical_pixel_scale = 3.0`
- change only Batman's runtime expected scale to 3.0
- composite Batman directly into the 1800×1200 surface instead of embedding him in the 600×400 base frame
- keep all other assets at scale 1.0
- produce direct 1×-promoted vs true-3× Batman comparison screenshots
- manually gate registration stability and sharpness before moving on to HUD/text
