# CobbleBot Native Fidelity Recovery Audit
Date: 2026-09-20

## Provenance

This audit is based on the Codex handoff archive supplied after the earlier native-Rust experiment.

- handoff archive SHA-256: `506661e5b9e07ddb0dbdd29bb9a39379e52ac5dfdb0b2f4c351f47284ab97454`
- handoff local branch: `codex/rust-native-bindings-experiment`
- reported handoff source HEAD: `6c4c54b73aeac28742d77d5e8b1e0fb7c4ab983a`
- prior full-campaign commit: `00ac031dfad67b141793e446eb9d2235ccca8bbb`
- handoff EXE SHA-256: `FEF53C18848FFAB79E9FE5C3F3F177CCA29F320BD74BD2F72AA1CADB45159325`

Important: those Codex commits were local to the handoff workspace. They are NOT present in the persistent SN-Tech Batman branch. The persistent upstream branch was still at `254c2675cf6a09aa6366906e077d61a0d688aaec` when this recovery branch was created.

## What the handoff genuinely got right

The handoff is not useless. It contains several valuable pieces that should be preserved:

- native Rust/Win32 executable architecture
- fixed 600x400 logical game surface and fixed 25 Hz simulation concept
- packed original Batman animation frames
- recovered frame-label table for the 271-frame Batman timeline
- collision-mask loading and native collision probes
- level/campaign state scaffolding for all nine gameplay stages
- extracted enemy/boss art packs and level tile packs
- a sizeable Rust test suite that is useful for regression checks after behavior is corrected

The Batman frame extraction itself is good. The 271-frame pack contains recognizable original stand/walk/run/jump/glide/punch/kick/gadget/hurt/death artwork. The primary problem is how those assets are selected, transitioned, composited and driven by gameplay.

## Root cause 1: visible level art contains invisible Flash control geometry

The biggest visual defect is not a renderer scaling issue anymore. The handoff's level tile baker renders every child of the Flash level display list as visible scenery.

For level1a, this bakes gameplay-only shapes directly into the artwork:

- symbol 149, named `pit`, renders as a giant bright red rectangle
- symbol 151, named `ground`, renders as blue collision geometry
- symbol 153 is checkpoint/control geometry
- symbols 146/147 are actual visible building scenery

Measured across the generated level1a tile set, approximately:

- 25.97% of visible non-transparent pixels are bright-red control geometry
- 25.28% are blue/collision-like geometry

So more than half the rendered content in the broken region is effectively debug/control geometry. This directly explains the user's "Batman standing on a red blob" report.

A test bake excluding symbols 149, 151 and 153 removes the red/blue junk and exposes the intended building art.

This must be fixed systematically for every stage. Collision, triggers, checkpoints, pits, camera volumes, damage regions and other invisible Flash instances must never be baked into visual scenery.

## Root cause 2: the native player state machine destroys action states

The handoff game tick performs approximately:

1. `handle_actions(input)`
2. `handle_horizontal(input, ground)`

`handle_actions` may enter Punch, Kick, Batarang, etc. On the same simulation tick, `handle_horizontal` can immediately replace that state with Stand, Walk or Run.

Consequences:

- attack animation may never visibly play
- melee transition detection can fail
- actions feel unresponsive
- animations appear static or nonsensical even though the correct frames exist

This is an architecture bug, not a missing-asset bug. Player action priority and state duration must follow recovered AVM1 `playerLogic`, not a sequence of loosely independent helper functions.

## Root cause 3: several original Batman states are missing

The original Batman timeline contains labels including:

`stand, walk, run, jump, fall, glide, land, duck, up, punch, highpunch, lowpunch, jumppunch, kick, highkick, lowkick, jumpkick, batarang, duckbatarang, jumpbatarang, grapplingup, grappling, capespin, hurt2, hurt, electro, die`

The handoff state enum omits important states such as:

- `land`
- `up`
- `grapplingup`
- `grappling`
- `hurt2`
- `electro`

Therefore even with all 271 original frames available, the native runtime cannot reproduce the original transition graph.

## Root cause 4: screen transitions and HUD are replacements, not ports

The handoff uses high-level screens such as:

- Title
- Instructions
- Playing
- GameOver
- AreaComplete
- Victory

but changes between them mostly happen as immediate Rust state switches.

There is no faithful implementation yet of the original Flash:

- fades
- level-title/card sequences
- transition timing
- overlays
- HUD animation/presentation
- original game-over/presentation flow

The runtime also draws a custom native HUD panel and bitmap-font text rather than reconstructing the original HUD display list. This is fundamentally incompatible with a 99%-fidelity goal.

Original audio is also not implemented.

## Root cause 5: keyboard focus fix is only a hypothesis

The handoff added `SetForegroundWindow` and `SetFocus`, but Windows is allowed to refuse foreground activation. The code:

- relies on `WM_KEYDOWN/WM_KEYUP`
- does not verify the return/effective foreground state
- does not robustly handle `WM_ACTIVATE`/`WM_SETFOCUS`
- does not recover focus on normal mouse activation
- was not validated with an automated real-keypress startup test

This matches the user's report: keyboard appears dead after launch and starts working only after clicking around.

The correct fix should not depend on a one-shot focus request.

## Root cause 6: campaign breadth was prioritized before one level was faithful

The handoff claims all nine gameplay stages, but its own documents explicitly acknowledge approximated AI, camera behavior, checkpoints/cards/presentation and missing audio.

Several AI routines are deterministic replacements for original random/branching behavior. Some later stages reuse generalized enemy logic rather than direct translations of the recovered stage-specific ActionScript.

This makes "all stages implemented" a misleading progress metric.

For this project, one correct stage is more valuable than nine approximate stages.

## Root cause 7: tests prove internal consistency, not original-game fidelity

The handoff reports 55 passing Rust tests. These are still useful, but most test the implementation's own state transitions and simulation assumptions.

Missing acceptance tests include:

- cold-launch keyboard input without clicking
- pixel/golden screenshot comparison against original game checkpoints
- animation-transition traces against original Flash behavior
- original HUD/card/fade timing
- full human playthrough
- clean-machine test
- audio/event timing

Passing unit tests therefore cannot be used as evidence of 99% fidelity.

## Recovery principle

Do not throw away the handoff, but do not treat it as a trustworthy implementation baseline either.

Keep:

- original asset extraction
- packed animation data
- collision infrastructure
- useful Rust plumbing/tests
- recovered RE evidence

Replace or re-derive:

- visible level tile baking
- player state machine
- input/focus path
- camera behavior
- HUD/presentation
- transitions
- AI approximations
- audio/event timing

The original SWF / recovered AVM1 is the behavioral oracle. The native runtime remains Rust and must not execute Flash at runtime.
