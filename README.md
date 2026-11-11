# Batman: The CobbleBot Caper — Native Rust Reimplementation

## Latest checkpoint — 2026-11-11 player-approved Level 1A freeze

The player has tested the native game and reports that everything now looks fine. **Level 1A is finished for now**, not the entire multi-level game. The frozen source is [`49e8fd343e4f37f045ba71f24fff0999211f0f11`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/49e8fd343e4f37f045ba71f24fff0999211f0f11) on permanent snapshot branch [`release/level1a-v0.2.0`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/tree/release/level1a-v0.2.0); matching [Windows CI #42936169736](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42936169736) completed SUCCESS (81/81 steps, 21/21 Rust tests). A one-shot release publisher is being configured for [`level1a-v0.2.0`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/releases/tag/level1a-v0.2.0). **Do not call the release published until that URL displays actual EXE, ZIP and checksum assets.** See [release notes](docs/releases/LEVEL1A_V0.2.0_RELEASE_NOTES.md) and [code-quality report](docs/releases/LEVEL1A_V0.2.0_RELEASE_QUALITY.md).

User acceptance is a practical approval of present gameplay/presentation, **not** proof of exact original-Flash equivalence, real speaker playback, pinned taskbar icon cache or every DPI. Issue #17 remains open for those stricter questions; PR #18 stays draft. No Rust/gameplay modifications were made during this release checkpoint; `main` and all Level1B branches remain untouched. Older below sections are preserved as historic status, not current release blockers.


Unofficial high-fidelity native Rust reimplementation of **The Batman: The CobbleBot Caper**, originally released as a Flash game.

The goal is not a loose remake. The project aims to preserve the original game's **visuals, animation timing, controls, camera behavior, collision, UI flow, and feel** while replacing the Flash runtime and ActionScript execution with native Rust wherever practical.

## Latest project checkpoint (2026-11-11)

**Windows local acceptance kit (Batch11):** Native [81-step Windows CI #42934164402](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42934164402) passed, including a test-only [PowerShell collector](scripts/collect-level1a-windows-evidence.ps1) that writes a local, non-uploaded Windows hardware/EXE provenance report and a player checklist. Read the [player acceptance guide](docs/revival/LEVEL1A_WINDOWS_ACCEPTANCE_GUIDE_2026-11-11.md). Original Flash side-by-side, actual audio playback, user pinned taskbar icon and mixed DPI remain unverified; Level1A issue17 stays open.



**Reproducible Windows Rust build (Batch10):** The native executable now tracks the exact Cargo-generated `Cargo.lock` (12 packages, generated on Windows CI), and the [fully successful 79-step Windows build #42899490952](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42899490952) uses `cargo test --locked` and `cargo build --locked` with strict lockfile-unchanged validation. Source/GUI/fidelity limits were preserved; real hardware sound/Flash acceptance remains open. See the latest [handoff](docs/AI_HANDOFF.md) and [build audit](docs/PROJECT_WIDE_AUDIT_2026-11-11.md).


**Active development:** [Draft PR #18](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/18) on `revive/level1a-intro-hud-fidelity-2026-11-09`; **Level 1A remains the only playable scope.** The game now has source-derived story intro/skip, a deliberately styled HQ HUD, native F/Esc fullscreen, audio-focus pause/resume, and a Batman emblem embedded into the native Windows EXE. These are original-source reconstruction candidates and user-requested usability additions, not proof of perfect Flash fidelity.

**Verified:** [75/75-step native Windows build #42888932281](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42888932281) and [documentation follow-up build #42891451043](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42891451043), both successful; 21 Rust tests, 15 original 1× screenshot regression hashes, real Win32 fullscreen/UI/EXE icon resource checks and a relocated standalone EXE smoke test. **Not verified:** physical speaker MCI audio (the hosted runner has zero audio endpoints and reports MCI277), human Explorer/taskbar cache display, original Flash side-by-side equivalence and multi-monitor physical DPI testing. Keep [acceptance issue #17](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/issues/17) open; do not merge PR #18 or start Level 1B.

**Start here:** [Current handoff](docs/AI_HANDOFF.md), [tasks](docs/AI_TASKS.md), [project-wide recovery audit](docs/PROJECT_WIDE_AUDIT_2026-11-11.md), [agent rules](AGENTS.md), [RE research ledger](docs/REVERSE_ENGINEERING_RESEARCH_LEDGER.md), and [Rust audit](docs/RUST_PURITY_AND_ENGINEERING_AUDIT.md). Use the permanent GitHub CLI installation command in AGENTS.md rather than historical artifact links.

**Build note:** the Windows CI fetches/validates the proprietary SWF and generates embedded game assets plus the ICO *before* running Cargo. A fresh checkout cannot run a standalone `cargo build` without that offline asset/resource-generation prerequisite. The playable released EXE requires neither Python nor Flash Player.

## Historical project scope and status

Development is deliberately frozen to **Level 1A** until the first playable area passes a strict fidelity gate.

Current recovery work includes:

- native Windows/Win32 runtime
- fixed 600×400 logical stage
- fixed 25 Hz simulation matching the SWF
- original Level 1A world/collision reconstruction
- separation of visible scenery from invisible Flash control geometry
- recovered Batman player state machine
- true child-animation timelines rather than the outer selector timeline
- robust foreground keyboard handling
- recovered Level 1A camera behavior
- original level-title, fade-in, tutorial overlay/checkpoints, and embedded-font tutorial text
- deterministic visual smoke tests

Later stages remain intentionally frozen until Level 1A is manually approved.

## Building

The repository contains **our Rust/Python source code and reverse-engineering/build tooling**, not the original proprietary game assets.

CI reconstructs development assets from a verified source SWF during the build. See `THIRD_PARTY_NOTICE.md`.

## License

Original project code is MIT licensed. Batman/DC/Warner/Cartoon Network game content is **not** covered by the MIT license.

This is an unofficial preservation/reimplementation experiment and is not affiliated with or endorsed by the original rights holders.

## Public CI status

The public repository is now the active build host for the Level 1A fidelity slice. Generated third-party game assets stay out of Git history and are reconstructed during CI.
