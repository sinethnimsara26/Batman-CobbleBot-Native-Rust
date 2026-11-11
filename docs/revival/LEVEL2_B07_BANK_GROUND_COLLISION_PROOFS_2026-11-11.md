# B07 — original Bank Street Level1B ground #782, spawn and camera source-probe verification

**Date:** 2026-11-11. **Scope:** ONLY B07 source-derived collision mask, native Rust source point probes and source camera, dormant until full Level1B is ready. **Branch:** `revive/level2-b07-bank-ground-probes-2026-11-11`; [draft/unmerged PR27](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/27) based on B06 documentation HEAD `0dd00b32094f62b1bd0affb1fce4d2000c2d6dcc`. **Exact WINDOWS-tested CODE SHA:** `d3aa66e8d37648d018fcf446dfc73e4f0cb5761b`. Final documentation SHA is a later commit, recover from live HEAD; it is never assumed to have the same tested-code CI.

## Original source and precise scope

Canonical SHA-locked original uncompressed FWS SHA256 `3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed`, 2,819,100 bytes; `reverse-engineering/manifests/level-instances.json`, `level-spec.json` and `level-constants-raw.json`. Zero-based ROOT frame **20**, game sprite **#1075**, visible Bank Street level **#783** and invisible original ground **#782**. Source `level` placement in game is **(1034.65,239.25)**; original `ground` placement depth1075 with matrix sx=sy=10.348342895507812, tx=-4338.2,ty=523.5. Original player #691 game-local position **(528.15,-18.75)** is **(-506.5,-258)** relative to the street level placement; facing **-1** (left). Original camera min **-6220**, max **600**, height **125**, jump **3**, native horizon 25Hz. Source `playerHeight=75`, ceiling height50 in existing native code, jump velocity -42 and gravity5. These are source facts and code model, NOT yet a playable stage.

## Changed paths and verified behavior

1. `src/collision.rs`: added dormant `CollisionMask::level1b()` byte-backed from original generated source mask origin(-11196,-886) and two deterministic native tests. Original generated source mask **12102×1413** binary pixels with **9,117,944 solid pixels**; sample floor points checked on all six source street positions. Rust exercises OOB detection, 75px ground correction, wall no-leak midpoint probes and 50px ceiling / -42 jump / +5 gravity landing. Native tests are source-mask simulations, not mouse/player game footage.
2. `src/stage.rs`: separate pure `SourceCamera::horizontal_step` / `jumping_y` methods + test. Strict min/max, 10% integer-truncated horizontal camera easing toward left-facing x=475 (right-facing x=125), vertical jump target `(125-player_game_y)/3`; sample camera step x26.10→19.10 for original player x528.15.
3. New **offline only** `reverse-engineering/tools/b07_probe_bank_ground.py`: rejects wrong source ID, mask origin/dimensions, malformed 1-bit image, empty/full or inconsistent occupancy and missing original floor sample geometry; logs source checksum and six floor points in `source-ground-report.json` and labels PNG source only/nonpackable.
4. `.github/workflows/build-level1a-fidelity.yml`: B07 trigger; existing original SHA-checked FWS baker renders original 1A AND 1B ground masks, runs B07 source assertion, uploads named B07 dev-only proof, retains old strict Windows native tests, images and p95 gates.

No current `Game::tick` / `Game::new` / `src/render.rs` / audio changes. 1A remains solely active. No new crates, original full SWF, binary Level1B art pack, dynamic props/enemies, boss, story, Level1B audio, or source screenshot golden edits. Original Level1B source art proof from B05/B06 is partial/nonpackable; B07 does not activate it.

## Genuine Windows CI evidence (exact tested code)

| Evidence | Actual outcome |
| --- | --- |
| [Full native Windows run #43077601981](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43077601981) | **COMPLETED SUCCESS 82/82 Windows steps** at exact `d3aa66e8d37648d018fcf446dfc73e4f0cb5761b` |
| Original source #782 bake and B07 preflight | `B07 original source Bank Street #782 mask OK: 12102x1413, origin=(-11196,-886), filled=9117944 floor probes=6` |
| Native release MSVC `cargo test --release --locked` | **32/32 PASS**, including 3 new collision/camera tests |
| `cargo build --release --locked` / Cargo.lock | **PASS**, no new dependencies or lock changes |
| Historical native Level1A 1× screenshot SHA | **15/15 identical** |
| Relocated Windows EXE HQ smoke screenshot SHA | **20/20 identical** |
| Native Win32 GUI | Title/Enter/Skip/F7/F8 HUD/fullscreen F/Esc/focus/OS icon/integration **PASS** under existing automation |
| Real EXE performance | render p95 **17.4411ms**, scenery scroll p95 **21.6610ms**; unchanged **<40ms** gates |
| Hosted speaker / WinMM | **BLOCKED_HOSTED_RUNNER_NO_AUDIO_ENDPOINT**, WinMM MCI error277; not audio PASS |

[Actually generated artifact #13883750256](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43077601981/artifacts/13883750256) `B07-Original-Bank-Street-Ground-Probe`: uploader succeeded and exposes source-only mask PNG plus report; **independent archive-byte inspection not performed**. CI logs verify dimensions, occupancy and six samples. This artifact is an original SWF vector-derived collision-mask test, *not* live original Flash gameplay or a complete native playable Level1B.

## Independent engineering audit and limitations

- **RE-20261111-017** records exact original source metadata, shape rasterization, actual tests and uncertainty; **AUD-20261111-017** separately reviews actual Rust/Python/CI diff, dependency/FFI model, 17,100,126 decoded-mask byte footprint (only when constructed), and no new unsafe code or game/runtime Flash/Python. Pure Rust gameplay remains native with previously documented Windows platform FFI.
- **Not yet verified:** original Flash player's exact point-hit semantics at sampled/unsampled geometry edges (existing SWF quadratic sampling + pixel rasterizer is an approximation); dynamic player/camera visuals, full street traversal and player feel; any Level1B actor/AI/prop hitboxes; physical audio playback and original Flash live side-by-side frame fidelity. Source sample proof alone does not make Level1B playable.
- Published Level1A `level1a-v0.2.0` immutable SHA `49e8fd343e4f37f045ba71f24fff0999211f0f11`, main and historical PR18/21–26 untouched, PR27 DRAFT/UNMERGED. Six records synchronized; two mandatory protocols read unchanged.

**STOP after B07.** Next separately authorized B08 only: original 4× #785 cans, 4× #790 trash and 1× #793 mailbox placement/depth and collision ownership, without changing active stage or implementing enemies/AI/music.