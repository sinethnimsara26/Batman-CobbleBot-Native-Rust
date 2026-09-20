# RE tools

All scripts are development-only Python utilities. The final Rust game must not invoke them.

- `run_all.py` — one-command pipeline
- `extract_projector.py` — finds the projector's outer zlib stream, extracts CWS, canonicalizes to FWS, hashes all source forms
- `swf_re.py` — recursive SWF/timeline parser and AVM1 disassembler; importantly handles function bodies outside ActionDefineFunction record lengths
- `extract_bitmaps.py` — decodes JPEG/JPEG2/JPEG3/Lossless/Lossless2 images
- `extract_sounds.py` — exports MP3 frames and losslessly preserves SWF ADPCM streams
- `generate_manifests.py` — builds function, animation, level-instance, routing, and compact raw-disassembly manifests
- `extract_shape_fills.py` — maps bitmap definitions into vector fill styles/matrices
- `render_collision_masks.py` — parses original vector ground shapes and creates level-space 1-bit masks

`Pillow` is the only third-party Python dependency and is used only for bitmap decode/PNG output and mask rasterization.
