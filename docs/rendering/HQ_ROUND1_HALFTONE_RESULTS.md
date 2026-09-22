# HQ Rendering Round 1 — HALFTONE Presentation Results

Status: **implementation + CI complete; manual live-window quality check pending**

Branch: `hq-rendering-phase1-halftone`

Base planning branch: `hq-rendering-plan`

Renderer-repair baseline: `level1a-cxform-fidelity`

## Scope

Round 1 was intentionally presentation-only.

Changed:

- added GDI `SetStretchBltMode` FFI.
- added GDI `SetBrushOrgEx` FFI.
- added `HALFTONE = 4`.
- immediately before final `StretchDIBits`:
  - call `SetStretchBltMode(hdc, HALFTONE)`.
  - call `SetBrushOrgEx(hdc, 0, 0, NULL)`.

Not changed:

- logical framebuffer: still exactly 600×400.
- gameplay simulation: still 25 Hz.
- collision.
- camera.
- Batman animation data.
- HUD assets.
- tutorial assets.
- audio.
- Level 1A routing/fade behavior.

## Why SetBrushOrgEx is present

Microsoft documents that after setting `HALFTONE` stretch mode, applications should call `SetBrushOrgEx` to avoid brush misalignment.

The project follows that contract directly.

## CI

GitHub Actions:

- workflow run: **#46**
- run ID: **35745928562**
- branch head: `490554210a6e600bceccb244679eaea9ae634765`
- result: **success**

The full Level 1A pipeline passed:

1. canonical SWF download/hash verification.
2. bitmap/sound extraction.
3. collision reconstruction.
4. 279-frame Batman reconstruction validation.
5. CXFORM/color-alpha gates.
6. control-geometry leak rejection.
7. Round 1 presenter contract check.
8. Windows x64 release build.
9. headless gameplay/visual smoke.
10. artifact upload.

## Regression proof

The complete Run #46 headless visual-smoke artifact was compared byte-for-byte with the renderer-repaired Run #45 baseline.

Result:

- file lists identical: **yes**
- screenshots compared: **15**
- byte/SHA-256 identical screenshots: **15 / 15**
- mismatches: **0**

This is the expected result because Round 1 changes only the final Win32 presentation path. The underlying 600×400 renderer must remain unchanged.

Representative hashes:

| Screenshot | SHA-256 |
|---|---|
| `00_fade_start.png` | `fe797743d114f6f3dba95067b563387e6140ba3fcd5ec4eed5800255aaec9989` |
| `04_running.png` | `2751693c5ad9ed5aeb4cc2716b536a00f24def31cacdf384e4869f644359052e` |
| `05_jump.png` | `0ec4c094ba0a337976ab82e2b8ff2d327ad9579dbcebb036df30292306d06b11` |
| `06_glide.png` | `e58f955f561a176314586b204216dfeee772b5f4fd1e3b9defa2bf611b5656c9` |
| `09_to_street.png` | `3b75f3eb28d18caebf710a771d0b6537952ac8b677da18e0c1a2c0865b656c95` |
| `11_exit_fade_mid.png` | `427e8c1108ba69fc86acf7b72abde0f4bf48d25521946070987fe545f11ea558` |

## Round 1 build

Artifact:

`Batman-CobbleBot-Level1A-Round1-HALFTONE-Windows-x64.zip`

ZIP SHA-256:

`7d2bcec331627e8d680c0512bdf1bce03a900188d6fd870eed63ff3683e1766b`

Executable:

`batman_cobblebot_native.exe`

EXE size:

`4,254,720 bytes`

EXE SHA-256:

`80d3e225bc4fa0dbebba5e97e5484fca4d702ab3dabcde03887a0f97dedaee33`

## Expected visual effect

Round 1 only changes interpolation of the final 600×400 image when Windows stretches it to the client viewport.

Expected:

- less harsh staircase/block boundaries at fractional window scaling.
- somewhat smoother Batman/UI/text edges.
- no new source detail.

Not expected:

- fully crisp Batman.
- fully crisp HUD labels.
- fully crisp tutorial text.

Those require the later 3× asset/presentation architecture.

## Manual acceptance test

Use the same large window size that originally exposed pixelation.

Check:

- Batman silhouette.
- HUD labels.
- score/lives/Batarang numbers.
- tutorial text.
- movement/jump/glide for temporal stability.

If the result is smoother with no obvious blur regression, HALFTONE remains part of the HQ architecture.

Even if the improvement is modest, Round 1 is technically successful because it establishes the correct high-quality final Windows stretch before the larger 3× renderer work.

## Round 1 verdict

Engineering gate: **PASS**

Regression gate: **PASS — 15/15 headless screenshots byte-identical**

Manual presentation gate: **PENDING USER TEST**

Next phase after manual confirmation: **Phase 2 — generalize render surfaces while retaining pixel-identical legacy output.**
