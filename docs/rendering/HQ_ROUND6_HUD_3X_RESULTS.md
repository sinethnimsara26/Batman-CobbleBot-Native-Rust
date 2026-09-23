# HQ Rendering Round 6 — True 3× HUD and Text Results

Status: **implementation + CI + artifact inspection complete**

Branch: `hq-rendering-phase6-hud-3x`

Parent: `hq-rendering-phase5-batman-3x`

Draft PR: **#9 — HQ Rendering Phase 6: true 3x HUD and text**

Validated implementation head:

`48677a89fdf6f231808a84f19f4aeaf199396f52`

Successful CI:

**Run #62 / `35841687312`**

## Goal

Round 6 replaces the remaining low-resolution gameplay HUD path with a true
3× HUD while preserving the original 600×400 simulation and the exact legacy
renderer as a regression oracle.

Scope is deliberately limited to:

- HUD shell, symbol 716.
- `BATARANG`.
- `HEALTH`.
- `SCORE`.
- `lives x`.
- dynamic Batarang count.
- dynamic score.
- dynamic life count.

Tutorial, level-title and fade overlays remain Round-7 work.

## Build-time HUD reconstruction

The existing HUD recovery already establishes that Flash device font
`_sans` maps to Windows Arial for this target.

Round 6 extends `bake_hud_assets.py` with explicit:

```text
--output-scale
--supersample
--downsample
--suffix
--pack-dir
```

The canonical HQ build is:

```text
symbol 716 rendered at 6×
+ Arial EditText rendered at matching 6× coordinates
→ premultiplied-alpha Lanczos downsample
→ final true 3× HUD
```

Pillow `RGBa` premultiplied-alpha filtering is used so translucent/vector/text
edges do not acquire dark or bright color fringes during downsampling.

The final HQ HUD shell is packed as:

```text
BCBFRM02
frame_count = 1
logical_pixel_scale = 3.0
```

## Runtime path

Legacy mode continues to load and draw the original:

- `hud_base.png`
- `hud_digits_small.png`
- `hud_digits_large.png`

HQ mode loads independent assets:

- `hud_shell_hq.bin`
- `hud_digits_small_hq.png`
- `hud_digits_large_hq.png`

The HQ shell and digit regions are copied **1:1** into the 1800×1200
presentation surface. They are not enlarged from the 1× assets at runtime.

Gameplay/HUD state remains in logical coordinates. Only the renderer converts
the original HUD placement to presentation pixels.

## Registration proof

The three dynamic field centers were compared in stage space.

### Batarangs — field 702

Legacy center:

```text
(211.125, 56.05)
```

HQ center:

```text
(633.375, 168.15)
```

Drift from exact 3×:

```text
(0.0, 0.0)
```

### Score — field 713

Legacy center:

```text
(386.6, 34.975)
```

HQ center:

```text
(1159.8, 104.925)
```

Drift is floating-point noise only, approximately:

```text
(0.0, 1.4e-14)
```

### Lives — field 714

Legacy center:

```text
(313.75, 56.95)
```

HQ center:

```text
(941.25, 170.85)
```

Drift is floating-point noise only.

The Phase-6 acceptance limit is one HQ pixel. Actual measured registration is
effectively exact.

## Visible-resolution proof

The raw PNG canvas cannot be compared with a literal 3× outer-size test because
the native-bounds baker uses fixed transparent padding.

CI therefore measures visible alpha bounds.

Legacy visible HUD bounds:

```text
279 × 77
```

HQ visible HUD bounds:

```text
837 × 233
```

Width is exactly 3×. Height differs from exactly 3× by two pixels because
Lanczos antialias support extends the visible alpha edge, which is expected.

The HUD's partially transparent panel also remains present and is explicitly
validated.

## Asset report

Run #62 produced:

```text
legacy HUD PNG              5,338 bytes
HQ HUD PNG                 73,102 bytes
HQ HUD BCBFRM02 pack       76,656 bytes

legacy small digits PNG     1,372 bytes
HQ small digits PNG         7,881 bytes

legacy large digits PNG     3,075 bytes
HQ large digits PNG        18,524 bytes
```

Build parameters:

```text
output scale = 3
supersample  = 2
temporary    = 6×
downsample   = Lanczos
```

## CI history

### Run #61

Run ID:

`35841159499`

The complete asset rebuild and all Round-1 through Round-5 gates passed.

The new Round-6 field-center checks also reported effectively zero registration
drift.

The job stopped on an incorrect validation assumption that the entire PNG
canvas must have a literal 3× width and height. Native-bounds fixed padding
makes that test invalid.

No rendering implementation failure occurred.

### Run #62

Run ID:

`35841687312`

Validated implementation head:

`48677a89fdf6f231808a84f19f4aeaf199396f52`

Result: **PASS**

Passed:

1. canonical SWF verification.
2. complete asset reconstruction.
3. collision/control-geometry gates.
4. original CXFORM/alpha gates.
5. Round-1 HALFTONE contract.
6. Round-2 surface contract.
7. Round-3 1800×1200 contract.
8. Round-4 sprite-pack compatibility contract.
9. Round-5 true-3× Batman contract.
10. Round-6 true-3× HUD/text contract.
11. Rust sprite-pack tests.
12. Windows x64 release compilation.
13. legacy gameplay/visual smoke.
14. HQ gameplay/visual smoke.
15. historical legacy pixel regression.
16. HQ 1800×1200 dimension/timing gate.
17. Batman A/B comparison generation.
18. HUD A/B comparison generation.
19. artifact uploads.

## Visual inspection

Run #62's A/B sheets show a clear improvement over bilinear enlargement.

The true-3× side has visibly cleaner:

- BATARANG lettering.
- HEALTH lettering.
- SCORE lettering.
- lives x text.
- Batarang-logo outlines.
- belt/HUD graphic edges.
- small numeric glyphs.
- large score glyphs.

The full-scene HQ smoke also places the new HUD correctly above gameplay.

A post-CI comparison of the complete Round-5 and Round-6 HQ smoke sets found
that all changed pixels are confined to the HUD region. Representative changed
bounds are approximately:

```text
x = 481..1320
y = 0..201
```

Frames with completely opaque fade coverage remain byte-identical because the
HUD is not visible through those frames.

No world, Batman, camera, tutorial or gameplay-region pixel changes were
observed outside the HUD region.

## Performance

Run #62, 60 HQ iterations:

```json
{
  "mean_ms": 25.4451,
  "p50_ms": 25.2213,
  "p95_ms": 26.2548,
  "max_ms": 32.2916,
  "framebuffer_bytes": 9600000
}
```

All reported timings are below the game's 40 ms / 25 Hz frame duration in this
run.

## Windows artifact

Run #62 artifact:

`Batman-CobbleBot-Level1A-Fidelity-Windows-x64`

Downloaded ZIP SHA-256:

`261ce01ad1006239381012dbe68af396cd5d8ef0f25f1e14debcf02708309de6`

Executable SHA-256:

`62d21324c2fa0f2481988817e32c1cefb5d8a98d64cffc3deff005e2e643dafe`

Executable size:

`20,438,016 bytes`

## Round 6 verdict

True 3× HUD shell: **PASS**

True 3× static HUD labels: **PASS**

True 3× dynamic number atlases: **PASS**

Dynamic field registration: **PASS**

Premultiplied-alpha downsample: **PASS**

1:1 HQ runtime compositing: **PASS**

Legacy renderer regression: **PASS**

Gameplay/camera/collision changes: **none**

## Next phase

Round 7 upgrades the remaining user-facing foreground overlays:

- tutorial overlay 737.
- level title 746.
- fade-in 747.
- fade-out 34.

Those assets must retain their original frame counts, labels, timing,
CXFORM/alpha behavior and embedded DefineText glyphs while being rebuilt at
true 3× presentation resolution.
