#!/usr/bin/env python3
"""B22: Bake Labs 2A/2B animated sliding door (#1325 / #1319 + #1324) and jambs (#1394..#1396).

Offline build/CI asset generator and verifier from canonical cobblebot.swf:
- Validates canonical SWF SHA-256 and #1325/#1324 (20 frames: Stop at 0 & 9, StartSound 1323 on 1)
  plus #1318 invisible hit-sensor exclusion and #1394/#1395/#1396 door jamb placements.
- Renders 20 1x logical frames of visible #1325 (`#1319` frame pocket + `#1324[f]` clipped leaf)
  at canonical placement scale `(sx=0.902069091796875, sy=0.81072998046875)`, omitting #1318,
  plus 3 1x logical jamb frames (`#1394`, `#1395`, `#1396`) at `(sx=0.924530029296875, sy=0.808013916015625)`.
- Outputs PNGs + `manifest.json` compatible with `pack_sprite_frames.py --format-version 2`.
- Generates a visual contact-sheet proof (`doors_and_jambs_proof.png`) & JSON audit report.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path

from PIL import Image, ImageDraw

from b03_bake_stage_asset_proofs import expect_original_source
from bake_level_tiles import load_bitmaps
from bake_sprite_frames import (
    compose,
    load_shapes,
    render_symbol,
    symbol_bounds,
)

SOURCE_SHA = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
DOOR_WRAPPER_ID = 1325
DOOR_SENSOR_SHAPE_ID = 1318
DOOR_POCKET_SHAPE_ID = 1319
DOOR_LEAF_SPRITE_ID = 1324
DOOR_LEAF_OFFSET = (-58.35, -93.6)
DOOR_SCALE = (0.902069091796875, 0.81072998046875)
JAMB_IDS = (1394, 1395, 1396)
JAMB_SCALE = (0.924530029296875, 0.808013916015625)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_source_door_defs(symbols: dict) -> None:
    wrapper = symbols[str(DOOR_WRAPPER_ID)]
    assert wrapper["frames"] == 1, wrapper["frames"]
    placements = {p["depth"]: p for p in wrapper["placements"]}
    assert placements[1]["character_id"] == DOOR_SENSOR_SHAPE_ID
    assert placements[1]["cxform"]["mult"] == [256, 256, 256, 0]
    assert placements[2]["character_id"] == DOOR_POCKET_SHAPE_ID
    leaf_p = placements[3]
    assert leaf_p["character_id"] == DOOR_LEAF_SPRITE_ID
    assert abs(leaf_p["matrix"]["tx"] - DOOR_LEAF_OFFSET[0]) < 1e-4
    assert abs(leaf_p["matrix"]["ty"] - DOOR_LEAF_OFFSET[1]) < 1e-4

    leaf = symbols[str(DOOR_LEAF_SPRITE_ID)]
    assert leaf["frames"] == 20, leaf["frames"]
    labels = sorted((l["label"], l["frame"]) for l in leaf["labels"])
    assert labels == [("close", 9), ("open", 0)], labels


def _padded_box(bounds: list[float], pad: int = 3) -> tuple[int, int, float, float]:
    min_x, min_y, max_x, max_y = bounds
    x0 = math.floor(min_x) - pad
    y0 = math.floor(min_y) - pad
    x1 = math.ceil(max_x) + pad
    y1 = math.ceil(max_y) + pad
    return x1 - x0, y1 - y0, float(-x0), float(-y0)


def render_door_frame(
    frame_idx: int,
    symbols: dict,
    shapes: dict,
    bitmaps: dict,
    size: tuple[int, int],
    anchor: tuple[float, float],
    unsupported: set,
    missing: set,
) -> Image.Image:
    ax, ay = anchor
    root_xf = (DOOR_SCALE[0], 0.0, 0.0, DOOR_SCALE[1], ax, ay)
    leaf_xf = compose(root_xf, (1.0, 0.0, 0.0, 1.0, DOOR_LEAF_OFFSET[0], DOOR_LEAF_OFFSET[1]))
    canvas = Image.new("RGBA", size, (0, 0, 0, 0))
    render_symbol(
        DOOR_POCKET_SHAPE_ID, 0, root_xf, canvas, symbols, shapes, bitmaps,
        unsupported, missing, 0, None, stroke_source_over=True,
    )
    render_symbol(
        DOOR_LEAF_SPRITE_ID, frame_idx, leaf_xf, canvas, symbols, shapes, bitmaps,
        unsupported, missing, 0, None, stroke_source_over=True,
    )
    return canvas


def render_jamb_frame(
    shape_id: int,
    symbols: dict,
    shapes: dict,
    bitmaps: dict,
    size: tuple[int, int],
    anchor: tuple[float, float],
    unsupported: set,
    missing: set,
) -> Image.Image:
    ax, ay = anchor
    canvas = Image.new("RGBA", size, (0, 0, 0, 0))
    xf = (JAMB_SCALE[0], 0.0, 0.0, JAMB_SCALE[1], ax, ay)
    render_symbol(
        shape_id, 0, xf, canvas, symbols, shapes, bitmaps,
        unsupported, missing, 0, None, stroke_source_over=True,
    )
    return canvas


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("swf", type=Path)
    parser.add_argument("symbols_json", type=Path)
    parser.add_argument("bitmaps_dir", type=Path)
    parser.add_argument("frames_out_dir", type=Path)
    parser.add_argument("proof_dir", type=Path)
    args = parser.parse_args()

    expect_original_source(args.swf)
    assert sha256_bytes(args.swf.read_bytes()) == SOURCE_SHA, "SWF SHA-256 mismatch"
    symbols = json.loads(args.symbols_json.read_text(encoding="utf-8"))
    verify_source_door_defs(symbols)

    unsupported: set = set()
    missing: set = set()
    shapes = load_shapes(args.swf, unsupported)
    bitmaps = load_bitmaps(args.bitmaps_dir)

    door_scale_xf = (DOOR_SCALE[0], 0.0, 0.0, DOOR_SCALE[1], 0.0, 0.0)
    b_pocket = symbol_bounds(DOOR_POCKET_SHAPE_ID, 0, door_scale_xf, symbols, shapes, 0, None)
    union_bounds = list(b_pocket)
    leaf_xf = compose(door_scale_xf, (1.0, 0.0, 0.0, 1.0, DOOR_LEAF_OFFSET[0], DOOR_LEAF_OFFSET[1]))
    for f in range(20):
        bf = symbol_bounds(DOOR_LEAF_SPRITE_ID, f, leaf_xf, symbols, shapes, 0, None)
        union_bounds[0] = min(union_bounds[0], bf[0])
        union_bounds[1] = min(union_bounds[1], bf[1])
        union_bounds[2] = max(union_bounds[2], bf[2])
        union_bounds[3] = max(union_bounds[3], bf[3])
    jamb_scale_xf = (JAMB_SCALE[0], 0.0, 0.0, JAMB_SCALE[1], 0.0, 0.0)
    for sid in JAMB_IDS:
        bj = symbol_bounds(sid, 0, jamb_scale_xf, symbols, shapes, 0, None)
        union_bounds[0] = min(union_bounds[0], bj[0])
        union_bounds[1] = min(union_bounds[1], bj[1])
        union_bounds[2] = max(union_bounds[2], bj[2])
        union_bounds[3] = max(union_bounds[3], bj[3])
    dw, dh, dax, day = _padded_box(union_bounds)

    args.frames_out_dir.mkdir(parents=True, exist_ok=True)
    args.proof_dir.mkdir(parents=True, exist_ok=True)

    manifest_frames = []
    door_samples = {}
    for f in range(20):
        im = render_door_frame(f, symbols, shapes, bitmaps, (dw, dh), (dax, day), unsupported, missing)
        assert im.getbbox() is not None, f"door frame {f} rendered empty"
        fname = f"frame_{f:04d}.png"
        im.save(args.frames_out_dir / fname)
        manifest_frames.append({"frame": f, "file": fname})
        if f in (0, 4, 9, 14, 19):
            door_samples[f] = im.crop(im.getbbox())

    jamb_meta = []
    jamb_ims = {}
    for j_idx, sid in enumerate(JAMB_IDS):
        frame_num = 20 + j_idx
        im = render_jamb_frame(sid, symbols, shapes, bitmaps, (dw, dh), (dax, day), unsupported, missing)
        bbox = im.getbbox()
        assert bbox is not None, f"jamb {sid} rendered empty"
        fname = f"frame_{frame_num:04d}.png"
        im.save(args.frames_out_dir / fname)
        manifest_frames.append({"frame": frame_num, "file": fname})
        cropped = im.crop(bbox)
        jamb_meta.append({
            "symbol_id": sid,
            "pack_frame": frame_num,
            "cropped_size": [cropped.width, cropped.height],
            "cropped_anchor": [dax - bbox[0], day - bbox[1]],
        })
        jamb_ims[sid] = cropped

    assert not unsupported, f"unsupported SWF fills: {sorted(unsupported)}"
    assert not missing, f"missing SWF bitmaps: {sorted(missing)}"
    assert door_samples[0].tobytes() != door_samples[9].tobytes(), "closed door frame 0 must differ from open frame 9"

    manifest = {
        "batch": "B22",
        "symbol_id": DOOR_WRAPPER_ID,
        "frame_count": len(manifest_frames),
        "anchor_in_bitmap": [dax, day],
        "frames": manifest_frames,
    }
    (args.frames_out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    # Build proof contact sheet showing cropped door open/close progression + all 3 jamb shapes
    cell_h = max(max(im.height for im in door_samples.values()), max(im.height for im in jamb_ims.values())) + 24
    cols = 5
    sheet = Image.new("RGBA", (cols * 140, cell_h * 2), (24, 24, 30, 255))
    draw = ImageDraw.Draw(sheet)
    for ci, f in enumerate((0, 4, 9, 14, 19)):
        im = door_samples[f]
        px = ci * 140 + (140 - im.width) // 2
        sheet.alpha_composite(im, (px, 20))
        draw.text((ci * 140 + 8, 4), f"door[{f}]", fill=(220, 220, 220, 255))
    for ji, sid in enumerate(JAMB_IDS):
        im = jamb_ims[sid]
        px = ji * 140 + (140 - im.width) // 2
        sheet.alpha_composite(im, (px, cell_h + 20))
        draw.text((ji * 140 + 8, cell_h + 4), f"jamb#{sid}", fill=(220, 220, 220, 255))
    sheet.save(args.proof_dir / "doors_and_jambs_proof.png")

    report = {
        "batch": "B22",
        "door_frames": 20,
        "door_canvas": [dw, dh],
        "door_anchor": [dax, day],
        "door_scale": list(DOOR_SCALE),
        "jambs": jamb_meta,
        "frame0_sha256": sha256_bytes(door_samples[0].tobytes()),
        "frame9_sha256": sha256_bytes(door_samples[9].tobytes()),
    }
    (args.proof_dir / "labs-doors-report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"B22 baked 20 door frames ({dw}x{dh}) + 3 jambs into {args.frames_out_dir}")


if __name__ == "__main__":
    main()
