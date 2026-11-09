# Level 1A revival — intro story and HUD presentation engineering log

Status: **implementation candidate / manual fidelity review required**  
Working branch: `revive/level1a-intro-hud-fidelity-2026-11-09`  
Predecessor: `revive/round11-ci-baseline-2026-11-09`  
Original reference: verified 600 × 400, 25 FPS The Batman: The CobbleBot Caper SWF.  
Date: 2026-11-09 (UTC).

This document is a reproducible **engineering decision and test log**, not private internal deliberation. Every assumption is labeled and failures are recorded so future development can independently validate or revise it. No original proprietary SWF, bitmap, audio, or font has been committed to Git.

## Why this work was started

A real Windows playtest of CI Run #110 identified:
1. The native title/controls screen jumped directly into Level 1A; the original game has a phone/briefing story intro with animated panels.
2. The HUD's generic Arial lettering, white score/lives numbers, and visual spacing looked incongruent with the Batman artwork.

The priority remains *finish Level 1A before Level 1B*. Gameplay, enemy logic, collision, and camera should not be rewritten during the presentation sprint.

## Starting evidence and stable checkpoint

- A prior effort repaired the stale Round 11 string-based CI assertion, which was looking for `target.fill(0xff000000);` even though the HQ renderer had changed.
- [Run #110](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42561171602) passed all Windows checks and produced a working executable; it is the rollback reference, not a manually approved fidelity release.
- [PR #16](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/16) contains the isolated previous CI fix.
- Game code then had `AppScreen {Title,Instructions,Playing}`; pressing Enter called `Playing` immediately.
- The existing HUD was rebuilt from SWF sprite **716**. Static EditText fields: **700 BATARANG, 710 HEALTH, 711 SCORE, 715 lives x**. Dynamic fields: **702 Batarangs, 713 score, 714 lives**.
- The HUD panel's source translucency (CXFORM alpha 82/256) is real original artwork, not a placeholder. The 1× HUD remains the visual source oracle.

## Step-by-step changes

### 1. Preserve the prior stable branch and isolate this sprint

Created `revive/level1a-intro-hud-fidelity-2026-11-09` from the green Round 11 recovery branch. Do not merge into `main` until Windows manual evaluation passes.

### 2. Inspect the original scene structure, instead of recreating a fictional cutscene

Added `reverse-engineering/tools/inspect_intro_hud.py`; Windows CI emits `.generated/intro-hud-source-report.json`.

Findings from the verified SWF:
- The intro scene is the original **root frame 18**.
- Root frame 18 places source graphics/symbols **40, 122, 123, 126, 129, 24, 34, 35** at original depths/transforms.
- Recovered sprite **122 has 165 frames**; sprites **123 and 126 have 180 frames**.
- At **25 FPS**, 180 timeline samples represent **7.2 seconds**, before allowing for any original ActionScript pauses/branching.
- The original root dispatcher contains an `intro` route before `level1a`.

### 3. Restore source-derived intro art into the Rust build

Added `reverse-engineering/tools/bake_root_intro.py` to composite source root-frame-18 placements with the existing vetted Flash shape, bitmap, depth and text rasterizer at native 600 × 400. The build packs the result as `assets/intro_frames.bin` (generated during CI, **not committed**) and loads 180 frames lazily. The native gameplay screen is never covered with a hand-drawn story substitute.

Known source limitation:
- **Symbol 129** is an interactive Flash `DefineButton2` rather than a normal sprite. The initial pipeline correctly failed on a missing symbol, so the first green candidate temporarily omitted that control. Inspecting the downloaded `00_intro_call.png` then revealed the **blank story button**, an unacceptable incomplete presentation. Follow-up change `5babab8` parses original button records, recovers the **UP-state graphics** with original child placement matrices, depths and color transforms, then injects them as a synthetic drawable one-frame sprite. Hover/down/click ActionScript is still not simulated; native Enter/Space/Escape remains the interaction/skip behavior until the original is compared manually.
- Original sound #130 is now wired to intro entry, with source-motivated stop before gameplay. **Remaining** ActionScript-driven holds/pauses and original interaction timing have **not** been fully translated. The current code advances 180 visual samples, then enters Level 1A.
- A **10-frame contact sheet** at source ticks 0,20,...,160,179 is now generated and uploaded by CI to catch source-asset omissions/abrupt cuts that the two smoke screenshots cannot reveal.
- **Symbol 34** is the root scripted fade-out clip. It must not auto-play and repeatedly black out the story; the intro visual baker now defers this clip entirely. Its exact original transition remains an acceptance item.

### 4. Restore the pre-level runtime route

Changed `src/game.rs`:
- `AppScreen::Intro` added; Enter/Space on Title or Instructions enters intro rather than gameplay.
- The intro frame advances every original 40 ms tick.
- At the end of the 180-sample presentation, the game enters Level 1A and dispatches one `MusicStart` event.
- Enter/Space/Escape can skip the intro during testing.
- Existing movement, physics, collision, attacks, camera, score, pickups and Level 1A completion behavior otherwise remain unchanged.

Changed `src/assets.rs` and `src/render.rs` to embed and display the original 1× intro frames. The existing HQ 3× presentation path upscales the original logical frames; no alternate vector HQ rebuild is claimed for the intro yet.

### 5. Add visible and automated regression checks

Changed `src/headless.rs`:
- Verify initial Enter reaches `Intro` with **no premature Level 1A music**.
- Capture `00_intro_start.png` and `00_intro_call.png`.
- Advance 180 ticks; verify gameplay starts with ticks reset to 0 and exactly one music event.
- Test manual intro skip and normal gameplay smoke captures.
- Keep the HQ render benchmark targeted to gameplay, not the opening story.

Windows CI reconstructs intro from the downloaded, hash-verified SWF; validates 180 frames, expected scene frame, no undocumented unsupported fills or missing glyph/bitmap references, and a visible difference between first and mid-scene samples.

### 6. HUD typography candidate — explicit design tradeoff

The user requested a presentation that better fits Batman's art style. The actual original HUD panel/icons, field source geometry, colors/alpha transforms and 600×400 placement are retained.

`reverse-engineering/tools/bake_hud_assets.py` now supports:
- `--style source` (unchanged legacy 1× source-device-font comparison path);
- `--style themed` (proposed HQ-only alternative): Arial Bold at the original source field sizes, dark text edging, muted gold labels, and high-contrast digit atlases. CI uses only the installed Windows `arialbd.ttf` font; no font files are distributed.

**This is a deliberate optional stylistic departure, not a false claim that the original Flash used gold Arial Bold.** The HQ output uses the candidate styling by default for testing; the existing legacy assets remain the reference. An [original-game HUD screenshot](https://images.igdb.com/igdb/image/upload/t_1080p/scymjx.png) confirms the Flash version used **white** text and digits, not gold. Compare both screenshots manually; if strict pixel fidelity wins, revert HQ to `--style source`. Never claim the gold candidate is a restoration of the original palette.

## Failures / corrective actions

| Evidence | Problem | Action | Verification |
|---|---|---|---|
| Original Round 11 CI | Validator referenced deleted source text | Repair validator on separate recovery branch | Run #110: PASS |
| Intro bake first CI run | Sprite renderer cannot draw Flash button 129; missing symbol reported | Temporary omission allowed source-animation validation; tracked explicitly | Later green CI, but screenshot review found a blank button |
| Visual audit after Run #42569523565 | `00_intro_call.png` showed Batman and “I'M ON MY WAY,” **but button control was visibly blank** | Parse the actual SWF DefineButton2 UP-state records into original sprite art; require button record presence in CI | New Windows CI + ten-frame contact sheet pending |
| HUD screenshot | Label/digit color and weight look synthetic | Add HQ-only source-geometry-preserving themed style, keep original 1× oracle | Inspect HUD A/B screenshot artifacts and perform Windows playtest |
| Intro source behavior | Timeline frames known, but ActionScript stops/sounds not translated | Mark as incomplete and preserve source analysis for next correction | Manual original-vs-native cutscene timing/audio gate pending |
| First complete integration run (Run #42568860291) | Original CI smoke expected 17 screenshots; two new intro snapshots correctly produced 19 | Update both screenshot-count gates to 19, preserve all 15 historical gameplay SHA-256 pixel checks, and allow *only* the two named intro additions | Re-run Windows CI; Rust compile/test succeeded before this integration-only failure |

### 7. Make the recovered SKIP button actually clickable

Visual review of the [successful button recovery run #42571136795](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42571136795) confirmed the original `SKIP` lettering, original bank-alert text, Batman's reply, and multiple camera/panel scenes in the generated ten-frame contact sheet. The full Windows pipeline passed **55 steps**.

However, `WM_LBUTTONDOWN` previously only set focus: the restored SKIP graphic could not respond to mouse input. The subsequent change:
- derives a visible UP-state rectangle from original sprite 129's original root placement matrix via `symbol_bounds`;
- emits `assets/intro_button_bounds.rs` **during CI only**, with no guessed native hotspot;
- maps mouse pixels through the existing DPI-aware, letterboxed 600×400 stage viewport;
- allows only clicks inside that source-derived rectangle, while `AppScreen::Intro` is active, to call `Game::skip_intro`;
- tests stage mapping, click bounds, and a single `MusicStart` even if the transition is triggered more than once.

**Implementation correction caught before CI:** the first button-bounds edit wrote the generated file before the output directory was created. The source was immediately updated to create the directory first. The Windows build is the definitive validation; no local Windows playtest has been claimed.

The source-up-state hitbox approximates the Flash button's clickable `HitTest` shape; it is grounded in original visible geometry, but a true `HitTest`-state polygon match is still a manual-fidelity follow-up.

### 8. Recover the exact original intro soundtrack from SWF sound events

The focused source audit `reverse-engineering/tools/inspect_intro_behavior.py` was committed and executed in fully green [Run #42574699640](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42574699640). Its `Batman-CobbleBot-Level1A-Intro-Behavior-Report` artifact records:

- **Root frame 18** contains exactly one directly reached `StartSound` event for **DefineSound #130** (`0130_sound_0130.mp3`). The original source encodes `sound_flags=4` (loop information present).
- Source root frame **18** executes `Stop`: the root clip deliberately stops here while the nested story sprites animate.
- Nested sprite **122** executes `Stop` on frame **164**.
- Nested sprite **123** executes `Stop` and `StopSounds` on frame **179**.
- No other direct `StartSound` events were identified in the scanned reachable intro sprite timelines. There is no evidence requiring speech/voice acting or invented effects.

Corrections committed:
1. `assets/intro_sound.mp3` is extracted from verified source sound #130 **during CI** (not checked into Git).
2. `src/audio.rs` opens a dedicated WinMM/MCI `bcb_intro` alias from the extracted original MP3; `IntroStart` seeks to the beginning and plays it exactly at native intro entry.
3. `src/game.rs` dispatches `IntroStop` followed by `MusicStart` when the intro finishes or is skipped, so the source audio does not overlap with gameplay music.
4. The headless regression tests verify event order for normal ending, keyboard skip, and mouse-click skip, including idempotent mouse handling.
5. CI fails if the original `StartSound` or final `StopSounds` evidence changes, or if the source MP3 cannot be extracted.

**Remaining uncertainty:** the native 180-sample sequence currently auto-advances to Level 1A. The original movie's root `Stop` and nested sprite stops show intentional timeline control; the exact user-interaction sequence and on-stage holds are not yet reproduced. WinMM audio playback and synchronization still require real Windows speaker testing. We must not claim manually verified fidelity from the action log alone.

## Acceptance gate before merge or Level 1B

- [x] First full intro build **Run #42569523565** passed all 54 Windows steps, including original gameplay screenshot hashes and 19 visual-smoke screenshots. The next button-art revision needs its own full green run.
- [x] Source-button gallery build Run #42571136795 passed all 55 Windows steps; the original SKIP lettering is present in downloaded HQ screenshots.
- [x] Clickable SKIP build **Run #42572833961** passed all 55 Windows workflow steps; source-derived mouse hitbox tests passed.
- [x] Focused original intro audio/actions audit **Run #42574699640** passed Windows CI and identified sound #130 / frame179 StopSounds.
- [ ] Latest intro-audio-wired Windows CI passes compilation, gameplay regressions and artifact publication.
- [ ] CI intro screenshots show meaningful original call/briefing art with no giant blank/missing portions.
- [ ] Original Flash story text, button visuals, timing/stops, and sound effects are compared against actual native Windows playback; audio #130 is source-backed but not yet speaker-tested.
- [ ] Title → intro → gameplay works with keyboard alone, including skip and music timing.
- [ ] Themed HQ HUD labels and score are visually acceptable **or** restored to source style.
- [ ] Player can finish Level 1A with movement, attacks, camera, pickups, audio and fade unchanged.
- [ ] Human acceptance recorded in [issue #17](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/issues/17).
- [ ] Only then merge an approved presentation fix and consider Level 1B.

## Safe rollback

The original Windows artifact is from [Run #110](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42561171602). This branch changes no assets in Git history and can be abandoned without affecting `main` or the preceding experimental PR chain.
