# Explicit user-authorized Level1A release request — 2026-11-11

The user reports that Level 1A game looks good and requests a final-for-now release before planning Level 1B. Release only **the already tested executable/source**, not an unverified new game change.

- Desired tag: `level1a-v0.2.0`; target source: `49e8fd343e4f37f045ba71f24fff0999211f0f11`.
- Permanent source branch: `release/level1a-v0.2.0`, already created at exact code SHA.
- Successful Windows build: [#42936169736](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42936169736), 81/81 success, EXE artifact ID 13819117162, name `Batman-CobbleBot-Level1A-Fidelity-Windows-x64`.
- Assets: raw `batman_cobblebot_native.exe`, portable Windows ZIP with third-party notice, SHA256SUMS, GitHub automatic source archives.
- No redistribution license for third-party Batman assets is asserted. No code changes, no main/Level1B edits, no merge of draft PR18. Fidelity issue17 remains open for stricter checks.
- Publisher workflow is one-shot/idempotent. Treat the release as **PENDING** until the actual GitHub tag, executable, ZIP and hashes have been verified.
