# AI agent operating guide — Batman: The CobbleBot Caper

This file is the provider-neutral entry point for **ChatGPT, Codex, Claude, Gemini, or any other contributor**. Do not depend on any previous chat, account memory, scratchpad, or ephemeral local file.

## Mandatory recovery order

1. Read **[docs/AI_HANDOFF.md](docs/AI_HANDOFF.md)** for the most recent evidence-based checkpoint.
2. Read **[docs/AI_TASKS.md](docs/AI_TASKS.md)** for the small prioritized work items and their acceptance tests.
3. Consult **[docs/AI_WORKLOG.md](docs/AI_WORKLOG.md)** and the existing detailed [Level 1A presentation engineering log](docs/revival/LEVEL1A_PRESENTATION_FIDELITY_LOG_2026-11-09.md) for earlier work, mistakes, and rationale.
4. Verify the **actual remote branch head, PR state, CI runs, and artifacts** against those notes before making changes. Correct stale documentation rather than trusting it.

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

## One-message / one-batch lifecycle — mandatory

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
- `gh run view 42575731190 -R sinethnimsara26/Batman-CobbleBot-Native-Rust`
- `gh run download 42575731190 -R sinethnimsara26/Batman-CobbleBot-Native-Rust -n Batman-CobbleBot-Level1A-Fidelity-Windows-x64 -D ./batman-test`
- On a completed Windows CI asset build: `cargo test`, `cargo build --release`, `cargo run --release -- --render-smoke <path>` **only after checking the project's actual CLI usage**; prefer the exact workflow commands rather than guessing flags.
- Manual gate and reproduction checklist: [issue #17](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/issues/17).

Engineering documentation should record **observable evidence, decisions, alternatives, fixes, and errors**, not hidden/private chain-of-thought or whole conversation transcripts.
