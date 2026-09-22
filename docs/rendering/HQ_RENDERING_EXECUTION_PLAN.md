# HQ Rendering Execution Plan

Status: **planning gate**

No implementation should begin until this document is reviewed against the current renderer-repair branch.

## Phase 0 — Preserve baseline

### Goal

Freeze the renderer-repaired Level 1A as the comparison oracle.

### Actions

- base HQ work on `level1a-cxform-fidelity`, not stale `main`.
- record current head SHA.
- retain PR #1 unmodified during planning.
- download/retain Run 45:
  - Windows EXE.
  - visual smoke screenshots.
  - renderer diagnostics.
- keep the user's manually tested screenshot as a real-world quality reference.

### Baseline facts to record

- logical stage: 600×400.
- simulation: 25 Hz.
- user's observed window viewport scale: approximately 2.32×.
- current Batman frame count: 279.
- renderer-repair CI is green.
- Level 1A gameplay is manually playable.

### Exit gate

No gameplay, camera, collision or audio code changes in the first HQ branch.

---

## Phase 1 — Presentation-only proof

### Goal

Measure how much quality comes from better final Windows scaling before touching asset packs.

### Files

- `src/win32.rs`

### Changes

Add:

- `SetStretchBltMode` FFI.
- `SetBrushOrgEx` FFI.
- `HALFTONE = 4` constant.
- call HALFTONE + brush origin immediately before the final bitmap stretch.

Keep the framebuffer 600×400.

### CI outputs

Produce:

- legacy current screenshot set.
- HALFTONE screenshot set.

### Acceptance

- zero gameplay differences.
- exact same smoke-state hashes/trace data.
- no paint artifacts.
- visually smoother fractional scaling.

### Decision

Keep HALFTONE regardless of later HQ work unless it causes a verified regression.

---

## Phase 2 — Generalize render surfaces

### Goal

Decouple gameplay resolution from presentation resolution.

### Files

- `src/render.rs`
- `src/win32.rs`
- possibly new `src/surface.rs`

### Changes

Introduce:

```rust
RenderSurface
RenderConfig
RenderMode
```

Replace all hard-coded framebuffer indexing assumptions with the surface dimensions.

Split rendering into:

1. base scene.
2. HQ composition.
3. transitions.

### Mandatory invariants

- `Game` stays untouched.
- logical camera math stays untouched.
- all world coordinates remain 600×400 logical coordinates.
- collision mask lookup stays untouched.

### Smoke test

Render legacy mode after refactor and require pixel-identical output to the pre-refactor baseline.

This is the most important safety gate in the entire quality project.

---

## Phase 3 — 3× HQ framebuffer

### Goal

Create `1800×1200` presentation without yet changing Batman assets.

### Changes

- allocate `base_fb = 600×400`.
- allocate `hq_fb = 1800×1200`.
- render scenery to base.
- implement `upscale_opaque_base()`.
- begin with bilinear.
- present the HQ DIB through HALFTONE GDI.

### Validation

- screenshot dimensions exactly 1800×1200 in headless HQ mode.
- letterboxing aspect remains 3:2.
- camera positions line up with legacy screenshots after 3× coordinate conversion.
- no one-pixel logical drift.

### Performance telemetry

Add headless command:

```
--benchmark-render N
```

Report:

- mean render milliseconds.
- p50.
- p95.
- max.
- framebuffer memory.

Do not hard fail CI until at least two runs establish a stable baseline.

---

## Phase 4 — Sprite pack v2

### Goal

Make asset resolution explicit.

### Files

- `reverse-engineering/tools/pack_sprite_frames.py`
- `src/assets.rs`

### Format

Introduce `BCBFRM02`.

Header:

```text
magic[8]
frame_count u32
logical_pixel_scale f32
```

Then existing per-frame records.

### Compatibility

Rust decoder:

- `BCBFRM01` → scale 1.0.
- `BCBFRM02` → read embedded scale.

### CI gate

Attempting to load a v2 pack with scale != expected HQ profile must fail with a clear message.

---

## Phase 5 — High-resolution Batman

### Goal

Fix the most visible gameplay pixelation first.

### Build changes

Add explicit baker options:

```
--output-scale 3
--supersample 2
--downsample lanczos
```

Do not encode those semantics as another magic multiplication in YAML.

The player-state baker still derives all 279 frames from the same source timelines.

### Output contract

- logical scale metadata = 3.0.
- frame count = 279.
- state frame counts unchanged.
- anchors scale proportionally.
- no missing bitmaps.
- no unsupported fills.
- CXFORM values applied before downsampling.

### Runtime

Batman is composited 1:1 onto `hq_fb`.

No runtime nearest-neighbor scaling.

### Visual gates

Create paired screenshots for:

- stand.
- walk.
- run.
- jump.
- glide.
- punch.
- kick.

Compare:
- legacy 1× enlarged.
- HQ 3×.

### Manual gate

Batman must look materially cleaner at the user's ~2.32× viewport before continuing to all UI.

---

## Phase 6 — HUD and text

### Goal

Fix the most objectionable text/UI pixelation.

### Rebuild at 3×

- HUD shell.
- BATARANG.
- HEALTH.
- SCORE.
- lives x.
- score digits.
- life digits.
- Batarang count.

### Font strategy

For Flash device `_sans`:

- Windows target uses Arial as the current faithful substitute.
- render at final 3× font size.
- if edge quality is inadequate:
  - rasterize at 6×.
  - Lanczos downsample to 3×.

### Runtime

Dynamic numeric glyphs are copied 1:1 into HQ surface.

### Gates

- numeric field centers must match legacy logical placement ×3.
- no baseline shift greater than 1 HQ pixel.
- zero nearest-neighbor resampling of HUD assets in HQ mode.

---

## Phase 7 — Tutorial / title / fades

### Goal

Make every user-facing foreground overlay crisp.

HQ rebake:

- tutorial overlay 737.
- level title 746.
- fade-in 747.
- fade-out 34.

### Preserve

- frame counts.
- label behavior.
- alpha/CXFORM sequence.
- embedded DefineText glyphs.
- frame timing at 25 Hz.

### CI

Existing alpha gates must still pass semantically. Pixel alpha values may need scale-aware sampling rather than fixed coordinates.

---

## Phase 8 — Pickups, Batarang, moon/small foreground symbols

Rebuild the small moving vector-derived pieces at 3×.

Do not rebuild the giant city panorama yet.

---

## Phase 9 — DPI-awareness upgrade

Only after HQ screenshots are approved:

- prefer `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)`.
- fall back to `SetProcessDPIAware`.
- verify on 100%, 125%, 150% Windows scaling.
- verify moving the window between monitors if available.

No gameplay changes.

---

## Phase 10 — Evaluate scenery

After Batman/UI are fixed, ask one question:

> Does the world now look noticeably worse than Batman/UI?

If no:
- keep current world assets.

If yes:
- upgrade selectively.

Priority:

1. foreground roof/platform vector layers.
2. moon/sky edge assets.
3. city panorama.
4. full tile pack last.

Never multiply all 168 tiles blindly before measuring archive size and runtime cache impact.

---

## Phase 11 — Optional bitmap-only restoration experiments

Only after canonical HQ rendering is complete.

Candidates:

- truly bitmap-only title/instruction art if still visibly soft.

Experiment order:

1. Lanczos.
2. bicubic.
3. optional Real-ESRGAN offline comparison.

AI result can only be adopted after manual side-by-side approval and must remain a build-time artifact, never a runtime dependency.

---

# File-by-file implementation map

## `src/render.rs`

Planned:

- `RenderSurface`.
- `RenderMode`.
- logical→HQ coordinate helpers.
- split base and HQ passes.
- bilinear opaque base scaler.
- HQ 1:1 sprite blitter.
- optional premultiplied-alpha filtered blitter.

Do not change:

- gameplay state.
- camera math.
- animation timing.

## `src/win32.rs`

Planned:

- HQ DIB dimensions.
- HALFTONE.
- SetBrushOrgEx.
- render quality CLI/debug selection.
- later PerMonitorV2 DPI awareness.

## `src/assets.rs`

Planned:

- SpriteSet / pack-scale metadata.
- BCBFRM02 decoder.
- legacy BCBFRM01 compatibility.
- expose HQ scale for validation.

## `reverse-engineering/tools/bake_sprite_frames.py`

Planned:

- generalized supersample/downsample stage.
- premultiplied-alpha-safe resampling.
- metadata records final/output scale and supersample factor.

## `reverse-engineering/tools/bake_player_states.py`

Planned:

- same HQ output controls.
- no changes to state discovery.

## `reverse-engineering/tools/bake_hud_assets.py`

Planned:

- scale-aware font generation.
- scale-aware field metadata.
- HQ digit atlases.

## `reverse-engineering/tools/pack_sprite_frames.py`

Planned:

- BCBFRM02.
- write logical pixel scale.

## GitHub Actions

Planned:

- build both legacy and HQ during transition.
- validate HQ metadata.
- upload HQ visual-smoke set.
- upload A/B contact sheet if practical.
- report asset sizes.
- report performance benchmark.
- keep all current fidelity gates.

---

# Acceptance checklist

The HQ pass is not finished until all of these are true.

## Visual

- [ ] Batman no longer looks blocky at ~2.3× desktop viewport.
- [ ] HUD labels are clearly smoother.
- [ ] score/lives/Batarang numbers are clearly smoother.
- [ ] tutorial text is clearly smoother.
- [ ] level title is clearly smoother.
- [ ] no new alpha halos.
- [ ] no color-transform regression.
- [ ] no reappearance of leaked control geometry.
- [ ] no registration wobble across Batman animation frames.
- [ ] large background remains at least as good as current renderer-repair build.

## Gameplay

- [ ] cold-launch keyboard still works.
- [ ] walk/run unchanged.
- [ ] jump/glide unchanged.
- [ ] punch/kick/Batarang timing unchanged.
- [ ] pickups unchanged.
- [ ] pit/life restart unchanged.
- [ ] Level 1A exit/fade unchanged.
- [ ] audio timing unchanged.

## Technical

- [ ] legacy refactor screenshot is pixel-identical before HQ content lands.
- [ ] BCBFRM01 remains readable during migration.
- [ ] HQ packs declare their logical pixel scale.
- [ ] no Flash/Ruffle/WebView runtime.
- [ ] no AI runtime dependency.
- [ ] build remains reproducible from verified SWF.
- [ ] CI reports HQ render performance.
- [ ] working set stays within target budget.

## Manual user gate

The user tests the HQ Level 1A build at the same large window size that exposed the pixelation.

Only after that test is approved should:

- HQ mode become the only packaged default.
- the branch be merged.
- work resume on Level 1B.

---

# Rollback strategy

Every phase should be independently revertible.

Do not combine:

- render-surface refactor,
- new pack format,
- Batman HQ assets,
- HUD HQ assets,
- DPI-awareness migration

into one giant commit.

Expected commit structure:

1. docs / baseline.
2. GDI HALFTONE.
3. surface refactor with legacy pixel-identical output.
4. 3× framebuffer.
5. pack v2.
6. Batman HQ.
7. HUD HQ.
8. overlay HQ.
9. small world objects HQ.
10. DPI awareness.
11. scenery improvements only if needed.

This makes regressions bisectable.

---

# Stop conditions

Pause implementation immediately if any of these occur:

- collision/gameplay behavior changes.
- player registration shifts.
- camera changes.
- renderer-repair color/alpha gates fail.
- 3× mode consistently misses the 40 ms game tick budget.
- decoded memory exceeds 500 MB before scenery upgrades.

Fix the regression before proceeding.

---

# Final deliverable of the HQ phase

A new Level 1A test ZIP containing:

- native Rust EXE.
- HQ mode enabled by default.
- no external runtime.
- same gameplay state as renderer-repaired Level 1A.
- visibly cleaner Batman/text/HUD at the user's normal large window size.

Alongside it, CI should publish:

- HQ smoke screenshots.
- legacy/HQ comparison screenshots.
- performance report.
- asset-size report.
