# Recovered game specification

## Movie/runtime

- SWF version: **7**
- logical stage: **600 × 400 px**
- simulation/timeline rate: **25 FPS**
- root timeline: **36 frames**
- root frame label contains `BATMAN // the cobblepot caper / Produced by ImageryMedia.com`
- gameplay is AVM1 / ActionScript 1/2-era bytecode

A faithful native implementation should therefore run gameplay on a **fixed 40 ms tick**. Rendering may be faster, but simulation must not be tied to monitor refresh rate.

## Player controls

Recovered directly from `playerLogic` `Key.isDown` calls:

- **Left (37):** move left
- **Right (39):** move right
- **Up (38):** high-attack modifier and contextual vertical/camera use
- **Down (40):** low-attack modifier / duck
- **Space (32):** jump; while descending and allowed, enter cape glide
- **S (83):** punch
- **D (68):** kick
- **A (65):** current gadget
- **D + S:** cape spin when the current state permits it
- hidden sequence **L I F E** (`76,73,70,69`) sets lives to **99**

Punch variants: `punch`, `highpunch`, `lowpunch`, `jumppunch`.
Kick variants: `kick`, `highkick`, `lowkick`, `jumpkick`.
Gadget animation variants include `batarang`, `duckbatarang`, `jumpbatarang`, `grapplingup`, and `grappling`.

## Player physics

- `playerHealth = 100`
- `playerStrength = 10`
- collision dimensions: **50 × 75**
- normal `playerSpeed = 19`
- normal starting `playerWalkSpeed = 6`
- Kabuki/highway stage uses `playerWalkSpeed = 8`
- normal gravity = **5 units/tick²**
- jump sets vertical velocity to **-42**
- glide entry: when descending fast enough (`playerDy > 5`) and allowed, vertical velocity is multiplied by **0.5**, gravity becomes **1**, and player state becomes `glide`
- `playerDir` uses ±1 facing

Root start values are score 0, lives 5, hitCount 0, Batarangs 0. Gameplay frames also initialize `playerLives = 3`; the original uses both root lives and level-local life/respawn state, so the port should preserve that distinction until behavior tests prove they can be safely unified.

## Batman animation symbol

Primary Batman sprite **691**, **271 frames**. Recovered labels:

`stand`, `walk`, `run`, `jump`, `fall`, `glide`, `land`, `duck`, `up`, `punch`, `highpunch`, `lowpunch`, `jumppunch`, `kick`, `highkick`, `lowkick`, `jumpkick`, `batarang`, `duckbatarang`, `jumpbatarang`, `grapplingup`, `grappling`, `capespin`, `hurt2`, `hurt`, `electro`, `die`.

Alternate/highway Batman sprite **1862**, **291 frames**, contains the same core states plus labels such as `jumpup` and `grapplingtemp`.

## Collision

The original does not use generic rectangular platform boxes for Batman/ground contact. It calls Flash shape `hitTest` against `game.level.ground` through global helpers:

- `checkGround(character,height,loop)`
- `checkCeiling(character,height,loop)`
- `checkEdge(character,height,dx)`
- `checkWall(character,height,dx)`
- `checkPlayer(character,dir,offset)`

`checkGround`/`checkCeiling` recursively correct penetration one pixel at a time with a hard limit of **16** iterations. `checkGround` references `onPlatform` and zeroes `playerDy` as appropriate. The native implementation should use the generated level-space ground masks and reproduce the original point probes/penetration correction, not substitute a generic physics engine.

`generated/collision_masks/` contains masks for every gameplay stage and an origin/size manifest. These masks are regenerated directly from SWF vector shapes by `tools/render_collision_masks.py`.

## Combat

Enemy AI contains repeated, verifiable damage paths:

- Batarang hit: **5 HP**
- ordinary punch path: **10 HP**
- `lowkick`: **15 HP**, usually knockdown/fall
- `highpunch`: **15 HP**, usually knockdown/fall
- `highkick`: **15 HP**, usually knockdown/fall
- a close-range `playerAttacking` clash/knockdown path: **20 HP**

The 20-HP branch is conditional on AI/collision state and should not be blindly attached to one animation name.

Standard enemy attacks generally subtract each enemy's `strength` from `playerHealth` and set `playerInvincible`. Bird/eagle contact uses a fixed **10 HP** path.

Standard goon/maid/bird/biker deaths award **100 points**. Penguin major death branches award **500**. CobbleBot death branches award **3000** and ultimately set `gotoNext = card:end`.

## Combo/invulnerability

The AI increments root `hitCount` on successful player attacks and updates `comboCount` (20 on standard-goon paths, 50 on several boss/advanced-enemy paths). Multiple branches gate damage with enemy `invincible` and/or player `playerInvincible`. Preserve those state variables; do not merely add a modern fixed invulnerability timer without matching the original timeline-driven clear conditions.

## Gadgets

`A` dispatches through `currentWeapon`:

- `batarang`: requires ammo, decrements root `batarangs`, and chooses standing/duck/air animation before calling `shootWeapon`
- `grappling`: enters grappling state and calls `shootGrappling`

Global grappling helpers recovered: `shootGrappling`, `startSwing`, `grapplingSwing`, `grapplingLines`, `radiansToDegrees`, `degreesToRadians`. The swing function references numeric limits/constants including 20, 80, 90 and 160 plus a `swingSpeed` state variable; raw bytecode is preserved in `core-functions.disasm.json.gz` so implementation can be translated branch-for-branch.

## Enemies and bosses

Animation/state symbols:

- goons **880/964/1365/1772**, 89 frames: `stand`, `walk`, `retreat`, `attack`, `hurt`, `fall`, `die`, `idle`, `intro`
- Penguin **1250**, 99 frames: `stand`, `walk`, `jump`, `dodge`, `block`, `attack`, `attack2`, `hurt`, `fall`, `die`
- eagle/bird **1393**, 50 frames: `fly`, `glide`, `attack`, `hurt`, `die`
- Kabuki **1919**, 154 frames: `idle`, `greet`, `stand`, `walk`, `jump`, `jumpOver`, `land`, `retreat`, `block`, `duck`, `attack`, `attackLand`, `attackHit`, `hurt`, `fall`, `die`
- maid **2037**, 99 frames: `stand`, `walk`, `retreat`, `attack`, `attack2`, `hurt`, `fall`, `die`, `idle`, `intro`
- CobbleBot **2135**, 104 frames: `idle`, `stand`, `walk`, `retreat`, `block`, `attack`, `attack2`, `attack3`, `hurt`, `dying`, `die`

Global AI functions are `bikerAI`, `goonAI`, `maidAI`, `penguinAI`, `kabukiAI`, `roboAI`, `birdAI`.

The exact per-level health/strength/speed setup is machine-readable in `manifests/combat-ai.json` and `manifests/level-constants-raw.json`.

## Camera

Five global camera routines exist: `cameraLogic`, `cameraLogic2`, `cameraLogic3`, `cameraLogic4`, `cameraLogic5`. They reference player position/direction/jumping, stage width, background scrolling, `camHeight`, `camJump`, `camMin`, `camMax`, `camOffset`, easing, and timing/FPS variables.

Do not replace camera movement with a generic "center Batman" camera. Per-level camera routine selection and constants are part of the original feel and are preserved in the manifests/raw bytecode.

## Levels and route

Gameplay root frames and core symbols:

1. frame 19 — `level1a` — game 696, level 152
2. frame 20 — `level1b` — game 1075, level 783
3. frame 21 — Penguin (`level1c`) — game 1251, level 1095
4. frame 22 — `card:escape` — card sprite 1267 -> `level2a`
5. frame 23 — `level2a` — game 1402, level 1317
6. frame 24 — `card:tv` — card sprite 1479 -> `level2b`
7. frame 25 — `level2b` — game 1496, level **1485**
8. frame 26 — `card:kabuki` — card sprite 1569 -> `level3a`
9. frame 27 — `level3a` — game 1779, level 1665
10. frame 28 — Kabuki (`level3b`) — game 1922, level 1812 -> `card:mansion`
11. frame 29 — `card:mansion` — card sprite 1947 -> `level4a`
12. frame 30 — `level4a` — game 2040, level 1962
13. frame 31 — CobbleBot (`level4b`) — game 2141, level 2051 -> `card:end`
14. frame 32 — ending movie/card sprite 2216 -> `start`

`card:bank`, `continue`, and `gameover` also exist in the dispatcher/checkpoint system.

## Tutorial overlay

Sprite **737**, 149 frames. Labels:

`blank`, `goRight`, `goLeft`, `jump`, `wrongway`, `glidedown`, `gameover`, `walk`, `findLab`, `fightBtns`, `escapeLab`, `useShaft`, `fight`, `toStreet`.

## Environment systems

Recovered global functions show bespoke level mechanics that must be translated rather than omitted:

- `pitLogic`
- `lavaLogic`
- `highwayLogic`
- `elevatorLogic`
- `liftLogic`
- `beltLogic`
- `stompLogic`
- `doorOpen`
- `bgTouch`, `bgHit`, `bgLight`, `bgCage`
- `clockTimer`
- `respawn`, `respawn2`

The named instance manifests identify which stages contain doors, elevators, belts, lifts, stomps, lava, lights, cages, highway, clock and checkpoint triggers.

## Original assets

The SWF contains:

- 342 DefineSprite definitions
- 1,218 DefineShape3
- 314 DefineShape
- 117 DefineShape2
- 79 bitmap definitions total
- 38 sounds
- 164 frame labels
- 37,067 PlaceObject2 records when recursively scanning sprite timelines

Bitmap decode status: **79/79 successful, zero errors**.

- 57 JPEG3
- 11 Lossless2
- 6 Lossless
- 3 JPEG2
- 2 JPEG/JPEGTables-style

The bitmap-fill linker reports **79 shapes with bitmap fills**, zero parse errors, preserving each bitmap ID and Flash fill matrix. This is enough to reproduce original raster-backed scenery during asset baking instead of drawing replacements.

Sound status: **38/38 indexed/extractable**:

- 33 MP3
- 5 SWF ADPCM

Named exported sounds include `swish`, `punch2`, `kick`, `item`, `eagle`, `ignition`, `bike`, `metal3`, `bam`, `metal2`, `explode`, `punch`, `metal1`.
