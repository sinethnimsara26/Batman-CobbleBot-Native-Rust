# Level 1A — physical Windows acceptance guide

**Status:** manual acceptance is OPEN, not proven by GitHub CI. This guide is a companion to [issue #17](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/issues/17) and the [mandatory automated testing protocol](../AUTOMATED_WINDOWS_TESTING_PROTOCOL.md). The project is deliberately frozen to Level 1A, PR #18 is draft and Level1B must not start until the player accepts fidelity.

## 1. Install the most recent verified native EXE

Follow the **permanent Windows game delivery contract** in [AGENTS.md](../../AGENTS.md) (GitHub CLI, latest successful `build-level1a-fidelity.yml` artifact, `%LOCALAPPDATA%\BatmanCobbleBot` install with SHA256 and backup). Do not use the obsolete pinned historical QA ZIP, do not install Flash/Python for the playable game, and close any old Batman process before replacing the EXE.

**Build evidence matters:** `cargo test --locked` + `cargo build --locked`, actual 1×/HQ/GUI/icon checks and CI SUCCESS are prerequisites, **not** physical/source-Flash acceptance. Always select a live successful run. Do not assume a documentation-only run produces the same executable checksum as a previous run.

## 2. Optional, safe, local evidence collector (PowerShell 5.1+)

The repository ships [`scripts/collect-level1a-windows-evidence.ps1`](../../scripts/collect-level1a-windows-evidence.ps1). Review it before running. It reads the installed EXE SHA256 and the installer-created `last-run.txt`, `last-commit.txt`, `last-sha256.txt` when available; records only operating-system version, coarse monitor resolutions/system DPI and count/status of enabled audio endpoints; writes a local JSON report and a Markdown player checklist.

It **does not** request administrator privileges, capture screen/voice/audio, send network requests, upload files, change Windows settings, install software, or assert human acceptance. An audio endpoint found by Windows is *not* proof that music actually played.

With a repository checkout, run from the repo root:

```powershell
powershell.exe -NoProfile -File .\scripts\collect-level1a-windows-evidence.ps1 -Launch
```

Without a checkout, open the linked script on GitHub, review/download its **Raw** source, then run from its downloaded location:

```powershell
powershell.exe -NoProfile -File "$env:USERPROFILE\Downloads\collect-level1a-windows-evidence.ps1" -Launch
```

Adjust the downloaded filename/path if necessary. If PowerShell's execution policy blocks locally downloaded scripts, right-click the script, select Properties and use **Unblock** only if you reviewed and trust this repository version; do not globally relax execution policy. The `-Launch` switch opens Batman after the report; omit it to collect information without opening the game.

Reports stay in `%LOCALAPPDATA%\BatmanCobbleBot\AcceptanceReports\level1a-<UTC timestamp>` by default. Do not post their contents without choosing to share them. They intentionally omit local full paths/usernames, microphone input and desktop imagery.

## 3. Actual player checklist — do each applicable item

The generated checklist starts with every condition unverified. Here is the intended order:

1. **Title + story:** start normally; compare the original Flash source story pacing, dialogue text, panel changes, sound and end-of-story transition. Confirm no unexpected black-screen hold, flashing, or silent desynchronization. Test original-style on-screen SKIP **mouse release-inside**; distinguish native user-requested automatic advancement from original Flash ActionScript behavior.
2. **HUD:** inspect BATARANG/HEALTH/SCORE and the large score/lives digits, including the **X immediately beside its number** and yellow/blue icon alignment. Test low and multi-digit values, not just the default HUD. Styled HQ HUD is a candidate; approval of original Flash white HUD equivalence is separate.
3. **Fullscreen/focus:** press F then Esc, confirm no cropped scene or broken window mode. During intro audio, Alt+Tab or open Windows' volume flyout and return; record whether soundtrack and cutscene **both** pause and resume together, and whether audio is audible. Successful CI visual/focus tests on an audio-less server do not establish this.
4. **Gameplay:** walk/run/jump/glide, melee/gadget attacks, collectibles, score/lives, collision, camera, fade and Level1A exit. The game intentionally stops at Level1A; Level1B is not part of this approval.
5. **Windows icon:** view the current EXE in Explorer and an actual pinned taskbar shortcut. Windows may cache an old icon even when the new EXE has the right native resource—unpin/re-pin if needed and compare.
6. **Display settings:** if available, test at 100%, 125% and 150% scaling and on different monitors/DPI; record the environment and clipping/alignment. Not every player owns multiple monitors; mark those checks *not available* instead of pass.

Use the original SWF/source screenshots only as lawful reference material; this project does **not** package an executable Flash runtime. Align screenshots to the original 600×400 stage when comparing, note whether the Rust HQ view is rendered at 3× resolution, and distinguish actual source differences from font anti-aliasing differences.

## 4. Share results only by choice

For a defect, include **build commit/run, exact reproduction steps, expected original Flash behavior, native observed behavior, severity and optional chosen screenshots**. You can copy relevant excerpts of the generated JSON, but do not share anything private unintentionally.

When the player has personally approved Level1A original-game fidelity and physical sound, record that decision in issue #17. Until then, **keep PR #18 draft/unmerged, issue #17 OPEN, main and Level1B untouched**. A 100% green CI is not a substitute for this decision.
