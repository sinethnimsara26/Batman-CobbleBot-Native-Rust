# Pure-Rust Implementation and Engineering Audit — Batman: The CobbleBot Caper

**Separate role:** Review the **actual implemented code** after a development batch: Rust-only compliance, crate/FFI composition, correctness, idiomaticity, safety, architecture, performance and independent original-game fidelity. Research findings, source bytes, alternative explanations and failed source experiments live in [the research ledger](REVERSE_ENGINEERING_RESEARCH_LEDGER.md); current tasks and recovery live in [AI_TASKS](AI_TASKS.md) and [AI_HANDOFF](AI_HANDOFF.md).

**Mandatory from 2026-11-09 onward:** Every CODE-changing batch requires (1) completed `RE-YYYYMMDD-NNN` entry, (2) post-implementation review of its **actual changed code and dependencies**, (3) completed `AUD-YYYYMMDD-NNN` entry, and (4) the existing task/worklog/handoff checkpoint, before being described as fully verified. A docs-only batch records **“No implementation changes — Rust code audit not applicable”** while it may review existing architecture. Audit classifications, with independent evidence:
- **PURE-RUST COMPLIANCE:** PASS / PASS WITH DOCUMENTED PLATFORM/BUILD QUALIFICATIONS / FAIL / UNVERIFIED
- **ENGINEERING QUALITY:** PASS / PASS WITH NON-BLOCKING CONCERNS / NEEDS IMPROVEMENT / UNVERIFIED
- **ORIGINAL-GAME FIDELITY:** VERIFIED FOR THE TESTED SCOPE / PARTIALLY VERIFIED / KNOWN MISMATCH / UNVERIFIED

These labels do not replace actual tests. Confidence: HIGH / MEDIUM / LOW / UNKNOWN with reasons. List issue severity and required remediation. Do not claim compiler success is fidelity proof, or recover hidden chain-of-thought.

## AUD-20261109-001 — Baseline Rust runtime and dependencies

**Date / scope:** 2026-11-09, documentation-only bootstrap. Parent code SHA [`c2b872e9858080fb360fa26af4164b2923632f9d`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/c2b872e9858080fb360fa26af4164b2923632f9d) on `revive/level1a-intro-hud-fidelity-2026-11-09`, draft PR #18. Linked research: [RE-20261109-001](REVERSE_ENGINEERING_RESEARCH_LEDGER.md#re-20261109-001--initial-sourcearchitecture-baseline-documentation-bootstrap), [RE-20261109-002](REVERSE_ENGINEERING_RESEARCH_LEDGER.md#re-20261109-002--intro-visual-repetition-routing-and-source-fade-historical-evidence-reconstruction), [RE-20261109-003](REVERSE_ENGINEERING_RESEARCH_LEDGER.md#re-20261109-003--two-successive-windows-render-benchmark-failures-new-cross-check).

**Audit applicability:** **No implementation changes — Rust code audit not applicable to this bootstrap commit.** The following is an initial *retrospective architecture and failure-risk baseline*, not a claim to have run after-the-fact tests for all historic batches.

### Implementation actually inspected

| Repo module / API | Role observed at pinned SHA | Language/runtime classification | Findings / limits |
|---|---|---|---|
| `src/main.rs::main` | Dispatches native Windows interactive app and headless Rust tests/benchmarks | Rust executable | No embedded Flash VM or script runtime visible in entry point |
| `src/game.rs::Game::{tick,skip_intro,begin_level1a}` | Game states, fixed 25Hz story animation, transition, gameplay and audio events | Rust implementation | `AppScreen::Intro` holds at frame179, keyboard/mouse SKIP starts source 41-frame fade; prior benchmark expects obsolete auto-jump |
| `src/render.rs` + `src/surface.rs` | CPU sprite composition, legacy 600×400 and HQ 1800×1200, source placement and fade overlay | Rust implementation | CPU performance requires measured benchmark; source art remains baked data |
| `src/assets.rs::Assets::load`, `LazySpriteSet` | `include_bytes!` compiled assets, PNG decoder, 2-entry `Rc`/`RefCell` LRU | Rust implementation plus Rust crate | Runtime lazy decoding/caching visible; allocation/throughput impact not yet profiled |
| `src/collision.rs::CollisionMask` | Embedded PNG ground collision mask and coordinate tests | Rust implementation plus PNG crate | Source geometry encoded as generated asset, not Flash runtime |
| `src/win32.rs::wndproc` | Window, focus, keyboard polling, native mouse-down/up release-inside, presentation | Rust with Windows user32/gdi32/kernel32 **platform FFI** | Significant `unsafe`, raw pointers and `static mut APP`; full soundness audit NOT done |
| `src/audio.rs::Audio` | Rust event controller; writes compiled MP3 bytes to temp files; MCI playback via `winmm` | Rust control using **platform media API** | WinMM/MCI decodes audio; acceptable *platform qualification*, not a custom Rust MP3 decoder; actual speaker timing unverified |
| `src/headless.rs` | Rust smoke snapshots, 41-frame transition tests, render and scrolling benchmarks | Rust tests | One benchmark contradicts current state machine; see blocker below |
| `reverse-engineering/tools/*.py` and Windows workflow | Offline SWF parsing/asset baking (Pillow), source hash check, sprite packing | **Non-Rust build-time tooling only** | No evidence of Python imported/invoked from playable `src/` runtime; full packaged binary import scan still outstanding |

**Runtime purity assessment:** The inspected gameplay, UI, animation orchestration, collision, and renderer are Rust. Platform Win32/GDI and WinMM/MCI APIs do not constitute a disguised non-Rust game engine. Original SWF frames are **data embedded in compiled Rust**, not dynamically executed ActionScript. No C/C++ engine wrapper, JS gameplay, Electron, Python runtime, Ruffle or Flash loader was found in inspected `Cargo.toml`/modules. **Do not claim complete dependency-chain purity without Cargo feature/tree and built-binary inspection.**

### Dependency and ecosystem assessment

- **Direct Cargo dependency:** `Cargo.toml` declares `png = "0.17.16"` (Cargo-compatible semver requirement, not necessarily exact resolved version); Rust edition **2021**, package version `0.2.0`, release `lto="thin"`, `codegen-units=1`, `strip=true`, `panic="abort"`. No tracked `Cargo.lock` was returned at this head (GitHub contents 404); exact resolved graph **UNKNOWN**. No newly added dependency in this docs batch.
- **Crate:** `png`, used for image decoding in `assets.rs`/`collision.rs` and PNG encoding in headless tests. Maintainers: image-rs. Version documentation reviewed: https://docs.rs/crate/png/0.17.16 and https://docs.rs/png/0.17.16/png/ (accessed 2026-11-09, official docs.rs for 0.17.16, release 2024-12-20). Page describes pure-Rust PNG decoder/encoder, no unsafe in crate; dependencies include `flate2`, `miniz_oxide`, `fdeflate`, `crc32fast`, `bitflags`. Licence: Apache-2.0 OR MIT, per crate page. **LIMIT:** transitive `flate2` features/backend, actual resolved versions, Windows target imports and dependencies were not inspected. Do not mark each transitive crate pure Rust yet.
- **Windows FFI:** `#[link(name="user32")]`, `gdi32`, `kernel32` from `win32.rs`; `#[link(name="winmm")]` from `audio.rs`. Official specification *The Rust Reference — External blocks*, https://doc.rust-lang.org/reference/items/external-blocks.html (Rust Project; accessed 2026-11-09, publication date not shown) clarifies `extern "system"` and unsafe requirements; it does NOT audit our signatures.
- **Dependency-graph method:** *The Cargo Book — cargo-tree(1)*, https://doc.rust-lang.org/cargo/commands/cargo-tree.html (Rust Project, accessed 2026-11-09; date not shown) documents `cargo tree -e features`. This was read as method guidance, **not run** on the repository. Recommended exact follow-up on properly provisioned Windows worktree: `cargo metadata --format-version 1 > cargo-metadata.json`, `cargo tree -e features`, `cargo tree --target x86_64-pc-windows-msvc`; inspect crate sources/features and FFI before final dependency claim.
- **No migration endorsed:** Existing minimal Rust/Win32 renderer avoids unapproved engine complexity; a larger Rust game framework is not justified by this documentation batch. No current community consensus, issue or Reddit opinion was investigated; do not invent such claims.

### Focused safety, correctness and performance findings

| ID / severity | Evidence (pinned source / CI) | Assessment / proposed remediation | Blocks what? |
|---|---|---|---|
| AUD-BLOCK-001 / **HIGH** | `src/headless.rs::benchmark_render` sends Enter from title, executes 180 intro ticks and asserts `AppScreen::Playing`; `src/game.rs::tick` holds the final intro sample pending explicit SKIP. CI [#42589238951](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42589238951) and [#42591045102](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42591045102) fail at release `--benchmark-render 60`, OS exit `-1073740791` | **Direct logical contradiction HIGH confidence; crash mechanism MEDIUM.** Update benchmark to call SKIP and advance source fade (41 frames) before asserting Playing; preserve benchmark measurement and run full Windows CI. `panic="abort"` may explain process termination; do not label OS code a proven memory corruption cause. | **Full CI / release readiness blocked** |
| AUD-RISK-002 / **MEDIUM** | `src/win32.rs` FFI, mutable global `APP`, casts, event callbacks, Win32 `unsafe` | Manually review pointer lifetime, message ordering, destroy path, focus invalidation, struct layout and ABI; mark unknown until tested; consider safer Rust encapsulation only if justified | Rust safety signoff, not automatic purity fail |
| AUD-RISK-003 / **MEDIUM** | `src/audio.rs::Audio::new` writes four temporary MP3 files and passes paths through `mciSendStringW`; `Drop` removes temp dir | System API permitted, but review failure reporting, quoting and cleanup for unusual Windows paths; speaker timing/UI interaction needs manual playtest | Audio fidelity verification |
| AUD-RISK-004 / **MEDIUM** | `Cargo.toml` no tracked lockfile at current head, version constraint `png="0.17.16"`; only direct dependency checked | Obtain resolved `cargo tree -e features`, document exact transitive versions/native backends, consider lockfile strategy separately | Complete transitive purity and reproducibility certification |
| AUD-RISK-005 / **MEDIUM** | `src/assets.rs::LazySpriteSet::frame` decodes cached PNGs lazily, cache capacity 2; `src/render.rs` composites 3× mode | Potential workload impacts require measured benchmarks, not speculative performance claims | Performance acceptance |
| AUD-RISK-006 / **HIGH for fidelity** | User visual report and draft [issue #17](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/issues/17); full CI failure at latest SHA | Compare on user's Windows machine and original Flash: HUD type, story, mouse release, audio, timing, frames, Level1A playthrough. Never equate unit test success with source fidelity | Manual fidelity gate, merge, Level1B |

**Alternatives evaluated (scope-specific):** For source fade, (A) retain existing Rust fixed-step transition using baked original frame data, versus (B) embed Flash/Ruffle or external interpreter. B is disallowed by project requirement and introduces runtime dependency; A best preserves explicit Rust control and original extracted frame content, subject to visual fidelity check. For benchmark fix, eliminate assertion vs make benchmark follow new state: retain the assertion and fix setup; removing it would hide regression. A broad crate/engine migration is not justified without measured failures.

**What was actually verified now:**
- Remote code, `Cargo.toml`, FFI points, renderer and benchmark source inspected against pinned `c2b872e`; both GitHub workflow statuses and failure step independently checked; both failure-job logs inspected. **CI-003** confirms release compilation, **9 unit tests passed**, but subsequent benchmark failed, so full CI **FAILED**.
- Docs.rs PNG 0.17.16 and the official Rust/Cargo references above consulted. No dependency solver, local target build, `cargo fmt --all -- --check`, `cargo clippy`, `cargo tree`, `cargo audit`, source code security scanner, binary disassembler, benchmark success, original-player video capture or manual Windows smoke executed in this documentation batch.
- This baseline's classifications are limited to **inspected source composition**, NOT final game quality.

### Required independent determinations for AUD-20261109-001

- **PURE-RUST COMPLIANCE: PASS WITH DOCUMENTED PLATFORM/BUILD QUALIFICATIONS** (**MEDIUM confidence**). All inspected game-specific runtime control and rendering are Rust. Windows APIs implement platform presentation/audio; Python only runs during offline asset generation. Transitive `png` features and packaged binary imports remain unverified.
- **ENGINEERING QUALITY: NEEDS IMPROVEMENT** (**HIGH confidence for blocking test contradiction**). Code has bounded state and tests, but the benchmark has a demonstrated invalid assumption and the latest **two complete runs failed**. The dangerous `unsafe` surface also needs focused review.
- **ORIGINAL-GAME FIDELITY: PARTIALLY VERIFIED** (**MEDIUM confidence in recovered source fields, LOW confidence in full experience**). Source identifiers/AVM1 route and earlier baked-frame visual contract were checked in older CI, but no user approval, real audio playback or full Level1A source-by-source comparison; current build is not full green.

**Disposition:** This is a DOCUMENTATION-ONLY bootstrap. No remediation patch applied. Highest-priority next code-changing batch: `AUD-BLOCK-001` benchmark setup mismatch; write `RE-...` and post-change `AUD-...` when that batch is actually performed. Preserve draft PR #18; do not merge or expand scope.

## Mandatory audit entry template — copy and complete AFTER implementation

`## AUD-YYYYMMDD-NNN — [task and referenced RE entry]`
1. Date; development branch; parent SHA; implemented SHA; exact files/functions and pinned diff link; source reference `RE-...`.
2. Actual code inspection: state/input/render/audio/collision changes; dependency, feature, build.rs, transitive and FFI diff; known pure Rust vs Windows API vs native wrapper vs offline tools vs unknown.
3. Correctness and invariants; Rust ownership/borrowing/lifetimes/allocations; unsafe justification, resource cleanup; architecture/modularity; frame-time/performance (measured vs hypothesized).
4. Dependency/alternative comparison where nontrivial: exact crate/version, source URLs, maintainer, license, features, FFI, activity/limitations, alternative and adoption rationale; no fabricated community consensus.
5. Actual commands run and outputs (fmt/check/test/clippy/build/tree/benchmark/CI) plus unrun tools and reasons; original-game evidence and user-playtest status **separately**.
6. Findings by severity/file/evidence/remediation/blocking status; classifications **PURE-RUST COMPLIANCE**, **ENGINEERING QUALITY**, **ORIGINAL-GAME FIDELITY**, confidence and reasons.
7. Next one bounded task; record superseded audit findings as dated corrections, not silent rewrites.
8. Explicit for documentation/research-only batches: **“No implementation changes — Rust code audit not applicable”**; may record relevant baseline facts with truthful limits.
