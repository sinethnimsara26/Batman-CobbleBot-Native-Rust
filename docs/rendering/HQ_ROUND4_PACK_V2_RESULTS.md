# HQ Rendering Round 4 — BCBFRM02 Sprite-Pack Results

Status: **implementation + CI complete**

Branch: `hq-rendering-phase4-pack-v2`

Parent implementation branch: `hq-rendering-phase3-3x-surface`

## Goal

Round 4 makes sprite-frame resolution explicit before any true high-resolution
Batman/UI assets are introduced.

The runtime must be able to distinguish:

- legacy 1× packs
- future 2×/3×/other packs

without guessing from image dimensions.

## New container format

### BCBFRM01

Existing format remains supported.

Header:

```text
magic[8]    = BCBFRM01
frame_count u32
frames...
```

Logical pixel scale is implicitly:

```text
1.0
```

### BCBFRM02

New format:

```text
magic[8]              = BCBFRM02
frame_count u32
logical_pixel_scale f32
frames...
```

Frame records remain compatible with the previous record layout.

## Packer changes

`reverse-engineering/tools/pack_sprite_frames.py` now supports:

```
--format-version {1,2}
--logical-pixel-scale <float>
```

Defaults:

```text
format version = 2
logical pixel scale = 1.0
```

Validation:

- scale must be positive.
- BCBFRM01 rejects any requested scale other than 1.0.
- JSON sidecars record both format and logical pixel scale.

## Rust runtime changes

Introduced:

```rust
pub struct SpriteSet {
    pub frames: Vec<SpriteFrame>,
    pub logical_pixel_scale: f32,
}
```

`SpriteSet` dereferences to `[SpriteFrame]`, preserving existing indexing,
`.len()`, `.is_empty()`, and slice-coercion behavior with minimal renderer
churn.

The decoder now accepts:

- `BCBFRM01` → scale 1.0.
- `BCBFRM02` → reads the embedded finite positive f32 scale.

Additional integrity checks now reject:

- unsupported magic.
- truncated v2 headers.
- invalid scale values.
- truncated frame headers.
- truncated PNG payloads.
- trailing data after the declared frame set.

## Expected-scale enforcement

Asset loading goes through:

```text
decode_sprite_frames_expected(...)
```

which compares the pack's declared scale against the scale expected by that
runtime asset.

A mismatch fails with an explicit error such as:

```text
sprite pack scale mismatch for scale_probe: expected 1, got 3
```

This is important for Round 5: a 3× Batman pack cannot silently be consumed by
the old 1× placement path.

## Live compatibility strategy

The normal CI build intentionally contains both formats.

### BCBFRM02 / scale 1.0

Normal packs including:

- Batman
- pickup
- level title
- fades
- tutorial overlay

are generated using the new v2 default.

Batman validation:

```text
format = BCBFRM02
frame_count = 279
logical_pixel_scale = 1.0
```

### BCBFRM01 / implicit scale 1.0

The live Batarang projectile pack is intentionally generated as
`BCBFRM01`.

The normal executable loads this pack during `Assets::load()`, so backward
compatibility is exercised in the real application, not just by a Python parser.

### BCBFRM02 / scale 3.0 probe

CI also builds:

```text
assets/scale_probe.bin
```

from the recovered Batarang frame set with:

```text
format = BCBFRM02
logical_pixel_scale = 3.0
```

It exists specifically to test scale metadata and mismatch rejection.

## Rust compatibility tests

Run #58 executed three Rust tests:

```text
legacy_v1_defaults_to_one ... ok
v2_reports_embedded_scale ... ok
v2_scale_mismatch_panics ... ok
```

Result:

```text
3 passed
0 failed
```

Therefore:

- V1 decoding: **PASS**
- V2 decoding: **PASS**
- explicit 3× metadata: **PASS**
- mismatched-scale rejection: **PASS**

## CI history

### Run #57

Run ID: `35820168989`

The Round 4 pack-format source/data contract passed.

The job then failed because the initial test command used:

```
cargo test ... --lib
```

This package is binary-only and has no library target.

No renderer/asset implementation failure occurred.

### Run #58

Run ID: `35820368342`

Head:

`c9f393e199c53c7ea3960f5e78c0cc2d65876604`

Result: **PASS**

Passed:

1. canonical SWF verification.
2. recovered asset reconstruction.
3. control-geometry rejection.
4. 279-frame Batman gate.
5. CXFORM / alpha gates.
6. Round 1 HALFTONE contract.
7. Round 2 render-surface contract.
8. Round 3 1800×1200 HQ contract.
9. Round 4 V1/V2 pack metadata contract.
10. Rust sprite-pack compatibility tests.
11. Windows x64 release build.
12. legacy smoke.
13. HQ 3× smoke.
14. exact legacy SHA regression gate.
15. HQ logical-center alignment gate.
16. render benchmark.
17. artifact uploads.

## Visual regression proof

Round 4 HQ smoke was compared to the optimized Round 3 HQ smoke.

Result:

- filenames: same 15
- byte/SHA-identical PNGs: **15 / 15**
- mismatches: **0**

Round 4 is therefore visually inert, as intended.

## Performance

Run #58 HQ benchmark:

```json
{
  "mean_ms": 21.4581,
  "p50_ms": 20.9503,
  "p95_ms": 24.9973,
  "max_ms": 28.6215,
  "framebuffer_bytes": 9600000
}
```

The pack metadata work does not add render-time scaling cost.

## Windows artifact

Artifact:

`Batman-CobbleBot-Level1A-Round4-PackV2-Windows-x64.zip`

ZIP SHA-256:

`4ea216184e4b680463fe6815a6d40ccf90aad21b05f8bdc3b2be05472a8500ec`

Executable:

`batman_cobblebot_native.exe`

EXE size:

`4,293,632 bytes`

EXE SHA-256:

`5c73ae47105be1026cd03c43e1d036a468272af2344fc8b9fc8eea1e7455b83c`

## Round 4 verdict

BCBFRM02 format: **PASS**

BCBFRM01 compatibility: **PASS**

Explicit scale metadata: **PASS**

Scale mismatch rejection: **PASS**

Legacy/HQ visual regression: **PASS**

Gameplay/collision/camera changes: **none**

## Next phase

Round 5 is the first true asset-resolution upgrade:

- extend the Batman baker with explicit HQ output controls.
- rerasterize all 279 true Batman animation frames from the original SWF at
  3× presentation resolution.
- use supersampling + high-quality downsampling.
- pack them as BCBFRM02 with `logical_pixel_scale = 3.0`.
- composite the new Batman frames 1:1 onto the 1800×1200 surface.
- keep scenery and UI on the existing Round 3 base path for this round.
- produce legacy-vs-HQ Batman comparison screenshots and inspect animation
  registration before continuing to HUD/text.
