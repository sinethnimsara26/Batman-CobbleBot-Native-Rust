# B02 — immutable source-backed stage descriptors (no gameplay changes)

## FINAL WINDOWS-CI VERIFICATION — SUCCESS (supersedes provisional wording below)

**Actual code SHA:** 18920bc12b8f27d835d8a574f0b47548c9a3b59b. **Genuine native Windows build and original-source regression** [Actions #42990726444](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42990726444) was **COMPLETED/SUCCESS, all 81/81 steps**. Six new B02 stage tests passed; **27/27 total Rust unit tests**, no failures. Source hash verification, asset recreation, tracked locked Cargo, historical **15/15 logical 1× identical pixel hashes**, **20/20 relocated HQ 3× EXE screenshot hashes**, Win32 title/HUD F8/fullscreen, icon as real PE resource and Windows Shell icon, and native installed-EXE integration all passed. p95 rendering **17.3369 ms**, p95 scenery scrolling **21.2708 ms** (existing performance gate is <40 ms). [Actual Windows EXE build artifact #13841717700](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42990726444/artifacts/13841717700); independently downloaded [installed-EXE report artifact #13842356242](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42990726444/artifacts/13842356242) reports real game binary **SHA256 2b9bcccadf8c836dff9826bb93d165b3899f2d971218d3d05f5580b06c6c807f**, child_exit=0 and result PASS_AUTOMATED_INTEGRATION_WITH_EXPLICIT_AUDIO_LIMIT.

**Physical sound caveat:** Hosted Windows still has no actual output audio endpoint; source WinMM MCI failed error277. Visual focus freeze/resume passed but original sound timing/loopback is **not** a physical test pass. Manual speaker, Explorer taskbar pin, original-Flash gameplay fidelity and mixed-display hardware checks remain separate.

**Visual QA performed:** Retrieved actual current-run image ZIPs ([1× 20 images artifact #13840764839](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42990726444/artifacts/13840764839), [HQ 20 images #13842410411](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42990726444/artifacts/13842410411)), inspected contact sheet of intro start/fade, landed with HUD, Level1A exit black and world-objects, side by side at normalized scale. No apparent B02-specific visual regression in these images; golden hashes supply stronger native-to-native identity evidence. The source-defined black intro/exit samples are not automatically defects. Visual inspection is not a live original Flash-versus-Rust comparison and doesn't certify hidden frames.

**Final engineering verdict:** B02 fully verified on real Windows **at the code commit**, no gameplay/rendering changes, pure Rust metadata with no new dependency/unsafe/FFI. The final six-record documentation commit is a later docs-only checkpoint; its own CI, if triggered, is separate and must never be represented as the code-run already completed. B03 remains NOT STARTED.


Date: 2026-11-11. Batch scope: B02 ONLY. B01 original source gate: https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42981445349 (SUCCESS). New B02 implementation SHA: 18920bc12b8f27d835d8a574f0b47548c9a3b59b.

B02 **genuine Windows full regression run:** https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42990726444. Read the final run result before marking B02 source/playback regression successful. Draft PR #22: https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/22; base draft PR #21, while prior PR #18, main, historical Level1B branches and frozen level1a-v0.2.0 tag/source (49e8fd3) remain untouched.

## Exact code change / frozen boundary

- Added src/stage.rs, a pure Rust std-only module defining immutable StageId for Level1A, Level1B, Penguin Level1C, Labs Level2A, and Labs Level2B, with a static five-stage descriptor table.
- The original Flash **zero-based parser root frames 19/20/21/23/25** map to five original source game/level/ground symbols, player spawn game-local positions, level object game-local origins, original player facing, source camera height/jump/min/max and verified original root StartSound IDs. The B01 original source verified four target-stage essential placements/IDs/matrices; Level1A values come from the existing native implementation/source manifests.
- Added source-preserving stage label-to-enum mapping. Original AVM1 names use "level1c" while the extracted JSON key is "level1c_penguin" (distinct). The StageId route table contains Level1A→Level1B→Level1C→escape story card→Labs2A→TV story card→Labs2B→Kabuki story card. The three story cards are source symbols 1267, 1479, 1569. No Level3A gameplay or card execution is added.
- Added only an inaccessible-from-input, private **Game.stage** field initialized to Level1A and read-only Game::active_stage getter. No route is called by Game::tick. The existing Level1A renderer, collision, audio, gameplay control flow, progress, death and final-black-frame exit remain unchanged. State lifetime distinction documented: score/lives/Batarang progress root-scoped; camera/health/enemy wave/checkpoint/timer are stage-specific and are NOT yet copied or reset generically by B02, pending actual source transition proof in B04.
- Added six unit tests inside stage.rs: unique original root indexes/IDs, original label serialization, route and card order, camera/SoundStart data, coordinate origin conversion, and default native Level1A Game initialization. No serde crate or runtime dependency introduced.
- Changed only the existing Windows build workflow branch filter to enable FULL unchanged Level1A tests for this feature branch; no test gate, expected image hash, benchmark or original-source integrity check relaxed.

## Source-backed stage descriptors

| Stage | Root index (0 based) | game / level / ground | source level origin (game local) | player spawn (game local) | face | camera height / jump / X min..max | root sound |
|---|---|---|---|---|---|---|---|
| 1A | 19 | 696/152/151 | 2467.65,239.25 | 284.95,-661.50 | +1 | 150 / 1.25 / -1350..7650 | 751 |
| 1B | 20 | 1075/783/782 | 1034.65,239.25 | 528.15,-18.75 | -1 | 125 / 3 / -6220..600 | 1076 |
| 1C Penguin | 21 | 1251/1095/1094 | 1485.65,239.25 | 122.05,281.25 | +1 | 270 / 4 / 250..2150 | 1252 |
| 2A Labs | 23 | 1402/1317/1316 | 1210.10,238.25 | 227.20,115.40 | -1 | 275 / 3 / 200..55252 | 1409 |
| 2B Labs | 25 | 1496/1485/1484 | 1210.10,238.25 | 4413.70,6854.30 | +1 | 275 / 3 / 250..55252 | 1504 |

The level-local player position is source game-local player placement MINUS the source game-local placement of the level. This matches preexisting 1A coordinate arithmetic but does not use screen-space camera or ground-shape local offsets. Root StartSound refers to a native Flash tag, not proven physical audio playback or loop count.

## Verification scope

Real game regressions must be checked on actual GitHub-hosted Windows MSVC (not the earlier B01-only source job). Historical Level1A green at SHA 49e8fd3 proves the frozen release, not B02. Expected B02 code tests: preexisting 21 Rust tests plus six new stage metadata tests. The unmodified workflow has source hash, asset generation, 15/15 legacy 1x hash, 20/20 relocated HQ comparisons, actual game process/GUI/icon/F8/fullscreen, original intro/HUD/focus and renderer p95 limits <40ms. Hardware speaker audibility on hosted runners still remains unverified and should not be called passed. Any real failed CI is a blocker, not a reason to rewrite golden hashes.

B02 does not use a Flash player at runtime and cannot yet establish source-perfect Level1B/Labs gameplay. No actual Level1B scenes are reachable; that is by design. No native unsafe/FFI change; original Win32/WinMM code untouched. Issue #19 retains older code-hardening and supply-chain tasks.

**Next batch B03 (not started or authorized here):** source-faithful stage-aware asset pipeline and visual proof generation, not B04 active runtime route and not Level1B gameplay.
