# Mandatory six-document development and AI handoff protocol
**Project:** The Batman: The CobbleBot Caper — standalone native Rust reconstruction  
**Authority:** User-directed permanent repository workflow, 2026-11-09  
**Applies to:** ChatGPT Chat mode, Codex, Claude, Gemini, any other AI coding agent, and human collaborators.  
**Canonical entry point:** [../AGENTS.md](../AGENTS.md). This protocol is referenced by AGENTS.md and is binding whenever an agent works in this repository.

> **START HERE.** Assume the previous AI conversation has been lost. Read AGENTS.md, this protocol, AI_HANDOFF, and AI_TASKS before any implementation. Verify live GitHub state. Work in one small batch; inspect code and tests; maintain all six records; checkpoint in GitHub; report; STOP. A new user "continue" authorizes only the NEXT batch.

## 1. Non-negotiable project goals and scope
1. Reconstruct the original *The Batman: The CobbleBot Caper* Flash mini-game as faithfully as demonstrated original evidence permits: animation, intro, dialogue, scene transitions, HUD, typography, effects, input hitboxes, timing, sound, original state transitions, collision and gameplay.
2. The **finished playable game and its gameplay logic remain genuinely Rust-based**, without embedding a Flash runtime, JS/Python/C/C++ gameplay substitute, or external emulator. Ordinary Win32/platform libraries and development-time inspection tools are evaluated separately; they are not automatically a violation.
3. Prioritize original-source fidelity over arbitrary visual redesign. Distinguish exact recovered Flash behavior from hypotheses and styled candidates; never claim they are interchangeable.
4. Respect existing development branch and **draft PR #18**, its original source-fidelity acceptance issue **#17**, and existing test gates. Do not merge, force-push, delete branches, bypass approval, or start unrelated Level 1B/boss work.
5. Chat-mode connection interruption is a real failure mode. Never design a task that requires one unbounded response to preserve the whole project state.

## 2. SIX mandatory records — different responsibilities, one workflow
The required files are:
1. [AGENTS.md](../AGENTS.md) — **permanent rules and repository orientation.** Canonical instruction index, build/test conventions, repository safety, Rust/fidelity mandates, and both protocols linked here. Re-read each batch; modify only if enduring policy/architecture/procedure has changed.
2. [AI_TASKS.md](AI_TASKS.md) — **roadmap and acceptance gates.** Keep statuses (TODO / IN PROGRESS / BLOCKED / IMPLEMENTED-UNVERIFIED / VERIFIED), small task boundaries, dependencies, priority, technical blockers, and separate automated vs manual acceptance.
3. [AI_HANDOFF.md](AI_HANDOFF.md) — **current reproducible checkpoint.** Keep current branch, PR, last verified source/code SHA and last docs/CI SHA where different, CI/artifact links, remote state, incomplete edits, validation limits, blockers, and EXACT first next step. This is a living current-state document, not the history archive. Note a document cannot contain its own future commit SHA; determine final SHA from Git.
4. [AI_WORKLOG.md](AI_WORKLOG.md) — **chronological engineering activity.** Append batch ID, scope, changed paths, experiments, failures, focused tests, concrete outcomes, relevant research/audit IDs, commits and the following step. Do not substitute a research ledger for the worklog.
5. [REVERSE_ENGINEERING_RESEARCH_LEDGER.md](REVERSE_ENGINEERING_RESEARCH_LEDGER.md) — **detailed investigation evidence.** Provide stable RE-YYYYMMDD-NNN entries and exact original SWF asset/symbol/frame/timeline/source paths, repository paths, evidence IDs, dates and source URLs, tested alternatives, decisions and tradeoffs, failed hypotheses, reproducible calculations and experiments, confidence and uncertainty. Explain an evidence-based decision trail; do NOT request or fabricate hidden chain-of-thought transcripts. For external research record exact URL, section/version, what it supported, how used, provenance quality, and whether verified locally.
6. [RUST_PURITY_AND_ENGINEERING_AUDIT.md](RUST_PURITY_AND_ENGINEERING_AUDIT.md) — **post-implementation independent audit.** Append AUD-YYYYMMDD-NNN after code inspection, not before. Examine touched Rust modules, actual code/diffs, Cargo crates/features/transitive native dependencies/FFI, quality and sound Rust practices, appropriate alternatives, benchmark claims, source-fidelity scope and real test results. Verdicts separate pure-Rust runtime compliance, engineering quality and original-game fidelity; state evidence and confidence. If no code changed, record honestly "No implementation changes — Rust code audit not applicable" and do not invent a new implementation audit.

**All six files MUST be considered in EVERY batch**. AGENTS.md need not get repetitive changes each time. The other documents receive the updates that actually matter; "reviewed; unchanged because X" is valid ONLY with truthful explanation. Research-only tasks must still update research as appropriate; code-changing tasks must get a completed post-change Rust audit.

Additional substantial original project diaries (such as docs/revival), manifests and source analyses remain valuable; do not delete or flatten them.

## 3. Required per-batch procedure — 11 stages
**Stage 1: RECOVER.** Read AGENTS.md and its linked protocols, then AI_HANDOFF, AI_TASKS, relevant AI_WORKLOG, RE and AUD entries. Verify remote branch HEAD and PR, latest CI, artifacts and changed files before assuming another agent's statements are correct. Distinguish current findings from stale notes. Do not overwrite concurrent or uncommitted work.

**Stage 2: SELECT.** Choose exactly ONE bounded objective with a measurable test. The next action should address the highest-priority real blocker. Build a scoped acceptance checklist. A user "continue" means one task, not a license to finish the game.

**Stage 3: INVESTIGATE.** Read original Flash evidence and code/test locations relevant to the task; record exact symbol IDs, extraction/asset paths, frame indices (specify zero- or one-based), source code functions and source comparisons. If the approach hinges on external Rust technical advice, consult the precise docs.rs crate version, Rust docs, maintainers' releases/issues and appropriate Rust community discussion. Distinguish official specifications, credible maintainer explanations, community anecdotes, and local reproduction. Record alternatives and falsifiable hypotheses.

**Stage 4: IMPLEMENT.** Make the smallest useful Rust-native change compatible with the existing architecture. Avoid new frameworks, changing the rendering foundation, large refactors or unnecessary dependencies. Do not make changes outside the current acceptance scope, including to deployment and historical branches.

**Stage 5: VERIFY.** Test actual change by relevant Rust test/build/lint, deterministic simulation, original-source reference comparison, Windows CI and, where possible, Windows window/input tests. Distinguish compile pass, unit pass, screenshot regression pass, GUI automation pass, physical audio/input test and user-approved original-game fidelity. No invented execution results. Capture failure logs, screenshot artifacts and stable URLs. If test unavailable, say so.

**Stage 6: RESEARCH RECORD.** Append a meaningful source-traceable RE entry (or state there are no new findings for a purely administrative change). Include exact location, evidence, experiments, alternatives, decision, confidence and unresolved limitations. Avoid raw tool output dumps.

**Stage 7: POST-CHANGE RUST AUDIT.** Inspect the finished changed code and affected dependencies, not only the intended design. Evaluate pure-Rust game runtime, unsafe/FFI/native code, idiomatic Rust patterns and alternatives proportional to the task. Separate Rust engineering from Flash source-fidelity assessment.

**Stage 8: AUDIT RECORD.** Append independently reasoned AUD entry, include executed commands, outcomes, linked code/evidence and explicit qualified verdicts. For a docs-only task, say no code audit applies. If a serious failure exists, block "VERIFIED"; fix within bounded scope only when safe or create a dedicated task.

**Stage 9: SYNCHRONIZE THE SIX DOCUMENTS.** Update roadmap, worklog, handoff, and AGENTS.md if durable rules changed. Cross-reference stable batch, RE and AUD IDs. Check all six exist and no statement contradicts the real branch, actual test result or the currently selected task. Use existing records, do not endlessly repeat unchanged research. Do not replace detailed research/audit with shorthand "done".

**Stage 10: CHECKPOINT.** Commit and push implementation plus relevant records to an authorized work branch, verifying remote existence and linked CI/artifacts. Preserve green baseline and separate speculative changes. Never force-push, silently merge or edit others' changes. If a remote write fails, state the specific failure; a text reply is NOT proof of a saved checkpoint. Re-read branch before retries to prevent duplicated operations.

**Stage 11: REPORT AND STOP.** Provide objective, changed files, actual tests, Rust compliance/fidelity results, six-file review statuses, commit/PR/run artifact links, blockers, and EXACT next small task. STOP; do not start a second batch until the next user instruction.

## 4. Conservative operation budget and failure recovery
- Prefer a compact batch with approximately 6–8 substantive tool operations; ~10 is a conservative operating target, NOT a ChatGPT service guarantee. Keep capacity for docs, commits, checks and final response. Research-only or audit-only batches are valid.
- Before lengthy builds/downloads/complicated external operations, save a durable earlier checkpoint where appropriate. Never run an unbounded chain of agent tool calls. Avoid excessive test logs in chat: put logs and image evidence in artifacts and link them.
- On "Connection interrupted", compare remote HEAD, PR changes, checks and artifacts FIRST. An apparent failure may have already committed work or run tools. Avoid retrying destructive or non-idempotent external operations until their results are checked. Repair stale handoff. A new AI must recover from GitHub, not memory.
- The active ChatGPT conversation does NOT receive a push notification just because AGENTS.md was edited. An already-running agent must reread it at its next safe checkpoint. New chats/other providers must be explicitly directed to the repository and AGENTS.md if their tools do not auto-discover agent files.
- If concurrent agents are editing, avoid collisions: verify remote head, use optimistic concurrency when supported, commit scoped changes, and explicitly surface conflicts rather than overwriting remote work.

## 5. Minimum evidence and classification discipline
- Every important assertion must be tagged in a meaningful way: directly observed, reproduced by automated test, original-source-extracted, inferred, externally sourced or unknown.
- Maintain source fidelity and engineering quality as distinct dimensions. Source screenshot hashes can catch unintended render changes, but **a Rust screenshot identical to another Rust screenshot does not prove correspondence with the original Flash game**.
- Keep source of truth where reproducible. Original asset IDs, frame times, SHA-256, library versions, CI run URLs and file paths must be stable and truthful; do not invent them.
- Respect real-world audio/speaker, mouse, DPI and manual acceptance limitations. Flag "UNVERIFIED" if only headless tests ran.
- On discovering a regression or stronger source evidence that invalidates an earlier finding, preserve the historical entry and append a correction, priority follow-up and rationale.

## 6. Six-file compliance report format
Report per batch:
| Record | Status | Evidence/reason |
| --- | --- | --- |
| AGENTS.md | Reviewed / Updated | Current permanent rule reference |
| AI_TASKS.md | Reviewed / Updated | Task ID and state |
| AI_HANDOFF.md | Updated / Blocked | Actual branch and next step |
| AI_WORKLOG.md | Updated / Blocked | Batch ID |
| REVERSE_ENGINEERING_RESEARCH_LEDGER.md | Updated / No new findings / Blocked | RE ID or reason |
| RUST_PURITY_AND_ENGINEERING_AUDIT.md | Audited / Not applicable / Blocked | AUD ID or reason |

Report the actual code SHA and documentation SHA separately when applicable; final commit SHA is determined after push. Incomplete/blocked obligations are not silently considered successful.

## 7. Governed extensions and pre-existing priority
Read the second mandatory document: [AUTOMATED_WINDOWS_TESTING_PROTOCOL.md](AUTOMATED_WINDOWS_TESTING_PROTOCOL.md). It adds free automated Windows executable QA, screenshot/video evidence, regression comparison, and staged real-GUI testing. It **does not relax** the six-file protocol, source-original fidelity rules or Rust-native runtime requirement.

**Existing priority remains the Level 1A intro/HUD/source-fidelity acceptance gate.** Automation is to reduce manual debugging, not manufacture fidelity approval. Keep draft PR #18 and issue #17 constraints. New permanent user rules belong in AGENTS.md or linked authoritative protocols, not only in chat.
