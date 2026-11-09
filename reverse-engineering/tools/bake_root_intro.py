#!/usr/bin/env python3
"""Bake SWF root frame 18's original intro composition as 600x400 native frames.

The source SWF is verified/downloaded by CI. This script uses the original
depths, placement matrices and child timelines: no substitute art or dialogue.
Flash's script-driven stops, interactive text and event sounds remain a
separate fidelity gate and must not be confused with the visual bake.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

from bake_sprite_frames import load_shapes, load_text_defs, render_symbol


ROOT_INTRO_FRAME = 18
INTRO_FRAMES = 180
STAGE_SIZE = (600, 400)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("symbols_json", type=Path)
    ap.add_argument("root_placements", type=Path)
    ap.add_argument("bitmaps", type=Path)
    ap.add_argument("out", type=Path)
    ns = ap.parse_args()

    symbols = json.loads(ns.symbols_json.read_text(encoding="utf-8"))
    placements = json.loads(ns.root_placements.read_text(encoding="utf-8"))
    source_scene = [p for p in placements if p.get("frame") == ROOT_INTRO_FRAME]
    # Symbol 129 is a DefineButton2 interactive control, not a drawable
    # DefineSprite/DefineShape. The SWF sprite baker cannot render button
    # up/over/down state graphics yet. Preserve this omission explicitly.
    unsupported_controls = {129}
    # The root's fadeout movie clip (34) is script-stopped by Flash until
    # a dispatcher transition. Automatically playing all its frames would
    # cover the briefing with black and then loop it repeatedly.
    script_controlled_transitions = {34}
    scene = [
        dict(p,frame=0) for p in source_scene
        if p.get("character_id") not in (unsupported_controls | script_controlled_transitions)
    ]
    expected = {40, 122, 123, 126, 129}
    found = {p.get("character_id") for p in source_scene}
    if not expected.issubset(found):
        raise ValueError(f"Missing source scene characters: {sorted(expected-found)}")
    if int(symbols["123"]["frames"]) != INTRO_FRAMES or int(symbols["126"]["frames"]) != INTRO_FRAMES:
        raise ValueError("Intro child timeline duration changed; inspect source before changing the port")

    # Use the same source parser and composed transforms as all existing
    # original-Flash bake jobs. This synthetic 180-frame parent holds the
    # original root-frame-18 display list while child timelines advance.
    synthetic_id = -180
    symbols[str(synthetic_id)] = {
        "frames": INTRO_FRAMES,
        "placements": scene,
        "removals": [],
    }
    unsupported = set()
    shapes = load_shapes(ns.swf, unsupported)
    load_text_defs(ns.swf)
    bitmap_manifest = json.loads((ns.bitmaps/"manifest.json").read_text(encoding="utf-8"))
    bitmaps = {}
    for item in bitmap_manifest:
        if "file" not in item:
            continue
        with Image.open(ns.bitmaps/item["file"]) as image:
            bitmaps[item["id"]] = image.convert("RGBA")

    ns.out.mkdir(parents=True, exist_ok=True)
    manifest_frames = []
    missing = set()
    for tick in range(INTRO_FRAMES):
        stage = Image.new("RGBA", STAGE_SIZE, (0, 0, 0, 255))
        frame_missing = set()
        render_symbol(
            synthetic_id,tick,(1.,0.,0.,1.,0.,0.),stage,
            symbols,shapes,bitmaps,unsupported,frame_missing,
        )
        missing.update(frame_missing)
        filename = f"{tick:03}.png"
        stage.save(ns.out/filename,format="PNG",optimize=True)
        manifest_frames.append({"frame":tick,"animation":"root-intro","file":filename})

    manifest = {
        "symbol": synthetic_id,
        "source_root_frame": ROOT_INTRO_FRAME,
        "frame_count": INTRO_FRAMES,
        "size": list(STAGE_SIZE),
        "scale": [1.0,1.0],
        "source_bounds": [0,0,600,400],
        "anchor_in_bitmap": [0.0,0.0],
        "unsupported_fills": sorted(unsupported),
        "missing_bitmap_ids": sorted(missing),
        "root_placements": source_scene,
        "omitted_interactive_button_ids": sorted(unsupported_controls),
        "deferred_script_controlled_transition_ids": sorted(script_controlled_transitions),
        "frames": manifest_frames,
        "timing_note": "180 timeline samples, 25 fps; ActionScript-directed pauses/sound are not reproduced by visual baking",
    }
    (ns.out/"manifest.json").write_text(json.dumps(manifest,indent=2),encoding="utf-8")
    print("Intro source bake:",{k:manifest[k] for k in ("source_root_frame","frame_count","size","unsupported_fills","missing_bitmap_ids")})
    if unsupported or missing:
        raise SystemExit("Original intro assets incomplete; do not accept partial frames")


if __name__ == "__main__":
    main()
