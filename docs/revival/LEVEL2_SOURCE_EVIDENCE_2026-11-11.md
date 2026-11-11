# Original Flash source evidence: the route from Level 1A to Gotham Labs

## POST-PLAN VERIFICATION 2026-11-11 — B01 ORIGINAL SOURCE ACTUALLY RECHECKED

The planning inventory below was written before actual B01 source access. It is superseded for newly verified original bytecode and audio by [the complete B01 source report](LEVEL2_B01_ORIGINAL_SOURCE_GATE_2026-11-11.md), [successful Windows original FWS CI #42981445349](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42981445349), and [artifact #13837522331](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42981445349/artifacts/13837522331). Confirmed canonical FWS hash, original root20/21/23/25 AVM1, stage semantic placements and source library images; older whole level-instances JSON has metadata drift but four stage-critical symbol/transform projections match. Specifically, Level1B has an eight-goon second wave and eKills-conditional Penguin progression, and Labs2B has a root-level 80-second clockTimer which the original game-instance manifest did not list by name. These two former assumptions are corrected in the B01 report. Source clips and sound IDs verified; **original live Flash interactive playback and exact gameplay screenshots are still not claimed**.


**Date:** 2026-11-11  
**Source inspection:** existing source-extracted, verified SWF manifests and Rust code on `ba53299c0cf618b20bc418d89973009138b1c7e0`; **not** a claim that the original SWF was freshly downloaded, executed, or disassembled in this session.  
**Frozen game release:** `level1a-v0.2.0`, SHA `49e8fd343e4f37f045ba71f24fff0999211f0f11`, UNTOUCHED.  
**Planning branch:** `plan/level2-source-roadmap-2026-11-11`.  
**Certainty convention:** SOURCE-MANIFEST = values extracted by earlier verified SWF parser; CURRENT-RUST = code inspected now; INFERENCE = consequence inferred but not yet demonstrated in live Flash; PENDING = requires raw AVM1/visual/audio proof.

## 1. Critical numbering distinction

The player accepted **Level 1A**, not all of the original **Gotham Bank** chapter. The parser starts its frame counter at zero before the first ShowFrame; all root-frame values in these manifests are parser indices, **not** human 1-based frame counts. `main_route` in [timeline-flow.json](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/timeline-flow.json) and per-root placements in [root-placements.json](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/root-placements.json) encode:

| SWF root frame (0-based parser frame index; the 20th displayed frame corresponds to index 19) | Route/state | Source game sprite | Level sprite | Verified next step |
|---|---|---:|---:|---|
| 19 | `level1a`: Gotham Bank rooftops | 696 | 152 | `gotoNext=level1b` |
| 20 | `level1b`: Gotham Bank streets | 1075 | 783 | Stage route continues to `level1c`; **exact completion predicate requires raw AVM1 proof** |
| 21 | `level1c`: Penguin encounter | 1251 | 1095 | `card:escape` |
| 22 | `card:escape` | card 1267 | — | sprite 1267 frame 224 (zero-based) calls `gotoNext=level2a` |
| 23 | `level2a`: Gotham Labs first section | 1402 | 1317 | `card:tv` |
| 24 | `card:tv` | card 1479 | — | sprite 1479 frame 151 (zero-based) calls `gotoNext=level2b` |
| 25 | `level2b`: Gotham Labs next section | 1496 | 1485 | `card:kabuki` |
| 26 | `card:kabuki` | card 1569 | — | sprite 1569 frame 159 (zero-based) calls `gotoNext=level3a` |

The old [game spec](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/GAME_SPEC.md) and [routing evidence](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/routing-evidence.json) corroborate these labels. Therefore **the immediate implementation target is Level 1B**, while the requested complete named Level 2 requires first finishing Level 1B, Penguin 1C and escape card, then both Labs sections 2A and 2B. Do NOT skip those original story beats or call 1B "Gotham Labs."

## 2. Source-extracted level metadata (not guesses)

| Stage | player placed at root game-local coordinates | original stage facing | camera min / max | camera height / jump | original ground ID and generated mask |
|---|---|---|---|---|---|
| 1B | (528.15, -18.75) | -1 (left) | -6220 / 600 | 125 / 3 | #782; origin (-11196,-886), 12102 × 1413 pixels |
| 1C | (122.05, 281.25) | +1 | 250 / 2150 | 270 / 4 | #1094; origin (-1731,-549), 2813 × 926 pixels |
| 2A | (227.2, 115.4) | -1 (left) | 200 / 55252 | 275 / 3 | #1316; origin (-1542,-2307), 8781 × 10060 pixels |
| 2B | (4413.7, 6854.3) | +1 | 250 / 55252 | 275 / 3 | #1484; origin (-1542,-2307), 8781 × 10060 pixels |

**Provenance:** [level-instances.json](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/level-instances.json), [level-constants-raw.json](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/level-constants-raw.json) and [pipeline verification](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/PIPELINE_VERIFICATION.md). Camera properties are initialization push-pairs, **not automatically verified frame-by-frame camera trajectory**. Coordinates are Flash game-local/root placement values, **not** global screen pixels; use correct parent matrices and avoid copying directly to renderer coordinates. Original movie logical screen 600×400, 25 FPS (40-ms fixed step), SWF7, verified uncompressed FWS SHA256 `3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed`.

## 3. Level 1B object census — exact named source placements

Root frame20: game sprite #1075, level #783, ground #782; named scene object count **33** (includes level/player/HUD helper entries). [Source](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/level-instances.json).

- **Ten standard goons** (six symbol #964, four symbol #880; named goon1..goon10), mostly at negative local X: -746.15, -1086.15, -1734.85, -2074.85, -3126.85, -3466.85, -4976.85, -5364.85, -5970.75 and -6110.45, each Y=398.1. These are *original placements*; actual activation/movement requires `goonAI` trace.
- **One biker** named `biker1`, symbol #1005, x=-570.25, y=375.85; use `bikerAI`, not a standard goon reskin.
- **Four cans** symbol #785; **four trash items** symbol #790; **one mailbox** symbol #793. Treat visible static/animated prop depth and collisions independently.
- **Three `checkpoint_1b_*`** named triggers from symbol #153; one lies at x=-8214.9, beyond ordinary camera min: inspect original trigger bounds/branch logic rather than treating every named checkpoint as a simple sequential platform.
- **Four ammo pickups** `item_br01..04`, source character #695. One `respawn` symbol #966, one projectile/Batarang source #1006, `bam` #1009, `combo` #1074.
- Player source symbol #691; unlike level1a, source init has `playerDir=-1`, so entering 1B should not default to moving right.
- [combat-ai.json](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/combat-ai.json) gives goons **32 HP, strength 5, speed 9**; biker **strength 10**, other biker dynamic details PENDING. Enemy source animation symbols #880/#964 each have 89 outer frames with stand/walk/retreat/attack/hurt/fall/die/idle/intro labels, but actual nested clip durations require renderer/source baking.

## 4. Level 1C — Penguin encounter and escape transition

Root frame21: game #1251, level #1095, ground #1094; one `boss` at (1035.65,316.2) with symbol **#1250**, Batman #691 at (122.05,281.25), four source ammo pickups #695 and existing combo/BAM effects. Penguin **200 HP, speed 6** and a `stength` property (misspelled in source) paired with value 5: do NOT silently "correct" source semantics without confirming whether runtime uses the typo. [Combat manifest](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/combat-ai.json), [function index](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/function-index.json) show `penguinAI` references `dodge`, `block`, `attack`, `attack2`, `laser`, `electro` and completion `card:escape`. Outer boss #1250 has 99 frames. Exact attack delays/laser hit timing and death-frame dispatch remain PENDING raw AVM1 analysis.

**Nontrivial HUD detail:** source [root-placements.json](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/root-placements.json) places HUD sprite #716 at approx. scale **0.8636** on root frame21 rather than ~1 on frame20. This is a real source-specific layout difference; test it rather than carry Level1B HUD placement blindly.

Story card on root frame22 uses sprite **#1267**, with end-route source at **zero-based frame224** to `level2a`. Need bake/render its distinct timeline with correct audio/interaction, not skip directly to Gotham Labs.

## 5. Level 2 proper — Gotham Labs (2A and 2B)

Source-backed scenery theme and chapter name are consistent with independent historical game listings ([Toonami Wiki](https://toonami.fandom.com/wiki/The_Batman%3A_The_CobbleBot_Caper)) and screenshots. These pages are external **context**, not more authoritative than the binary manifests for placements, timing or code.

**Level 2A:** root23/game #1402/level #1317/ground #1316. [Named placements](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/level-instances.json): **18 door clip instances** symbol #1325 (often mirrored pairs); **3 elevators** symbols #1328/#1330; **10 camera triggers** symbol #153; **4 checkpoints** symbol #153; **9 goons** symbol #1365, **1 eagle** symbol #1393; **9 ammo pickups** #695 and **1 health pickup** #1401. The 2A goons initialize at **70 HP, strength 10, speed 6**; eagle at **30 HP, speed 5**. `elevatorLogic`, `doorOpen`, `cameraLogic2` and `birdAI` must be traced from actual AVM1; two-dimensional/highly vertical world must not be reduced to a simple horizontal run. Camera bounds 200..55252 are source constants, not a claim of horizontal world width. TV card #1479 root24 end (zero-based frame151) advances to 2B.

**Level 2B:** root25/game #1496/level #1485/ground #1484. **16 door instances, 3 elevators, 10 camera triggers, 4 checkpoints, 1 named `lava1` (#1486), 10 Batarang pickups (#695)**. Player starts near (4413.7,6854.3); distinct section and traversal route should be reconstructed rather than assuming 2A initial coordinates. The compact [level-spec.json](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/reverse-engineering/manifests/level-spec.json) describes `clock:1`, while the named-instance census contains **no directly named clock instance**. **OPEN DISCREPANCY:** timer may be script-created, nested or otherwise omitted by named-placements-only extraction; verify `clockTimer` in raw bytecode before asserting a visible clock/hazard behavior. The combat manifest gives eagle group defaults for 2B but **no named eagle in 2B placements**; do not fabricate a spawned enemy without sprite/action evidence. Completion routes to `card:kabuki`, not directly to the next battle.

## 6. Existing native engine gap at the frozen release

[game.rs](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/src/game.rs) declares `AppScreen{Title,Instructions,Intro,Playing}`, has `begin_level1a`, 7 fixed Level1A pickups and a `level1a_complete` flag. **Actual current Level1A exit:** `update_tutorial_checkpoints` triggers a 41-frame fade on `CP_NEXT`, then `level1a_complete=true` and leaves native game in final black/frozen state **instead of loading Level1B** (explicit comment in lines around 788–797). [assets.rs](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/src/assets.rs) and [render.rs](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/src/render.rs) load/render **Level1A-specific tiles/scenery** and [collision.rs](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/src/collision.rs) loads only Level1A mask. [win32.rs](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/src/win32.rs) supports F5–F8 QA only for title/intro/Level1A, not arbitrary stages. [audio.rs](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/blob/ba53299c0cf618b20bc418d89973009138b1c7e0/src/audio.rs) owns Level1A music/intro; other level cue mapping not implemented.

Thus **the next task is not just baking new images**. The native scene/collision/music/state model needs minimal source-grounded generalization, with full Level1A backwards compatibility. Never retrofit approximated 1B/2A scene contents into `LEVEL_X`, `PICKUPS` or `level1a_tiles` constants.

## 7. Questions the static manifests CANNOT establish alone

1. Exact order and predicates in `respawn2`, `goonAI`, `bikerAI`, `penguinAI`, `cameraLogic2`, `elevatorLogic`, `doorOpen`, `clockTimer` and `birdAI`; especially root20 1B→1C condition.
2. Per-frame source animation action callbacks, hit-box geometry, enemy reaction delay/invincibility, original collision events, Batman HUD health/lives carry across all transitions and special music/FX start-stop. Numeric init values and referenced strings do NOT themselves prove the whole AVM1 control flow.
3. Layering, alpha transforms, visible frames and audio cues at stage entry, exit and story cards; exact static-vs-animated prop separation. Only a live original SWF player or a verified source renderer and raw action audit can certify these.
4. 2B named `clock` discrepancy and possible script-created enemies. Evidence must be verified before coding.
5. Full source/CI re-extraction from the **canonical SHA-verified FWS**, reading `core-functions.disasm.json.gz`, `level-init-actions.disasm.json.gz`, `symbols.json` and audio tags; these compressed per-action reports are **produced by existing pipeline but not tracked in this Git tree**. The present tool runtime could inspect committed extracted manifests but could not fetch/play the original SWF, so call these a **required first implementation-preparation gate**, not already passed in this specific session.

**Follow-up criteria:** First next bounded batch performs original-source action cross-check, captures machine-readable evidence and representative source frames, links the actual CI artifact and corrects any mismatch; no Level1B or Labs Rust behavior until the relevant source uncertainty is closed. No new proprietary SWF binary committed to public Git history.
