# AI engineering worklog — chronological, evidence-linked

This is the **short index of meaningful, verified progress and mistakes**. Detailed source analysis, source IDs, full decisions and original screenshots remain in [the original Level 1A presentation diary](revival/LEVEL1A_PRESENTATION_FIDELITY_LOG_2026-11-09.md). Current state and exact next action are in [AI_HANDOFF.md](AI_HANDOFF.md); prioritized task statuses in [AI_TASKS.md](AI_TASKS.md).

## 2026-09 — foundational recovery and fidelity strategy (repository history)

- Reverse-engineering phase recovered original 600×400/25FPS structure, stage routes/collision geometry, Flash AVM1 functions, frame assets, sounds, UI and original transforms. See `reverse-engineering/RE_COMPLETE.md`, manifests, tools and `README.md`. Treat recovery of **source data** as different from implementation of all stages.
- Early broad-game prototype was not accepted as faithful: visible collision/control geometry leaked into scenery, some attack-state transitions/camera/UI behaviors were approximated. Decision: freeze implementation to an **authentic Level 1A vertical slice** and retain a deterministic baseline for comparison.
- Rendering rounds reconstructed source-shaped 3× player/overlay/HUD/world layers and Windows DPI presentation through branches/PRs. Their outputs are staged; not automatically merged or approved.

## 2026-11-09 — Round 11 CI repair; verified build

- **Problem:** validator searched for deleted Rust source text `target.fill(0xff000000);` after renderer composition changed.
- **Fix:** changed only the CI source anchor/order check on `revive/round11-ci-baseline-2026-11-09`; no Rust gameplay changes. [PR #16](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/16) draft/unmerged.
- **Evidence:** [Run #42561171602](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42561171602) succeeded. User then ran a local Windows build and reported a **missing pre-level story/call** and an awkward HUD typography/score/icon layout. This older build's successful launch does not approve later versions.

## 2026-11-09 — source-derived intro & HUD candidate, iterative checks

- **Scope:** `revive/level1a-intro-hud-fidelity-2026-11-09`, [draft PR #18](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/18), under issue #17. Original intro art from SWF root frame18 is baked into a 180-frame pack; title/instructions enter `AppScreen::Intro`, then gameplay, with keyboard skip. HQ HUD proposes gold/Arial Bold text while retaining a white/source-style legacy oracle.
- **Failure 1:** sprite baker could not render Flash button #129 (`DefineButton2`). Temporary omission left a **blank SKIP outline**, spotted by screenshot review. Fix: parse the actual SWF UP-state button art; create ten-frame contact sheet. [Run #42571136795](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42571136795) passed and showed restored `SKIP` lettering.
- **Failure 2:** original automated smoke asserted exactly 17 screenshots; new intro added 2 => 19. Fix: accept exactly 19, preserve historical gameplay screenshot hashes. Earlier failing [Run #42568860291](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42568860291); [Run #42569523565](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42569523565) fully green.
- **Failure 3 (caught during implementation):** source-derived button rectangle was initially written before its output folder existed; creation order corrected before full CI. Win32 now hit-tests mouse clicks in the letterboxed/DPI-aware logical stage and routes only intro clicks to `skip_intro`. [Run #42572833961](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42572833961) passed **55 steps**. No *physical mouse* test yet.
- **Audio evidence:** audit `inspect_intro_behavior.py` verified SWF root frame18 sound #130 and sprite123 frame179 StopSounds. Sound #130 extraction/WinMM intro start/stop and headless audio ordering were committed. [Run #42574699640](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42574699640) proved the focused audit worked.
- **Latest game code:** commit `16b341a2287a17f5a0e147efae0b2b304477caf3` passed **all 56 Windows CI steps** in [Run #42575731190](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42575731190). Validated: Rust code/build, generated original assets, headless screenshot hashes, transitions, source metadata, smoke, HQ checks, uploaded EXE. **Not validated:** human Windows audio/video/interactions, Flash script pauses, original-accurate styled HUD, full Level1A playthrough.
- **Policy:** no Level1B, bosses, new renderer rewrite or merging draft PRs before manual Level1A fidelity acceptance.

## 2026-11-09 — permanent cross-AI workflow bootstrap (this batch)

- **Objective:** make the repository independently recoverable after ChatGPT timeouts, context loss, new chats, and different AI providers; **no gameplay feature changes**.
- **Recovery verified before write:** public repository; draft PR #18 head `16b341a...` based on draft PR #16's recovery branch; last full successful Run #42575731190 at that same game-code SHA; issue #17 open. The four required portable documentation files were absent on that branch. The existing `docs/revival/LEVEL1A_PRESENTATION_FIDELITY_LOG_2026-11-09.md` was kept intact.
- **New paths:** `AGENTS.md` (safe operating rules), `docs/AI_TASKS.md` (small prioritized actions), `docs/AI_HANDOFF.md` (verified checkpoint and next action), `docs/AI_WORKLOG.md` (this concise chronology).
- **Reproducibility:** documentation-only commit on the existing PR #18 development branch; confirm its actual GitHub commit SHA and these four paths exist. A triggered new GitHub workflow may still be running; **do not claim it green until completed**. Previous full green game commit remains `16b341a...`.
- **Next batch (ONLY when user says 'continue'):** `HUD-01`, evidence-backed A/B audit of one HQ HUD typography/score-layout mismatch. Keep the workflow changes separate from gameplay work.

## 2026-11-09 — HUD-01, font family and labels/digits (current batch)

- **Evidence:** user's 1920×1080 Windows screenshot shows the tutorial \`PRESS THE SPACE BAR TO JUMP\` as heavy/condensed, gold; top HUD labels in generic, thin outlined Arial; large SCORE number also resembles generic Arial. User explicitly requested the **same visual font family** for labels and the numbers above/below icons, with size hierarchy and better spacing.
- **Selection:** one isolated visual-baker change; no intro-timing work in this batch, even though the user also reported **rapid blinking and premature cut-off** (recorded as next bug).
- **Technical change:** the HQ HUD baker uses Windows \`impact.ttf\` instead of \`arialbd.ttf\` as a close *visual match* to the tutorial's condensed display lettering. The generator uppercases themed labels and fits each glyph run inside the **original SWF label field box**, avoiding overlap with bat/batarang/health artwork. Both numeric atlases use the same typeface, at deliberately different sizes, with warm cream/gold and dark outline. Original legacy \`arial.ttf\` HUD, text content metadata, original sprite/camera/UI positions and all Rust code are unchanged.
- **Verification:** the workflow now asserts the exact source/HQ font filenames, keeps all older legacy screenshot SHA-256 checks, and creates existing Round6 A/B comparisons. **This entry records implementation only; full Windows CI and manual on-screen approval are not yet claimed.**
- **Risk/limitation:** Impact is *not proved to be the actual embedded Flash tutorial font*. If the user wants pixel-perfect glyph identity, find and re-rasterize the original SWF font in a separate task. If Windows runner lacks Impact, the new build must fail explicitly; do not hide a fallback substitution.
- **Next distinct batch:** INTRO-01 to diagnose the flickering, short pre-game cutscene. Stop after the HUD checkpoint.

## 2026-11-09 — INTRO-01, original action offsets and every-frame flicker data

- Recovered current PR #18 head `3138d02a` and completed Windows Run #42580731847, confirming HUD condensed-font CI success. Reviewed original intro behavior report, 10-panel contact sheet, and new A/B HUD assets. Manual Windows approval still outstanding.
- Original root frame18 explicitly `Stop`s; sprite122 frame164 `Stop`s; sprite123 frame179 `Stop`s and `StopSounds`. Native `src/game.rs` transitions on tick179 without checking original Flash event logic. This is the first concrete discrepancy; final original transition rules remain unknown.
- Added source ActionScript tag/action byte offsets and 180-frame grayscale brightness/change data to Windows diagnostic artifacts. Purpose is to determine whether rapid blinking arises between the ten previously sampled contact-sheet frames. **No intro behavior patched yet**; new diagnostic CI pending.
- Next batch: inspect uploaded reports, identify the single source-backed cause, implement and test INTRO-02 without touching Level1B or main.
