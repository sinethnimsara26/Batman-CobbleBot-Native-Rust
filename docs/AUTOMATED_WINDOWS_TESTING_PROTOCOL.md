# Mandatory free automated Windows executable and fidelity QA protocol
**Project:** Native Rust implementation of *The Batman: The CobbleBot Caper*  
**Authority:** User-directed persistent testing protocol, 2026-11-09; governed by [../AGENTS.md](../AGENTS.md) and [DEVELOPMENT_WORKFLOW_PROTOCOL.md](DEVELOPMENT_WORKFLOW_PROTOCOL.md).  
**Status on creation:** **INSTRUCTION AND IMPLEMENTATION PLAN ONLY**. Adding this file does NOT mean any new automated test has been implemented or executed. Never pretend it does.

## 1. Objective and restrictions
The user should not be required to download the new Windows EXE, repeatedly play it, manually screenshot errors and report every basic regression. The project's free GitHub-based development cycle should, as far as feasible, build the real executable, run deterministic scenarios, collect real visual/behavior evidence and let an AI inspect failures before asking for human testing.

Constraints:
- Use free tooling, preferentially existing public-repository GitHub Actions Windows runners. No subscription, new paid cloud service, or secrets without explicit authorization.
- Preserve a truly native standalone **Rust gameplay/runtime**. Development-time CI Python, image analysis, FFmpeg or OS testing utilities may be used solely in build/QA; disclose them and do not add them to shipped runtime requirements.
- Do not replace the game with Ruffle, Flash runtime, JavaScript, Electron or an emulator. Original SWF is an analysis and source-fidelity reference only.
- Reuse existing harness and artifacts; never throw away established deterministic tests, original bitmap hashes, source frame manifests or the old accepted baseline.
- **Automated CI passing does not establish pixel-perfect source-original fidelity, physical speaker behavior, actual mouse feel or release acceptance**. Keep PR #18 draft and issue #17 as manual source-equivalence gate until evidence and user approval justify a change.

## 2. What already exists — inspect live branch before extending
At the time this protocol was drafted, development branch **revive/level1a-intro-hud-fidelity-2026-11-09** and draft **PR #18** had:
- **src/main.rs** dispatching executable CLI flags **--render-smoke**, **--render-smoke-hq**, **--benchmark-render**, and **--benchmark-scenery-scroll**.
- **src/headless.rs** using real native Rust Game/Assets/CollisionMask/RenderSurface/render modules to tick scripted inputs and output actual PNG images, including intro frames, early skip behavior, gameplay fade, walk/run, jump/glide/attacks, world objects, pickup and level exit.
- **.github/workflows/build-level1a-fidelity.yml** on **windows-latest**, source-SWF integrity verification, original-asset baking, Cargo tests and Windows release build, actual EXE execution in smoke modes, checks expecting **20 1x PNGs and 20 HQ PNGs**, regression/hash checks, benchmarks, and multiple uploaded screenshot and evidence artifacts.
- Existing artifacts such as **Batman-CobbleBot-Level1A-Visual-Smoke**, **Batman-CobbleBot-Level1A-HQ3x-Visual-Smoke**, contact sheets, source reports and renderer diagnostics.

These are verified **repository implementation observations**, not a claim that the present latest CI succeeded. Fetch the actual branch and workflow before citing a success. The headless mode renders actual engine frames but not necessarily a real Win32 desktop window and physical OS input path.

## 3. Required TWO levels of automated QA
**Level A — deterministic executable/in-process QA (first priority).** Execute the compiled native Windows Rust EXE in its CLI smoke/testing modes on existing GitHub-hosted Windows runners. Script gameplay inputs and state transitions; capture every critical checkpoint and short contiguous frame sequences around animations. This can be reliable without a GUI desktop. Test real engine Game/render code and assert state/time transitions, explicit event order and image artifacts.

**Level B — real Win32 window/system-input QA (experimental, separate).** Use a free Windows GUI automation pilot when the hosted runner offers a suitable interactive session. Launch the actual EXE normally, send keyboard/mouse events, capture the app window through genuine OS screen/window capture, test DPI/window scaling, log exit and error state. Candidate tooling includes Microsoft WinApp CLI, Win32 input/screenshot APIs, and limited external Windows UI automation tools. First PROVE whether desktop is unlocked and capture works; if it does not, mark ENVIRONMENT / INFRASTRUCTURE LIMITATION. Do not treat black captures or rejected synthetic inputs as game defects without independent evidence. A Win32 app may expose no semantic accessibility tree; coordinate/input testing may be necessary.

The two levels are complementary: Level A is mandatory, stable and cheap; Level B is opportunistic and must not block all ordinary development until feasibility is proven.

## 4. Prioritized scenario matrix
1. **Boot/title:** native EXE starts; title/instructions are visible; no crash, blank frame or missing resource.
2. **Natural intro:** original-source-derived intro timeline is entered; actual 180 samples and frame progression are correct for the verified source; correct audio start/stop and script-controlled pause/hold behavior; detect unintended black frame streaks and repeated flicker. Do NOT arbitrarily assume the last nearly black frame is erroneous: check original Flash behavior.
3. **Intro skip keyboard:** test Enter, Space and Escape on relevant states, event ordering and fade frame count. Distinguish input down/up and state-specific effects.
4. **Intro skip mouse:** Win32 release-inside hit region, letterbox coordinate conversion, DPI scaling, click outside/inside and idempotency; headless simulated clicks are not proof of OS event routing.
5. **HUD:** score, batarangs and lives at multiple values and transitions. Test glyph bounding boxes, numbers/icons not overlapping, alignment, font appearance and color under 1x/HQ/DPI scenarios. A candidate styled HUD is NOT automatically original-source faithful.
6. **Gameplay controls:** walk acceleration/run, jump/air time/glide, punch/hold gate, kick, Batarang release and position, state rendering.
7. **World:** player/camera scrolling, parallax/depth, collision geometry invisibility, pickups, pits and spawn, level exit fade. Keep source-asset depth and collision geometry separate.
8. **Audio:** deterministic event sequence and resource load; distinguish this from physical playback and speaker audibility, which still needs a valid loopback or human test.
9. **Failure paths:** missing/invalid assets, bad arguments, wrong resolution/mode, unhandled panic, nonzero exit; fail with actionable logs.
10. **Performance:** benchmark realistic gameplay and intro frames, warm caches as appropriate, report mean/p50/p95/max and bounded CI environment noise; compare normalized trends rather than claiming stable user GPU performance from VMs.

Start with a single valuable missing assertion. Do NOT attempt this entire matrix in one ChatGPT turn.

## 5. Capture format and fidelity evidence
- Keep exact game/build SHA, test-scenario ID, input-event schedule, tick/time and capture-mode metadata with every frame.
- Save numbered PNGs of critical frames and short consecutive PNG sequences around intro/fade/animation transitions. If practical, generate an MP4/GIF/contact sheet as a *test artifact* with sufficient temporal sampling.
- Prefer output from the actual Rust framebuffer for Level A, avoid dependence on a desktop that may be unavailable, and make tests deterministic by stepping the game simulation at known ticks.
- Preserve original-game frame references separately from current Rust output; label **ORIGINAL SWF/verified footage**, **LEGACY RUST**, **CURRENT RUST** distinctly. A legacy Rust golden is regression evidence only.
- For visual comparisons include source vs reconstructed image, aligned overlay, diff image, numeric discrepancy and potentially relevant bounding rectangles. Record crop/scale/normalization, dimensions and frame identifiers. Don't use a naive exact pixel diff as sole acceptance when font rasterization/antialiasing differs: use tolerances and human review with documented exceptions.
- For temporal checks compare consecutive frames and expected animation structure. Detect black/nonblack oscillation, unchanging frames in states expected to advance, unexpected dropouts, repeated sequences, large discontinuities, missing frames and timing errors. Explicitly allow known original fade-to-black and intentional still frames.
- For HUD inspect glyph/icon bounding geometry if available; image-difference alone won't reliably diagnose overlapping text. Store evidence for any automatically flagged collision.
- Capture failures reliably: record nonzero exit, process timeout, crash exception/code, stderr/stdout if available, Windows Event Log where practical, and complete test step metadata. Ensure CI uploads evidence **even on failed checks** with appropriate artifact-upload conditions, while avoiding publishing secrets or huge original copyrighted assets.

## 6. Tests must actually execute the EXE
Automated testing should verify the built Windows artifact, not merely pass source inspection. The current CI executable modes are illustrative:
- Windows smoke: native executable with **--render-smoke** and destination directory.
- Windows HQ smoke: native executable with **--render-smoke-hq** and destination directory.
- Render benchmark: **--benchmark-render** and iterations/output JSON.
- Cargo tests and checks should run against the correct workspace/target and generated assets, using actual CI commands, not guessed local flags.
- Preserve existing 20 screenshot outputs per mode until a scenario expansion intentionally adjusts both the test harness and count validation. Do not rewrite golden checks just to make a failure green.
- Never claim "launched real GUI" when only headless CLI mode ran. Never claim "visual inspected" if only file presence/count/hash was checked.
- Validate the actual changed asset input provenance: the repository workflow downloads the original SWF and checks canonical hashes; don't bypass this step.

## 7. Free hosted Windows runner limitations and pilot
GitHub-hosted **windows-latest** can execute Windows programs but is not automatically an unlocked full interactive human desktop. Start a narrow, disposable feasibility pilot:
1. Establish runner OS/session, user, display/resolution/DPI and whether Win32 desktop session can interact with an ordinary window.
2. Launch the actual game window with a bounded timeout and process cleanup.
3. Capture a single trustworthy PNG of the titled game window and verify its pixel content plus process exit state.
4. Attempt one deterministic title screen keyboard or mouse interaction through OS APIs.
5. Capture BEFORE/AFTER screenshots and logs. Classify outcome:
   - GUI_AUTOMATION_VERIFIED: real app window + input + valid screenshot observed.
   - PARTIAL: window/capture supported, input not proven, or vice versa.
   - INFRASTRUCTURE_UNAVAILABLE: session security/runner limitations.
   - GAME_FAILURE: independent evidence points to actual application failure.
   - UNKNOWN: insufficient evidence.
6. Report source URL, tested tool version and exact run/artifact IDs. If unsuitable, keep Level A and investigate a different free runner/service in a later bounded batch; do not buy a VM or install a persistent remote-control service automatically.

Candidate alternatives (research their current terms/availability before choosing) include open-source app automation on a disposable Windows runner and open-source-oriented Windows CI hosts. No blanket claim that any of them provides a persistent unlocked desktop.

## 8. AI visual review and continuous feedback
After CI completes, the agent should (where its tools and context allow) retrieve the actual current-run evidence artifacts. Inspect the relevant PNG/contact sheet/short clips, compare with a grounded original-game reference, and identify real differences and uncertainty. If binary artifacts cannot be retrieved or visualized, record "NOT INSPECTED" rather than claiming success.
- For each discovered issue, create/synchronize a bounded bug in **AI_TASKS.md**, provide screenshot/tick/run evidence and correct scope and severity; record investigation in the research ledger and changed Rust audit as relevant.
- Review the FIRST priority fidelity discrepancy rather than launching unrelated development.
- A passing technical smoke test may still contain an obvious missing intro/button/font; visual review must remain independent from automated hash/count gates.
- Do not blindly revise the expected screenshots to fit output. Changes to golden references require a defensible source/fidelity rationale and review.
- Keep full screenshot/diagnostic artifacts on GitHub Actions; do not routinely commit large binary frame archives to Git.
- GitHub Actions artifacts expire; add stable pointers, hashes, summary evidence and reproducible regeneration commands to handoff/research docs.

## 9. CI safety, runner efficiency and branch protection
- Favor extending **.github/workflows/build-level1a-fidelity.yml** with small test stages when justified. Preserve currently green jobs and existing windows-latest asset rebuild.
- CAUTION: This workflow uses branch-level concurrency with **cancel-in-progress: true**. An unrelated documentation push to the same active branch can cancel an existing CI run and start a new run. Coordinate commits, avoid gratuitous pushes, and NEVER misinterpret a cancelled run as a game regression.
- Keep bounded duration, process cleanup, artifact retention, and minimal storage/bandwidth. Use public-repo standard runner capacity responsibly.
- Avoid adding new CI triggers, expensive matrices, or long recordings by default. Keep diagnostic recording focused on failure-prone animations.
- If auto-generated assets are needed, produce them in CI as already established; do not require user installation of Flash/Python just to play the game.
- Treat untrusted game assets/CI pull request content with proper runner isolation. Never expose GitHub tokens, user credentials or a permanent remote desktop endpoint.
- A manual player acceptance step remains necessary before merging to stable branch.

## 10. Mandatory integration with the SIX documents
For each bounded code-changing QA development batch:
1. Review **AGENTS.md** and this protocol.
2. Select one QA improvement or failure.
3. Record original-vs-native evidence and relevant sources in **REVERSE_ENGINEERING_RESEARCH_LEDGER.md**.
4. Implement and actually run what is feasible, then independently inspect Rust/native dependencies in **RUST_PURITY_AND_ENGINEERING_AUDIT.md**.
5. Update tasks, worklog, current handoff, and AGENTS.md if permanent procedures change.
6. Check all six documents, commit safely, verify CI/remote status, report and STOP.

For a no-code feasibility study, record the actual findings, exact URLs and limitations; mark code audit N/A. The first implementation action should be **one scoped improvement to existing headless Windows CI evidence or a small separate real-window feasibility pilot**, whichever the active source-fidelity blocker warrants. Do not silently re-prioritize away from intro/HUD manual gate.

## 11. Evidence-rich batch report template
- Batch ID and specific test scenario: ...
- Existing baseline/run SHA: ...
- Changed paths and purpose: ...
- Test mode: native executable CLI / Win32 GUI / simulation only / static review: ...
- Exact commands executed: ...
- Tests actually passed/failed/blocked: ...
- Visual frames personally inspected by AI: ...
- Original source reference used: ...
- Report and artifact URLs, expiry and hashes: ...
- Rust compliance and engineering verdict: ...
- Original-game fidelity status: ...
- Infrastructure/environment limitations: ...
- Six-document review status: ...
- Next single bounded task: ...
- Git commit (if successfully persisted): ...

**Never describe this protocol's existence as proof that an automation pipeline is implemented. Its future work must be built and verified in separate small batches.**
