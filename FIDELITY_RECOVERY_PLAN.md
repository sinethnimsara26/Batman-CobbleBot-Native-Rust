# CobbleBot Native Fidelity Recovery Plan

## Goal

Produce a portable native Windows executable whose gameplay and presentation are visually and behaviorally close enough to the original Flash release that ordinary play looks like the same game, while replacing the Flash runtime and ActionScript execution with Rust wherever practical.

Target: **99% look-and-feel fidelity**, not "all levels technically reachable".

## Development rule

No new stage may be counted as implemented until the previous stage passes its fidelity gate.

The old "full campaign" handoff is reference material, not the new definition of done.

## Phase 0 — Preserve the handoff safely

- preserve handoff hashes, source commit IDs and executable hash
- retain useful Rust source, packed original animation assets, collision code and tests
- do not merge the known-bad handoff over the verified RE evidence
- regenerate reproducible assets from the canonical SWF whenever possible instead of depending on opaque one-off binaries

Exit gate: handoff provenance documented and no unrelated SN-Tech project changed.

## Phase 1 — Make input boringly reliable

Replace one-shot focus assumptions with a robust foreground input layer:

- track `WM_ACTIVATE`, `WM_SETFOCUS`, `WM_KILLFOCUS`
- clicking the client should explicitly establish focus
- use foreground-gated physical keyboard sampling (for example `GetAsyncKeyState` with edge detection) as the gameplay source, or an equally robust native solution
- clear held keys immediately on focus loss
- ignore gameplay keys when the game is not the foreground window
- keep event messages for text/menu behavior where appropriate

Acceptance:
- launch EXE and press Enter without clicking: works
- immediately hold Right: Batman moves
- Space/S/D/A work without clicking
- alt-tab away/back does not leave stuck keys
- mouse click is not required to "wake up" controls

## Phase 2 — Rebuild level1a visible art correctly

Separate Flash display-list content into:

1. visible scenery
2. collision-only geometry
3. triggers/checkpoints/pits/camera controls
4. dynamic actors/props

Do not render categories 2/3 into visual tile packs.

Generate an explicit per-stage visibility manifest from RE evidence rather than maintaining arbitrary manual symbol exclusions.

For level1a specifically, the initial known exclusions include control symbols 149/151/153. Verify all remaining instances against an original-game reference capture.

Also fix unsupported fill handling (notably gradient fills) before declaring scenery complete.

Acceptance:
- no red pit rectangles
- no blue collision shapes
- no visible checkpoint/control volumes
- canonical level1a spawn screenshot visually matches the original
- visible art and collision masks are independent assets

## Phase 3 — Translate Batman's player state machine faithfully

Stop layering generic movement helpers over action helpers.

Translate recovered `playerLogic` into one explicit state machine with action priority, state entry, state duration and transition rules.

Support the complete recovered state set, including:
- stand/walk/run
- jump/fall/glide/land
- duck/up
- all punch/kick variants
- all Batarang variants
- grapplingup/grappling
- capespin
- hurt2/hurt/electro/die

Animation should be driven by the original label ranges and original 25 Hz timeline semantics.

Gameplay events such as attack hit windows, projectile spawn, landing, hurt recovery and death must be tied to original state/frame timing rather than arbitrary Rust timers.

Acceptance:
- standing visibly animates where the original does
- walk/run cycles advance smoothly
- jump -> fall -> glide -> land transitions match the original
- attacks visibly complete instead of being overwritten by Stand/Walk
- holding movement while attacking follows original behavior
- recorded state/frame trace can be compared tick-for-tick to an original reference scenario

## Phase 4 — Implement exact camera and registration behavior

Use recovered `cameraLogic*()` functions and original placement matrices.

No guessed center-on-player smoothing.

Preserve:
- world coordinate system
- sprite registration points
- stage-specific camera bounds
- camera look-ahead/vertical behavior
- special-stage camera functions

Acceptance:
- level1a canonical camera checkpoints align with original screenshots
- Batman and scenery do not jitter from frame registration
- camera movement during run/jump/glide matches original reference footage

## Phase 5 — Restore the original presentation layer

Reconstruct the original visual flow instead of custom native substitutes:

- title/menu behavior
- instructions/tutorial overlays
- HUD
- level title/card sequences
- fade-in/fade-out
- area transitions
- game over
- ending/victory flow

Rust owns the state machine, but original artwork/layout/timing is preserved.

Acceptance:
- no custom placeholder HUD
- no custom text-only Area Complete/Game Over substitutes where original presentation exists
- transitions animate at original cadence rather than hard-cutting

## Phase 6 — Restore audio as event-driven native playback

Use extracted original sounds and map them to recovered ActionScript events.

Implement:
- punches/kicks/swishes
- item/gadget sounds
- enemy/boss effects
- environmental/vehicle sounds
- explosions/metal hits
- any music/loops present in the original source

Runtime playback must be native; Flash is never executed.

Acceptance:
- sound events line up with the same animation/gameplay frames as the original
- repeated sounds obey original restart/overlap behavior where recoverable

## Phase 7 — Freeze level1a as the fidelity vertical slice

Before touching level1b, create a deterministic reference suite:

- reference screenshots at several fixed positions
- native screenshots at the same positions
- player state/frame traces for movement/jump/glide/attacks
- camera coordinate traces
- collision checkpoints
- cold-start input test

The user should manually play this build and approve the feel before campaign expansion.

## Phase 8 — Expand stage-by-stage

Only after level1a is approved:

- level1b
- Penguin
- level2a
- level2b
- level3a
- Kabuki
- level4a
- CobbleBot/ending

For each stage:
- visible asset bake
- collision
- camera
- stage-specific objects
- enemy AI
- transitions
- audio
- golden screenshots/playtest

Do not substitute a generic AI system when recovered stage-specific ActionScript exists.

## Phase 9 — Release gate

Before calling the project complete:

- full human playthrough
- clean Windows machine test
- no Flash/Ruffle/WebView/runtime dependency
- portable single-folder or single-EXE distribution as practical
- no focus/input startup bug
- no debug/control geometry visible
- original assets and transitions consistently displayed
- all stages and bosses completable
- compare representative gameplay video against original reference footage

## Immediate next coding target

Do NOT continue expanding campaign content.

The next implementation sequence is:

1. import/preserve the handoff Rust source into this recovery branch without treating it as approved
2. fix foreground keyboard input
3. fix the level1a tile baker so invisible control geometry is excluded
4. rewrite the player state/update order around recovered `playerLogic`
5. rebuild level1a only
6. produce a visual/playable checkpoint for user testing

Only after that checkpoint looks and feels correct should work resume on later stages.
