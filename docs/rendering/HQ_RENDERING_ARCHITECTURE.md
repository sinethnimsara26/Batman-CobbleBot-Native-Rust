# HQ Rendering Architecture

Status: **approved design candidate; implementation has not started on this branch**

This document converts the rendering research into a concrete architecture for the native Rust port.

## 1. Non-negotiable constraints

1. Gameplay simulation remains exactly 600×400 logical coordinates at 25 Hz.
2. Collision masks, player physics, checkpoints, camera equations and recovered ActionScript behavior must not change.
3. The runtime remains native Rust/Win32 with no Flash, browser, WebView, Ruffle or external runtime.
4. The renderer-repair work in PR #1 must remain intact.
5. Level 1B and later content remain frozen.
6. The existing deterministic headless gameplay smoke must keep passing.
7. High resolution must not change sprite registration points in logical coordinates.
8. We must be able to fall back to the old renderer during development for A/B comparisons.

## 2. Chosen rendering model

### Logical simulation surface

```
LOGICAL_W = 600
LOGICAL_H = 400
```

All game state continues to use logical stage coordinates.

### Base scene surface

```
600 × 400
```

Contains the visually acceptable low-frequency world:

- root sky.
- large city background.
- Level 1A tile pack.

This pass stays cheap and preserves all existing camera math.

### HQ presentation surface

Default:

```
HQ_SCALE = 3
HQ_W = 1800
HQ_H = 1200
```

Render sequence:

```
1. render_base_scene(base_fb, game, assets)
2. upscale_opaque_base(base_fb → hq_fb)
3. render_hq_world_objects(hq_fb, game, assets)
4. render_hq_ui(hq_fb, game, assets)
5. render_hq_transitions(hq_fb, game, assets)
6. present hq_fb → Win32 viewport using HALFTONE
```

### Why split the scene

If we rendered every existing 600×400 tile separately into the 1800×1200 buffer with filtered scaling, each visible tile would cost a large resample.

Instead, all low-resolution scenery is composited once at 600×400, then the completed opaque scene is enlarged once.

Batman and UI are then drawn from true HQ source assets over that base.

## 3. Coordinate contract

Gameplay values are never multiplied inside `Game`.

Only the renderer converts logical coordinates to HQ pixels.

```text
hq_x = logical_x * HQ_SCALE
hq_y = logical_y * HQ_SCALE
```

Camera remains logical:

```text
render_x = (world_x + camera_x) * HQ_SCALE
render_y = (world_y + camera_y) * HQ_SCALE
```

The stage transform is therefore visually scaled without changing collision or timing.

## 4. Sprite registration contract

A sprite pack must know its **presentation scale**.

Current v1 frame packs contain image dimensions and anchors but assume 1× stage pixels.

The HQ pipeline should introduce pack version 2:

```text
magic: BCBFRM02
frame_count: u32
logical_pixel_scale: f32
frames...
```

For a 3× Batman pack:

```text
logical_pixel_scale = 3.0
```

Anchors remain stored in actual image pixels.

Placement becomes:

```text
dest_x = round(logical_x * pack.logical_pixel_scale - anchor_x)
dest_y = round(logical_y * pack.logical_pixel_scale - anchor_y)
```

The decoder must accept both:

- BCBFRM01 → implicit 1.0
- BCBFRM02 → embedded scale

That lets CI compare legacy and HQ packs and avoids hidden magic constants.

## 5. Batman asset strategy

Current stage-size bake:

```
0.163742
```

3× output target:

```
0.491226
```

Do not simply rasterize once at 0.491226 and call it finished.

The build tool should support:

```
final_scale = original_stage_scale * HQ_SCALE
supersample = 2 initially
temporary_scale = final_scale * supersample
```

For Batman:

```
0.163742 × 3 × 2 = 0.982452 temporary
→ Lanczos downsample by 2
→ final 3× Batman frame
```

If text/UI still needs more edge quality, use supersample 4 for those smaller assets.

### Required preservation

- 279 frame total.
- exact state boundaries.
- exact anchor/registration semantics.
- CXFORMWITHALPHA results.
- no interpolation between animation frames.
- no AI-generated detail.

## 6. HUD strategy

The HUD is unusually sensitive because the eye knows immediately when text is bad.

Rebuild at 3×:

- vector HUD shell.
- embedded/shape graphics.
- `BATARANG`, `HEALTH`, `SCORE`, `lives x`.
- dynamic number atlas.

Device-font generation:

```text
current font_px × HQ_SCALE
```

If build-time font antialiasing at exactly 3× is not clean enough, render the font atlas at 6× and Lanczos-downsample to 3×.

Runtime numbers are copied 1:1 into the 3× surface.

No nearest-neighbor scaling of the HUD should occur in normal HQ mode.

## 7. Tutorial/title/fade strategy

Bake the following at 3×:

- symbol 737 tutorial overlay: 149 frames.
- symbol 746 level title: 100 frames.
- symbol 747 fade-in: 43 frames.
- symbol 34 fade-out: 41 frames.

Embedded DefineText glyph rendering must happen before the final supersample/downsample stage.

The renderer places them using original logical coordinates multiplied by HQ_SCALE.

Alpha/CXFORM fidelity from PR #1 must be applied **before** downsampling.

This ordering matters because antialiasing should include the transformed alpha edges.

## 8. Pickups/projectiles

Rebake at 3× output:

- pickup symbol 695: original stage scale ×3.
- projectile symbol 694: original stage scale ×3.

They are small and move over the scene, so they benefit strongly from clean edges.

## 9. Base-scene scaler

The base scene is already fully composited and opaque.

Implement a dedicated `upscale_opaque_base()`.

### First implementation

Bilinear RGB interpolation.

Reasons:

- background is already visually acceptable.
- no alpha fringe problem because the base scene is opaque.
- much cheaper than high-quality cubic/Lanczos every frame.
- only one 600×400 → 1800×1200 operation per rendered frame.

### Do not use nearest-neighbor in HQ mode

Nearest should remain available only in legacy/debug mode.

## 10. Transparent sampling rules

Important assets should normally be 1:1 in HQ pixels.

If any transparent asset must be resampled:

1. convert/interpolate as premultiplied alpha.
2. interpolate premultiplied RGB + alpha.
3. source-over composite into the destination.

Do not linearly interpolate straight-alpha RGB across fully transparent pixels; that risks dark/colored halos.

## 11. Final Windows presentation

Current final path:

```
600×400 DIB → StretchDIBits → viewport
```

HQ path:

```
1800×1200 opaque DIB → HALFTONE StretchDIBits → viewport
```

Before `StretchDIBits`:

```text
SetStretchBltMode(hdc, HALFTONE)
SetBrushOrgEx(hdc, 0, 0, NULL)
```

The final source is opaque, so GDI's lack of alpha-aware compositing is irrelevant at this stage.

## 12. DPI awareness

After the HQ path is stable:

- try `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)` before window creation.
- fall back to `SetProcessDPIAware` if unavailable/fails.
- handle `WM_DPICHANGED` only if moving across monitors reveals a real issue.

Do not bundle DPI migration into the first HQ-render commit.

## 13. Renderer API refactor

Introduce:

```rust
struct RenderSurface {
    w: usize,
    h: usize,
    pixels: Vec<u32>,
}

struct RenderConfig {
    logical_w: usize,
    logical_h: usize,
    hq_scale: usize,
    mode: RenderMode,
}

enum RenderMode {
    Legacy1x,
    HighQuality,
}
```

Recommended functions:

```text
render_base_scene(...)
upscale_opaque_base(...)
render_hq_world_objects(...)
render_hq_ui(...)
render_hq_transitions(...)
present_surface(...)
```

Do not keep growing the existing one-line-heavy `blit()` functions.

## 14. Legacy A/B mode

For development only:

```
--render-quality legacy
--render-quality hq
```

Default must be HQ after validation.

The legacy path gives us:

- exact regression comparison.
- emergency fallback if a high-resolution placement is wrong.
- direct screenshot pairs in CI.

It is not intended as a permanent user-facing graphics setting unless performance testing proves it useful.

## 15. CI asset profiles

Build two profiles during the transition:

### legacy

Current 1× assets for regression screenshots.

### hq

3× runtime assets with supersampled vector rasterization.

Once HQ is approved manually, CI may stop packaging legacy assets but should retain a small legacy visual reference if useful.

## 16. Expected memory

### Framebuffers

- base: ~0.92 MiB.
- HQ 3×: ~8.24 MiB.
- total working framebuffers: ~9.2 MiB plus minor scratch data.

### Batman

Historical 1× handoff data decoded to ~16.7 MB RGBA for 271 frames.

3× linear dimensions imply roughly 9× decoded pixels, or ~150 MB for Batman alone.

The current 279-frame pack is slightly different.

This is acceptable for the target desktop class, but it must be measured in the new CI build.

If the exact current 3× Batman set exceeds the memory budget materially, fallback choices are:

1. decode animation frames lazily.
2. cache only the active state plus adjacent frames.
3. use 2× runtime HQ scale on lower-memory systems.

Do **not** immediately reduce visual quality before measuring.

## 17. Target budgets

Initial non-hard targets:

- startup decoded memory: < 350 MB.
- full HQ renderer working set: < 500 MB.
- render time: < 25 ms average on GitHub Windows runner.
- 95th percentile render time: < 35 ms.
- simulation budget remains 40 ms/tick at 25 Hz.

If 3× passes comfortably, keep it.

If it fails:
- optimize base upscaler and asset loading first.
- only then consider 2×.

## 18. Direct2D fallback criteria

Do not migrate to Direct2D unless at least one of these is true:

- GDI HALFTONE presentation visibly fails at fractional viewport sizes.
- software base upscaling exceeds the frame-time budget.
- 3× software composition cannot remain under the memory/performance budget.
- future stages require transforms that are prohibitively expensive in the software renderer.

If needed, Direct2D should initially replace **presentation/scaling only**, not gameplay or asset logic.

## 19. AI-upscale fallback criteria

AI may be tested only for an asset if:

1. it is genuinely bitmap-only in the original SWF.
2. no higher-resolution/vector representation exists.
3. deterministic Lanczos/bicubic scaling is visibly inadequate.
4. output can be manually compared to the original.
5. it does not affect text, Batman, HUD geometry, hitboxes or gameplay.

AI output must never silently replace canonical source-derived assets.
