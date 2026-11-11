# B04 — Level 1A → Level 1B pending-stage handoff (native Rust, no playable 1B)

**Date:** 2026-11-11. **Draft PR:** [#24](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/24) (OPEN/DRAFT/UNMERGED), based on B03 PR23.
**Windows-tested code SHA:** `7841968805fdcd21ced0d43514beaa7766de3e9a`; **run:** [#43023265336](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43023265336) COMPLETED SUCCESS. Final documentation-only HEAD is a later commit and must be resolved live from GitHub.

## Source-first acceptance boundary
- Canonical original FWS SHA256 `3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed` was verified by B01 and rechecked in the B04 native workflow.
- Original parser root index **19** (zero-based) is Level1A; `routing-evidence.json` root19 DoAction offset **402595** sets `gotoNext=level1b`, entering parser root index **20**. The original 41-frame fadeout sprite #34 uses frame39 StopSounds then its terminal frame40. Level1B source #1075 game/#783 level/#782 ground, StartSound #1076; Level1A StartSound #751.
- Exact body of original live player-control/audio dispatcher outside the tested source evidence is not claimed; original-vs-native live gameplay comparison remains open.

## Implemented source/paths
- `src/stage.rs`: only immutable `RootProgress` {score,lives,batarangs} and `PendingStageEntry` {from,to,progress,outgoing/incoming original StartSound IDs}; stage-/scene/card-type route already source-backed by B02.
- `src/game.rs`: read-only `Game::pending_stage_entry`; frame39 native MusicStop on exit path only, pending entry latched after t40 dispatcher; StageId and screen/render stay Level1A. Both added unit tests cover exact fade ticks, sound-event idempotence and pit/continue isolation.
- `.github/workflows/build-level1a-fidelity.yml`: only new B04 feature branch on push allowlist. Assets, goldens, game audio resources, Cargo dependencies, Win32 input and renderer are unchanged.
- **Explicit non-feature:** B03 previews were partial (`partial_proof_only`, `never_embed_as_complete_level`) and are NOT loaded as a stage. Incoming #1076 sound is a deferred source cue, not a playable native file. Root stat carry is a handoff contract, not proof of exact 1B spawn-health timing.

## Verified Windows CI (actual B04 code SHA)
- 81/81 reported workflow steps SUCCESS; 29/29 Rust tests, including two new B04 tests, cargo locked and canonical source SHA PASS.
- Historical native 1× screenshot expected hashes 15/15 PASS; installed-copy HQ3× screenshot parity 20/20 PASS; genuine Win32 window/start/focus/F7/F8/fullscreen/F/Esc/PE and Shell icon tests PASS.
- Render p95 **21.678ms**, scrolling p95 **23.8795ms** (both below unchanged 40ms); actual installed executable integration `PASS_AUTOMATED_INTEGRATION_WITH_EXPLICIT_AUDIO_LIMIT`.
- Real artifact metadata available from [run #43023265336](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/43023265336) includes `Batman-CobbleBot-Level1A-Fidelity-Windows-x64` artifact **#13855859171**, legacy image artifact **#13856297798**, HQ screenshot artifact **#13856896317**, fullscreen Windows QA **#13856801560** and Windows GUI pilot **#13856756984**. CI checked screenshot byte identities; original-Flash visual equivalence was NOT independently demonstrated.
- Hosted runner has no operational audio endpoint; `INTRO-AUDIO-QA-05 BLOCKED_HOSTED_RUNNER_NO_AUDIO_ENDPOINT` due WinMM MCI error277. Native visual focus checks passed but actual audio playback is UNVERIFIED.

## Independent audit, protected work, and next one task
- Source record `RE-20261111-014`; independent code audit `AUD-20261111-014`. No new Rust dependency/unsafe/third-party engine, no implementation of 1B collision/enemy/scoreboard/audio asset; no release issued.
- Frozen accepted `level1a-v0.2.0` tag/branch SHA `49e8fd343e4f37f045ba71f24fff0999211f0f11` unchanged, main and historical feature branches untouched, PR18/21/22/23 unchanged and PR24 stays DRAFT/UNMERGED.
- **STOP. Next separately approved B05:** source-authentic original Level1B Bank street 1× static scenery tile proof, with original #783 provenance and exclusion of script actors/control geometry; NOT activate playable Level1B.
