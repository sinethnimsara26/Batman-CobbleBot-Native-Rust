# Native implementation milestone 0.1

Status: **BUILDS SUCCESSFULLY ON WINDOWS**

Validated source head:

`5cedfc90fa8a9158cb558db13bda55ad34d5d328`

GitHub Actions:

- workflow: `build-batman-cobblebot-native`
- run: **#2**
- run id: `35433092621`
- Windows x64 compile: success
- smoke-check: success
- artifact upload: success

Executable:

- name: `batman_cobblebot_native.exe`
- size: **601,600 bytes**
- SHA-256: `cf3eb0612c404c42edabb998878ebe64db0dc6b13ee6581d70fea9b8f9d47fc2`
- PE: x86-64 Windows GUI
- MSVC CRT: statically linked
- observed imports: USER32.dll, GDI32.dll, KERNEL32.dll, api-ms-win-core-synch-l1-2-0.dll, ntdll.dll

## Implemented native shell

- 600 x 400 logical framebuffer
- aspect-preserving Win32 presentation
- DPI-aware native window
- fixed 25 Hz simulation / 40 ms ticks
- independent render cadence
- key-down/key-up input state
- no Flash Player, browser, WebView, Java, Ruffle, Electron, SDL or game engine at runtime

## Gameplay foundation

Translated from the recovered specification:

- player dimensions 50 x 75
- walk speed 6
- run speed 19
- jump velocity -42
- gravity 5
- glide gravity 1
- glide entry while descending
- Left / Right movement
- Space jump / glide
- Up / Down attack modifiers
- S punch states
- D kick states
- A gadget/Batarang animation states
- D+S cape-spin state
- LIFE cheat -> 99 lives
- 16-pixel penetration-correction limit
- collision probes use the exact recovered level1a SWF ground mask
- wall/feet/head probes separated for later branch-for-branch collision fidelity

## Original assets already used

No hand-drawn replacement character art was introduced.

The milestone embeds:

- recovered Batman artwork
- recovered level1a proof artwork
- exact SWF-derived level1a collision mask

The `png` crate is the only non-std Cargo dependency. It is a Rust library used only to decode embedded recovered PNGs.

## Fidelity warning

This milestone is **not** a 99% visual build yet and must never be presented as one.

It establishes the native engine while the complete artwork/timeline bake is being built. Batman currently uses a recovered static proof render rather than the full 271-frame original animation. The visible level layer is the recovered level1a proof render rather than the complete original display-list/timeline bake. The camera currently uses a temporary native smoothing function rather than the recovered `cameraLogic` AVM1 translation.

These are explicit implementation placeholders, not unknown behavior.

## Next hard milestone

1. Reconstruct the canonical SWF during asset baking.
2. Export Batman sprite 691, all 271 frames, preserving transparency and registration.
3. Bake complete level1a display-list scenery at native fidelity.
4. Replace static Batman proof render with state-label/frame ranges from `animation-labels.json`.
5. Translate `cameraLogic` branch-for-branch.
6. Translate exact `checkGround/checkCeiling/checkWall/checkEdge` probe behavior from AVM1.
7. Add tutorial overlay sprite 737 and original HUD.
8. Only then begin level1b enemy-combat validation.

The 99%-fidelity goal remains the acceptance criterion for the finished port.
