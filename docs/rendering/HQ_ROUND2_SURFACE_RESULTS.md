# HQ Rendering Round 2 — Render Surface Refactor Results

Status: **implementation + CI complete**

Branch: `hq-rendering-phase2-surfaces`

Parent implementation branch: `hq-rendering-phase1-halftone`

## Goal

Round 2 decouples **presentation-surface dimensions** from the game's original
600×400 logical coordinate system without changing a single rendered legacy
pixel.

This is the structural prerequisite for Round 3, where the presentation target
can become 1800×1200 while gameplay/collision/camera remain 600×400 logical.

## Changes

### New `src/surface.rs`

Introduces:

```rust
pub struct RenderSurface {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<u32>,
}
```

The surface owns its dimensions, storage, clear operation and byte length.

### `src/render.rs`

Introduces:

```rust
RenderMode
RenderConfig
render_with_config(...)
```

Current active mode is still:

```text
Legacy1x
600×400
hq_scale = 1
```

The renderer no longer assumes framebuffer stride/bounds through
`LOGICAL_W`/`LOGICAL_H` while writing pixels.

Blit and sub-region operations now use:

```text
surface.w
surface.h
surface.pixels
```

Gameplay coordinates remain logical.

`RenderMode::HighQuality` intentionally exists as a non-functional future
entry point and fails explicitly if selected before Round 3.

### `src/win32.rs`

The application now owns a `RenderSurface` rather than a raw `Vec<u32>`.

The DIB source dimensions, image byte count, pointer, and StretchDIBits source
rectangle are taken from the surface itself.

The active surface is still constructed as:

```rust
RenderSurface::new(LOGICAL_W, LOGICAL_H)
```

Therefore Round 2 does not change presentation resolution yet.

Round 1's GDI HALFTONE + SetBrushOrgEx path is retained.

### `src/headless.rs`

Headless smoke output now renders through the same `RenderSurface`
abstraction.

PNG width, height, capacity and pixel iteration are sourced from the surface,
not hard-coded framebuffer storage assumptions.

This makes the headless path ready to emit 1800×1200 screenshots in Round 3.

### `src/main.rs`

Registers the new `surface` module.

## CI gates added

### Surface architecture contract

CI verifies that:

- `RenderSurface` owns width, height and pixels.
- `RenderConfig` / `RenderMode` exist.
- renderer indexing uses surface stride.
- Win32 DIB dimensions come from the surface.
- headless PNG dimensions come from the surface.
- the active legacy surface still starts at LOGICAL_W × LOGICAL_H.

### Pixel-identity regression gate

The complete 15-image headless visual-smoke set is hashed after rendering.

Each image must exactly match the known-good renderer-repair baseline from
Run #45.

The gate compares SHA-256, not an image similarity threshold.

## CI history

### Run #49 — expected guard failure

Run ID: `35749324680`

The asset pipeline passed, but the inherited Round 1 CI guard still required
the Win32 DIB header itself to literally contain:

```text
biWidth: LOGICAL_W
biHeight: LOGICAL_H
```

That assertion was no longer architecturally valid once dimensions moved into
`RenderSurface`.

No compile/render regression was exposed by this failure.

The guard was updated so it proves both:

1. HALFTONE + SetBrushOrgEx remain enabled.
2. the currently instantiated legacy RenderSurface is still LOGICAL_W × LOGICAL_H.

### Run #50 — PASS

Run ID: `35749594364`

Head SHA:

`9f9b124ccf525f9ff271fb1b687343f015d10e69`

Result: **success**

Passed:

1. canonical source SWF verification.
2. asset reconstruction.
3. sound reconstruction.
4. collision-mask generation.
5. control-geometry rejection.
6. 279-frame Batman timeline gate.
7. CXFORM / alpha fidelity gates.
8. Round 1 HALFTONE contract.
9. Round 2 surface-abstraction contract.
10. Windows x64 release compilation.
11. gameplay + visual smoke execution.
12. pixel-identical legacy screenshot gate.
13. artifact uploads.

## Pixel identity result

Expected screenshots: **15**

Matched screenshots: **15 / 15**

Mismatches: **0**

Representative baseline hashes remain:

| Screenshot | SHA-256 |
|---|---|
| `00_fade_start.png` | `fe797743d114f6f3dba95067b563387e6140ba3fcd5ec4eed5800255aaec9989` |
| `04_running.png` | `2751693c5ad9ed5aeb4cc2716b536a00f24def31cacdf384e4869f644359052e` |
| `05_jump.png` | `0ec4c094ba0a337976ab82e2b8ff2d327ad9579dbcebb036df30292306d06b11` |
| `06_glide.png` | `e58f955f561a176314586b204216dfeee772b5f4fd1e3b9defa2bf611b5656c9` |
| `09_to_street.png` | `3b75f3eb28d18caebf710a771d0b6537952ac8b677da18e0c1a2c0865b656c95` |
| `11_exit_fade_mid.png` | `427e8c1108ba69fc86acf7b72abde0f4bf48d25521946070987fe545f11ea558` |

This proves the surface refactor is behaviorally and visually inert in legacy
mode.

## Round 2 Windows artifact

Artifact name:

`Batman-CobbleBot-Level1A-Round2-Surfaces-Windows-x64.zip`

ZIP size:

`2,228,677 bytes`

ZIP SHA-256:

`6812c205f9dd212aa9a36c2b65f58596dfc713e1052c0b81ed5c0a8fcb1a3963`

Executable:

`batman_cobblebot_native.exe`

EXE size:

`4,255,232 bytes`

EXE SHA-256:

`9f1788f815bf4301ea2e0223d29ea2d2c7d8a2e9325a9dbb09b722ddad8852b4`

## Round 2 verdict

Architecture gate: **PASS**

Compilation gate: **PASS**

Gameplay smoke: **PASS**

Legacy visual identity: **PASS — 15/15 exact**

Presentation quality change expected in Round 2: **none**

This is intentional. Round 2 is a pure structural refactor.

## Next phase

Round 3 can now safely introduce:

- a 600×400 base scene surface.
- a 1800×1200 HQ presentation surface.
- a first bilinear opaque base-scene upscale.
- HQ DIB presentation.
- render-performance telemetry.

The first Round 3 gate must prove logical camera/gameplay positions are
unchanged while headless HQ screenshots become exactly 1800×1200.
