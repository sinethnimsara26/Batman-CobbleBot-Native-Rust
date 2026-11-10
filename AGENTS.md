# AI agent operating guide — Batman: The CobbleBot Caper

## FS-QA-03 — native fullscreen desktop verification gate (2026-11-10)

Authorized batch 03/08 adds the CI-only Python probe at reverse-engineering/tools/windows_fullscreen_probe.py and a STRICT Windows step to .github/workflows/build-level1a-fidelity.yml. It launches the ordinary game EXE without --qa-navigation; SendInput F/Enter/Esc; asserts full monitor WS_POPUP geometry, saved RECT/style/WINDOWPLACEMENT, proper centered 3:2 viewport and real OS screenshot letterbox pixels, unchanged Intro after Esc past its 41-frame skip window, and maximized-window restoration. It always uploads separate JSON and eight desktop PNGs on success/failure; legacy Windows GUI pilot and headless/source gates remain intact. Code commit [61b960a](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/61b960a540063c624f8b470cde844715bdbf75f9). CI-only Python is not a shipped runtime dependency. Full multi-monitor/mixed-DPI/physical speaker tests, user 1920x1080 review and source Flash equivalence remain outside this test.

**Actual evidence:** [Windows CI #42838310455](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42838310455) completed SUCCESS at 4445e81eaa709a8e71216f2a2b2a3f5fd4c82f08: **65/65 steps**, native MSVC Rust **18/18**, historical legacy screenshot SHA **15/15**, old Win32 pilot **9 images/errors=[]** with all 3 real F8 HUD captures, title held-Enter probe PASS, and FS-QA-03 native fullscreen **8 real desktop images/errors=[]**, status PASS_NATIVE_FULLSCREEN_TITLE_INTRO_RESTORE_MAXIMIZED. Windows hosted desktop: ONE 1024x768 monitor; full window and monitor bounds [0,0,1024,768], centered 3:2 viewport [0,43,1024,682] with RGB [0,0,0] letterbox pixels. Normal pre/post Esc rect identical [94,108,1024,768]; maximized pre/post identical [-8,-8,1032,728] and showCmd=3. Intro remained visible more than 1.9 sec after fullscreen Escape; screenshots independently downloaded and inspected. Render p95 **20.8318ms**, scenery p95 **24.5818ms**, unchanged strict <40ms. [EXE #13775795700](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42838310455/artifacts/13775795700); [old GUI #13774992700](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42838310455/artifacts/13774992700); [fullscreen GUI/report #13775735822](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42838310455/artifacts/13775735822).

Do not erase two historical CI failures: documentation-only #42834460076 and initial FS probe #42836412666 failed the OLD strict F8 HUD assertion because the preceding old GUI pilot reported NO_VISIBLE_GAME_WINDOW_UNDETERMINED after its 8-second detection window; the NEW fullscreen probe already passed 8/8 desktop captures in #42836412666. Batch03 focused reliability follow-up [4445e81](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/4445e81eaa709a8e71216f2a2b2a3f5fd4c82f08) changes ONLY the old CI-side windows_gui_pilot.py startup timeout 8s -> 30s, records elapsed native window discovery; DOES NOT relax the F8 assertion or game code. On the passing run old GUI detection took 7.516s, suggesting the old timeout was marginal, while exact root cause of earlier no-window detection is not proven. Trace RE-20261110-021, AUD-20261110-021. Draft PR18 and source issue17 stay open, main/Level1B untouched. No other roadmap batch authorized; next 04 Intro-Audio only after another user instruction.


## FS-CORE-02 — user-authorized borderless fullscreen (2026-11-10)

Batch 02 of the independent eight-batch 2026-11-10 roadmap is now implemented in **native Rust Win32 only** (two implementation commits [7574f35](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/7574f35928c662ab8a58ec0d943591f0c6dcfa76) and [7d5c527](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/7d5c52759ff985e1375eb530c45a1acfcd194171)). Ordinary F enters borderless fullscreen on the nearest intersecting monitor; Esc restores the previous window style and WINDOWPLACEMENT (including maximized state). Existing 600x400/3:2 stage viewports, title startup gate, focus gating, intro/Instructions Escape semantics outside fullscreen, QA F5-F8 navigation and defaults are preserved. Fullscreen Esc is reserved for mode exit and consumed until released, preventing the same press from skipping a story scene. Focus regain requires release-then-press for F/Esc fullscreen commands. A standalone AtomicBool communicates fullscreen mode to reentrant WM_DPICHANGED handling without introducing another overlapping App borrow. No new crates, game logic, assets, source-SWF geometry, audio or icon changes.

**Windows CI:** [Windows Actions #42832410562](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42832410562) completed SUCCESS at code SHA 7d5c52759ff985e1375eb530c45a1acfcd194171: 63/63 workflow steps, native MSVC Rust 18/18 (including two new fullscreen key/viewport unit tests), historical 1x Rust screenshot-hash regressions 15/15, real Win32 legacy GUI pilot 9 screenshots and errors=[], ordinary inherited-Enter title probe errors=[], HQ renderer p95 21.3317 ms and scenery-scroll p95 26.5107 ms (both strictly <40 ms). [Executable artifact #13771439858](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42832410562/artifacts/13771439858); [genuine legacy-GUI evidence #13772866479](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42832410562/artifacts/13772866479). The GUI pilot never pressed F or Esc to test fullscreen, so window-mode OS geometry/focus/taskbar correctness remains specifically UNVERIFIED and is the sole Batch03 target. Fullscreen physical Win32 F/Esc toggling, alternate-monitor scaling, taskbar interactions and fullscreen-specific screenshot automation are **NOT covered by the existing GUI pilot**; dedicated separately authorized Batch 03 FS-QA must build that evidence. Intro-focus/audio repair is Batch04/05; EXE art/icon is Batch06/07; final combined QA Batch08. Do not start any later batch without a new user request. Keep PR #18 OPEN/DRAFT, issue #17 open and main/Level1B untouched. Trace [RE-20261110-020](docs/REVERSE_ENGINEERING_RESEARCH_LEDGER.md) and [AUD-20261110-020](docs/RUST_PURITY_AND_ENGINEERING_AUDIT.md). This is a user-requested native window feature, not an assertion of original Flash fullscreen behavior.


This file is the provider-neutral entry point for **ChatGPT, Codex, Claude, Gemini, or any other contributor**. Do not depend on any previous chat, account memory, scratchpad, or ephemeral local file.

## HUD-REAL-07A — permanent rule for numeric-sized Lives X (2026-11-10)

**Scope:** This is user-authorized mini-batch 01/08 of the HUD/fullscreen/audio/icon roadmap. The player wanted X next to the Lives number to be as large and as styled as the number, not the tiny caption. The only game-code-facing changes are offline 3x themed-HQ HUD asset baking/layout and its Windows CI checks. **Do not start mini-batch 02 or later without another user instruction.**

**Verified native code:** [87f2d29](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/87f2d29df2138f9ae570136e4264f7f02276f6b7), real Windows [CI #42824607288](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42824607288) completed SUCCESS, all **63/63 steps**; Rust **16/16**; original 1x image hashes **15/15**; three real Win32 F8 HUD cases and mouse SKIP, F7, ordinary Enter-held title tests passed. The alpha-ink report shows overlaps=[] across the existing three multi-value scenarios, including Lives 10,12,99. Source 6x baker independently measures zero X-to-caption ink collision and exactly one 6x pixel row of contact with source opaque shell art, subject to a strict maximum of one. Renderer p95=17.7125ms and scrolling p95=21.3369ms against unchanged <40ms thresholds. GUI [artifact #13769361511](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42824607288/artifacts/13769361511), executable [artifact #13768557537](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42824607288/artifacts/13768557537).

**Themed HQ implementation detail:** Original SWF static text #715 is unmodified source data, but the styled-HQ baker draws only LIVES at same size based on the previous LIVES X width fitting, offset [-8,+5] stage pixels. Generate a temporary eleventh glyph, uppercase X, alongside the 115% small Impact numeral atlas, crop only invisible atlas margins, and composite X into the original-sized themed HUD shell at logical center (306,63). Emitted numeric atlas remains ten cells; existing Rust lives number is still x313, y63. Original source 1x, symbol716, panel #698, blue #693, yellow #705/#709, score, batarang, and no-duplicate-black-band rule remain unchanged. The 1-row **source-art edge contact is documented and narrowly limited, not described as zero intersection**. The larger X must never intersect independently baked static-caption ink.

**Permanent workflow:** For any next batch read this guide and both mandatory protocols, verify PR and branch head and last real Windows evidence, implement exactly one bounded task, run genuine Windows tests, update all SIX records (this file, AI_TASKS, AI_HANDOFF, AI_WORKLOG, research ledger, independent Rust audit), inspect actual results, commit with safe branch lease, report and STOP. Detailed source analysis: RE-20261110-019; audit: AUD-20261110-019. Five preceding failed HUD bake runs #42812461244, #42813657463, #42814943691, #42817311142, #42822810554 must remain visible in audit history. PR #18 must remain DRAFT, source-fidelity issue #17 open, main and Level1B untouched.

**NEXT ONE BATCH (only after separate authorization): 02 FS-Core**, add native Win32 F borderless fullscreen and Esc restore while preserving window placement, key routing, aspect ratio, intro behavior and default launch. Do not include its later fullscreen QA, audio repair, EXE icon or final integration within 02.

## Opt-in native Windows QA navigation (GUI development tool)

**Ordinary game builds launch unchanged.** For developer/testing sessions ONLY, run the same native Rust Windows EXE with `--qa-navigation`. This enables in-window scene shortcuts, preserving the original default experience when the flag is absent:

- **F5:** restart at title.
- **F6:** restart at the first story-intro frame.
- **F7:** restart Level 1A using the normal Rust level-entry routine and **150 neutral fixed-step gameplay updates** so Batman's physics and camera reach a stable gameplay/HUD inspection frame. This is not merely a display-clock jump.
- **F8:** only in --qa-navigation, cycle three **test-only** native Level 1A HUD fixtures: (Batarangs 20, Lives 10, Score 1234); (99,12,123456); (20,99,2147483647). Every F8 press resets via Game::new and Game::prepare_level1a_for_qa (150 neutral gameplay updates), then sets the counters for display. F7 resets F8 case index. Ordinary game launches do not accept these navigation shortcuts. The CI Win32 pilot uses actual SendInput F8 down/up to capture three separate real desktop screenshots; a second, offline-only Pillow script measures digit-to-digit and counter-to-opaque-shell alpha-mask intersections from the actual Windows-built 3x assets (see HUD-REAL-05C, RE-20261110-015). QA fixtures do not constitute original-game fidelity.

Use the opt-in mode in desktop GUI automation when moving between original Flash scenes; it is NOT a source-accurate gameplay shortcut and must not be part of original-game acceptance. The CI-only script `reverse-engineering/tools/windows_gui_pilot.py` launches the EXE with this flag, uses real mouse LEFTDOWN/LEFTUP inside the source-derived SKIP button #129, captures OS desktop PNGs after the fade and in gameplay, then captures a **separate** F7 shortcut frame. **Never treat F7 navigation as evidence that SKIP itself succeeded.** Review `pilot-report.json` and images independently. Normal game F5/F6/F7/F8 inputs should have NO effect without the flag.

Source: `src/main.rs` opt-in CLI flag, `src/win32.rs` gated `poll_qa_navigation` using real Win32 `GetAsyncKeyState` rising edges; existing `Game::new` is used to reset state without shipping any non-Rust gameplay code. New QA behavior must pass real Windows CI before being reported as verified.

## Title-screen startup-input protection (HUD-REAL-06A, 2026-11-10)

The ordinary **native Rust Windows EXE must NEVER advance from Title merely because Enter or Space was already held by the launching shell** (for example, PowerShell's final Enter key). The `src/win32.rs::TitleStartGate` starts unarmed, sees BOTH start keys released after launch/focus regain and then accepts only a subsequent fresh Enter/Space down-edge through the existing `InputState`. On `WM_KILLFOCUS` / inactive `WM_ACTIVATE`, or opt-in F5 QA return to title, it re-arms this release requirement. No additional countdown, splash-screen change, pause, or user-facing runtime dependency is permitted. Other input keys (including `I` instructions) are not masked. Original root-frame18 Enter/Space behavior remains intact once the start gate is armed. This native Windows-specific guard is not a recovered Flash logic change.

**Permanent verification:** Windows CI must execute `reverse-engineering/tools/windows_title_startup_probe.py` using **the ordinary executable with no `--qa-navigation`**, synthetically hold Enter with real Win32 `SendInput` BEFORE `Popen`, capture actual OS title images while held and after releasing, press new Enter and prove the intro appears. The script classifies the known title's colorful upper-center art versus the source intro's black upper band using a documented sample ROI and reports both images and ratios; it also cleans up the injected key and process. This is backed by a Rust unit test for Enter AND Space. Tested [Windows run #42764880940](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42764880940) passed **63/63 steps**, **16/16 Rust tests**, **15/15 historical 1x image hashes**, and real Win32 held-key probe with errors=[]; screenshots/report at [artifact #13739209445](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42764880940/artifacts/13739209445). See RE-20261110-016 and AUD-20261110-016 for evidence/limits. This protects title input only; do not equate it with original Flash-runtime full fidelity.

## HUD-REAL-06C verified — compact large SCORE counter glyph advance (2026-11-10)

**Current verified game code:** [`7df83428`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/7df83428b9f66a31170b09808028521fe0c4fb89) on `revive/level1a-intro-hud-fidelity-2026-11-09`. This is a **separately user-authorized** mini-batch after 06B. [Windows CI #42806477547](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42806477547) completed SUCCESS **63/63 steps**, **16/16 Rust tests**, **15/15 historical original-1x image hashes**, native Win32 mouse-SKIP/F7/F8 and ordinary held-Enter startup regressions. HQ render p95 **17.5360 ms**; scrolling p95 **21.6214 ms**; unchanged caps strictly **<40 ms**. [Native Windows EXE](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42806477547/artifacts/13758354942); [real GUI screenshots + alpha report](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42806477547/artifacts/13759800193); [renderer asset diagnostics](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42806477547/artifacts/13759725337).

Only themed HQ large SCORE digits use `large_digit_advance=16` **600x400 logical pixels**, or **48 true-3x pixels**, instead of the prior atlas-cell **66px**. Original large cell dimensions, glyph raster pixels, source icon geometry, translucent backing and labels (#700 BATARANG, #710 HEALTH, #711 SCORE, #715 LIVES X), logical SCORE center (387,35) and width budget92 are unchanged. For values not fitting the original full-cell selection (e.g. 6/10-digit scores), the prior small-atlas fallback/natural-step behavior is **unchanged**. The legacy/source 1x HUD and SWF are untouched. The 100 ordered large-digit pair alpha-BBox tests show **minimum 3 true-3x pixels of horizontal clearance at alpha>=32**; all three F8 combinations also report zero digit/digit, digit/opaque-shell and inter-counter thresholded overlaps. The [real 930x660 Windows F8 screenshot](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42806477547/artifacts/13759800193) confirms narrower score gaps without a full-width black counter ribbon. Source 1x and 94% icon + 121%-height Batarang 06B modifications stay intact.

**Permanent cautions:** This is a *themed Impact/gold HQ candidate*, not exact original Flash-white font reproduction. CI cannot substitute for the user's 1920x1080 monitor review, physical audio test, original Flash-runtime A/B or complete Level1A acceptance. PR #18 stays OPEN/DRAFT, issue #17 open, `main`/Level1B untouched. The immediately following **06B section is historical**: its "06C NOT IMPLEMENTED" and "requires user authorization" statements describe the earlier 06B checkpoint only; 06C is now committed and verified. See RE-20261110-018 and AUD-20261110-018; next user-authorized task is **player visual approval / original-style comparison**, not autonomous Level1B development.

## HUD-REAL-06B permanent HUD source-fidelity guard (2026-11-10)

The blue Batarang source child #693 nested under #703 and yellow Health **colored** components #705/#709 (under sprite #716) were scaled to 94% during themed-HQ vector baking from a **copy** of the canonical SWF symbols metadata. Crucially **#698 is the original translucent panel** and was restored unchanged; original static labels #700 BATARANG, #710 HEALTH, #711 SCORE, and #715 LIVES X were never transformed. Final `bake_sprite_frames.py --bounds-symbols-json` uses the **original unmodified SWF canvas bounds** when rasterizing the HQ icon changes, preserving labels/panel anchor. An independent Batarang-only 121%-vertical-ink atlas (previous shared small glyph 115%) is drawn at logical center (211,58), one logical pixel above (211,59), with unchanged 9px digit pitch. Lives/small-score retain their 115% atlas and the large SCORE digit advance remains unchanged: 06C NOT IMPLEMENTED. Original source SWF, 1x rendering, gameplay, intro and inputs unchanged.

Windows-verified code SHA `0c4cf23992b9b120454b3f4b3742488775890bf8`, [CI run #42774124754](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42774124754): success 63/63 steps, Rust 16/16, historical original 1x screenshots 15/15, real native Win32 SKIP/F7/F8 and inherited-Enter normal-launch regression passed, 3 alpha-mask F8 cases with zero reported ink intersections (glyph alpha>=32; shell opaque alpha>=208), HQ render p95 21.5922ms and scroll p95 26.1790ms (both below the unmodified 40ms limit). [Game EXE](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42774124754/artifacts/13742814809), [genuine Windows screenshots plus alpha report](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42774124754/artifacts/13742894598), [baked renderer diagnostics](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42774124754/artifacts/13743058332).

Independent old/new artifact pixel comparison: both RGBA HUD shell PNGs 837x246; all four BATARANG/HEALTH/SCORE/LIVES X text-label regions have exactly **0 changed RGBA pixels**; panel sample at (750,150) is unchanged [96,115,115,82]; old/new `assets/hud_base.png` legacy SHA-256 and shared small/large HQ atlas bytes match. Modified shell union diff box (19,77,534,203) localizes new icon shapes. Real Windows 930x660 F7 and F8 image crops show smaller blue/yellow icons, slightly taller/higher Batarang numerals, untouched top captions and no duplicate full-width black bar. This is NOT a new 1920x1080 user's-monitor capture or original-Flash-runtime parity proof.

Keep draft PR #18 OPEN/DRAFT and source-fidelity issue #17 open. No `main` merge, Level 1B development, 06C score-spacing change, generic full-width black ribbon, source-1x modification, new gameplay language/runtime dependency, or unsafe Rust. The 94% icons and Impact gold typography are a deliberate themed HQ candidate, not original Flash exact typography. The player has not yet approved the new styling on their 1920x1080 machine. Research RE-20261110-017; audit AUD-20261110-017. Next batch HUD-REAL-06C requires **separate user authorization**. Never scale backing SWF child #698 when changing the yellow Health emblem; preserve source bounds, original 1x and all labels.

## PERMANENT MANDATORY PROTOCOLS — READ BEFORE EVERY BATCH (2026-11-09)

**USER-DIRECTED, BINDING PROJECT RULES.** This repository's six-document workflow and free Windows executable testing policy must not depend on any ChatGPT conversation. ALL current and future coding agents must read these two self-contained documents, then follow them on EVERY bounded development batch:

1. **[docs/DEVELOPMENT_WORKFLOW_PROTOCOL.md](docs/DEVELOPMENT_WORKFLOW_PROTOCOL.md)** — complete **mandatory SIX-document lifecycle**, including AGENTS.md, AI_TASKS.md, AI_HANDOFF.md, AI_WORKLOG.md, REVERSE_ENGINEERING_RESEARCH_LEDGER.md, and RUST_PURITY_AND_ENGINEERING_AUDIT.md. Every batch: recover actual Git state → select ONE small task → investigate → implement → verify → document research → independently audit changed Rust → synchronize six files → commit/verify → report → STOP. Every file must be reviewed; update when applicable; never silently omit the older four because the two newer files are more recent.
2. **[docs/AUTOMATED_WINDOWS_TESTING_PROTOCOL.md](docs/AUTOMATED_WINDOWS_TESTING_PROTOCOL.md)** — mandatory staged, free executable QA objective: build/run actual Windows Rust EXE in GitHub Actions, exercise deterministic gameplay, capture screenshots and short frame sequences, compare against verified original evidence, archive failure artifacts, let an agent actually inspect the images where tools permit, and pilot real Win32 window/input testing separately. Existing headless/CI infrastructure is not a substitute for a real GUI test and does not prove original fidelity. Implementation is a separate small-batch roadmap item, not a claim of completed automation.

**Priority and continuity:** These protocols EXTEND all earlier project constraints, do not remove the existing Windows delivery contract, and do not authorize a risky mid-task rewrite. Preserve pure-Rust game runtime, draft PR #18, current gameplay branch and source-original acceptance issue #17. Agents continuing work already in progress should finish the current safe checkpoint, then READ BOTH PROTOCOLS and synchronize required records before their next batch. A GitHub push does NOT automatically inject instructions into an active ChatGPT chat; reread on the next turn. Do not overwrite another agent's in-flight work.

**Every final batch report must include the six-file compliance table and a truthful test/CI/artifact summary.** Distinguish actual tests from plans, original-Flash proof from Rust-on-Rust regression, and docs-only status from implementation verification.

## Mandatory recovery order

1. Read **[docs/AI_HANDOFF.md](docs/AI_HANDOFF.md)** for the most recent evidence-based checkpoint.
2. Read **[docs/AI_TASKS.md](docs/AI_TASKS.md)** for the small prioritized work items and their acceptance tests.
3. Read the relevant entries in **[docs/REVERSE_ENGINEERING_RESEARCH_LEDGER.md](docs/REVERSE_ENGINEERING_RESEARCH_LEDGER.md)** and **[docs/RUST_PURITY_AND_ENGINEERING_AUDIT.md](docs/RUST_PURITY_AND_ENGINEERING_AUDIT.md)**, then [AI_WORKLOG](docs/AI_WORKLOG.md) and the [detailed Level 1A presentation log](docs/revival/LEVEL1A_PRESENTATION_FIDELITY_LOG_2026-11-09.md).
4. Verify the **actual remote branch head, PR state, CI runs, and artifacts** against those notes before making changes. Correct stale documentation rather than trusting it.

## Permanent Windows game delivery contract — user-preferred workflow (2026-11-09)

**Do not make a manually repackaged conversation ZIP the primary delivery method when the user asks for the playable game.** The standard is the GitHub CLI → latest successful Actions artifact → `%LOCALAPPDATA%\BatmanCobbleBot` → launch workflow. On every future playable-build report:

1. Verify the real latest **successful** run on the development branch for workflow `build-level1a-fidelity.yml`; confirm an unexpired artifact named `Batman-CobbleBot-Level1A-Fidelity-Windows-x64`, its actual head SHA, real test results and that the user is not accidentally getting a pending/failed run.
2. Say which actual run, revision, Windows test totals and remaining human-fidelity blockers were verified. Provide a clickable **GitHub Actions run URL** that exposes the artifact; label a direct ZIP download as optional, not the preferred installation route.
3. **Always give a complete copy-paste PowerShell 5.1+ installer command** in the reply, which automatically selects the latest successful run (never a hard-coded stale run ID), downloads the artifact with authenticated `gh`, installs/replaces `batman_cobblebot_native.exe` in `%LOCALAPPDATA%\BatmanCobbleBot`, writes `last-run.txt`, `last-commit.txt`, `last-sha256.txt`, and starts the game. It must not rebuild locally or install Flash/Python/browser runtimes.
4. Warn the user to close any running Batman window. Require GitHub CLI (`winget install --id GitHub.cli --exact`) and `gh auth login` if needed. Download to a disposable temp directory; verify exe exists and source/staged SHA-256 match before replacing the installed game. Preserve the previous executable as `.bak` where possible and do not claim successful installation until the user runs the script; the assistant does **not** control the user's Windows machine.
5. If there is no confirmed successful run/artifact, **do not pretend to deliver** an unverified latest executable; report the blocker and prioritize fixing/retrying CI. A documentation-only run may rebuild a byte-different EXE even if Rust source is unchanged; use the actual latest-run binary/hash instead of assuming historical SHA-256 values still match.

**Canonical PowerShell block to include in user-facing game deliveries (copy intact unless verified to need a scoped fix):**

```powershell
$ErrorActionPreference = "Stop"
$repo = "sinethnimsara26/Batman-CobbleBot-Native-Rust"
$branch = "revive/level1a-intro-hud-fidelity-2026-11-09"
$artifact = "Batman-CobbleBot-Level1A-Fidelity-Windows-x64"

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw "GitHub CLI missing. Install with: winget install --id GitHub.cli --exact"
}
gh auth status --hostname github.com *> $null
if ($LASTEXITCODE -ne 0) { throw "Log into GitHub CLI first: gh auth login" }

$runJson = gh run list --repo $repo --branch $branch --workflow build-level1a-fidelity.yml --status success --limit 1 --json databaseId,headSha
if ($LASTEXITCODE -ne 0) { throw "Could not query successful Windows builds." }
$run = @($runJson | ConvertFrom-Json) | Select-Object -First 1
if (-not $run) { throw "No successful Windows build found on the development branch." }

if (Get-Process -Name "batman_cobblebot_native" -ErrorAction SilentlyContinue) {
    throw "Close the currently running Batman game before updating it."
}
$dest = Join-Path $env:LOCALAPPDATA "BatmanCobbleBot"
$temp = Join-Path $env:TEMP ("BatmanBuild-" + [guid]::NewGuid().ToString("N"))
$staged = Join-Path $dest ("batman_cobblebot_native.exe." + [guid]::NewGuid().ToString("N") + ".new")
New-Item -ItemType Directory -Force -Path $dest,$temp | Out-Null
try {
    gh run download $run.databaseId --repo $repo --name $artifact --dir $temp
    if ($LASTEXITCODE -ne 0) { throw "GitHub artifact download failed." }

    $source = Join-Path $temp "batman_cobblebot_native.exe"
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Expected Windows executable was not in the downloaded artifact."
    }
    $hash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
    Copy-Item -LiteralPath $source -Destination $staged -Force
    if ((Get-FileHash -LiteralPath $staged -Algorithm SHA256).Hash -ne $hash) {
        throw "Copy integrity check failed; the previous game was not replaced."
    }

    $exe = Join-Path $dest "batman_cobblebot_native.exe"
    if (Test-Path -LiteralPath $exe) {
        [System.IO.File]::Replace($staged,$exe,(Join-Path $dest "batman_cobblebot_native.exe.bak"))
    } else {
        [System.IO.File]::Move($staged,$exe)
    }
    Set-Content -LiteralPath (Join-Path $dest "last-run.txt") -Value $run.databaseId
    Set-Content -LiteralPath (Join-Path $dest "last-commit.txt") -Value $run.headSha
    Set-Content -LiteralPath (Join-Path $dest "last-sha256.txt") -Value $hash
    Start-Process -FilePath $exe -WorkingDirectory $dest
    Write-Host "Launched Batman build $($run.databaseId) ($($run.headSha)); SHA256 $hash"
}
finally {
    Remove-Item -LiteralPath $staged -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
```

**Six-file documentation integrity check:** The 2026-11-09 WIN-01-QA-PACK-01 checkpoint actually added/modified exactly these six persistent files: `docs/WINDOWS_QA_PACKAGE_2026-11-09.md`, `docs/REVERSE_ENGINEERING_RESEARCH_LEDGER.md`, `docs/RUST_PURITY_AND_ENGINEERING_AUDIT.md`, `docs/AI_TASKS.md`, `docs/AI_HANDOFF.md`, and `docs/AI_WORKLOG.md`. Before the next handoff inspect all six in the actual remote branch, confirm current source/CI/file-hash claims and cross-references, preserve old pinned evidence as historical data, and repair any stale *current* delivery instruction. `AGENTS.md` is the **seventh governance file** and must contain these permanent instructions. Future batches must continue the two detailed RE/AUD records, the task/handoff/worklog lifecycle and any relevant QA manifest amendments; do not artificially rewrite all six in unrelated tasks.

## Mandatory research ledger and post-implementation Rust audit (permanent protocol)

**Effective 2026-11-09. This EXTENDS, not replaces or weakens, all rules below.**

Every batch must use the two authoritative, durable Markdown files:
- **[docs/REVERSE_ENGINEERING_RESEARCH_LEDGER.md](docs/REVERSE_ENGINEERING_RESEARCH_LEDGER.md)** — detailed, independently reproducible original-game evidence, exact SWF symbol/tag/frame/action-offset and code locations, hypotheses and alternatives, negative results, test outputs, external-source provenance and confidence-rated decisions.
- **[docs/RUST_PURITY_AND_ENGINEERING_AUDIT.md](docs/RUST_PURITY_AND_ENGINEERING_AUDIT.md)** — independent **post-implementation** inspection of actual Rust changes, architecture, dependencies/features/transitives, FFI/native components, idiomaticity, safety, correctness, efficiency, engineering suitability and separately assessed fidelity.

Existing `AI_HANDOFF.md`, `AI_TASKS.md`, `AI_WORKLOG.md`, detailed revival and rendering logs remain mandatory and must cross-reference these files. Do not flatten older research into short summaries. Never invent external citations, measurements, crash explanations or exact sources; never present private thought transcripts as evidence. Instead record reproducible observations, candidate explanations, supportive and contradictory evidence, decisions and testable trade-offs. Record direct vs historical evidence and confidence (**HIGH/MEDIUM/LOW/UNKNOWN**) with justification. On contradiction preserve a dated correction and link to the superseded claim.

**Strict playable-game implementation:** All game-specific runtime mechanics (state, input, animation, renderer orchestration, collision, audio-event dispatch, menus, progress, UI, cutscenes) are Rust. No Flash VM, Ruffle as engine, ActionScript execution, JS/TS/Electron gameplay, Python runtime, C/C++ gameplay engine hidden behind FFI, or external gameplay executable. Native OS APIs, drivers and system media facilities are permissible **with documented qualification**, not falsely called pure Rust decoding/driver implementations. Evaluate what each crate actually implements rather than trusting its Rust API surface. Legacy Python SWF parsing/Pillow baking is OFFLINE BUILD/CI ONLY; source data is embedded into Rust. Do not replace offline tools or add a new runtime engine without a separate authorized batch.

**New blocker-first correction rule (user reaffirmed 2026-11-09):** If ANY change, test, workflow, fidelity comparison, implementation audit, or new symptom reveals something wrong, broken, incomplete, regressed, below a measured limit, or otherwise blocked, **prioritize diagnosing, fixing, re-testing and documenting that problem before moving to the next unrelated batch**. Within the current bounded task, perform safe targeted retries or revisions against the same defect, preserving exact evidence and the existing test thresholds rather than weakening/skipping tests. If genuinely unable to resolve within the current context (e.g., unavailable inputs, environmental nondeterminism, unsafe or disproportionately broad scope, persistent CI failures), explicitly document the exact reproducible blocker, attempted remedies, measurements and why it remains unresolved, mark the task BLOCKED/UNVERIFIED, and keep it as the NEXT task. Never silently advance scope, mislabel a flaky rerun as proof of a definitive fix, or claim success before the actual runner reports it. This strengthens both the one-batch rule and the source-faithfulness/strict Rust runtime requirements; it does not authorize unauthorized merges, infinite/background execution, deleting history, or changing unrelated code.

**Every code-changing batch — expanded exact lifecycle (one small item per user instruction):**
1. **RECOVER:** read this guide, current `AI_HANDOFF`, `AI_TASKS`, relevant RE and AUD entries, `AI_WORKLOG`; independently verify actual remote SHA/PR/CI/artifacts.
2. **SELECT:** one bounded task with fidelity and engineering acceptance criteria, and any previous source-based findings.
3. **INVESTIGATE:** compare original SWF evidence, current code, exact source paths and CI; consult official Rust/crate docs and qualified community sources *only as material to the decision*, recording titles, URLs, access dates, relevant sections, caveats.
4. **IMPLEMENT:** smallest appropriate Rust/source-backed change; do not rewrite unrelated code or depend on non-Rust game functionality.
5. **TEST:** execute relevant build, unit, smoke, performance, source and/or visual checks; classify actual pass/fail/NOT RUN. Compilation ≠ fidelity.
6. **RESEARCH:** append full `RE-YYYYMMDD-NNN` technical evidence and decision record with confidence and exact source locations, or explicitly mark research-only applicability.
7. **INDEPENDENT AUDIT:** inspect the **finished changed code**, diff, relevant dependency graph and FFI; evaluate correctness, ownership, safety, performance, alternatives and exact crate version/features. Run applicable Cargo checks (`fmt --check`, `check`, `test`, `clippy`, `build`, `tree`, `metadata`) when feasible; explain unrun checks.
8. **RECORD AUDIT:** append cross-linked `AUD-YYYYMMDD-NNN` with documented evidence and explicit verdicts: **PURE-RUST COMPLIANCE** (PASS / PASS WITH DOCUMENTED PLATFORM/BUILD QUALIFICATIONS / FAIL / UNVERIFIED), **ENGINEERING QUALITY** (PASS / PASS WITH NON-BLOCKING CONCERNS / NEEDS IMPROVEMENT / UNVERIFIED), **ORIGINAL-GAME FIDELITY** (VERIFIED FOR THE TESTED SCOPE / PARTIALLY VERIFIED / KNOWN MISMATCH / UNVERIFIED). Include confidence, severity, file, remediation and blockers. If a blocker cannot safely be fixed in this batch, track it; do not claim completed verification.
9. **EXISTING MEMORY:** update task status, chronological worklog and concise handoff with links to both new records; preserve important older history.
10. **CHECKPOINT:** commit actual code/docs to the authorized development branch, verify remote SHA and CI as appropriate, never merge/force-push/unlock unrelated work. Where a later doc checkpoint audits an earlier code commit, link the earlier SHA. A commit cannot include its own SHA in its contents; record final hash in final report/Git history.
11. **REPORT & STOP:** deliver verified result, evidence, three verdicts, RE/AUD IDs, exact commit/PR/run, open risks and one next bounded task; never autonomously proceed.

**Research-only / documentation-only batch:** Add relevant RE entry and explicitly write **"No implementation changes — Rust code audit not applicable"** in the audit, without inventing a post-code audit. Existing baseline architecture review is permitted if labeled retrospective and incomplete.

**Provenance requirement:** For materially used external sources record page title, exact URL/permalink, author/maintainer, publication/update date if known, access date, source type, relevant section, technical claim, exact application to this game, agreement/disagreement, reliability/limits, tested status, relevant crate/API versions. Prefer original data, Rust/Cargo/reference/API docs, docs.rs and maintainers' source; community/Reddit posts are contextual evidence, not universal rules. No exhaustive ecosystem survey for a trivial patch. For exact source findings, record original file, SWF IDs and frame indices, byte offsets/coordinate systems, generated artifact/SHA when available, Rust file/function/line at pinned commit, actual tests and uncertainty.

**Interruption protection:** Keep the previous one-batch limit. An entire batch may be research/audit; checkpoint before starting an unrelated fix. Never let a long audit crowd out saving documentation. Do not report artifact/job health without checking current run. No unauthorized merges/branch rewrites, secrets or proprietary binaries in Git.

## Purpose and technical design

- Unofficial native **Rust/Win32** preservation of the original Flash game, not a loose remake. No Flash/browser runtime.
- **Pure Rust playable runtime contract (user reaffirmed 2026-11-09):** All game state, rendering, frame timing, inputs, collision, audio dispatch and transition logic must be implemented in Rust (calling native Windows APIs where necessary). Do not ship or invoke Python, Flash Player, JavaScript, Electron, a browser, or external script interpreters to run the game. Python tools under `reverse-engineering/tools/` are **offline build/CI-only original-SWF extraction and sprite baking**, producing byte assets that are embedded via Rust `include_bytes!`; they are not game code. If replacing these preparatory tools with Rust becomes a separate authorized milestone, keep that separate from playable runtime fidelity.
- Source-derived **600 × 400 logical pixels**, **25 Hz fixed simulation** (40 ms), optional true 3× (1800 × 1200) artwork/HQ presentation. Faithfulness includes scene flow, game feel, control input/focus, animation, camera, collision, music, HUD, and text.
- `src/game.rs`: game screens (`Title`, `Instructions`, `Intro`, `Playing`), state machine, physics, Level 1A only. `src/win32.rs`: native Windows window, input/focus, DPI, click mapping. `src/render.rs` and `src/surface.rs`: compositor, legacy/HQ drawing. `src/assets.rs` and `src/collision.rs`: embedded generated packs and collision. `src/audio.rs`: original audio via WinMM/MCI. `src/headless.rs`: deterministic screenshot, transition and render-performance tests.
- `reverse-engineering/manifests/` contains documented source metadata; `reverse-engineering/tools/` parses/bakes the hash-verified original SWF. Generated proprietary sprites, music, and executable are **CI artifacts**, not source files to commit. Respect `THIRD_PARTY_NOTICE.md`.
- GitHub Actions: `.github/workflows/build-level1a-fidelity.yml` on `windows-latest`. It downloads/verifies the canonical original SWF, rebuilds all assets, compiles Rust, runs tests/visual comparisons, and uploads Windows x64 and diagnostic artifacts. **Local `cargo build` / `cargo test` alone may fail without the CI-generated `assets/` tree**.

## Fidelity and branch protection

- **Level 1A is the only active game scope.** Do not implement Level 1B, bosses, or the campaign until the manual Level 1A gate is satisfied.
- Compare original Flash behavior/art with native before changing it. Preserve control geometry as invisible; do not render debug collision rectangles as visible scenery. Do not invent dialogue, soundtrack, or timing.
- Original Flash uses mostly **white HUD lettering**. Gold/bold HQ HUD is a design *candidate*, not verified source fidelity; preserve the legacy source-style oracle for A/B evaluation.
- `main` and earlier rendering branches remain untouched until explicitly approved. Current draft work is chained through [PR #16](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/16) and [PR #18](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/18).
- Never force-push, reset destructively, delete/merge branches, rewrite other contributors' work, change deployment, or add proprietary assets/secrets without explicit authorization.

## Original one-message / one-batch lifecycle (still binding; expanded above)

Each user message saying **"continue" authorizes exactly ONE bounded, independently verifiable task**. Avoid unrelated fixes and sprawling investigations. Prefer approximately 6–8 substantive tool operations; ~10 is a conservative checkpointing target, not a strict platform ceiling.

1. **RECOVER:** read handoff/tasks/worklog and independently inspect remote branch/CI/artifacts.
2. **SELECT:** choose the highest-priority small unblocked task; state a measurable acceptance criterion.
3. **IMPLEMENT:** smallest appropriate code/docs change; preserve known-good behavior.
4. **VERIFY:** use targeted tests and relevant screenshots; distinguish **code committed**, **compile/test green**, **full CI green**, **runtime observed**, **manual source fidelity approved**, and **unverified**. CI does not prove Windows audio or gameplay feel.
5. **CHECKPOINT:** update handoff, task status, and concise worklog entry **before stopping**, commit/push to the authorized branch, confirm remote SHA. Mark interrupted/incomplete work WIP/UNVERIFIED. Avoid a failing commit on a release/deployment branch.
6. **REPORT AND STOP:** say what changed, what evidence actually passed, commit/run links, remaining blockers, and the exact next small batch. **Do not automatically start another task.**

On interruption, inspect GitHub first: if a prior operation may have succeeded, confirm it before repeating. If GitHub writes fail, provide recovery notes explicitly and do not claim they were persisted.

## Useful reproducible checks

- Repository: `https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust`
- `git fetch origin && git status --short && git rev-parse HEAD` (local clone, if present).
- `gh pr view 18 -R sinethnimsara26/Batman-CobbleBot-Native-Rust`
- `gh run list -R sinethnimsara26/Batman-CobbleBot-Native-Rust --branch revive/level1a-intro-hud-fidelity-2026-11-09 --limit 5`
- Do not hard-code obsolete build `42575731190` or older SHA in game installation instructions. Use the **Permanent Windows game delivery contract** above and verify the latest successful run and artifact before printing the installer.
- On a completed Windows CI asset build: `cargo test`, `cargo build --release`, `cargo run --release -- --render-smoke <path>` **only after checking the project's actual CLI usage**; prefer the exact workflow commands rather than guessing flags.
- Manual gate and reproduction checklist: [issue #17](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/issues/17).

Engineering documentation should record **observable evidence, decisions, alternatives, fixes, and errors**, not hidden/private chain-of-thought or whole conversation transcripts.
