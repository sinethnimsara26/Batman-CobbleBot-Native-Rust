# Reverse-engineering phase: COMPLETE

Status: **complete for implementation handoff**.

This marker means the port no longer needs guesswork about the original game's runtime rules. Asset baking can continue during implementation, but the game structure and source-asset relationships needed to perform that baking are known and reproducible.

## Completion checklist

- [x] Correct title identified: **The Batman – The CobbleBot Caper**
- [x] Windows projector fingerprinted
- [x] embedded zlib/CWS location identified and extraction automated
- [x] canonical FWS reproduced and fingerprinted
- [x] movie dimensions/FPS/root timeline recovered
- [x] recursive sprite/timeline/placement inventory recovered
- [x] AVM1 parser fixed to preserve DefineFunction/DefineFunction2 code bodies
- [x] all **45** global gameplay functions indexed
- [x] player controls recovered from `Key.isDown` bytecode
- [x] player physics constants recovered
- [x] Batman animation labels/frame count recovered
- [x] enemy/boss animation labels recovered
- [x] enemy/boss initialization stats recovered per level
- [x] combat damage/score paths recovered
- [x] camera functions and per-level camera constants recovered
- [x] level progression and story-card routes recovered
- [x] all named level/game instances and transforms exported
- [x] collision algorithm functions recovered (`checkGround`, `checkCeiling`, `checkEdge`, `checkWall`, `checkPlayer`)
- [x] every named `ground` symbol identified for all nine gameplay stages
- [x] collision masks reproducibly generated from original vector geometry
- [x] all **79/79** bitmap definitions decoded with **0 errors**
- [x] bitmap-fill shape relationships and matrices inventoried with **0 errors**
- [x] all **38** sounds indexed/extractable: **33 MP3 + 5 SWF ADPCM**
- [x] proof renders preserved
- [x] full pipeline rerun successfully from the original projector in a fresh output directory

## Source fingerprints

Canonical values are in `manifests/source-fingerprints.json`:

- projector: 3,111,471 bytes
- projector SHA-256: `03bb56b8e88345ef80c2e2e7b896195fb53c53d3d6ebb9a44a3c6487d7c4f9cf`
- outer zlib stream offset: **1,044,550** (`0xff046`)
- original embedded CWS: 2,078,586 bytes
- CWS SHA-256: `92a803fe7958cae87dcc6b610d78b716d09bc545331830a16d7158c39c312dcf`
- canonical FWS: 2,819,100 bytes
- FWS SHA-256: `3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed`

The original proprietary projector/SWF is intentionally **not duplicated in this repository**; the tooling works from the user's supplied copy and the hashes prevent accidental analysis of a different revision.

## What "complete" does not mean

It does not mean every original vector frame has already been converted into the final Rust asset-pack format. That is an **asset-build/implementation** task, not an unknown behavior problem. We now know which symbols, timelines, bitmaps, fills, placements, sounds, collision geometry, and state machines must be exported.

The implementation must therefore start from these manifests rather than drawing placeholder Batman art or inventing physics.
