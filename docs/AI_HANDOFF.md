# AI handoff — current recovery checkpoint

**READ THIS FIRST.** This is a provider-neutral, evidence-based snapshot for an agent that has never seen the original chat. Follow `AGENTS.md`; consult `docs/AI_TASKS.md` and `docs/AI_WORKLOG.md`. **Always verify current remote GitHub state before trusting this snapshot.**

## Repository / actual recovered status

- Repo: **https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust** (public).
- Active work branch: **`revive/level1a-intro-hud-fidelity-2026-11-09`**; source branch `revive/round11-ci-baseline-2026-11-09`; `main` is **not** the new intro/HUD build.
- PR: **[#18](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/18)**, **OPEN / DRAFT / UNMERGED**, base `revive/round11-ci-baseline-2026-11-09`. Earlier Round11 CI repair: [PR #16](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/16), also draft/unmerged.
- **Last VERIFIED code+asset-test commit:** `16b341a2287a17f5a0e147efae0b2b304477caf3` (intro, clickable SKIP, original sound and HQ HUD candidate).
- **Latest documentation checkpoint:** this four-file workflow bootstrap is committed *after* the preceding game-code SHA; determine its exact SHA from the **live branch head** (`git rev-parse HEAD` or GitHub PR head). Do not mistake a docs-only green CI for human gameplay approval.
- Last verified **full GitHub-hosted Windows x64 CI:** [Run #42575731190](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42575731190) — **COMPLETED SUCCESS; 56 successful steps, zero failed**, head `16b341a...`. Rust compile/tests, source asset baking, legacy screenshot hash checks, HQ smoke/benchmark/visual comparisons, and artifact upload passed.
- Playable artifact: `Batman-CobbleBot-Level1A-Fidelity-Windows-x64` (Run #42575731190, artifact ID **13655706241**, ~29.3 MB compressed, expiry as reported by GitHub **2027-02-07**). **This is a CI-built candidate, not an approved release.**
- Other relevant artifacts on that run: `Batman-CobbleBot-Level1A-Intro-Contact-Sheet`, `Batman-CobbleBot-Level1A-Intro-Behavior-Report`, `Batman-CobbleBot-Level1A-HQ3x-Visual-Smoke`, `Batman-CobbleBot-Level1A-Round6-HUD-Comparisons`.
- Manual acceptance issue: **[#17](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/issues/17)**, OPEN. No complete Windows playtest or original-vs-native fidelity approval is recorded for the latest commit.

## Goal, recent changes, and what is proven

**Overall objective:** source-faithful native Windows Rust port of original Flash game with 600×400 logical stage at 25 Hz, true-3× HQ art where source-grounded, and no Flash runtime. **Current priority is finishing Level 1A presentation fidelity before Level 1B, Penguin, renderer rewrites, or campaign expansion.**

- `src/game.rs`: `AppScreen::Intro` inserted between Title/Instructions and Level1A, 180 source timeline samples, keyboard skip, one music transition, `IntroStart`/ `IntroStop` audio event routing. **Compiled + headless passed**; native input feel and original timing **not manually validated**.
- `src/win32.rs`: original SKIP button hitbox from generated SWF metadata; clickable via DPI-aware letterboxed mouse coordinate conversion. **Compiled/unit checks passed**, but actual physical Windows click **not yet observed**.
- `src/render.rs`, `src/assets.rs`, `reverse-engineering/tools/bake_root_intro.py`: original root-frame-18 intro composites baked into 180-frame native pack, button #129 UP state recovered, scripted fade clip #34 excluded until true timing is established; contact sheet and screenshots exist. **CI+artifact visual review** passed, not frame-perfect source approval.
- `src/audio.rs`: original SWF DefineSound **#130** MP3 embedded via CI; starts at intro, stops before level1a music. Source audit finds root-frame-18 `StartSound` #130 and sprite123 frame179 `StopSounds`. **CI + event-order checks passed; physical speaker playback UNVERIFIED**.
- `reverse-engineering/tools/inspect_intro_hud.py`, `inspect_intro_behavior.py`: source evidence reports. `reverse-engineering/tools/bake_hud_assets.py`: legacy white/source-style HUD oracle and proposed **gold/Arial Bold HQ** typography. Original Flash HUD text is **white**. HQ styling/spacing **NOT YET APPROVED**.
- `.github/workflows/build-level1a-fidelity.yml`: reconstructs copyright-protected game assets only at build time, validates all rendering contracts, creates 19 smoke captures (17 prior + 2 intro), Windows executable, source reports, comparisons and benchmarks.
- Existing fuller engineering diary: `docs/revival/LEVEL1A_PRESENTATION_FIDELITY_LOG_2026-11-09.md`. **Caution:** its older introductory/unchecked status lines may be stale; run #42575731190 is authoritative for recent CI status. Preserve history, update stale assertions when working on that file.

## Outstanding/unknown (do not invent successes)

1. **HUD visual mismatch:** user's screenshot of the old build showed awkward BATARANG/HEALTH/SCORE labels, score digits, icon alignment and spacing. The newer themed HUD may improve aesthetics but departs from the original white Flash design. Needs A/B review and possibly *one* bounded source-driven correction.
2. **Original intro routing:** root frame18 stops; clips 122,123/126 have 165/180-frame sequences; SWF sprite123 ends with StopSounds. The Rust player currently **auto-advances at 180 samples** and allows skip; original game may wait for specific ActionScript/button transitions. Not verified.
3. **Cutscene effects:** Source sound #130 is present, but intro speaker playback, synchronization, any nontrivial stops/holds and true Flash click behavior remain untested on the user's PC.
4. **Full Level 1A playthrough:** physical focus, animation feel, camera, collision, score, batarang, pickup, jump/item sounds, fade/exit all await user's manual acceptance against original Flash. There is no verified Level 1B in this branch.
5. Multiple earlier HQ PRs remain a chain of experimentation; do not merge them automatically. No permission was given to change `main` or deploy releases.

## Current HUD-01 batch / user screenshot and CI checkpoint

- **New player evidence (2026-11-09):** The Windows game launches and displays gameplay, but the intro **rapidly blinks** and seems **cut off / shorter than the original Flash game**. This is a user-observed issue, not an independently diagnosed root cause; original ActionScript stops/timing are the next task. The user also provided a screenshot where the tutorial prompt is heavy condensed gold but BATARANG/HEALTH/SCORE and ammo/lives/score digits still use generic outlined Arial.
- **This bounded HUD-01 change:** The generated HQ-only HUD font switches from Windows Arial Bold to Windows **Impact**, a *close visual match* to the game's tutorial display style (not a proven exact SWF embedded font). Gold uppercase labels, warm high-contrast numeric glyphs, and original SWF text-field size constraints are applied in \`reverse-engineering/tools/bake_hud_assets.py\`. The Windows build command and Round6 contract explicitly require \`impact.ttf\`. Original 1× Arial _sans render and all Rust gameplay/rendering logic remain unchanged.
- **Verification state at commit:** source-level checks passed before push; **the new Windows CI run and A/B screenshot review are pending**. Previous proven green gameplay commit: \`16b341a2287a17f5a0e147efae0b2b304477caf3\`, Run #42575731190. Check the **live branch head** and subsequent workflow run for actual result before declaring success.
- **If the runner lacks Impact or visual size tests fail:** fix only this HUD-01 failure; preserve the prior green artifact/commit and record the error. Do not disable regression gates.

## November 9 continuation — HUD verified; INTRO-01 source diagnostics pending CI

- **HUD:** [Windows Run #42580731847](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42580731847) passed at `3138d02a4c788cb2e5efe0101e354cdd50cba76c`. Downloaded and reviewed the new HQ shell and digit A/B artifacts: the font is visibly condensed. Physical gameplay/HUD appearance is not user-approved.
- **Intro evidence:** Original root frame 18 contains `Stop`; sprite122 frame164 contains `Stop`; sprite123 frame179 contains `Stop` + `StopSounds`. Native `Game::tick` currently calls `skip_intro()` automatically at `intro_tick>=179`. That is a structural timing/flow mismatch, but additional Flash clip/button events may still control the original transition.
- **Flicker investigation:** 10-panel contact sheet includes dark transition panels, but is insufficient to diagnose rapid alternating flashes. The baker now reports grayscale mean, near-black fraction, and changed-pixel fraction for every one of 180 native baked frames. Original ActionScript audit now includes tag and action byte offsets. New workflow artifact: `Batman-CobbleBot-Level1A-Intro-Frame-Diagnostics`.
- **No native gameplay timing or renderer changes in INTRO-01.** Fresh diagnostic CI must pass and report reviewed before fixing one event/timing/render-frame discrepancy.
- **Exact next task:** confirm this commit's Windows Actions outcome, download `Intro-Frame-Diagnostics` and `Intro-Behavior-Report`; locate anomalous one-frame brightness reversals and source action offsets, then implement and test **INTRO-02**. Keep PR #18 draft, `main` unchanged.

## November 9 — INTRO-02 source-stop correction (CI pending)

- The preceding diagnostic [Windows Run #42582404902](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42582404902) **passed 57 steps** at `47d8db0d5c5d6224e5fe4b0240e0f44082e95248`. Full 180-frame metrics show near-black panels at frame multiples of 15 (0.6 seconds at 25FPS). Dark cycles are real in the current baked output; not all are necessarily source mistakes.
- Source actions: root frame18 Stop at FWS byte 282288; sprite122 frame164 Stop at nested offset47244; sprite123 frame179 Stop/StopSounds at offsets673/674; fade34 frame0 Stop at offset16.
- **Selected verified bug:** source sprite122 has 165 frames, stops at index164, but generic rasterization uses frame modulo and restarts that parent at tick165. In the intro baker only, source Stop is checked and that parent visual list is held through tick179; all other sprites and game logic remain unchanged. CI asserts the manifest.
- **Diagnostic follow-up:** root sprite35 is a 15-frame clip; new `inspect_intro_behavior.py` also includes sprite35 for exact source actions. The 15-frame black pattern makes it a suspect, not an established fault. No unsupported scene/fade removal.
- **Next batch:** inspect this new commit's Actions outcome and sprite35 Stop actions. If source supports it, correct its looping fade in another isolated batch. User still must approve timing and on-screen appearance. Preserve unmerged PR #18 and untouched `main`.

## November 9 — INTRO-03: source-stopped fade clip #35 (CI pending)

- **Recovery verified:** previous [Run #42583921359](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42583921359) completed SUCCESS at commit `5215b2239b53ce7e75606674637f31fbfffebffc`; it confirmed sprite122 stop handling, 180-frame diagnostic report, and base gameplay regressions. [PR #18](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/18) still open/draft, `main` untouched.
- **Original source evidence:** downloaded `Batman-CobbleBot-Level1A-Intro-Behavior-Report` (artifact **13658473742**) shows `sprite:35`, 0-based frame **14**, `Stop` at nested DefineSprite payload byte **207** (DoAction tag offset **201**). Native baker previously wraps its 15 frames via generic `frame % frames`, replaying the opening black transition every 15 ticks. Earlier diagnostics (artifact **13658682539**) show near-black fractions of 1.0 at ticks **15, 30, 45, 60** and later multiples, with abrupt frame deltas ~0.39.
- **Targeted implementation:** `bake_root_intro.py` validates sprite35's frame count and terminal Stop against verified SWF; holds its final source display list through the 180-sample intro pack (the initial source fade remains intact). It extends only this intro bake's virtual sprite duration. Generic renderer, audio, Rust gameplay, camera, controls and intro cutoff remain unchanged. Source sprite122's previous stop still holds.
- **Focused CI checks:** build manifest includes held sprite35, behavior audit asserts its final Stop, and per-frame luminance/change checks require no near-total-black pulse at frames **15,30,45,60**. These checks and preserved historical hashes must pass before marking this batch CI verified.
- **Open issues:** actual Windows playback and original Flash comparison have not yet confirmed the flicker or story timing; the native 180-tick automatic transition may still end the story prematurely. **Next batch:** inspect new CI artifact and compare actual intro, then audit original frame179 transition/skip event handlers before changing timing. Do not merge PRs or begin Level1B.

## 2026-11-09 — INTRO-04: source button transition audit (fresh CI pending)

- **Preserve prior evidence:** sprite35 hold is committed at \`0787e55e1180a822b01093a409923b85e1856d7b\`; [Windows Run #42586294097](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42586294097) was still rebuilding assets when this batch began. A new push may supersede/cancel the run due to GitHub's \`cancel-in-progress\` concurrency. Do **not** mark sprite35's prior CI green unless the run is actually completed successfully.
- **Intro ending issue:** Rust \`src/game.rs\` automatically calls \`skip_intro()\` on tick179. Original SWF root frame18 executes \`Stop\`; sprite123 frame179 executes \`Stop\` and \`StopSounds\`; neither alone establishes that a 180-tick automatic root transition occurs. Source dispatcher supports intro and level1a routes, but the actual trigger is still unresolved.
- **This bounded batch:** source evidence script \`inspect_intro_behavior.py\` now decodes original DefineButton2 #129 event conditions and AVM1 action records at exact source byte offsets, including push strings/function calls. It uses the documented SWF ActionOffset/CondActionSize coordinate system and writes the data into the existing \`Intro-Behavior-Report\` artifact. New CI checks that the button was found and its handler schema is valid. It does **not** invent a timed pause or change any Rust gameplay, audio, or scene transition behavior.
- **Next batch:** verify the full Windows run, inspect recovered button action handlers and source-triggered \`gotoNext\`/fade routes. Only then decide whether to remove auto-transition at tick179 and replace it with explicit source-backed interaction/transition behavior. Manual Windows user acceptance remains separate. Keep PR #18 draft and \`main\` unchanged.

## 2026-11-09 — INTRO-05, original button release and held intro (CI pending)

- **Last Windows-green source:** [Run #42587338561](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42587338561) passed for `d8a1e004e4c9772cb0ca4a20fb6e3e8d94f6df1e`. Its frame diagnostics artifact **13660266670** verifies no periodic black resets at 15,30,45,60 (near-black around 0.609, down from 1.0); this is NOT human playback confirmation.
- **Original source action, recovered:** behavior artifact **13660301491** contains DefineButton2 #129 at FWS byte 376676, conditional record at 376745. UB[16] flag is **0x0800, OverDownToOverUp (release inside)**. Previous script read the flags as little-endian 0x0008 and incorrectly reported End-key 4. Handler sets `_root.gotoNext="level1a"`, calls `_root.fadeout.play()`. Fade clip #34 later calls gotoNext on its final frame. Root frame18 and sprite123 final frame179 stop, so a timer-only jump at 180 samples was unsupported.
- **This bounded implementation:** `src/game.rs` no longer automatically transitions at tick179; hold source last sample until explicit SKIP. Stop original intro sound precisely once upon frame179 or on an earlier skip. Avoid repeating IntroStop when a later skip begins music. Headless smoke tests verify 100+ extra idle ticks, one-shot sound events, explicit transition, and earlier skip.
- **Original fade STILL NOT implemented:** clicking the source SKIP button should trigger clip #34's ~41-frame transition, but native SKIP currently jumps immediately. This is the exact next bounded feature after CI; do not misrepresent it as restored.
- **Verification:** `inspect_intro_behavior.py` now interprets the button's condition bits in MSB order; Windows workflow checks actual release-inside and AVM1 fadeout route. New full Windows CI pending on this commit. Preserve draft [PR #18](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/18), main untouched, no Level1B.

## Exact next ONE bounded batch

**Task ID: Verify INTRO-05 Windows CI then restore source fadeout clip #34.** After green build inspect held story and user Windows video; implement source button's actual fadeout animation and route to Level1A only when the fade finishes. Preserve original sound order and keep Level1B blocked.


## Reproduction / first commands and links

```powershell
$repo = "sinethnimsara26/Batman-CobbleBot-Native-Rust"
$branch = "revive/level1a-intro-hud-fidelity-2026-11-09"
gh pr view 18 -R $repo
gh run list -R $repo --branch $branch --limit 5
gh run view 42575731190 -R $repo
gh run download 42575731190 -R $repo --name "Batman-CobbleBot-Level1A-Fidelity-Windows-x64" --dir "./batman-test"
```

The full workflow is `.github/workflows/build-level1a-fidelity.yml` using `windows-latest`; code runs with generated `assets/` created from the SHA-256-verified original SWF. Local `cargo test` without those assets is not a complete reproduction. The user's local game launcher may use `%LOCALAPPDATA%\BatmanCobbleBot`; verify what is actually installed instead of assuming it was updated.

**External/uncommitted uncertainty:** As of this checkpoint, all project changes discussed here are persisted in GitHub; no local Windows uncommitted files, actual user runtime results on the latest version, or external branch changes have been verified. GitHub is the shared source of truth.

At **every batch** update this file with actual last verified SHA, links, the next exact step, and WIP status; commit the changes with the worklog and task status and STOP.
