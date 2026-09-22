# High-Quality Rendering Research

Status: **design only — no runtime changes in this branch yet**

Base: `level1a-cxform-fidelity` at the renderer-repaired Level 1A state.

## 1. Problem statement

The native port currently preserves the original 600×400 logical stage and 25 Hz gameplay simulation, but it also rasterizes the final presentation at only 600×400.

The current path is effectively:

```
Flash/SWF vectors and symbols
        ↓ build-time bake
1× PNG / sprite frames
        ↓
Rust software renderer
        ↓
600×400 Vec<u32> framebuffer
        ↓
Win32 StretchDIBits
        ↓
large desktop window
```

The renderer-repair pass fixed color transforms, alpha transforms, missing backdrop layers, and several vector reconstruction bugs. Those fixes solved corruption, but they did not change the fundamental resolution pipeline.

In the user's latest screenshot, the client area is approximately 1648×928. A 600×400 stage with preserved aspect ratio occupies approximately 1392×928, which is a linear scale of roughly **2.32×**. Thus one logical framebuffer pixel becomes more than five display pixels in area.

This particularly hurts:

- Batman, because his 279 true child-animation frames are baked near final 1× stage size.
- HUD labels and dynamic numbers.
- tutorial text and level-title text.
- sharp vector edges such as Batman's cape, cowl and emblem.
- small UI graphics above the playfield.

The large city/background artwork is less objectionable because it contains broad dark shapes and low-frequency detail.

## 2. What Flash itself did differently

Flash's rendering model was not equivalent to rendering one 600×400 bitmap and enlarging it.

The Flash/AIR Stage quality documentation states that:

- Medium uses a 2×2 antialiasing grid.
- High uses a **4×4 antialiasing grid**.
- High was the desktop Flash Player default.
- Best also uses a 4×4 grid and applies higher-quality bitmap downscaling when bitmap smoothing is enabled.
- newer StageQuality modes also expose 8×8 and 16×16 grids.

Sources:

- AIR / Flash Stage reference: https://airsdk.dev/reference/actionscript/3.0/flash/display/Stage.html
- StageQuality reference: https://airsdk.dev/reference/actionscript/3.0/flash/display/StageQuality.html

The important design implication is that the original experience was based on **subpixel/vector antialiasing and smoothing**, not on nearest-neighbor enlargement of a 600×400 final bitmap.

We should therefore reproduce the principle rather than bolt an AI model onto the final frame.

## 3. Current native bottlenecks found in the repository

### 3.1 Fixed 600×400 final framebuffer

`src/win32.rs` allocates:

```rust
fb: vec![0; LOGICAL_W * LOGICAL_H]
```

with `LOGICAL_W = 600` and `LOGICAL_H = 400`.

The DIB passed to GDI is also exactly 600×400.

### 3.2 Final presentation uses StretchDIBits without an explicit high-quality stretch mode

The current Win32 paint path calls `StretchDIBits` directly.

Microsoft documents `HALFTONE` as the highest-quality GDI stretch mode: it maps source pixels into destination blocks so the average destination color approximates the source. Microsoft also requires `SetBrushOrgEx` after enabling HALFTONE.

Sources:

- SetStretchBltMode: https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setstretchbltmode
- SetBrushOrgEx: https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setbrushorgex
- GDI image scaling overview: https://learn.microsoft.com/en-us/windows/win32/gdi/scaling-an-image

This is a useful low-risk improvement, but it cannot manufacture detail that was already lost when Batman/text were rasterized into a 600×400 frame.

### 3.3 Runtime blit is nearest-neighbor

The current software `blit()` chooses source pixels by integer division:

```text
src_x = ox * source_width / destination_width
src_y = oy * source_height / destination_height
```

That is nearest-neighbor resampling.

It is appropriate for exact 1:1 copies, but poor for arbitrary scaling of antialiased cartoon/vector artwork.

### 3.4 Important assets are baked at approximately 1× logical resolution

Examples from the current workflow:

- Batman: `--scale 0.163742`
- pickup: `--scale 1.130691`
- Batarang projectile: `--scale 0.364471`
- HUD/tutorial/title/fades: `--scale 1.0`

Those values reproduce the original stage-space size, but not a high-resolution presentation of that size.

### 3.5 Device-font HUD text is rasterized at low resolution

The HUD builder recreates Flash's `_sans` device-font fields with Windows Arial. This is correct structurally, but the generated labels/digit atlases are currently close to 1× stage resolution and then enlarged with the rest of the game.

## 4. Options researched

### Option A — AI upscale the final framebuffer

Examples include Real-ESRGAN / RealESRGAN anime models.

Real-ESRGAN is designed for practical real-world image restoration and super-resolution and provides x4 models, including an anime-oriented model.

Source:
https://github.com/xinntao/Real-ESRGAN

#### Advantages

- can improve genuinely low-resolution bitmap-only art.
- can be useful as an offline experiment for source assets with no vector or higher-resolution representation.

#### Problems for this project

- it invents detail rather than reconstructing the original vector geometry.
- text is a poor candidate; hallucinated glyph edges are unacceptable.
- frame-by-frame inference can introduce inconsistent detail/flicker.
- an AI runtime would violate the lightweight native-port philosophy.
- baking all 279 Batman frames through an AI model is less faithful than simply rerasterizing Batman from the SWF at higher resolution.
- it would complicate reproducibility and CI.

**Decision: reject as the primary pipeline. Keep as an optional offline experiment only for bitmap-only source art if deterministic conventional scaling proves insufficient.**

### Option B — xBRZ / pixel-art scaling

xBRZ is explicitly a pixel-art scaling family.

Example:
https://github.com/atheros/xbrzscale

Batman is not pixel art; he originates primarily from Flash vector/shape timelines. xBRZ intentionally reshapes pixel clusters and would alter the source aesthetic.

**Decision: reject for Batman, UI text and vector-derived art.**

### Option C — GDI HALFTONE only

Microsoft describes HALFTONE as slower but higher-quality than COLORONCOLOR for bitmap stretching.

#### Advantages

- tiny code change.
- no new dependency.
- native Windows.
- improves arbitrary window-size presentation.

#### Limitation

The source remains only 600×400. Batman/text would become smoother, but still lack high-resolution edge information.

**Decision: mandatory final-presentation improvement, but not sufficient by itself.**

### Option D — move the whole renderer to Direct2D

Microsoft Direct2D supports:

- per-primitive high-quality antialiasing.
- nearest, linear, cubic, multisample linear, anisotropic and high-quality cubic interpolation.
- GPU acceleration with software fallback.

Sources:

- Direct2D overview: https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-overview
- interpolation modes: https://learn.microsoft.com/en-us/windows/win32/api/d2d1_1/ne-d2d1_1-d2d1_interpolation_mode
- high-quality scale effect: https://learn.microsoft.com/en-us/windows/win32/direct2d/high-quality-scale

#### Advantages

- excellent native Windows image scaling.
- future path to GPU acceleration.
- Direct2D can exceed GDI visual quality.

#### Problems

- large architectural change.
- significant COM/FFI surface if kept dependency-light.
- unnecessary for the immediate quality defect because our primary issue is premature rasterization.
- risks destabilizing a Level 1A build that finally plays correctly.

**Decision: reserve as Plan B. Do not replace the renderer before trying a supersampled software path.**

### Option E — supersample vector-derived assets + higher-resolution native composition

This uses the original SWF vectors/timelines, not generated detail.

The principle is:

1. keep gameplay coordinates at 600×400.
2. rasterize important vector content at several times its logical resolution.
3. render the final presentation into a higher-resolution framebuffer.
4. downscale to the actual window with a high-quality filter.

This most closely matches what Flash's high-quality antialiasing was doing conceptually.

**Decision: chosen approach.**

## 5. Supersampling and resampling research

Pillow provides deterministic filters including bilinear, bicubic and Lanczos. Its documentation describes Lanczos as an available high-quality resampling filter and supports a reducing-gap optimization for downscaling.

Source:
https://pillow.readthedocs.io/en/stable/reference/Image.html

For build-time vector baking, our preferred model is:

```
SWF vector geometry
→ rasterize at high temporary resolution
→ Lanczos downsample to runtime HQ resolution
→ PNG/frame pack
```

This means antialiasing quality is obtained from the original geometry.

For runtime scaling of transparent images, interpolation must respect alpha. Microsoft documents premultiplied alpha as the preferred form for filtering/compositing because transparent pixels no longer carry unrelated RGB color that can bleed into edges.

Sources:

- WIC pixel formats: https://learn.microsoft.com/en-us/windows/win32/wic/-wic-codec-native-pixel-formats
- premultiplied alpha explanation: https://learn.microsoft.com/en-us/windows/apps/develop/win2d/premultiplied-alpha

The selected architecture avoids most runtime transparent-image scaling by making the important HQ assets match the HQ framebuffer scale exactly.

## 6. Scale choice

We considered 2×, 3× and 4× runtime presentation.

A framebuffer uses four bytes per pixel:

| Render scale | Framebuffer | Approx. bytes | Approx. MiB |
|---|---:|---:|---:|
| 1× | 600×400 | 960,000 | 0.92 |
| 2× | 1200×800 | 3,840,000 | 3.66 |
| 3× | 1800×1200 | 8,640,000 | 8.24 |
| 4× | 2400×1600 | 15,360,000 | 14.65 |

The user's current screenshot implies a viewport around 1392×928, or ~2.32× logical scale.

Therefore:

- 2× still requires a small final upscale.
- 3× produces a source larger than the current viewport, so presentation becomes a downscale.
- 4× provides more headroom but multiplies decoded sprite memory substantially.

Historical Batman frame data from the earlier handoff had approximately 4.18 million decoded pixels across 271 frames (~16.7 MB RGBA at 1×). Linear 4× frame dimensions would theoretically expand that to roughly 267 MB for Batman alone; 3× is roughly 150 MB. The current project has 279 true frames, so exact modern values will differ slightly but the order of magnitude remains useful.

### Chosen target

**Runtime HQ scale: 3×**

Reasons:

- enough to turn the user's current ~2.32× window presentation into a downscale rather than an upscale.
- dramatically cleaner Batman/text/UI.
- ~8.2 MiB final framebuffer.
- materially lower decoded sprite memory than 4×.
- leaves headroom for the CPU software compositor.
- practical on the project's target Windows hardware.

### Chosen build-time antialiasing scale

**4× sampling quality relative to the final HQ target where practical.**

Do not interpret this as storing 12× assets.

The baker should render vector geometry at an internal supersample resolution and downsample to the final 3× asset using Lanczos. Start with 2× internal supersampling of the 3× target (6× temporary raster) for expensive Batman sets; evaluate 4× internal sampling only for text/UI if needed.

The temporary resolution affects CI time, not runtime memory.

## 7. High-priority assets versus low-priority scenery

### Tier A — must become HQ

- all 279 Batman frames.
- HUD shell.
- HUD static labels.
- HUD dynamic digit atlases.
- tutorial overlay frames and embedded text.
- level-title timeline.
- fade-in/fade-out timeline.
- pickup/Batarang moving graphics.
- small root foreground symbols such as the moon if their edges are visibly aliased.

### Tier B — initially keep at existing resolution

- large Level 1A tile pack.
- large city background panorama.
- broad red sky layer.

Reason: these already look acceptable to the user and dominate storage/bandwidth if multiplied.

The base scene will be smoothly promoted to the HQ framebuffer once per frame before Tier A is composited.

### Tier C — evaluate after the first HQ build

- title screen.
- instructions screen.
- large bitmap-only backgrounds.

Only upgrade them if A/B comparison still exposes obvious pixelation.

## 8. Why not just rebake the entire world at 3×

The Level 1A visual tile system contains 168 600×400 tiles.

A 3× tile is 1800×1200. Uncompressed that is ~8.24 MiB per tile. Even though PNG compression and the tile cache reduce practical memory/disk use, blindly tripling every tile would produce a disproportionate storage/cache cost.

The proposed two-pass pipeline makes the most objectionable foreground/UI elements truly HQ first while keeping the already-acceptable scenery cheap.

## 9. DPI behavior

The current app calls the legacy `SetProcessDPIAware`.

Windows supports `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`, and Microsoft recommends setting the desired process DPI awareness before creating UI.

Sources:

- DPI awareness contexts: https://learn.microsoft.com/en-us/windows/win32/hidpi/dpi-awareness-context
- SetProcessDpiAwarenessContext: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setprocessdpiawarenesscontext
- GetDpiForWindow: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow

We should migrate to Per-Monitor-v2 with a safe legacy fallback, but this is a separate subphase from asset supersampling so DPI changes cannot be confused with renderer regressions.

## 10. Research conclusion

The pixelation is primarily an architectural resolution problem, not missing source quality.

The project still has better source information than an AI upscaler:

- SWF vector geometry.
- embedded fonts.
- placement matrices.
- color/alpha transforms.
- true child animation timelines.

The highest-fidelity path is therefore:

```
keep gameplay at 600×400
+ preserve original vectors/timelines
+ bake important assets at HQ
+ compose into a 3× framebuffer
+ use high-quality final Windows scaling
```

AI super-resolution remains an optional last resort for bitmap-only material, not part of the canonical Level 1A renderer.
