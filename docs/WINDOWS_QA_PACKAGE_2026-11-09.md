# WIN-01 — Native Rust Windows QA package and evidence manifest

**Batch date:** 2026-11-09. **Task:** prepare a usable on-PC test bundle from the last successful Rust Windows CI artifact for the still-open manual Level1A acceptance gate. **No game code, Rust dependencies, workflow, original SWF or gameplay source modified in this batch.** This report describes the independent ChatGPT conversation ZIP; it is **not** a binary committed to GitHub and has no guaranteed permanent Git URL. The original CI artifact is the canonical durable retrieval path until GitHub's artifact retention expires.

## Canonical and repackaged binary provenance

| Item | Exact value and verification |
|---|---|
| Repository | https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust |
| Authorized branch | `revive/level1a-intro-hud-fidelity-2026-11-09` |
| Last green playable CODE SHA | `acccd0b70327e2ea1911885f97e21f5e5a5398b3` |
| CI | https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42613807273 |
| CI outcome | 57 successful Windows workflow steps; 11 passing Rust unit tests; source and screenshot checks, strict HQ render budget passed |
| Original executable artifact | `Batman-CobbleBot-Level1A-Fidelity-Windows-x64`, GitHub Actions artifact ID **13672241525** |
| Original GitHub artifact ZIP, retrieved this batch | `github-actions-artifact-13672241525.zip`, SHA-256 `e02c091de15f36624ae1e35246ad64ec6ebe29d55f1001f1e8882b8165175eb6` |
| Exact EXE archive member | `batman_cobblebot_native.exe`, **34,697,216 bytes** |
| EXE SHA-256 | `348098af93ce0eb28924835245f29532d49390cace301880caf1d57f92fef1ed` |
| QA ZIP created for the conversation | `Batman_Cobblebot_Native_Rust_Windows_QA_2026-11-09.zip`, **30,074,539 bytes**, SHA-256 `2b2d6276e0b7ca72d24f7f583ff3e962375bce2099d86306778919439ecc4938` |
| Game-candidate status | **Automated checks PASS; human source-equivalence and physical Windows audio/input tests UNVERIFIED** |

**How package was actually assembled:** Downloaded original binary ZIP via connected GitHub artifact service, extracted the executable **unchanged**, and computed SHA-256 in the working container. Downloaded current native intro contact-sheet artifact **13672106844** and 1800×1200 HQ visual-smoke artifact **13671807314** from the same CI run. Selected native generated contact sheet and frame `04_running.png`. Created the two Markdown handoff files, checksum manifest and archive using Python standard-library ZIP packaging outside the game; Python is neither inside nor invoked by the playable executable. Verified archive members and CRC via `zipfile.ZipFile.testzip() == None` and inspected Windows x64 PE architecture and import names.

**QA ZIP members:** `batman_cobblebot_native.exe`, `START_HERE.md`, `REPORT_TEMPLATE.md`, `SHA256SUMS.txt`, `references/intro-contact-sheet.png`, `references/hq-running-reference.png`. SHA-256 for reference images respectively:
- contact sheet: `479b001a89248fc078e8495216e753c46f49026922b7e1cbbec12ae89a4ea415`
- native HQ running frame: `4bfbe8e7f5641d20af2e0ce38e0321f87601437d32eb4130e1971960e329cc39`

**Verified binary type/import inspection:** `file` classified the executable as `PE32+ executable for MS Windows 6.00 (GUI), x86-64, 6 sections`. `objdump -p` reported imported DLL names `GDI32.dll`, `KERNEL32.dll`, `USER32.dll`, `WINMM.dll`, `api-ms-win-core-synch-l1-2-0.dll`, `bcryptprimitives.dll`, `ntdll.dll`. **Limited claim only:** these are Windows OS/system imports; no separate browser/Flash/Python engine import was observed. This is not proof that all transitive dependency source code is Rust, nor does it prove absence of dynamically loaded modules at runtime. Cargo code source and `AGENTS.md` still establish the native Rust game objective.

## Manual acceptance procedure — do not substitute CI for original comparison

1. Get the original artifact from the above GitHub run if the conversation ZIP is unavailable. Unpack the native EXE; verify SHA using PowerShell `Get-FileHash .\batman_cobblebot_native.exe -Algorithm SHA256`.
2. Launch the EXE on actual Windows, with keyboard and speakers enabled. The original source story begins on title **Enter**, plays its recovered 180 samples and **holds its stopped final frame**. It must not automatically enter Level1A at 7.2s; original sprite Stop actions and source button event explain why.
3. Test **Enter/Space/Escape** or original bottom-right button **press then release inside**. Original source button #129 release-inside launches fadeout sprite #34 with 41 samples (0–40), then starts Level1A. Listen for correctly sequenced intro StopSounds and Level1A music.
4. Compare story dialogue/images, lack of repeated 15-frame black flicker, source-style HUD typography, Level1A title, running/jumping/gliding/attacking, pickups, camera and exit fade against **actual original Flash footage**, not just the included native smoke images.
5. Record Windows version, screen resolution/DPI, tested code SHA and exact timestamp/steps for first difference. Attach screenshot or 10–20-second video and mark expected-source vs observed-native. Use included `REPORT_TEMPLATE.md` and open issue https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/issues/17.
6. **Blocker-first rule:** Fix the first reproducible discrepancy and rerun source-appropriate Rust/Windows tests before starting any unrelated game feature; keep PR #18 draft, `main` untouched and Level1B blocked until issue #17 explicitly accepted.

## Limits and next bounded task

The original CI ZIP and native image artifacts were successfully retrieved and independently hashed. The repackaged QA ZIP passed an internal archive-integrity check. **No Windows process was run in this Linux environment**, and there has been no physical mouse/audio/screen original-vs-native acceptance test. A checksum checks identity/corruption, not source-game fidelity or trustworthiness of an unfamiliar executable; follow normal Windows security guidance and verify provenance.

The next task remains **WIN-01 / FIDELITY-GATE-01 (user's real Windows playtest)**. Once evidence identifies a concrete bug, the project must resolve and retest it before moving on. See [research RE-20261109-007](REVERSE_ENGINEERING_RESEARCH_LEDGER.md) and [docs-only audit AUD-20261109-005](RUST_PURITY_AND_ENGINEERING_AUDIT.md). Keep this text aligned with the exact tested game SHA rather than a later documentation-only SHA.

## 2026-11-09 — WIN-DELIVERY-01: preferred automatic Windows installer (supersedes ZIP as PRIMARY delivery)

**Current verified completed latest workflow:** [Run #42619759787](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42619759787), Git head `ba3a18f140e1519d9c7308df2e5e1ab1aba96dc3`, **57/57 successful workflow steps**, Windows x64 executable artifact **13676630727**, name `Batman-CobbleBot-Level1A-Fidelity-Windows-x64`, not expired at inspection.

**Important separate identities:** From prior code commit `acccd0b70327e2ea1911885f97e21f5e5a5398b3` to `ba3a18f...`, GitHub compare showed **only `AGENTS.md` and documentation files changed, no Rust source/runtime implementation changes**. However the newest CI binary's bytes differ from the earlier build, so do not use the earlier archive's SHA-256 to verify the new one. The following hashes were calculated by downloading and reading the newly produced artifact, not guessed from Git history:

| File | Actual bytes | SHA-256 |
|---|---:|---|
| GitHub Actions ZIP artifact 13676630727 | 29,375,231 | `e0180ec868a62e521d31c25a52226fec937c51f50aa8875dfaf691b739f278ca` |
| Unmodified member `batman_cobblebot_native.exe` | 34,697,216 | `e5394d6e4f044e289023386eae37483e075c587a2d1fd172095bd1b7fd38b28c` |

The ZIP was inspected and **passed ZIP CRC validation** with exactly one member, the expected EXE. Its PE header begins with `MZ`. The previous test archive and hashes in the sections **above remain historically correct for their own earlier CI run**. This addendum does not overwrite them.

**User-preferred primary delivery restored:** The canonical PowerShell 5.1+ installer is now in [AGENTS.md](../AGENTS.md), section *Permanent Windows game delivery contract*. It uses authenticated GitHub CLI (`gh`) to find the most recent successful Windows workflow on the development branch, download its single named executable artifact, verify the SHA-256 across the temporary and staged files, replace `%LOCALAPPDATA%\BatmanCobbleBot\batman_cobblebot_native.exe` with a backup of the old file, write `last-run.txt`, `last-commit.txt` and `last-sha256.txt`, and launch the newly installed game. **Future user-facing playable deliveries must include that copy-paste installer block and a current successful run link, not just an manually repackaged chat ZIP.** The conversation ZIP reported above is optional and bound to its own historical SHA.

**Source documentation and audit:** [RE-20261109-008](REVERSE_ENGINEERING_RESEARCH_LEDGER.md), [AUD-20261109-006](RUST_PURITY_AND_ENGINEERING_AUDIT.md). **No game logic changed.** User-side execution and original Flash visual/audio fidelity remain **UNVERIFIED** until physical Windows feedback. The installer command has been statically reviewed against the official [GitHub CLI run list](https://cli.github.com/manual/gh_run_list), [artifact download](https://cli.github.com/manual/gh_run_download), and [Microsoft File.Replace](https://learn.microsoft.com/dotnet/api/system.io.file.replace) documentation; it has **not** been executed on the user's machine or tested with an installed local PowerShell interpreter in this environment.
