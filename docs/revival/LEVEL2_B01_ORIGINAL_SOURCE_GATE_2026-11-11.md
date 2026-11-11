# B01 — original Flash source verification and next-stage evidence

**Date:** 2026-11-11. **Outcome:** source-evidence B01 **PASSED** on genuine Windows CI; **no native gameplay changes**.

**Verified final Windows run:** https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42981445349  
**Real downloadable evidence artifact #13837522331:** https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42981445349/artifacts/13837522331  
**Code SHA of successful source test:** ea5e75727703c6cc66584be42559815bebd3d3a9.  
**Tool:** reverse-engineering/tools/b01_source_probe.py.  
**Workflow:** .github/workflows/b01-original-source-gate.yml.  
**Frozen accepted Level1A release:** level1a-v0.2.0 / source SHA 49e8fd343e4f37f045ba71f24fff0999211f0f11 — unchanged.

## What the real Windows runner verified

The existing Archive.org source URL was downloaded again, canonicalized to uncompressed FWS7, and freshly SHA-256 matched **3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed** (2,819,100 bytes; 600×400; 25FPS). The source parser recovered **342 sprites, 231 ActionScript DoAction blocks, 79/79 successfully decoded bitmaps, and 38 sounds** (33 MP3, 5 ADPCM).

It regenerated six historical source-manifest families. Five — movie, routing-evidence, function-index, animation-labels, and level-constants-raw — **matched exactly**. The sixth, level-instances, **did not match as complete JSON**: newly extracted CXFORM and other placement metadata change the representation; the first difference is $.level1a.ground.cxform. Crucially, a STRICT semantic equality assertion verified all 4 relevant stages (1B, Penguin1C, Labs2A and Labs2B) retained exact game/level IDs, player/ground, all named objects, source IDs, depths, flags, frame indices, and original six-component transforms (33, 11, 61 and 49 named objects respectively). Do not silently claim all nine stages or every metadata field were identical.

The new source evidence artifact includes full raw AVM1 action records for root indices **20/21/23/25**, 13 targeted source functions with branch windows and original byte offsets, direct timeline sound cues, manifest-parity JSON, explicit critical-source assertions and **five source-symbol first-frame PNGs**. Images depict original goon #880, biker #1005, Penguin #1250, Lab door #1325 and Lab elevator #1328, rendered in 600×400 canvases at 0.5 scale. These are **static original-library sprite renderings**, NOT original Flash gameplay screenshots or verified camera movements. It does **not** upload the proprietary SWF, full bitmap or soundtrack packs.

**Attempt history:** CI #42980621296 FAILED on an overly strict full-JSON equality assertion; CI #42980830711 PASSED after honestly reporting drift; final CI #42981445349 PASSED after asserting actual gameplay-critical four-stage semantic parity and original exit/timer bytecode presence. Preserve history, don't hide the initial failure.

## Original-source mechanics newly established

### Level 1B — second wave and transition

Source root parser frame **20** contains Gotham Bank streets game #1075, level #783, ground #782, ten initial goons #880/#964, one biker #1005, four Batarang pickups and three checkpoints. The root initialization creates a goon-only enemy array by identifying instance names beginning with “goon”; the biker receives a separate bikerAI callback. Original initial goon stats are health **32**, strength **5**, speed **9**; biker strength **10**.

At original FWS byte **741171**, the root script binds the respawn object's onEnterFrame to global **respawn2**, function-body start **145903**. The source AVM1 stack control checks **respawned == 0**, **eKills > 0** and **eKills modulo enemies.length == 0** (offsets roughly **145903–145998**). When met, the function loops **eight entries**, reactivating enemies with health **50**, speed **9**, state **walk**, and reset attack/fall/visibility; their X positions depend on player X by **500 ± random(600)** and a side/direction random branch (loop source **146040–146515**). It then resets eKills and increments respawned.

Another original AVM1 branch checks **respawned > 0** and **eKills == 8**, then increments camHeight by **15**, writes **gotoNext = level1c** at **146625**, and runs fadeout.play near **146635**. This explains why the level requires a **second eight-enemy wave** before the Penguin, rather than automatically finishing after walking to an exit or simply defeating the ten initial goons. This is direct stack-bytecode interpretation, not a live Flash-player observation. Future QA must still verify activation/race conditions and attack timing.

### Level 2B — root-level 80-second timer

Root parser frame **25** explicitly attaches global **clockTimer** to **_root.clock.onEnterFrame** near FWS byte **1292741**. This resolves the old “clock listed in compact spec but no named game clock object” puzzle: the clock belongs to the **root**, not the game-level named-placement list.

Global clockTimer code begins at **148826** (460 bytes). It initializes **secs = 80** and **millisecs = 99** (near offsets 148882 and 148895), uses AVM1 **GetTime** / elapsed real milliseconds, and on timeout displays 00 :00, sets **playerHealth = 0** (149188), **playerState = die** (149205), triggers explosion/halo-fire effects and removes timer callback. This time pressure must be implemented in B29; unlike a fixed-tick approximation, native focus-pause/background behavior needs specific handling.

### Other source evidence

**DoorOpen** global source function starts at **152905** (140 bytes). It uses **player hitTest** and door clip frame predicates: when currentframe equals **1**, play the door.open animation; at **10**, play door.close. Do not use generic permanently-opening doors.

**PenguinAI** starts at **124287** (3920 bytes); source action at **128128** routes to **card:escape**. Penguin source init is 200 HP/speed 6 and a misspelled property “stength” associated with value 5; preserve original semantics until usage is audited. Goons, biker, bird, cameraLogic2 and elevatorLogic source disassemblies, exact offsets and conditional branch neighborhoods are included in artifact. Their complete behavior has **not** been ported or frame-timed in B01.

**Verified story-card routes:** escape sprite #1267, local zero-based frame **224** → Level2A; TV sprite #1479 frame **151** → Level2B; Kabuki sprite #1569 frame **159** → Level3A. TV card #1479 has a local StartSound for exported **explode #11** at local frame 106. The nested clip timeline cannot be equated with global playback without reading its script start/stop controls.

### Root-stage StartSound cues (verified tags, not verified audible playback)

| Source stage, parser frame index | Source sound ID | Indexed MP3 duration |
|---|---:|---:|
| Bank1B — 20 | **1076** | approximately **3.902s** |
| Penguin1C — 21 | **1252** | approximately **10.786s** |
| Gotham Labs2A — 23 | **1409** | approximately **5.889s** |
| Gotham Labs2B — 25 | **1504** | approximately **5.889s** |

Actual root StartSound tag offsets in source: **870268**, **1010504**, **1222394**, **1325645**, respectively. These MP3 IDs are source-timeline cues, not assertions of soundtrack looping, audible physical device operation, or original gameplay elapsed time. Source SWF and source-renderer proof are available through the CI workflow for future reproducibility.

## What B01 does NOT verify

B01 did **not** run a Flash projector or Ruffle session to compare actual camera, input, enemy randomness, hitboxes, exact fight frames or full-stage screenshots, and did **not** compile or play Level1B, Penguin or Labs in Rust. The isolated Windows job is a real **source validation** gate only; it deliberately does not mutate the finished native Level1A source, executable, public release, main, historical Level1B branches or PR #18. Complete metadata differences beyond stage-critical placements, precise audio loops, exact AI action triggers and visual parity remain explicitly open. Historical Python dependency/action pinning and unsafe native-runtime risks remain in issue #19; no security certification claimed.

## Next task — B02 only (separate user authorization)

Add the smallest immutable stage-ID/source-descriptor model to Rust while leaving the default running game **exactly Level1A**. Distinguish persistent root score/lives/ammo from stage-local enemy waves, player health, camera, collision and checkpoints; source info now proves important stage-specific behavior. B02 must not bake Lab artwork (B03), wire live Level1A→1B gameplay transition (B04), rewrite original movement/combat or start B05. Verify all original 1×/HQ snapshot hashes, Windows build/GUI/fullscreen/icon/focus and performance via genuine Windows CI, then update six records and STOP.
