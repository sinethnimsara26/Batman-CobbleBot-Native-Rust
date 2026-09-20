# Reverse-Engineering Gate: CLOSED

Status: **COMPLETE AND VERIFIED**

Canonical RE package verification commit before this marker:

`2ca37883de0b717830d37a4dc9b1e9e4c49b15f2`

Integrity result:

- canonical package files checked: **48**
- GitHub package files checked: **48**
- Git blob SHA-1 matches: **48 / 48**
- size/path mismatches: **0**
- missing files: **0**
- unexpected package files: **0**

Scope verification:

- branch: `flash-native-batman-cobblebot`
- base: `main`
- all project changes are contained under `batman-cobblebot-native/`
- no Scooby project files or unrelated projects are modified by this branch

Recovered/validated RE coverage includes:

- original projector/SWF source fingerprints
- SWF movie/timeline structure
- 45 recovered AVM1 gameplay functions with raw disassembly evidence
- Batman controls, movement, jump/glide/grapple and physics constants
- melee/gadget combat state machine and damage logic
- enemy/boss behavior evidence
- level progression and routing
- level initialization constants and named instance placements
- camera behavior evidence
- animation labels/states
- 79 bitmap definitions / bitmap-fill mapping
- 38 sound definitions / sound manifest
- collision geometry/masks for all 9 gameplay stages
- original-art proof renders
- reproducible extraction/analysis toolchain

The reverse-engineering phase is therefore a **hard-complete milestone**.

## Rule for the next phase

Do **not** spend implementation time rediscovering already recovered behavior or replacing verified values with guesses.

The native Rust port should treat the manifests, disassembly evidence, masks, tools, and `GAME_SPEC.md` in this directory as the source of truth. If implementation uncovers an apparent contradiction, document it explicitly and re-check the raw evidence before changing the recovered specification.

No native Batman gameplay implementation existed when this gate was closed.
