# Batman: The CobbleBot Caper — Native Rust Reimplementation

Unofficial high-fidelity native Rust reimplementation of **The Batman: The CobbleBot Caper**, originally released as a Flash game.

The goal is not a loose remake. The project aims to preserve the original game's **visuals, animation timing, controls, camera behavior, collision, UI flow, and feel** while replacing the Flash runtime and ActionScript execution with native Rust wherever practical.

## Current status

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
