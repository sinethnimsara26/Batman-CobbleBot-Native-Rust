# Independent Full Rescan & Rust Engineering Audit of Level 2 Work (`B01`–`B09`)

- **Audit Date:** 2026-11-12
- **Auditor:** Antigravity (Independent Pair-Programming Agent Audit requested by repository owner `sinethnimsara26`)
- **Audited Range:** `origin/revive/level1a-intro-hud-fidelity-2026-11-09` (`ba53299`) .. `origin/revive/level2-b09-bank-goon-sprites-2026-11-12` (`9176e3f`)
- **Canonical Source Baseline:** Uncompressed Flash 7 `FWS` (`2,819,100` bytes, SHA-256 `3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed`)
- **Immutable Release Baseline Verified Untouched:** `release/level1a-v0.2.0` (`49e8fd343e4f37f045ba71f24fff0999211f0f11`) and `main` (`639a75f`)

---

## 1. Executive Summary

This document records an independent, full-repository rescan of everything implemented by the previous AI agent across the Level 2 microbatches **B01 through B09** (`plan/level2-source-roadmap-2026-11-11` through `revive/level2-b09-bank-goon-sprites-2026-11-12`).

### 1.1 What Has Actually Been Done So Far (`B01`–`B09`)

> [!IMPORTANT]
> **No playable Level 1B (Gotham Bank Street), Level 1C (Penguin Boss), or Level 2A/2B (Gotham Labs) gameplay is active yet.**
> Every Rust addition in `B02`–`B09` is either gated behind `#![allow(dead_code)]` / `#[allow(dead_code)]` or latched into an inert `PendingStageEntry` struct after Level 1A finishes its 41-frame fadeout. At runtime, the compiled executable still plays **only Level 1A** and freezes on `level1a_complete = true`.

| Batch | Branch / PR | Actual Deliverable | Runtime Status |
| :--- | :--- | :--- | :--- |
| **B01** | `plan/level2-source-roadmap-2026-11-11` (PR #21) | Offline Python AVM1/timeline probe (`reverse-engineering/tools/b01_source_probe.py`) + 32-batch roadmap (`docs/revival/LEVEL2_IMPLEMENTATION_ROADMAP_2026-11-11.md`). Discovered `respawn2` (8-kill second wave in 1B) and `clockTimer` (80s timer in 2B). | Offline tool only |
| **B02** | `revive/level2-b02-stage-descriptors-2026-11-11` (PR #22) | Added `src/stage.rs` (`StageId`, `StoryCard`, `SourceScene`, `SourceCamera`, `StageDescriptor`, `STAGES[5]`). | Dormant (`#![allow(dead_code)]`) |
| **B03** | `revive/level2-b03-source-asset-proofs-2026-11-11` (PR #23) | Added `reverse-engineering/tools/b03_bake_stage_asset_proofs.py` rendering 2 sample tiles (`1x` and `3x`) + 1 actor still per stage (`1B`, `1C`, `2A`, `2B`), plus negative guard in `pack_level_tiles.py`. | Offline tool only |
| **B04** | `revive/level2-b04-pending-transition-2026-11-11` (PR #24) | Added `PendingStageEntry` / `RootProgress` in `src/stage.rs` and wired `Game::tick` (`src/game.rs`) to stop music on fade tick 39 and latch `pending_stage = Some(Level1B)` on tick 40 while keeping `level1a_complete = true`. | Latches metadata only; no transition |
| **B05** | `revive/level2-b05-bank-street-source-1x-2026-11-11` (PR #25) | Added `b05_bake_bank_street_1x.py` + `--include-source-static-text` in `bake_level_tiles.py` (recovering `DefineText #774` and excluding `DefineButton2 #758/#771`) across **9 sampled viewports** of Bank Street (`#783`). | Offline partial sample only (`never_embed_as_complete_level: true`) |
| **B06** | `revive/level2-b06-bank-street-hq3x-2026-11-11` (PR #26) | Added `b06_bake_bank_street_hq3x.py` (`6x -> 3x` premultiplied Lanczos on the same **9 sampled viewports** + 2-viewport depth-split proof at `cut = 626`). | Offline partial sample only (`never_embed_as_complete_level: true`) |
| **B07** | `revive/level2-b07-bank-ground-probes-2026-11-11` (PR #27) | Added `CollisionMask::level1b()` (`12102 × 1413` mask from `#782`) in `src/collision.rs` and `SourceCamera::horizontal_step` / `jumping_y` in `src/stage.rs`. | Dormant (`#[allow(dead_code)]`, called only in `#[cfg(test)]`) |
| **B08** | `revive/level2-b08-bank-props-source-2026-11-12` (PR #28) | Added `src/bank_props.rs` (`BANK_STREET_PROPS: [BankStreetProp; 9]` for 4× `#785` cans, 4× `#790` trash, 1× `#793` mailbox) + `b08_bake_bank_props.py` frame-0 cutouts. | Dormant (`#![allow(dead_code)]`) |
| **B09** | `revive/level2-b09-bank-goon-sprites-2026-11-12` (PR #29) | Added `src/bank_goons.rs` (`GOON_LABELS`, `GOON_TIMELINES`, `BANK_GOON_SPAWNS: [BankGoonSpawn; 10]` for `#880` and `#964`) + `b09_bake_bank_goons.py`. | Dormant (`#![allow(dead_code)]`) |

### 1.2 Summary of Audit Verdicts

| Audit Dimension | Verdict | Summary of Key Findings |
| :--- | :--- | :--- |
| **1. Rust Runtime Purity** | **PASS (Qualified)** | Runtime executable (`batman_cobblebot.exe`) is 100% native Rust (`edition = "2021"`, `png = "0.17"`, `flate2 = "1"`) with only Win32/GDI (`user32`, `gdi32`) and WinMM (`winmm`) OS FFI. All Python scripts (`b01`–`b09`) are strictly offline/CI tools. |
| **2. Source Fidelity & Correctness** | **3 CRITICAL ISSUES FOUND** | **L2-CRIT-01:** `src/bank_goons.rs` (B09) models goon animations using outer 89-frame label spans (`10` frames/state) instead of the true depth-1 child MovieClip timelines (`55` frames for `#880`, `66` frames for `#964`), truncating `#964` walk/retreat and causing `hurt`/`fall`/`die` to wrap around and pop dead/fallen goons upright.<br>**L2-CRIT-02:** B05/B06 only baked 9 non-packable sample tiles, and the B10–B13 roadmap has **no batch scheduled to bake/pack the complete 48-tile Level 1B stage** before B13 stage activation.<br>**L2-CRIT-03:** `bake_sprite_frames.py` uses `ImageDraw.Draw(canvas).line()` which overwrites RGBA alpha instead of Porter-Duff `source-over` blending, causing the 22,382-pixel discrepancy masked in `b06_bake_bank_street_hq3x.py`. |
| **3. Rust Code Quality & Architecture** | **6 IMPROVEMENTS NEEDED** | **R-01:** `CollisionMask` uses 1 byte/pixel (`Vec<u8>`), costing `17.1 MB` for 1B and **`88.3 MB` each** for 2A/2B; bit-packing into `Box<[u64]>` cuts memory **8×** (`2.1 MB` / `11.0 MB`).<br>**R-02:** Dormant module sprawl (`stage.rs`, `bank_props.rs`, `bank_goons.rs`) with untyped `(f32, f32)` coordinates risks game-local vs level-local bugs; replace with zero-cost `GamePos`/`LevelPos` newtypes.<br>**R-03:** `src/render.rs` allocates two 1,800-element `Vec`s (`xmap`) on the heap **every frame** and allocates `String`s for HUD digits.<br>**R-04:** `Assets::load()` eagerly loads all stage assets at startup instead of per-stage `StageAssets::load(StageId)`.<br>**R-05:** `static mut APP: *mut App` in `src/win32.rs` has a synchronous Win32 reentrancy/aliasing UB hazard during `toggle_fullscreen`, and `Audio::on_focus_changed` fails to pause gameplay music (`bcb_music`).<br>**R-06:** Minor idioms (recursive `check_ground`/`check_ceiling`, `format!("{:?}")` state assertions). |
| **4. Online / Technical Research** | **4 AREAS REQUIRED** | Full AVM1 control-flow decompilation (beyond 18-op windows) before B10/B11/B12/B15; AVM1 prop interaction verification for `#785/#790/#793`; Flash `hitTest` coordinate/winding edge cases; CI workflow consolidation & SWF caching. |

---

## 2. Part I — Critical Source-Fidelity & Architectural Issues (`B01`–`B09`)

### L2-CRIT-01 (High Severity — B09): `src/bank_goons.rs` & `b09_bake_bank_goons.py` Use Outer 89-Frame Label Spans Instead of True Depth-1 Child MovieClip Timelines

#### Evidence
When Level 1A was built, the project discovered a fundamental Flash authoring pattern in `cobblebot.swf` documented at the top of `reverse-engineering/tools/bake_player_states.py` (lines 4–8) and `src/game.rs` (lines 90–91):
> *"The outer Batman sprite (691) selects states. Each state places a child movie clip whose true frame count differs from the outer label span. Baking the outer 271 frames truncates several animations. This tool resolves each label to its depth-1 child, renders every real child frame through the exact parent placement matrix, and keeps one common registration anchor at the outer Batman origin."*

In **B09** (`src/bank_goons.rs` lines 47–80 and `reverse-engineering/tools/b09_bake_bank_goons.py` lines 24–25, 115–164), the previous agent modeled `GoonFamily::A` (`#880`) and `GoonFamily::B` (`#964`) as having **identical 89-frame timelines** (`GOON_LABELS`) where every state except `stand` (`0..9`, 9 frames) is a 10-frame slice (`9..19`, `19..29`, `29..39`, `39..49`, `49..59`, `59..69`, `69..79`, `79..89`).

Inspecting the actual `child_placement_timeline` in B09's own generated `B09_ORIGINAL_BANK_GOON_SOURCE_REPORT.json` (from CI run `43091960107`) proves that `#880` and `#964` use the **exact same nested child MovieClip architecture as Batman `#691`**, and that their true child frame counts **differ both from the outer 9/10-frame spans and between Goon A and Goon B**:

| State Label | Parent Span (`GOON_LABELS`) | `#880` (`GoonFamily::A`) Depth-1 Child | `#880` True Child Frames | `#964` (`GoonFamily::B`) Depth-1 Child | `#964` True Child Frames | Bug When Using Outer 89-Frame Span (`b09_bake_bank_goons.py` / `src/bank_goons.rs`) |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| `stand` | `0..9` (`9f`) | `#799` (`born=0`) | **`1`** | `#883` (`born=0`) | **`1`** | Renders **9 duplicate copies** of static frame `0`. |
| `walk` | `9..19` (`10f`) | `#824` (`born=9`) | **`10`** | `#915` (`born=9`) | **`11`** | **`#964` (`GoonFamily::B`) `walk` is truncated!** Frame 10 (the 11th frame of `#915`) is never rendered (`cf=0..9` only). |
| `retreat` | `19..29` (`10f`) | `#834` (`born=19`) | **`10`** | `#923` (`born=19`) | **`11`** | **`#964` (`GoonFamily::B`) `retreat` is truncated!** Frame 10 (the 11th frame of `#923`) is never rendered (`cf=0..9` only). |
| `attack` | `29..39` (`10f`) | `#847` (`born=29`) | **`10`** | `#939` (`born=29`) | **`10`** | Matches 10-frame span. |
| `hurt` | `39..49` (`10f`) | `#854` (`born=39`) | **`6`** | `#944` (`born=39`) | **`6`** | **Wraps around (`% 6`) on parent frames `45..48`!** Re-renders child frames `0, 1, 2, 3` a second time (`cf = 0,1,2,3,4,5, 0,1,2,3`). |
| `fall` | `49..59` (`10f`) | `#873` (`born=49`) | **`8`** | `#959` (`born=49`) | **`8`** | **Wraps around (`% 8`) on parent frames `57..58`!** After falling to the floor on `cf=7`, parent frames `57..58` pop the goon back upright to `cf=0, 1`! |
| `die` | `59..69` (`10f`) | `#879` (`born=59`) | **`8`** | `#963` (`born=59`) | **`8`** | **Wraps around (`% 8`) on parent frames `67..68`!** Dead goon pops back upright to `cf=0, 1` on the last 2 parent frames! |
| `idle` | `69..79` (`10f`) | `#799` (`born=69`) | **`1`** | `#883` (`born=69`) | **`1`** | Renders **10 duplicate copies** of static frame `0`. |
| `intro` | `79..89` (`10f`) | `#799` (`born=69`) | **`1`** | `#939` (`born=79`) | **`10`** | `#880` keeps `#799` (`1` frame), while `#964` places `#939` (`10` frames, attack animation). |
| **Total** | **`89` outer** | — | **`55` true frames** | — | **`66` true frames** | Outer 89-frame bake has wrong lengths, truncates `#964`, and wraps one-shot states. |

#### Required Fix (Before / During B10–B11)
1. **Do not pack the 89 outer parent frames for runtime goon rendering.** Instead, bake `#880` (`55` true child frames) and `#964` (`66` true child frames) using `reverse-engineering/tools/bake_player_states.py --symbol 880` and `--symbol 964` (which already resolves depth-1 child MovieClips and renders their exact `child_source_frame_count` frames!).
2. **In `src/bank_goons.rs`**, replace the shared `GOON_LABELS` (`start..end` over `0..89`) with per-family child-clip descriptors (`GOON_A_CLIPS: [GoonClip; 9]` totaling `55` frames and `GOON_B_CLIPS: [GoonClip; 9]` totaling `66` frames), storing both `parent_label_frame: u16` (for provenance) and `child_symbol_id: u16` + `child_start: usize` + `child_len: usize` (mirroring `PLAYER_CLIPS` in `src/game.rs:92-120`).
3. Inspect the terminal frame `DoAction` inside child sprites `#854`/`#944` (`hurt`), `#873`/`#959` (`fall`), and `#879`/`#963` (`die`) to verify how each child clip signals completion to `_parent` (e.g. `_parent.gotoAndStop("stand")` vs stopping on the final frame or removing the clip).

---

### L2-CRIT-02 (High Severity — B05/B06/Roadmap Gap): No Full-Stage Level 1B Tile Pack Exists Before B13 Stage Activation

#### Evidence
- In **B05** (`b05_bake_bank_street_1x.py`) and **B06** (`b06_bake_bank_street_hq3x.py`), `bake_level_tiles.py` was invoked with `--only-tile` for only **9 sampled viewports** (`(-9000,-400), (-7200,0), (-5400,0), (-4200,0), (-2400,0), (-600,0), (0,0), (600,0), (1200,0)`).
- Bank Street (`#783`, placed at `(1034.65, 239.25)`) spans local bounds `[-11157.95, -838.75, 899.4, 527.0]`, which in game-world `600×400` tile coordinates spans `x in [-10200..=1800]` (21 columns) and `y in [-400..=400]` (`48` non-empty candidate tile origins).
- Furthermore, `bake_level_tiles.py` (lines 64–65, 206–209) currently **forbids** baking a complete depth-split layer:
  ```python
  if args.root_depth_range is not None:
      if args.only_tile is None:
          raise ValueError("partial root-depth layer may only be baked with --only-tile")
  if args.only_tile is not None or args.root_depth_range is not None:
      manifest["partial_proof_only"] = True
      manifest["never_embed_as_complete_level"] = True
  ```
- Yet `docs/revival/LEVEL2_IMPLEMENTATION_ROADMAP_2026-11-11.md` schedules:
  - `B10`: `goonAI`
  - `B11`: Goon combat & `respawn2`
  - `B12`: `bikerAI` (`#1005`)
  - `B13`: `level1b` checkpoints, music & `1B -> 1C` transition (**stage activation**)
  There is **no batch between B09 and B13** that bakes and packs the full 48-tile Level 1B `1x` (`level1b_tiles.bin`) and `HQ 3x` (`level1b_tiles_hq_base.bin` / `level1b_detail_tiles_hq.bin`) packs or wires them into `src/assets.rs` and `src/render.rs`!

#### Required Fix
1. Allow `bake_level_tiles.py` to bake a full stage with `--root-depth-range` when `--complete-depth-layer` (or omitting `--only-tile`) is explicitly requested for a bounded stage like `#783` (48 tiles), without setting `never_embed_as_complete_level = True`.
2. Schedule a dedicated **Level 1B Full Scenery & Actor Asset Pack Integration** step (either as part of B12/B13 or as B12b) before flipping `PendingStageEntry` into an active Level 1B transition in B13.

---

### L2-CRIT-03 (Medium-High Severity — B06 & `bake_sprite_frames.py`): Pillow `ImageDraw.line` Overwrites Destination Alpha Instead of Compositing

#### Evidence
In `reverse-engineering/tools/b06_bake_bank_street_hq3x.py` (lines 283–294), when comparing the flat 3× render of `#783` against the recomposition of the `lower` (`depth <= 626`) and `upper` (`depth > 626`) bands at `x = -4200`, standard alpha compositing (`Image.alpha_composite(layers[0], layers[1])`) produced **22,382 pixel mismatches**. The script worked around this by writing:
```python
combined = layers[0].copy()
binary_mask = layers[1].getchannel("A").point(lambda alpha: 255 if alpha else 0)
combined.paste(layers[1], (0, 0), binary_mask)
```
**Root cause in `reverse-engineering/tools/bake_sprite_frames.py` (lines 713, 804–808):**
- Solid/gradient/bitmap fills are composited onto `canvas` using `canvas.alpha_composite(...)` (Porter-Duff `source-over`).
- However, vector strokes (`line_paths`) are drawn directly onto `canvas` using `draw = ImageDraw.Draw(canvas)` and `draw.line(..., fill=style["color"], width=width)`.
- In Pillow, `ImageDraw.Draw(rgba_image).line(..., fill=(r, g, b, a))` **replaces** the destination pixel's `(R, G, B, A)` directly with `(r, g, b, a)` rather than alpha-blending it! When `a < 255`, drawing a semi-transparent stroke on `canvas` **erases the opaque alpha (`255 -> a`) of whatever background fill was already beneath it**!
- Worse, in `b06_bake_bank_street_hq3x.py`, using `combined.paste(layers[1], (0, 0), binary_mask)` means any anti-aliased or semi-transparent fill pixel `(r, g, b, a)` in `layers[1]` (`0 < a < 255`) overwrites the opaque street background in `layers[0]`, leaving holes with `alpha < 255` in the combined image—whereas at runtime, `src/render.rs::blit_tile_hq_1to1` uses `blend()` (true Porter-Duff `source-over` alpha blending)!

#### Required Fix
In `bake_sprite_frames.py` (around line 804), when `style["color"][3] < 255`, render strokes onto a temporary transparent RGBA layer (or use `alpha_composite`) rather than overwriting `canvas` pixels via `ImageDraw.Draw(canvas).line`—while keeping a legacy flag if needed to preserve exact historical Level 1A golden SHA-256 hashes.

---

### L2-ARCH-04 (Medium Severity — B07 `src/stage.rs` vs `src/game.rs`): Camera Easing & Grounded Vertical Tracking Divergence

#### Evidence
In **B07**, `SourceCamera::horizontal_step` and `SourceCamera::jumping_y` were added to `src/stage.rs` (lines 144–164). Comparing them against `Game::update_camera` in `src/game.rs` (lines 860–896) reveals two discrepancies that will cause camera bugs as soon as Level 1B is activated:
1. **`src/game.rs:881` hardcodes Level 1A's `150.0` (`CAM_HEIGHT`) in the grounded vertical camera branch:**
   ```rust
   if self.player.jumping {
       if ((self.camera_y + player_y - CAM_HEIGHT) / CAM_JUMP).abs() > 1.0 {
           self.camera_y -= ((self.camera_y + player_y - CAM_HEIGHT) / CAM_JUMP).trunc();
       }
   } else if (self.camera_y + player_y - 150.0).abs() > 1.0 {
       self.camera_y -= ((self.camera_y + player_y - 150.0) / 2.0).trunc();
   }
   ```
   Notice that `SourceCamera` in `src/stage.rs` only implemented `jumping_y` (the `if self.player.jumping` branch) and **omitted the `else if` grounded vertical camera return branch**, and `game.rs:881` hardcoded `150.0` instead of `self.height` (`125.0` for Level 1B, `270.0` for Level 1C, `275.0` for Level 2A/2B)!
2. **Parallax `background_x` / `background_y` (`* 0.1` in `game.rs:869, 875, 880, 883`):** `SourceCamera` does not track or return the `dx * 0.1` / `dy * 0.1` backdrop delta.

#### Required Fix
Add `SourceCamera::vertical_step(self, player_game_y: f32, camera_y: f32, jumping: bool) -> (f32, f32)` to `src/stage.rs` using `self.height` in both the jumping (`/ self.jump`) and grounded (`/ 2.0`) branches, returning `(new_camera_y, delta_y)` so `Game::update_camera` can use `SourceCamera` directly for all stages without duplicating camera math.

---

### L2-ARCH-05 (Medium Severity — B08 `src/bank_props.rs`): Verify Full Prop Timelines & AVM1 References Before B11/B13

#### Evidence
In **B08** (`src/bank_props.rs` and `b08_bake_bank_props.py`), only frame `0` of `#785` (`can_01..04`), `#790` (`trash_01..04`), and `#793` (`mailbox_01`) was inspected and rendered. While B08 honestly marked prop AVM1 interactions as `UNVERIFIED`, B10–B13 does not explicitly schedule a check of whether `#785`, `#790`, or `#793` have multi-frame timelines, nested child clips, or AVM1 references in `playerLogic` / `goonAI` / `bikerAI` / root frame 20 `DoAction`.

#### Required Fix
Before or during B10/B11, query `symbols.json` and `b01-original-avm1-functions.json` / `b01-original-root-stage-actions.json` for `"can_"`, `"trash_"`, `"mailbox"`, and symbols `785`, `790`, `793` (plus their child symbols) to definitively close whether they are purely static foreground/midground depth-sorted props (`1077..1097 < player 1127`) or interactive.

---

## 3. Part II — Rust Purity, Memory Architecture & Better Rust Code

### 3.1 Pure-Rust Runtime Purity Assessment

- **Crate Graph (`Cargo.toml`):** Only two external crates exist in `[dependencies]`: `png = "0.17"` and `flate2 = "1"`. Both are pure-Rust crates used for decoding embedded PNG/zlib assets.
- **Native / OS FFI Boundary:** Confined strictly to `src/win32.rs` (`user32`, `gdi32`, `kernel32`) and `src/audio.rs` (`winmm` `mciSendStringA`). Zero C/C++ gameplay libraries, zero embedded Flash/AVM1/Ruffle/SWF runtime, and zero Python/JS at runtime.
- **Verdict:** **100% Compliant with the Pure-Rust Runtime Mandate.**

---

### 3.2 Concrete Rust Engineering Improvements (`R-01` through `R-06`)

#### R-01 (High Impact — `src/collision.rs`): Bit-Pack `CollisionMask` into `Box<[u64]>` Instead of 1-Byte-Per-Pixel `Vec<u8>` (8× Memory Reduction) + Replace Recursion with Loops

**Problem:**
In `src/collision.rs` (lines 11–74), `CollisionMask` stores its binary solid mask as `solid: Vec<u8>`, allocating **1 full byte (`u8`) per pixel** (`0` or `1`), plus a temporary full-image `buf = vec![0u8; r.output_buffer_size()]` during `decode_mask`:
- **Level 1A (`11,036 × 1,044`):** `11,521,584` bytes (**11.5 MB** heap)
- **Level 1B (`12,102 × 1,413`, added in B07):** `17,100,126` bytes (**17.1 MB** heap)
- **Level 2A (`8,781 × 10,060`, planned in B21):** `88,336,860` bytes (**88.3 MB** heap!)
- **Level 2B (`8,781 × 10,060`, planned in B27):** `88,336,860` bytes (**88.3 MB** heap!)

Allocating nearly **200 MB of heap** just for 1-bit collision masks across stages—and thrashing the CPU L1/L2 cache during `check_ground` / `check_wall_clear` probes—is unnecessary in Rust. Additionally, `check_ground` (line 97) and `check_ceiling` (line 109) use recursion (`self.check_ground(x, y - 5.0, loop_count + 1)`) instead of a simple `for _ in 0..16` loop.

**Better Rust Code (`src/collision.rs`):**
Pack 64 pixels into each `u64` word (`words: Box<[u64]>`), reducing Level 1B from **17.1 MB -> 2.14 MB** and Level 2A/2B from **88.3 MB -> 11.04 MB** each (an **8× memory reduction**), and replace recursive ground/ceiling resolution with bounded loops:

```rust
pub struct CollisionMask {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
    words: Box<[u64]>,
}

impl CollisionMask {
    #[inline]
    pub fn hit_test_level(&self, x: f32, y: f32) -> bool {
        let px = x.floor() as i32 - self.origin_x;
        let py = y.floor() as i32 - self.origin_y;
        if px < 0 || py < 0 || px >= self.width as i32 || py >= self.height as i32 {
            return false;
        }
        let idx = py as usize * self.width as usize + px as usize;
        (self.words[idx >> 6] & (1u64 << (idx & 63))) != 0
    }

    pub fn check_ground(&self, x: f32, mut y: f32) -> (f32, bool) {
        let mut adjusted = false;
        for _ in 0..16 {
            if self.hit_test_level(x, y - 3.0) {
                y -= 5.0;
                adjusted = true;
            } else {
                break;
            }
        }
        if adjusted {
            for _ in 0..10 {
                if !self.hit_test_level(x, y + 1.0) {
                    y += 1.0;
                } else {
                    break;
                }
            }
        }
        (y, false)
    }

    pub fn check_ceiling(&self, x: f32, mut y: f32, height: f32) -> (f32, bool) {
        for _ in 0..16 {
            if self.hit_test_level(x, y - height) {
                y += 5.0;
            } else {
                return (y, false);
            }
        }
        (y, true)
    }
}
```

---

#### R-02 (High Impact — `src/stage.rs`, `src/bank_props.rs`, `src/bank_goons.rs`): Introduce Zero-Cost Coordinate & Depth Newtypes and Consolidate Stage Metadata

**Problem:**
Across B02 (`src/stage.rs`), B08 (`src/bank_props.rs`), and B09 (`src/bank_goons.rs`), the codebase has accumulated three separate `#![allow(dead_code)]` modules that each re-implement coordinate conversions between **GAME-local coordinates** (parent sprite `#1075`) and **LEVEL-local coordinates** (child sprite `#783` at `(1034.65, 239.25)`):
- `StageDescriptor::player_spawn_level_local` returns `(f32, f32)` in `src/stage.rs:124`.
- `BankStreetProp::origin_level_local` returns `(f32, f32)` in `src/bank_props.rs:58`.
- `BankGoonSpawn::level_local` returns `(f32, f32)` in `src/bank_goons.rs:106`.

Because both coordinate spaces are represented as raw `f32` or `(f32, f32)`, the compiler cannot catch passing a `game_local` coordinate (`528.15, -18.75`) into `CollisionMask::hit_test_level` (which expects `level_local` `-506.5, -258.0`)!

**Better Rust Code:**
Define zero-cost newtypes in `src/stage.rs` so the Rust type system enforces coordinate-space and display-list-depth invariants at compile time:

```rust
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GamePos {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LevelPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GameDepth(pub u16);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LevelDepth(pub u16);

impl StageDescriptor {
    #[inline]
    pub const fn to_level_pos(&self, pos: GamePos) -> LevelPos {
        LevelPos {
            x: pos.x - self.level_origin_x,
            y: pos.y - self.level_origin_y,
        }
    }
}
```
Additionally, group Bank Street's props and goon tables under a single `src/bank_street.rs` (or `stage::bank_street`) module referenced by `StageDescriptor`, and reference their invariants from a single deterministic registry check so `#![allow(dead_code)]` does not need to blanket entire files.

---

#### R-03 (Medium Impact — `src/render.rs`): Eliminate Per-Frame Heap Allocations in `upscale_opaque_base`, `upscale_alpha_layer`, and HUD Number Drawing

**Problem:**
1. In `src/render.rs`, both `upscale_opaque_base` (lines 110–120) and `upscale_alpha_layer` (lines 157–167) allocate a `Vec::with_capacity(dst.w)` (`1,800` tuples = `28.8 KB`) on the heap **every single frame** (50 heap allocations/sec at 25 Hz) and recompute 1,800 `div_euclid`/`rem_euclid` operations, even though `src.w == 600` and `dst.w == 1800` (`HQ_W`) are compile-time constants!
2. In `src/render.rs`, `draw_number_hq_1to1` (line 489) and `draw_number_centered` (line 532) call `value.max(0).to_string()`, allocating a `String` on the heap 3 times every frame to draw the HUD counters (`batarangs`, `lives`, `score`).

**Better Rust Code:**
1. Precompute the `600 -> 1800` bilinear horizontal lookup table at compile time (`const XMAP_600_TO_1800: [(usize, usize, u32, u32); HQ_W]`) or in a stack array, eliminating both per-frame heap allocations and the 3,600 per-frame integer divisions.
2. Format HUD integers into a small stack buffer (`let mut buf = [0u8; 10];`) without heap allocation.

---

#### R-04 (Medium Impact — `src/assets.rs`): Split `Assets` into `CommonAssets` and Lazy/Per-Stage `StageAssets`

**Problem:**
`Assets::load()` in `src/assets.rs` (lines 136–215) eagerly decodes all embedded binary packs (`level1a_tiles.bin`, `level1a_tiles_hq_base.bin`, `level1a_detail_tiles_hq.bin`, `level1a_ground.png`, etc.) into memory at startup. Once Levels `1B`, `1C`, `2A`, and `2B` (plus story cards `1267`, `1479`, `1569`, goon families `#880/#964/#1365`, biker `#1005`, Penguin `#1250`, eagle `#1393`, Lab doors/elevators `#1325/#1328/#1330`) are added across B10–B32, decoding all 5 stages' HQ 3× tiles and collision masks simultaneously at startup will balloon peak RAM by hundreds of megabytes and stall window creation.

**Better Rust Code:**
Separate stage-invariant assets from stage-specific assets:
- `CommonAssets`: Batman (`#691`), HUD (`#716`), Tutorial (`#737`), Title/Intro (`root_intro.bin`), Fonts (`impact_*`). Loaded once at startup.
- `StageAssets`: `level_tiles`, `level_tiles_hq_base`, `level_detail_tiles_hq`, `collision_mask`, and stage-specific enemy/prop sprite packs. Loaded on demand when entering a `StageId` (dropping the previous stage's `StageAssets` first).

---

#### R-05 (Medium Impact — `src/win32.rs` & `src/audio.rs`): Fix `static mut APP` Reentrancy Hazard (`L1A-QA-02`) and Pause Gameplay Music on Focus Loss (`L1A-QA-01`)

**Problem:**
1. In `src/win32.rs` (line 160), `static mut APP: *mut App = null_mut();` is dereferenced as `let app = &mut *APP;` inside `wndproc`. In `WM_KEYDOWN` (line 206), while `app: &mut App` is live on the stack, `app.toggle_fullscreen(hwnd)` calls `SetWindowPos(...)` and `InvalidateRect(...)`. On Win32, `SetWindowPos` **synchronously dispatches** `WM_WINDOWPOSCHANGED` / `WM_SIZE` / `WM_DPICHANGED` back into `wndproc` on the same thread, which executes `let app = &mut *APP;` a second time while the outer `&mut App` reference is still active—creating **aliased `&mut App` references (instant Undefined Behavior in Rust)**! Furthermore, `static mut` references are denied by default in Rust 2024 (`static_mut_refs`).
2. In `src/audio.rs` (lines 101–122), `Audio::on_focus_changed` pauses/resumes `bcb_intro` when the window loses/gains focus, but **ignores `bcb_music`** (the looping gameplay music track), so gameplay music keeps playing in the background when the user Alt-Tabs away!

**Better Rust Code:**
1. Store `Box<App>` in the window's user data (`SetWindowLongPtrW(hwnd, GWLP_USERDATA, ...)`), or wrap state access in `Cell`/`RefCell` so window-resizing Win32 API calls (`SetWindowPos`, `ShowCursor`, `InvalidateRect`) happen **after** the `&mut AppState` borrow is dropped.
2. In `Audio::on_focus_changed`, also issue `pause bcb_music` on focus loss and `resume bcb_music` on focus restore when `self.music_playing` is true.

---

#### R-06 (Low Impact — `src/headless.rs`): Replace `format!("{:?}", state)` String Allocations in Tests

In `src/headless.rs` (lines 147, 152, 165), tests use `assert_eq!(format!("{:?}", game.player.state), "Walk");`. Adding `#[derive(Debug)]` to `PlayerState` in `src/game.rs` allows idiomatic `assert_eq!(game.player.state, PlayerState::Walk);` with zero string formatting.

---

## 4. Part III — Where Additional Online / Technical Research & Tooling Are Needed

### 4.1 Decompile Full AVM1 Control Flow for `goonAI`, `bikerAI`, `respawn2`, and `penguinAI` (Before B10–B15)
- **Why needed:** `reverse-engineering/tools/b01_source_probe.py` (line 78) only extracted a flat `[-13..+5]` instruction window around each `If`/`Jump` (`windowed_conditions`) and explicitly warned: `"qualification": "Nearby stack-machine operations only; no proven logical predicate."`
- Specifically, `combat-ai.json` notes:
  > *"The 20-damage path is gated by playerAttacking/collision state in the AI; do not simplify it to a single named attack without reproducing the original conditions."*
- Before coding `goonAI` in **B10** and combat damage in **B11**, write a small offline AVM1 stack/branch analyzer (or inspect the full `actions` list in `b01-original-avm1-functions.json` for `goonAI`, `bikerAI`, and `respawn2`) to recover the exact branch targets, distance thresholds, attack cooldown counters, `eKills == 8` wave trigger, and knockback velocities.

### 4.2 Flash Player 7 / Ruffle Reference Research for `MovieClip.hitTest` & Child MovieClip Lifecycle
- **Why needed:**
  1. In AVM1 (Flash 7), `mc.hitTest(x, y, true)` tests `(x, y)` in **Root/Stage coordinates** (`_root._xmouse`, or `localToGlobal`), whereas `CollisionMask::hit_test_level` tests in **Level-local coordinates**. Verify in `goonAI` / `bikerAI` / `penguinAI` whether enemy ground/wall probes call `level.ground.hitTest(x, y, true)` and how `_parent` / `level._x` offsets are applied.
  2. In Flash 7, when an outer sprite executes `gotoAndStop("hurt")`, the depth-1 child clip placed at that frame starts at child frame `0` (`1` in ActionScript) and advances once per root frame tick (`25 Hz`), running its child frame scripts (`DoAction`) at the end of the child timeline. Cross-checking against Ruffle's `avm1/globals/movie_clip.rs` and `display_object/movie_clip.rs` confirms why **L2-CRIT-01** (baking child timelines rather than outer label intervals) is essential for faithful state durations.

### 4.3 Consolidate GitHub Actions Workflows & Cache the Canonical SWF
- **Why needed:** Batches `B01`, `B03`, `B05`, `B06`, `B08`, and `B09` each created a brand-new workflow file under `.github/workflows/b0X-*.yml` that independently downloads the 2.8 MB executable from `web.archive.org` via `curl`.
- If `web.archive.org` experiences transient throttling or downtime, CI fails across all workflows, and adding 23 more workflows for `B10`–`B32` will clutter `.github/workflows/`.
- **Recommendation:** Cache the extracted/verified SWF by its SHA-256 (`actions/cache@v4` keyed on `3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed`) with `web.archive.org` as fallback, and consolidate offline source-proof scripts into a single parameterized workflow or stage-verification job.

---

## 5. Priority Action Checklist for the Next Agent Turn (Before Starting B10)

1. **[MUST FIX FIRST — L2-CRIT-01]** Update B09/B10 goon sprite representation in `src/bank_goons.rs` and offline baking so `GoonFamily::A` (`#880`, 55 true child frames) and `GoonFamily::B` (`#964`, 66 true child frames) use their **depth-1 child MovieClip timelines** (via `bake_player_states.py` logic), fixing `#964` `walk`/`retreat` 11-frame truncation and `hurt`/`fall`/`die` 6/8-frame wraparound.
2. **[MUST PLAN — L2-CRIT-02]** Add full-stage Level 1B tile baking & packing (`48` tiles of `#783` at `1x` and `HQ 3x`) before B13 stage activation, and lift `bake_level_tiles.py`'s `--only-tile` restriction for bounded full-stage depth splits.
3. **[MUST VERIFY — L2-ARCH-04 & L2-ARCH-05]** Add the grounded vertical camera return (`(camera_y + player_y - self.height) / 2.0`) to `SourceCamera` in `src/stage.rs` (fixing hardcoded `150.0` in `src/game.rs:881`), and verify whether props `#785/#790/#793` have any AVM1 references or multi-frame states.
4. **[RECOMMENDED RUST UPGRADES — R-01..R-05]** Bit-pack `CollisionMask` into `Box<[u64]>` (8× less RAM before Labs 2A/2B), introduce `GamePos`/`LevelPos` newtypes in `src/stage.rs`, make `xmap` in `src/render.rs` zero-allocation, split `StageAssets` from `CommonAssets`, and fix `Audio::on_focus_changed` for `bcb_music`.

## 2026-11-12 addendum — independently measured L2-CRIT-02 tile count corrects earlier 48 estimate

The original static-scene analysis above called for **48 nonempty 1B tiles** before obtaining a complete full-grid original-source bake; that value was an **unverified/incorrect estimate**, not a true source fact. [Source Windows #43170888784](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43170888784) intentionally failed its strict old assumption after producing **75** actual nonempty original-source #783 1× PNGs. Downloaded failed artifact #13921972602 was inspected. New exact source code [`26dad6e`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/26dad6e07ef3dc1ca860aa98b4a8ccfb12912f72) and [Windows #43171300111](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43171300111) passed true full bounds-derived `19 columns × 4 rows = 76` candidates, **75 nonempty**, with **9/9 old B05 RGBA hashes unchanged**, a validated BCLVT001 75-tile 253090-byte test pack, negative incomplete-pack rejection, and downloaded/verified [source artifact #13922092240](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43171300111/artifacts/13922092240). y(-400,0,400,800) row counts=18/19/19/19; only blank origin (1800,-400). This is extracted source-library static art, not Flash-live gameplay. **L2-CRIT-02 stays PARTIAL:** full original 1× evidence/pack proof done only in CI, HQ3× full authored depth, Rust stage integration and activation still TODO; Level1A native [#43171300209](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43171300209) pending at first correction entry. See RE-20261112-008/AUD-20261112-008. Preserve original audit text as historical evidence rather than replacing it.


### 2026-11-12 final audit-correction verification (AUDIT-FIX-05)

Superseding the historic unmeasured "48 tiles" guess: actual original SWF Bank #783 has **75 nonempty/76 grid origins**, exact source CI [#43171300111](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43171300111) SUCCESS on CODE `26dad6e07ef3dc1ca860aa98b4a8ccfb12912f72`, verified 9/9 B05 old decoded 1× hashes and a dev-only complete 253090-byte BCLVT001 proof. Strict native [#43171300209](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43171300209) **SUCCESS 82/82** on same code, 42/42 Rust, 15/15 legacy1× screenshot SHA, installed 20/20 HQ screenshot equality and p95 20.84/26.1223ms under40. Hosted speaker audio MCI277/no output remains BLOCKED. Full HQ3× authored depth, runtime activation, original Flash player comparison not addressed. L2-CRIT-02 remains only PARTIAL; document source estimate mistake, not a dropped 27-tile asset set.

