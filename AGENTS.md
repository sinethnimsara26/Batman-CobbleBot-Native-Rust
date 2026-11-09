# AI agent operating guide — Batman: The CobbleBot Caper

This file is the provider-neutral entry point for **ChatGPT, Codex, Claude, Gemini, or any other contributor**. Do not depend on any previous chat, account memory, scratchpad, or ephemeral local file.

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
