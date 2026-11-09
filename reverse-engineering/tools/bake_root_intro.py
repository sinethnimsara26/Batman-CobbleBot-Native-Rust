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
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageChops, ImageStat
import struct

from bake_sprite_frames import load_shapes, load_text_defs, render_symbol, symbol_bounds
from render_collision_masks import affine
from swf_re import read_rect, read_matrix, read_cxform_with_alpha, tags


ROOT_INTRO_FRAME = 18
INTRO_FRAMES = 180
STAGE_SIZE = (600, 400)
GALLERY_TICKS = (0, 20, 40, 60, 80, 100, 120, 140, 160, 179)


def recover_button_up_state(swf: Path, button_id: int):
    """Read original DefineButton2 UP-state records into a synthetic sprite.

    The source sprite renderer supports shape/text placement but cannot read
    ButtonRecord arrays directly. Preserve child matrix/depth/CXFORM and
    exclude nonvisual hit-test-only/down/hover records.
    """
    data = swf.read_bytes()
    _, pos = read_rect(data, 8)
    pos += 4
    for code, payload, _, _ in tags(data, pos, len(data)):
        if code != 34 or len(payload) < 6:
            continue
        sid = struct.unpack_from("<H", payload, 0)[0]
        if sid != button_id:
            continue
        action_offset = struct.unpack_from("<H", payload, 3)[0]
        records = []
        up = []
        position = 5
        while position < len(payload):
            flags = payload[position]
            position += 1
            if flags == 0:
                break
            if flags & 0xF0:
                raise ValueError(f"DefineButton2 {sid} uses unsupported button flags 0x{flags:02x}")
            if position+4 > len(payload):
                raise ValueError("Truncated DefineButton2 ButtonRecord")
            character_id,depth = struct.unpack_from("<HH", payload, position)
            position += 4
            matrix,position = read_matrix(payload,position)
            cxform,position = read_cxform_with_alpha(payload,position)
            record = {"states":flags & 0x0F, "character_id":character_id,
                      "depth":depth, "matrix":matrix, "cxform":cxform}
            records.append(record)
            if flags & 0x01:
                up.append({"frame":0,"tag_offset":len(up),"depth":depth,
                           "character_id":character_id,"matrix":matrix,"cxform":cxform})
        if not up:
            raise ValueError(f"DefineButton2 {sid} has no visible UP-state records")
        return up, {"symbol":sid,"up_state_record_count":len(up),
                    "all_state_record_count":len(records),
                    "up_state_character_ids":[r["character_id"] for r in up],
                    "action_offset":action_offset}
    raise ValueError(f"Expected DefineButton2 source symbol {button_id} was not found")


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
    # Recover original button 129's visual UP state in source order instead of
    # presenting an empty outline or substituting a made-up native button.
    button_up,button_report = recover_button_up_state(ns.swf,129)
    symbols["129"] = {"frames":1,"placements":button_up,"removals":[]}
    # The root's fadeout movie clip (34) is script-stopped by Flash until
    # a dispatcher transition. Automatically playing all its frames would
    # cover the briefing with black and then loop it repeatedly.
    script_controlled_transitions = {34}
    scene = [
        dict(p,frame=0) for p in source_scene
        if p.get("character_id") not in script_controlled_transitions
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
    # Compute the clickable SKIP rect from original source UP-state geometry,
    # including the original button placement matrix on SWF root frame 18.
    button_placement = next(p for p in source_scene if p.get("character_id")==129)
    bounds = symbol_bounds(129,0,affine(button_placement.get("matrix",{})),symbols,shapes)
    if not all(math.isfinite(v) for v in bounds):
        raise ValueError("Recovered button has no visible geometry")
    click_rect = [
        max(0,math.floor(bounds[0])), max(0,math.floor(bounds[1])),
        min(600,math.ceil(bounds[2])), min(400,math.ceil(bounds[3]))
    ]
    if not (click_rect[0]<click_rect[2] and click_rect[1]<click_rect[3]):
        raise ValueError(f"Recovered SKIP button bounds invalid: {click_rect}")
    # Source-generated Rust constant; never hardcode a guessed hotspot in the
    # native window. The CI-generated file is embedded only at build time.
    ns.out.mkdir(parents=True,exist_ok=True)
    (ns.out/"intro_button_bounds.rs").write_text(
        "const INTRO_SKIP_RECT:(i32,i32,i32,i32)=("+",".join(map(str,click_rect))+");\n",
        encoding="utf-8"
    )
    bitmap_manifest = json.loads((ns.bitmaps/"manifest.json").read_text(encoding="utf-8"))
    bitmaps = {}
    for item in bitmap_manifest:
        if "file" not in item:
            continue
        with Image.open(ns.bitmaps/item["file"]) as image:
            bitmaps[item["id"]] = image.convert("RGBA")

    ns.out.mkdir(parents=True, exist_ok=True)
    manifest_frames = []
    frame_diagnostics = []
    previous_rgb = None
    missing = set()
    gallery = Image.new("RGB", (300*5,220*2),(18,18,22))
    gallery_draw = ImageDraw.Draw(gallery)
    for tick in range(INTRO_FRAMES):
        stage = Image.new("RGBA", STAGE_SIZE, (0, 0, 0, 255))
        frame_missing = set()
        render_symbol(
            synthetic_id,tick,(1.,0.,0.,1.,0.,0.),stage,
            symbols,shapes,bitmaps,unsupported,frame_missing,
        )
        missing.update(frame_missing)
        # Inspect all 180 frames: ten contact-sheet samples can miss one-frame flashes.
        # A dark source scene is not automatically a rendering bug.
        rgb = stage.convert("RGB")
        luma = rgb.convert("L")
        histogram = luma.histogram()
        changed_fraction = 0.0
        if previous_rgb is not None:
            diff = ImageChops.difference(previous_rgb, rgb).convert("L")
            changed_fraction = sum(diff.histogram()[24:]) / (STAGE_SIZE[0]*STAGE_SIZE[1])
        frame_diagnostics.append({
            "frame":tick,
            "mean_luma":round(ImageStat.Stat(luma).mean[0],3),
            "near_black_fraction":round(sum(histogram[:9])/(STAGE_SIZE[0]*STAGE_SIZE[1]),6),
            "changed_pixel_fraction_from_previous":round(changed_fraction,6),
        })
        previous_rgb = rgb
        filename = f"{tick:03}.png"
        stage.save(ns.out/filename,format="PNG",optimize=True)
        manifest_frames.append({"frame":tick,"animation":"root-intro","file":filename})
        if tick in GALLERY_TICKS:
            idx=GALLERY_TICKS.index(tick)
            x=(idx%5)*300
            y=(idx//5)*220
            gallery.paste(stage.convert("RGB").resize((300,200),Image.Resampling.BILINEAR),(x,y))
            gallery_draw.text((x+6,y+202),f"SWF frame {tick:03} / 180",fill=(245,240,220))

    (ns.out/"intro-frame-diagnostics.json").write_text(
        json.dumps({
            "source_root_frame": ROOT_INTRO_FRAME, "fps": 25, "frame_count": INTRO_FRAMES,
            "metrics": "grayscale RGB; near-black 0-8; changed difference >=24",
            "frames": frame_diagnostics,
            "note": "Diagnostic only; distinguish original scene fades from unwanted native flicker.",
        }, indent=2), encoding="utf-8")
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
        "recovered_button_up_state": button_report,
        "skip_button_stage_bounds": click_rect,
        "deferred_script_controlled_transition_ids": sorted(script_controlled_transitions),
        "frames": manifest_frames,
        "timing_note": "180 timeline samples, 25 fps; ActionScript-directed pauses/sound are not reproduced by visual baking",
    }
    (ns.out/"manifest.json").write_text(json.dumps(manifest,indent=2),encoding="utf-8")
    gallery.save(ns.out/"intro-contact-sheet.png",format="PNG",optimize=True)
    print("Intro source bake:",{k:manifest[k] for k in ("source_root_frame","frame_count","size","unsupported_fills","missing_bitmap_ids")})
    if unsupported or missing:
        raise SystemExit("Original intro assets incomplete; do not accept partial frames")


if __name__ == "__main__":
    main()
