# B05 — original source Level1B Bank Street 1× static scenery proofs (VERIFIED source-art, not playable)

**Date:** 2026-11-11. **Scope:** exactly B05, source-authentic 600×400 scenery proof views from original sprite #783, no live Level1B gameplay. **Branch:** `revive/level2-b05-bank-street-source-1x-2026-11-11`. **Draft PR:** [#25](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/25), based on B04 draft #24; **OPEN/DRAFT/UNMERGED**.

**Exactly Windows-tested CODE commit:** [`16cbd7b8adde1b2667110850abc9b2fdcd7b4f62`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/16cbd7b8adde1b2667110850abc9b2fdcd7b4f62). The later six-record/documentation commit is a distinct SHA and requires separate CI inspection; do not relabel the code runs as docs-HEAD tests.

## Source and implementation

Original canonical **FWS** SHA256 `3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed`, length **2,819,100 bytes** (fresh downloaded CWS/FWS normalized and rechecked on Windows). Source parser root **20 (ZERO-BASED)**: game **#1075**, static level **#783**, invisible collision ground **#782**, level game-local origin **(1034.65, 239.25)**. Source proof is level scene frame **0 (ZERO-BASED)**, not a live Flash gameplay capture.

Original source display-list analysis: **70 direct children**, **67 visible geometry child bounds**, **23 geometry-derived candidate tile locations**, **18 distinct candidate x-columns**. The B05 baker deterministically selects nine separated 600×400 original-scale views, world origins:
`(-9000,-400)`, `(-7200,0)`, `(-5400,0)`, `(-4200,0)`, `(-2400,0)`, `(-600,0)`, `(0,0)`, `(600,0)`, `(1200,0)`.
All **9/9** have nonempty alpha, valid original 600×400 dimensions, file SHA256 and decoded RGBA SHA256 in the downloadable report. This is a **sampled static-art proof, not the complete world tile pack**.

The earlier B03 source rasterizer exposed original IDs **758, 771, 774** as unresolved. B05 independently traced their actual canonical SWF definition tags: **758 and 771 are DefineButton2, SWF tag 34** (scripted interactive buttons, excluded from static scenery); **774 is DefineText, SWF tag 11** (original font/glyphs recovered via development-time `load_text_defs`). The new `--include-source-static-text` option is explicitly opt-in; default Level1A rasterization remains untouched. Actual corrected B05 manifest: **no missing source IDs and no unsupported fills**. The two old B03 coordinates `(0,0)` and `(600,0)` were independently re-rendered under unchanged default and verified pixel-identical SHA256 (`896f7373e114393a233ecd279dd8b143955f920913c8dde26013ebee32e59a6c`, `2381074e5468f5c0e7fd8326898fb2d8bb907981467433936046e47649e8c8c6`), and the corrected B05 views also share those decoded pixel hashes.

**Changed paths (exact code diff from B04):**
1. `reverse-engineering/tools/b05_bake_bank_street_1x.py` — original-source root/placement/definition-type validation, scene-depth/ownership guard, deterministic bounded tile selection, source static-text proof and two prior-view hashes, nine tile PNG/hash/provenance JSON, contact sheet, and packer negative test.
2. `reverse-engineering/tools/bake_level_tiles.py` — opt-in original DefineText load, manifest declaration. Default output for existing Level1A and B03 remains unchanged; no new production asset.
3. `.github/workflows/b05-original-bank-street-source-1x.yml` — fresh canonical download + SHA, source extraction, Python Pillow dev-only rasterization, proof/negative-packer gate, artifact on free Windows runner.
4. `.github/workflows/build-level1a-fidelity.yml` — adds only B05 branch to existing unchanged full Windows QA allowlist.
No `src/`, `Cargo.toml`, `Cargo.lock`, `build.rs`, native game asset pack or golden image modifications.

## Genuine Windows evidence — tested-code SHA, not documentation HEAD

| Gate | Actual result |
| --- | --- |
| [B05 source-art Windows #43029939309](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43029939309) | **COMPLETED SUCCESS, 9/9 steps**, exactly `16cbd7b8adde1b2667110850abc9b2fdcd7b4f62` |
| [Full native Windows Level1A #43029939394](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43029939394) | **COMPLETED SUCCESS, 81/81 steps**, same code SHA |
| `cargo --locked` Rust | **29/29 tests**, locked dependency resolution unchanged |
| Original 1× native screenshot hashes | **15/15 identical** against historical baseline |
| HQ 3× installed-copy EXE screenshots | **20/20 identical** |
| Native Win32 integration | Real executable, Windows input/window/start/skip, F/Esc fullscreen, F8 HUD, OS icon, focus **PASS** in automated QA |
| Render / scrolling p95 | **15.923ms / 21.1566ms**, unchanged **<40ms** thresholds |
| Windows installed-copy result | `PASS_AUTOMATED_INTEGRATION_WITH_EXPLICIT_AUDIO_LIMIT` |
| Hosted real audio | **BLOCKED_HOSTED_RUNNER_NO_AUDIO_ENDPOINT**, WinMM MCI277, physical sound and real MCI position **UNVERIFIED** |

**Actual source proof ZIP:** [artifact #13858908870](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43029939309/artifacts/13858908870) (`B05-CobbleBot-Original-Bank-Street-Source-1x-Proofs`, 30-day retention). Independently downloaded; archive contains **15 files**: 9 original 1x corrected PNGs, two B03 baseline PNGs, two flagged partial manifests, JSON provenance report and captioned contact sheet. Verified **9/9 PNG SHA256**, **9/9 RGBA SHA256**, and contact-sheet SHA256 **`0dfde689500572ccc2b34bfe6b929511481887c4e8a91b1a483c6c29ff726272`** / dimensions **1836×1338**. Inspecting the actual sheet shows original-source street windows, road, trees, lamps and readable `GOTHAM BANK` lettering, not an executed original-game screenshot. Both manifests mark `partial_proof_only=true` and `never_embed_as_complete_level=true`; actual negative `pack_level_tiles.py` test rejects these tiles, and the ZIP contains **no SWF, EXE or native BCLVT pack**. No original full media archive committed.

**Failure history preserved:** Initial [source run #43028683730](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43028683730) FAILED because rasterizer did not load/handle #758/#771/#774. Next [#43029559752](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43029559752) FAILED on an overly broad hypothesis that all three were DefineText (canonical FWS showed only #774 is text); corrected by recognizing #758/#771 as DefineButton2 controls and excluding them. Both failed runs are genuinely FAILED, not accepted. Earlier full Windows runs #43028683240 and #43029559723 were **CANCELLED by superseding pushes**, not falsely reported green. Final source/native runs both SUCCESS at the same final code SHA.

## Post-change audit / limitations / stop

Source evidence: **RE-20261111-015**. Independent completed engineering audit: **AUD-20261111-015**. Rust game purity preserved: changes are development/CI-only Python/Pillow and an opt-in source-asset bake flag, with *no new shipped game runtime library*; pre-existing Win32/GDI/WinMM FFI qualifications remain. Source static art is proven against real FWS, but **original Flash live camera/gameplay, full actor/ground animation, physical audio and playable 1B are NOT tested or implemented**. Source sample is incomplete by design. Physical speaker/input/monitor feel remains a manual separate gate. Version-range Pillow installation on CI is a reproducibility/hardening concern, mitigated for evidence by SHA-checked source inputs and outputs; not production dependency.

**Protected refs:** published `level1a-v0.2.0` release tag and branch at `49e8fd343e4f37f045ba71f24fff0999211f0f11`, `main`, PRs #18/#21/#22/#23/#24 and all older draft/history untouched; PR25 stays DRAFT and UNMERGED.

**STOP after B05. Next, ONLY on separate user authorization:** B06 original Level1B **independent 3× high-quality static scenery/depth proof**, retaining exact B05 1× samples and native Level1A goldens. Do not activate Level1B or merge/release within B06.
