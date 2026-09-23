# HQ Rendering Round 9 — Per-Monitor-V2 DPI Awareness

Status: **implementation + CI complete**

Branch: `hq-rendering-phase9-dpi-awareness`

Parent: `hq-rendering-phase8-world-objects-3x`

PR: **#12 — HQ Rendering Phase 9: Per-Monitor-V2 DPI awareness**

Validated implementation head:

`86c2fabc780b5821e5d2bd453a1cb6ea32ab8b2e`

Successful CI:

**Run #78 / `35869159185`**

## Goal

Round 9 makes the native Windows presenter DPI-aware without changing the
600×400 simulation, the 1800×1200 HQ framebuffer, game timing, camera math,
asset placement, or rendered scene content.

## Windows strategy

The process now prefers:

`DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` (`-4`)

`SetProcessDpiAwarenessContext` is resolved dynamically from `user32.dll`
with `GetProcAddress`. This avoids creating a hard import requirement on
older Windows versions.

If the newer API is unavailable or rejects the request, the existing
`SetProcessDPIAware` path remains as the fallback.

This preserves the project's Windows compatibility while opting into physical
client-pixel coordinates on modern Windows.

## Monitor-DPI changes

The window procedure now handles `WM_DPICHANGED`.

When Windows recommends a new window rectangle after a monitor/DPI transition,
the native window applies that rectangle with `SetWindowPos` using
`SWP_NOZORDER | SWP_NOACTIVATE`, then invalidates for repaint.

The HQ framebuffer itself is never resized by DPI. It remains exactly:

`1800×1200`

The game remains:

`600×400 @ 25 Hz`

Only final presentation into the physical client rectangle changes.

## Viewport contract

The original centered 3:2 viewport calculation was factored into the pure
`viewport_for_client()` helper so the DPI behavior can be unit-tested without
a live desktop.

CI models physical-pixel client sizes corresponding to common Windows scaling
levels:

- 100%: 900×600 → exact 900×600 viewport.
- 125%: 1125×750 → exact 1125×750 viewport.
- 150%: 1350×900 → exact 1350×900 viewport.

It also checks non-3:2 windows at those representative sizes:

- 930×660 → 930×620 centered with 20 px top/bottom bars.
- 1163×825 → 1163×775 centered with 25 px top/bottom bars.
- 1395×990 → 1395×930 centered with 30 px top/bottom bars.

Odd physical dimensions are allowed the unavoidable one-pixel integer aspect
rounding.

## Validation

Run #78 passed:

- canonical SWF verification.
- complete reproducible asset reconstruction.
- Round 1–8 rendering/fidelity contracts.
- Round-9 Per-Monitor-V2 source/viewport contract.
- **6/6 Rust tests**, including both DPI viewport tests.
- Windows x64 release compilation.
- historical legacy SHA-256 pixel regression.
- **17/17 legacy + 17/17 HQ** smoke screenshots.
- all Batman/HUD/overlay/world-object comparison generation.
- all artifact uploads.

The Windows build therefore proves the new dynamic DPI API declarations and
`WM_DPICHANGED` handling compile on the real x64 MSVC target.

## Performance

Run #78, 60 HQ iterations:

```text
mean  = 23.4805 ms
p50   = 23.4373 ms
p95   = 23.6819 ms
max   = 23.8233 ms
```

All samples are well below the game's **40 ms / 25 Hz** frame duration.

Framebuffer memory remains:

`9,600,000 bytes`

## Manual limitation

CI can prove the API wiring, fallback path, viewport math, Windows compilation
and unchanged renderer output.

A real transition between monitors with different Windows DPI settings cannot
be physically simulated by the hosted headless CI runner. That remains a live
Windows presentation check, not an untested renderer/gameplay dependency.

## Result

Per-Monitor-V2 preferred: **PASS**

Legacy DPI fallback retained: **PASS**

WM_DPICHANGED suggested-rectangle handling: **PASS**

100/125/150% physical viewport math: **PASS**

3:2 centered letterboxing: **PASS**

Windows x64 compilation: **PASS**

Legacy pixel regression: **PASS**

HQ render regression: **PASS**

Gameplay/camera/collision changes: **none**

## Next phase

Round 10 evaluates and selectively upgrades scenery.

Round-8 visual inspection shows that the now-sharp Batman/HUD/tutorial assets
make the remaining level geometry visibly softer by comparison. The next pass
must therefore target foreground roof/platform vector geometry first, while
avoiding a blind 3× rebuild of every world asset.

Current Level-1A level tile facts:

- source level symbol: **152**.
- visible frame-0 placements: **41**.
- visible repeated child symbols: primarily **144–147**.
- current logical tiles: **168 × 600×400**.
- current 1× packed tile archive: approximately **313 KB**.
- tile decoding is lazy with an LRU cache.

The moon/city panorama remain separate later priorities.
