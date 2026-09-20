# Reverse-engineering package

This directory makes the CobbleBot analysis durable and reproducible instead of leaving critical knowledge in a chat transcript.

## What is preserved

- exact source/projector/SWF fingerprints
- SWF movie metadata and tag inventory
- corrected AVM1 disassembly with nested function bodies preserved
- index of all 45 global gameplay functions
- Batman/enemy/boss animation labels and frame counts
- exact keyboard controls and core player-physics constants
- level-by-level camera and enemy initialization constants
- all named game/level placements and transforms
- root timeline and level/card routing evidence
- original collision `ground` symbols plus generated level-space collision masks
- all 79 bitmap definitions decoded without errors
- mapping from bitmap definitions to the 79 vector shapes that use bitmap fills, including fill matrices
- all 38 DefineSound records indexed and extractable (33 MP3, 5 SWF ADPCM)
- proof renders demonstrating that original vector/level/collision data can be recovered

## One-command reproduction

With Python and Pillow installed:

```powershell
python -m pip install -r reverse-engineering/tools/requirements-dev.txt
python reverse-engineering/tools/run_all.py "The Batman - The CobbleBot Caper.exe" re-output
```

The pipeline performs:

1. projector -> embedded CWS -> canonical uncompressed FWS extraction
2. recursive SWF/timeline/AVM1 inventory
3. bitmap decode
4. sound extraction
5. function/animation/level/routing manifests
6. bitmap-fill linkage extraction
7. collision-ground rasterization

See `RE_COMPLETE.md` for the completion checklist and implementation handoff.

## Optional independent cross-check

JPEXS FFDec is useful as an independent visual/decompilation cross-check, but is **not required** by this pipeline and is not vendored here. During this analysis the current upstream release was **FFDec 26.3.0** (2026-09-14); GitHub reported SHA-256 `35f4930eb7c380afe66f2117f90b006deac0631473ad7500bb39c78f68645ecd` for `ffdec_26.3.0.zip`.
