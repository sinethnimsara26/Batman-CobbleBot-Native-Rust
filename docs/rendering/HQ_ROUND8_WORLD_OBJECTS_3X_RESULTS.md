# HQ Rendering Round 8 — True 3× Moving World Objects

Status: **implementation + CI complete**

Branch: `hq-rendering-phase8-world-objects-3x`

Parent: `hq-rendering-phase7-overlays-3x`

PR: **#11 — HQ Rendering Phase 8: true 3x pickups and Batarangs**

Validated implementation head:

`6a31d6cce4cd79dbf6c2287075cb550d0a138609`

Successful CI:

**Run #74 / `35864769129`**

## Scope

Round 8 upgrades the two small moving vector-derived world objects that were
still being rasterized into the 600×400 base and then blurred by whole-frame
promotion:

- pickup symbol **695**.
- thrown Batarang/projectile symbol **694**.

The moon is deliberately deferred. It sits behind the city layer, so upgrading
it faithfully requires a separate depth-preserving world-layer split rather
than drawing a sharp moon over the city.

## Build pipeline

The legacy 1× packs remain unchanged as the regression oracle.

Each HQ object is reconstructed from the canonical SWF at **6× relative to its
recovered display scale**, then premultiplied-alpha Lanczos filtered to the
final true 3× object scale and packed as `BCBFRM02` with
`logical_pixel_scale = 3.0`.

Recovered scales:

- pickup: legacy **1.130691**, temporary **6.784146**, final SWF raster scale **3.392073**.
- Batarang: legacy **0.364471**, temporary **2.186826**, final SWF raster scale **1.093413**.

Frame counts are preserved:

- pickup: **1**.
- Batarang: **5**.

## Runtime composition

The HQ playing frame now follows this order:

```text
1× scenery without pickup/projectile
→ promote scenery to 1800×1200
→ true-3× pickup/projectile copied 1:1
→ true-3× Batman copied 1:1
→ true-3× HUD/tutorial/title/fades
```

This prevents a blurry duplicate of the moving objects from surviving inside
the promoted base while preserving their original depth below Batman.

## Registration

The final CI gate explicitly accounts for the generic baker/packer fixed
padding difference between legacy and supersampled crops.

Worst cropped-anchor bookkeeping delta:

- pickup: **5.0 HQ pixels**.
- Batarang: **5.0000000000000036 HQ pixels**.

Runtime stage placement is locked to the original logical world coordinates;
the crop delta is internal pack bookkeeping rather than world-space drift.

## Memory

Round-8 eager decoded RGBA additions, legacy + HQ:

- pickup legacy: **8,844 B**.
- pickup HQ: **68,288 B**.
- Batarang legacy: **7,280 B**.
- Batarang HQ: **71,224 B**.
- Round-8 total: **155,636 B**.

Previous conservative known sprite upper bound:

**375,492,464 B**

Round-8 conservative combined known upper bound:

**375,648,100 B (~358.2 MiB)**

This remains comfortably below the project's **500 MiB stop condition**.

## Performance

Run #74, 60 HQ iterations:

```text
mean  = 23.6719 ms
p50   = 23.5242 ms
p95   = 24.1689 ms
max   = 26.7421 ms
```

All samples remain below the game's **40 ms / 25 Hz** frame duration.

## Regression and visual gates

Run #74 passed:

- canonical SWF verification.
- complete reproducible asset rebuild.
- Round 1–7 contracts.
- Round-8 true-3× object contract.
- Rust sprite-pack tests.
- Windows x64 release build.
- historical legacy SHA-256 regression.
- **17 legacy + 17 HQ** smoke screenshots, including a dedicated
  `13_world_objects.png` checkpoint.
- pickup/Batarang A/B comparison sheets.
- memory and performance reports.
- all artifact uploads.

## Result

Pickup 695 true 3×: **PASS**

Batarang/projectile 694 true 3×: **PASS**

1:1 HQ runtime compositing: **PASS**

Depth ordering below Batman: **PASS**

Legacy 1× oracle preserved: **PASS**

Known decoded-memory upper bound <500 MiB: **PASS**

60-frame HQ benchmark <40 ms: **PASS**

Gameplay/camera/collision changes: **none**

## Next phase

Round 9 is the DPI-awareness pass:

- prefer `PER_MONITOR_AWARE_V2`.
- fall back to legacy `SetProcessDPIAware`.
- preserve the 3:2 viewport and HQ framebuffer.
- handle monitor-DPI changes without changing gameplay or render pixels.
- validate 100%, 125% and 150% scaling behavior as far as CI can simulate.
