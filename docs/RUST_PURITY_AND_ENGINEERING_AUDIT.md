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


## AUD-20261109-002 — CI-BENCH-01 post-implementation Rust audit

**Audit date:** 2026-11-09. **Linked evidence:** [RE-20261109-004](REVERSE_ENGINEERING_RESEARCH_LEDGER.md#re-20261109-004--ci-bench-01-rebuild-the-render-benchmarks-source-correct-entry-path); preceding baseline [AUD-20261109-001](#aud-20261109-001--baseline-rust-runtime-and-dependencies). **Base code:** `4ee103a6340bfea26d21e7857b9e80a9147e664a`. **Actual implementation inspected:** [`6e36914d2ab5c6025c40c72acca6789e3479d1e8`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/6e36914d2ab5c6025c40c72acca6789e3479d1e8), one-file diff: `src/headless.rs::benchmark_render` (+16/−3 lines); `src/game.rs` and all runtime modules unchanged.

### Independent changed-code inspection after implementation

**Behavior:** The benchmark first uses the existing title Enter transition into `AppScreen::Intro` and advances 180 ticks, intentionally stopping at original sampled intro frame179. It now asserts `Intro`, `intro_tick=179`, `intro_fadeout_tick=None`. It invokes existing `Game::skip_intro()` **once**, asserts fade frame0, advances to fade frame40, checks it is still in `Intro`, then ticks once more and asserts `Playing`, `intro_fadeout_tick=None`, `ticks=0`. It then preserves the original gameplay tick advance to105 and exact unchanged HQ rendering measurement/report loop. These are actual source assertions, not claimed user playback.

**Correctness and corner cases:** Existing Rust `Game::skip_intro` rejects non-Intro/double-skip; these are already tested by `render_smoke_mode`. Intro frames are zero-based; the new code requires all 41 fade frame samples and ensures no off-by-one jump to `Playing`. Fixed `InputState` release after title Enter prevents an accidental repeated Enter press during story. Time spent in the intro setup falls **outside** the benchmark's timed rendering samples, so it does not change the meaning of collected per-frame HQ metrics. It does **not** bypass the assertion by directly setting `game.screen=Playing`.

**Pure Rust / dependencies:** The *only code change* is safe Rust in `src/headless.rs`. It adds no `unsafe`, FFI, C/C++, Python, JS, ActionScript execution, external game engine, crate, feature flag, build.rs, code generation command, asset data or API binding. It uses existing `Game`, `InputState`, `CollisionMask` and built-in `assert!`/`assert_eq!` constructs. `Cargo.toml` still directly declares `png="0.17.16"` and `panic="abort"` in release, with no manifest/lockfile changes in this diff. The existing game keeps Rust state/render and documented Win32/GDI/MCI platform FFI plus offline Python SWF extraction; **full transitive dependency/binary composition is not re-audited**. No new crate suitability study is warranted when no crate is changed.

**Idiomatic Rust and maintainability:** Explicit `Game::skip_intro()` call rather than reaching into private states; compile-time enum comparisons and `Option<u16>` checks; deterministic bounded loops with no new allocation or cloning; no mutation to game physics; readable comment referencing original source stop/fade, with an expected explicit assertion for every transition. The function predates this patch and uses imperative setup; a shared helper with smoke tests could reduce duplication, but extracting one now would widen this bounded fix and make the failing benchmark harder to isolate. **Alternative considered:** set `game.screen=Playing` directly to continue benchmarking, rejected because that hides the original routing; alternatively modify `src/game.rs` to reinstate auto-transition, rejected as source fidelity regression. Keeping the benchmark's assertion is deliberate.

**Safety and performance:** No new raw pointer, `unsafe`, thread or resource handling, panic-prone indexing or crate dependency introduced. New `assert!` calls purposely abort/flag a test regression; they are on a dedicated `--benchmark-render` diagnostic CLI path, not inside actual Windows rendering loops. The benchmark still warms caches and times 60 HQ render calls after gameplay reaches its unchanged setup. **Performance improvement is NOT claimed**; retain the actual measured metrics and old baseline for comparing only after tests pass. Existing Win32 `static mut`/FFI and MCI risks from AUD-RISK-002/003 remain, but are **not changed**.

### Actual verification and limits

| Check | Evidence | Result |
|---|---|---|
| Remote code/diff | Exact GitHub commit `6e36914d...` inspected via commit API and fetched `src/headless.rs` | **PASS:** only `src/headless.rs` changed (+16/−3), source asserts present |
| Model/trace of source state transitions | Fetched `src/game.rs::Game::tick/skip_intro` and compared exact branch/index progression | **PASS at code-inspection level**, not independently run executable |
| Previous failures | Windows [#42589238951](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42589238951) and [#42591045102](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42591045102) | Both **FAIL**, stage22 benchmark; 9/9 prior unit tests passed |
| Current full Windows workflow | [#42595410918](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42595410918) | **COMPLETED / FAILURE overall; target fixed.** Windows steps20 (9 Rust tests),21 (release build),22 (legacy/HQ smokes + `--benchmark-render 60` and scenery benchmark programs) PASSED; step24 FAILED on separate scenery-scroll p95 check: 42.926ms ≥40ms. Gameplay HQ benchmark p95 36.4716ms PASSED. |
| Local fmt/check/test/clippy/tree/metadata | No generated Windows asset tree in this environment; modified test needs `include_bytes!` original asset packs to compile | **NOT RUN LOCALLY**; Rust compiler, Clippy and Cargo graph not independently executed here |
| Manual original Flash vs native Level1A | User Windows playback / issue #17 | **NOT RUN**, fidelity gate remains open |

**Applicable official sources consulted 2026-11-09:** *assert_eq in std*, Rust Project (std 1.99.1, code date 2026-10-13), https://doc.rust-lang.org/std/macro.assert_eq.html, confirms release `assert_eq!` is always active; *Profiles — The Cargo Book*, Rust Project, https://doc.rust-lang.org/cargo/reference/profiles.html, `panic` section, confirms release `panic="abort"` terminates the process; *Fail Fast Exception C0000409*, Microsoft Learn (2018-01-29), https://learn.microsoft.com/en-us/shows/inside/c0000409, warns that subcode not recorded here is needed to distinguish crash scenario. All are technical guidance, not community consensus or live project test output. The `png` crate and release profile already existed; no new libraries. Cargo/Win32/platform compliance notes inherited from pinned AUD-20261109-001.

### Determinations (post-code review; test outcomes must remain separate)

1. **PURE-RUST COMPLIANCE: PASS WITH DOCUMENTED PLATFORM/BUILD QUALIFICATIONS — HIGH confidence for this diff, MEDIUM for broader runtime.** The actual patch is strictly safe Rust and invokes established Rust gameplay; external platform APIs exist elsewhere and offline Python only prepares embedded data. Transitive `png` feature/native composition not reverified.
2. **ENGINEERING QUALITY: NEEDS IMPROVEMENT — HIGH confidence.** The targeted benchmark state-machine correction works in Windows CI (step22 success), but full workflow still **FAILS** step24 on the **independent HQ scenery-scroll p95** budget. Do not call the game release-ready. This batch did not modify rendering logic; prioritize separate measured performance investigation. The implementation itself has no new code-quality/unsafe dependency concerns.
3. **ORIGINAL-GAME FIDELITY: PARTIALLY VERIFIED — MEDIUM source-evidence confidence, LOW full-user-experience confidence.** Original action-based story exit is preserved by current native code; headless test-setup now follows it. The patch itself does not improve sprite/art, typography, audio capture or verified on-device gameplay.

### CI-closeout and new blocking finding

Actual [Run #42595410918](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42595410918), Windows job `129086770688`:
- `cargo test --release --target x86_64-pc-windows-msvc`: **PASS, 9 tests** (step20).
- `cargo build --release --target x86_64-pc-windows-msvc`: **PASS** (step21).
- Both `--render-smoke` and `--render-smoke-hq` child processes and the formerly aborting `--benchmark-render 60 .generated/render-benchmark.json`: **PASS** (step22), as did `--benchmark-scenery-scroll` executable invocation.
- Original gate for HQ render: p95 **36.4716ms**, under **40ms** (**PASS**). Existing 3× frame dimensions/reports were checked.
- Existing scenery-scroll gate: **41 samples**, p95 **42.9260ms**, p50 **16.3843ms**, mean **19.6801ms**, max **46.1800ms**; `assert scroll["p95_ms"] < 40.0` **FAIL**, step24. Specific high readings: x=0 **46.0933ms**, x=600 **46.1800ms**, x=1200 **42.9260ms**, x=1800 **42.5766ms**. The final CI verdict is **FAIL**, not a complete green build. Capturing the full Windows exception for the old abort is no longer a release prerequisite unless the abort returns.

**AUD-BLOCK-001 / HIGH (prior benchmark abort): RESOLVED for tested code SHA** (Windows stage22 passes on `6e36914d...`); do not erase history of the previous failed runs. **AUD-BLOCK-007 / HIGH (new performance gate): OPEN**, affected `src/headless.rs::benchmark_scenery_scroll`, `src/render.rs`, `.github/workflows/build-level1a-fidelity.yml`, observed p95 42.926ms vs <40ms CI bound. **Cause UNKNOWN:** could be complex-scene CPU compositing/cache cost or runner contention. **Remediation:** next **one** bounded task `CI-SCROLL-P95-01`, reproduce and profile offending camera windows; optimize in Rust only if supported by measurements, leaving gate intact. **Blocks:** full CI/release/merge; manual visual acceptance independently outstanding. **AUD-RISK-002..006** unchanged.

**Audit closed for this bounded code-changing batch:** post-implementation Rust source and diff inspected; compiled Windows run validates the corrected benchmark, though a separate performance failure prevents green full CI. No Python runtime, JS engine, new native dependency or non-Rust gameplay introduced. Next code batch must append a new RE and new post-code AUD entry after actual tests, per AGENTS.md.

---


## AUD-20261109-003 — CI-SCROLL-P95-01 post-implementation Rust audit

**Date:** 2026-11-09. **Review scope:** actual optimized `src/render.rs` diff, not a proposal. **Development branch:** `revive/level1a-intro-hud-fidelity-2026-11-09`, draft PR #18. **Base commit:** `6c6c2cb84731163c9bb147c0d9b82aec56ca079a`; **implementation:** [`acccd0b70327e2ea1911885f97e21f5e5a5398b3`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/acccd0b70327e2ea1911885f97e21f5e5a5398b3). **Linked independent research:** [RE-20261109-005](REVERSE_ENGINEERING_RESEARCH_LEDGER.md#re-20261109-005--ci-scroll-p95-01-repeated-windows-measurements-and-pixel-exact-hq-tile-clipping). **Changed code:** `src/render.rs::draw_tiles_hq` calls new `blit_tile_hq_1to1` (instead of generic `blit`), with 2 new unit tests under `render::tests`. No `Cargo.toml`, `Cargo.lock`, Win32, audio, collision, Python source, generated asset, workflow or game-state changes.

### Independent post-code change inspection

1. **Original call and changed path.** HQ scenery detail pack `Assets::level1a_detail_tiles_hq` is a `LevelTileSet` constructed with logical scale 3.0; `draw_tiles_hq` positions its source images at integer 3× stage-pixel origins after crop offsets. Previously passed each image to generic `blit(..., image.w, image.h, false)`. The new function takes source-sized images and clips once, then runs one blend per *visible* pixel. It does not replace the legacy 600×400 renderer, non-tile sprites, 3× sky/moon, 1× alpha city, or world object UI composition.
2. **Source parity.** For output size equal to source size and flip=false, old `blit` computes `sx=ox*src.w/src.w=ox`, `sy=oy*src.h/src.h=oy`, whereas the new bounded row intersection maps exactly those x/y to the same source pixel. Relative draw ordering in `draw_tiles_hq` is unchanged. The new function calls the **same** `blend` routine; RGBA alpha=0, fully opaque and partial opacity output math remain unchanged. Arbitrary resampling and mirrored sprites still invoke original `blit` elsewhere.
3. **Memory safety and arithmetic.** The clipped rectangle clamps to destination dimensions, checks empty source/output, then derives `sx`, `sy` and width. Access uses safe `src.pixels[a..b]` and `dst.pixels[a..b]` and `iter().zip(iter_mut())`. No `unsafe`, `get_unchecked`, raw pointers, FFI, shared mutable alias, new thread or cached global. i32 screen positions are widened to i64 for clipping additions, avoiding potential i32 overflow during camera offsets. Source dimensions and buffer consistency remain governed by existing image/asset decoding; pathological impossible >i64 image dimensions are out of realistic input scope, not formally proven. Runtime slices check indexing bounds.
4. **Rust API/design assessment.** The new specialized function is small and separate from the generic blitter; it has a single HQ tile call site, makes no new public API, and uses safe contiguous iteration. `zip` truncates to the shorter iterator, but both clipped ranges are constructed with identical `width`; this is supported by local invariants, not an excuse to skip parity tests. No new allocations per rendered tile. The helper itself is generic enough for cropped/scrolled tile coordinates but not silently used for other layers.
5. **Measured performance distinction.** A prior Windows run on the unoptimized code exceeded `scroll.p95_ms < 40.0`: [#42595410918](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42595410918) p95 42.9260ms and max46.1800ms, step24 failed. A later run with **the same Rust source**, [#42597104708](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42597104708), had p95 27.3516ms (all 57 steps passed). The difference shows test-run variability; it does NOT prove the optimized blitter is faster before a new measured run or isolate the PNG cache as the root cause.

### Correctness tests, expected compatibility gates and actual execution

- **Added test `render::tests::hq_tile_clipped_blit_is_identical_to_general_blit`:** iterates 10 x offsets × 10 y offsets (100 unique clipped/offscreen positions) with patterned 7×5 source/destination, transparent and multiple partial/opaque alpha values; old general `blit` and new specialized path must yield identical complete destination vectors. This is a stronger pixel-level equivalence condition than approximate screenshot review.
- **Added test `render::tests::hq_tile_clipped_blit_keeps_zero_sized_and_offscreen_tiles_noop`:** edge behavior and pixel preservation. Original renderer tests remain.
- **Existing workflow:** Windows `cargo test --release --target x86_64-pc-windows-msvc`, release `cargo build`, two native visual smokes, gameplay/HQ screenshot and hash comparisons, actual `--benchmark-render 60`, `--benchmark-scenery-scroll` 41-camera-sample run, pixel-level asset/oracle assertions and strict **40 ms p95**. These must not be disabled or diluted.
- **Current implementation CI:** [Run #42613807273](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42613807273), pinned to `acccd0b...`. **COMPLETED SUCCESS, all 57 Windows steps.** Job 129148589727 reports 11 passing Rust tests including both new tile tests, release build and existing source-asset/hash and visual smoke gates passing, full native benchmarks successful, 40ms p95 threshold unchanged. Scenery-scroll p95 **25.8716ms**, mean 14.3048ms, max 26.2268ms; ordinary HQ benchmark p95 29.6811ms. No physical Windows user verification was done.
- **Local limits:** No local Rust toolchain (both `rustc` and `cargo` absent in the available container), and the source-derived asset packs aren't mounted. Therefore `cargo fmt --all -- --check`, `cargo clippy`, `cargo tree`, `cargo metadata`, local project build, Windows debugger/perf profiler and actual Flash comparison were **NOT RUN** here. Do not report them as passed. Windows Actions is the execution evidence.

### Dependency suitability and safety

- Direct `Cargo.toml` dependency `png="0.17.16"` is **unchanged**. `LevelTileSet::tile` still uses Rust `Rc<Image>`, `RefCell` and four-entry LRU with PNG decode on cache miss. This batch does NOT upgrade crates, change resolution or claim exact transitive dependency graph proof; earlier `AUD-RISK-004` remains.
- Windows `user32/gdi32/kernel32` and `winmm` platform FFI remain confined to preexisting modules, not changed or used in the new helper. No C/C++ game engine, Ruffle, Flash player, ActionScript interpreter, JavaScript or Python gameplay dependency is introduced. Python and Pillow continue solely as original-asset build-time preprocessors, as permitted by `AGENTS.md`.
- Official reference used for the safe implementation: Rust Project, *Primitive Type slice*, https://doc.rust-lang.org/std/primitive.slice.html (**Iteration**, `iter_mut`), Rust Project *Iterator::zip*, https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.zip, and Rust Project *IterMut*, https://doc.rust-lang.org/std/slice/struct.IterMut.html; consulted Nov 9, 2026. These document safety semantics; they are not performance benchmarks and do not imply guaranteed auto-vectorization. No relevant external crate/community adoption claims needed.

### Alternatives, risks and severity

| Finding | Severity/status | Evidence and disposition |
|---|---|---|
| `AUD-BLOCK-007` scenery-scroll p95 | **HIGH, conditional until current CI measured** | Same code revision alternated 42.9260ms fail and 27.3516ms pass. New clipped one-to-one Rust path targets repeated work. Preserve unchanged 40ms CI threshold, check repeatability and any regression. |
| `AUD-RISK-008` renderer clipping/equivalence | **HIGH fidelity, guarded by new unit tests/CI screenshots** | Pixel-level parity verified logically from source mapping; automated test execution required. If source alpha/order differs, repair before moving on. |
| `AUD-RISK-009` broader performance path | **MEDIUM, not measured** | 3× full-frame alpha promotion and 4-entry cached PNG detail pack can also be expensive. No speculation about exact bottleneck without profiling. |
| Existing unsafe Win32 and WinMM scope | **Previously tracked, unchanged** | No new `unsafe` or platform API. Full transitive dependency and FFI soundness not recertified in this batch. |
| Manual game/source parity | **OPEN** | Physical Windows playthrough, real audio and pixel/original comparison for user issue #17 remain unverified; automated parity doesn't replace human confirmation. |

### Post-implementation verdicts and next action

- **PURE-RUST COMPLIANCE: PASS WITH DOCUMENTED PLATFORM/BUILD QUALIFICATIONS** — **HIGH confidence in changed file**, MEDIUM confidence across all transitive dependencies. Only safe Rust added; original permitted OS FFI and offline Python asset preprocessing remain outside game logic.
- **ENGINEERING QUALITY: PASS WITH NON-BLOCKING CONCERNS** — high confidence in tested code and existing Windows strict checks. 11 tests pass, screenshot/source-asset oracle tests pass, CI 57/57 steps green and scenery p95 25.8716ms (<40ms). Prior same-source variance means measured advantage is not robustly attributed to this change. Unchanged baseline platform/FFI, memory and dependency risks remain.
- **ORIGINAL-GAME FIDELITY: PARTIALLY VERIFIED** — original source data and tile placement unchanged; unit pixel equivalence plus CI screenshot comparisons support the tested 3× detail path, but manual Flash-vs-native verification remains open.

### Completed verification and issue disposition

Actual [Run #42613807273](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42613807273), source SHA `acccd0b70327e2ea1911885f97e21f5e5a5398b3`, **SUCCESS 57/57 workflow steps**. Rust `cargo test --release` on Windows **11/11 tests PASS**, including source-sized pixel-for-pixel comparison across 100 placements and the empty/offscreen tests. Original assets rebuilt, release executable and native smoke runs passed, HQ dimensions 1800×1200 and screenshot/hash contracts passed, normal HQ benchmark p95 **29.6811ms** and scrolling p95 **25.8716ms**, both below the same **40ms** gate. Scenery mean **14.3048ms**, maximum **26.2268ms**.

**AUD-BLOCK-007 (scenery 40ms threshold): CLOSED for this test/commit**, conditional on future repeatability; prior same-Rust run variability proves a single green CI should not establish durable speedup. **AUD-RISK-008 (pixel parity): PASS** under new focused unit tests and existing original-image CI comparisons; source art unchanged. Baseline Win32/WinMM FFI safety, complete transitive dependency audit and manual user Windows fidelity acceptance remain outstanding.

**Verdicts:** PURE-RUST COMPLIANCE **PASS WITH DOCUMENTED PLATFORM/BUILD QUALIFICATIONS**; ENGINEERING QUALITY **PASS WITH NON-BLOCKING CONCERNS**; ORIGINAL-GAME FIDELITY **PARTIALLY VERIFIED**, pending issue #17. Independent post-code review complete. User's new blocker-first rule is committed to `AGENTS.md` as part of this checkpoint. Next: source-vs-native Windows visual/audio gameplay acceptance, and fix any first concrete defect before unrelated work.

---


## AUD-20261109-004 — WIN-01 acceptance documentation only

**Date:** 2026-11-09. **Task:** Update stale source-vs-native Windows manual acceptance issue #17 with the current tested code SHA, GitHub Actions Windows executable, accurate original-Flash transition expectations, and evidence reporting protocol. **Related research:** [RE-20261109-006](REVERSE_ENGINEERING_RESEARCH_LEDGER.md#re-20261109-006--win-01-source-correct-windows-manual-acceptance-candidate). **Documentation-only: No implementation changes — Rust code audit not applicable.**

**Artifact and code basis:** User-run candidate [`acccd0b70327e2ea1911885f97e21f5e5a5398b3`](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/acccd0b70327e2ea1911885f97e21f5e5a5398b3), [Windows #42613807273](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42613807273), successful 57/57 CI checks, 11 passing Rust tests, artifact `Batman-CobbleBot-Level1A-Fidelity-Windows-x64` (ID 13672241525). This artifact is the compiled **native Rust** executable `batman_cobblebot_native.exe` per the actual workflow upload declaration, not a web or Python/Flash runtime. Actual user desktop acceptance was not performed here.

**What was inspected and changed:** Exact issue #17 body originally described prior candidate `16b341a`, obsolete automatic story→Level1A transition, and outdated claims regarding audio implementation. Updated existing *open* issue #17 with a superseding current-candidate section, clarified 180-sample intro hold + explicit 41-frame SKIP fade and release-inside mouse semantics, preserved older historical data for reproducibility, and stated manual visual/audio approval remains an independent blocker. The GitHub issue API returned successful update. All Rust implementation files, crate manifests, dependencies, FFI, assets, CI thresholds, tests, PR and `main` were left unchanged; this task should not trigger a native rebuild.

**Safety, dependency and source fidelity determination:** Since no Rust code or dependencies changed, no new crates, `unsafe`, allocation, concurrency, transitive graph changes, platform wrappers or runtime purity risk were introduced. Prior source-backed system behavior and CI green results remain the baseline, subject to manual testing. The issue instructions require reproducible step, expected-original vs observed-native, build SHA, Windows version/DPI, video/screenshot, and a blocker-first fix/retest process.

**Actual verification:** GitHub issue #17 read and updated; [#42613807273](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42613807273) artifact ID/name and uploader path retrieved from GitHub connector; PR #18 still draft; no executable downloaded locally and no user physical playtest occurred. No local `cargo test`, `cargo fmt`, `cargo tree`, `cargo clippy`, `rustc`, or flash-to-native visual comparison performed for this documentation-only task. Existing source and benchmark results were not silently replaced with new measurements.

**Audit classifications (docs-only task):**
- **PURE-RUST COMPLIANCE:** **UNCHANGED — PASS WITH DOCUMENTED PLATFORM/BUILD QUALIFICATIONS** from prior audited code. Not a new comprehensive Rust audit.
- **ENGINEERING QUALITY:** **UNCHANGED — PASS WITH NON-BLOCKING CONCERNS** at the tested prior code SHA, while physical gameplay remains unapproved.
- **ORIGINAL-GAME FIDELITY:** **PARTIALLY VERIFIED** from source-action and automated regression contracts; actual Windows original-side-by-side remains **UNVERIFIED**. No claim that updating the issue proves fidelity.

**Next exactly one task:** actual Windows Level1A manual test using the pinned successful artifact (or clear user-submitted video); if a concrete deviation is observed, repair it and re-test *before* unrelated game development. Keep issue #17 open, PR #18 draft and `main` unchanged. New code batches require their own complete independent post-code AUD and evidence-led RE entries.


---


## AUD-20261109-005 — WIN-01 provenance and playtest package (documentation only)

**Date:** 2026-11-09. **Scope:** create/review a directly usable Windows game QA ZIP from the already-green native executable and prepare reliable manual acceptance instructions. Parent documentation code head `d4da009697ca600ea9b58140f4a6b51494430f2f`; executable code SHA `acccd0b70327e2ea1911885f97e21f5e5a5398b3`; PR #18 draft. **Related research:** [RE-20261109-007](REVERSE_ENGINEERING_RESEARCH_LEDGER.md#re-20261109-007--win-01-provenance-verified-native-windows-acceptance-zip); persistent QA manifest [WINDOWS_QA_PACKAGE_2026-11-09.md](WINDOWS_QA_PACKAGE_2026-11-09.md).

**No implementation changes — Rust code audit not applicable.** No Rust game source, original SWF asset, crates, Cargo manifest/lockfile, FFI, Win32 state machine, renderer, tests or CI threshold modified or rebuilt as part of this QA-packaging step. No new user-confirmed gameplay defect was identified, so changing unrelated game functionality would conflict with the mandatory blocker-first protocol.

**Independent artifact inspection, not code certification:**
- Fetched exact Windows x64 artifact 13672241525 from successful 57-step [CI #42613807273](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42613807273), containing `batman_cobblebot_native.exe` (34,697,216 bytes; SHA-256 `348098af93ce0eb28924835245f29532d49390cace301880caf1d57f92fef1ed`).
- Independently inspected PE header using `file`: native PE32+ x86-64 GUI binary. `objdump -p` imported only Windows OS/system libraries `GDI32`, `KERNEL32`, `USER32`, `WINMM`, `api-ms-win-core-synch-l1-2-0`, `bcryptprimitives`, `ntdll`. This is supportive evidence that a separate Flash/JavaScript/Python engine is not linked through the static import table, **not** a proof of no delayed dynamic DLL loads or a fully Rust transitive graph.
- Downloaded original CI visual samples and bundled them as **native reference outputs**, clearly not authentic original Flash captures. Created `START_HERE.md`, `REPORT_TEMPLATE.md` and `SHA256SUMS.txt`. Verified six-member ZIP content and no ZIP CRC errors. ZIP SHA-256 `2b2d6276e0b7ca72d24f7f583ff3e962375bce2099d86306778919439ecc4938`, 30,074,539 bytes. The ZIP is an ephemeral conversation-delivered artifact; the official GitHub Actions artifact remains canonical.
- **Actual execution:** no Windows process was launched here, no keyboard/mouse/speaker tests were done and original Flash was not compared. CI remains a code/build oracle, **not** a human acceptance oracle. Windows signing/security reputation was not measured. No Cargo commands run; source code was unchanged and repo source assets require Windows CI to rebuild.

**Pure-Rust compliance:** **UNCHANGED — PASS WITH DOCUMENTED PLATFORM/BUILD QUALIFICATIONS** based on preceding inspected Rust runtime and static PE imports (HIGH confidence for asset identity, MEDIUM for full dependency purity). Windows graphics/audio OS calls and offline Python SWF baking remain documented exceptions outside gameplay source.

**Engineering quality:** **UNCHANGED — PASS WITH NON-BLOCKING CONCERNS** for prior successful 11-test Rust code SHA, strict 40ms CI gate passed. The package improves reproducibility by pinning SHA and providing a complete report template, but does not establish that unsolved Win32 unsafe/FFI, dependency lockfile and real-device performance issues are absent.

**Original-game fidelity:** **PARTIALLY VERIFIED / PHYSICAL ACCEPTANCE UNVERIFIED**. Source actions and native outputs covered in earlier audits and CI, but actual intro flicker, HUD, SKIP release, audio, controls and frame feel require a user physical Windows test.

**Disposition:** WIN-01 playtest material is prepared and independently package-verified; the **remaining blocker is the absence of actual user original-vs-native results**. Do not invent a mismatch, call issue #17 accepted, change unrelated gameplay or begin Level1B. Once user supplies an observed discrepancy, fix and retest it *before moving on*, and author a fresh detailed RE and post-code AUD for that Rust change.

---

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
