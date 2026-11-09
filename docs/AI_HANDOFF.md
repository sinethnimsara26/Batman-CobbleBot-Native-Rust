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

## Exact next ONE bounded batch

**Task ID: HUD-01 — Audit one HUD typography/score-layout discrepancy.** Without gameplay changes: inspect latest Round6 HUD A/B artifact and HQ smoke, compare against legacy/source HUD and credible original-game screenshot. Identify one measurable mismatch (e.g., label colour, alignment or digit location), record screenshot/coordinates and an evidence-backed keep/revert/fix choice. If feasible, make **one narrowly scoped change**, rebuild/check visual regressions, and checkpoint; otherwise label the fix TODO with reproducible evidence. Do not start intro timing, campaign, or other HUD items in the same turn.

**FIRST ACTION for the next AI:** Fetch `AGENTS.md`, this handoff, and `docs/AI_TASKS.md` from the **live PR #18 head**, then independently check PR #18, latest branch HEAD and its most recent CI run to rule out intervening commits. Only afterward retrieve the latest `Batman-CobbleBot-Level1A-Round6-HUD-Comparisons` and `HQ3x-Visual-Smoke` artifacts and perform HUD-01.

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
