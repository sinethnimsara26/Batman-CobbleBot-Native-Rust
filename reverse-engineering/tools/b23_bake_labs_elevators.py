#!/usr/bin/env python3
"""B23: Bake Labs 2A/2B elevator symbols (#1328 & #1330, excluding alpha-0 #1327 hitBox).

Offline build/CI asset generator and verifier from canonical cobblebot.swf:
- Validates canonical SWF SHA-256 and symbol #1328 (#1326 + hitBox #1327 cxform alpha=0)
  and symbol #1330 (#1329 + hitBox #1327 cxform alpha=0).
- Renders visible elevator frames (`frame 0 = #1328`, `frame 1 = #1330`) at 1.0x logical scale
  onto a shared union canvas with common `anchor_in_bitmap`, omitting #1327 hitBox geometry.
- Outputs PNGs + `manifest.json` compatible with `pack_sprite_frames.py --format-version 2`.
- Generates a visual side-by-side contact sheet (`elevators_proof.png`) & JSON audit report.
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
    load_shapes,
    render_symbol,
    symbol_bounds,
)

SOURCE_SHA = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
HITBOX_SHAPE_ID = 1327
ELEVATOR_SPECS = (
    (1328, 1326),
    (1330, 1329),
)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_source_elevators(symbols: dict) -> dict[int, dict]:
    verified = {}
    for sym_id, vis_shape_id in ELEVATOR_SPECS:
        sym = symbols[str(sym_id)]
        assert sym["frames"] == 1, (sym_id, sym["frames"])
        by_depth = {p["depth"]: p for p in sym["placements"]}
        vis = by_depth[1]
        hb = by_depth[2]
        assert vis["character_id"] == vis_shape_id, (sym_id, vis)
        assert hb["character_id"] == HITBOX_SHAPE_ID, (sym_id, hb)
        assert hb.get("name") == "hitBox", (sym_id, hb)
        assert hb["cxform"]["mult"] == [256, 256, 256, 0], (sym_id, hb["cxform"])
        verified[sym_id] = {"visual_placement": vis, "hitbox_placement": hb}
    return verified


def _padded_box(bounds: list[float], pad: int = 3) -> tuple[int, int, float, float]:
    min_x, min_y, max_x, max_y = bounds
    x0 = math.floor(min_x) - pad
    y0 = math.floor(min_y) - pad
    x1 = math.ceil(max_x) + pad
    y1 = math.ceil(max_y) + pad
    return x1 - x0, y1 - y0, float(-x0), float(-y0)


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
    verified = verify_source_elevators(symbols)

    unsupported: set = set()
    missing: set = set()
    shapes = load_shapes(args.swf, unsupported)
    bitmaps = load_bitmaps(args.bitmaps_dir)

    union_bounds = [math.inf, math.inf, -math.inf, -math.inf]
    hitbox_bounds = {}
    for sym_id, vis_shape_id in ELEVATOR_SPECS:
        vm = verified[sym_id]["visual_placement"]["matrix"]
        vxf = (vm["sx"], 0.0, 0.0, vm["sy"], vm["tx"], vm["ty"])
        vb = symbol_bounds(vis_shape_id, 0, vxf, symbols, shapes, 0, None)
        union_bounds[0] = min(union_bounds[0], vb[0])
        union_bounds[1] = min(union_bounds[1], vb[1])
        union_bounds[2] = max(union_bounds[2], vb[2])
        union_bounds[3] = max(union_bounds[3], vb[3])

        hm = verified[sym_id]["hitbox_placement"]["matrix"]
        hxf = (hm["sx"], 0.0, 0.0, hm["sy"], hm["tx"], hm["ty"])
        hitbox_bounds[sym_id] = symbol_bounds(HITBOX_SHAPE_ID, 0, hxf, symbols, shapes, 0, None)

    w, h, ax, ay = _padded_box(union_bounds)
    args.frames_out_dir.mkdir(parents=True, exist_ok=True)
    args.proof_dir.mkdir(parents=True, exist_ok=True)

    manifest_frames = []
    cropped_ims = {}
    report_elevators = []
    for f_idx, (sym_id, vis_shape_id) in enumerate(ELEVATOR_SPECS):
        vm = verified[sym_id]["visual_placement"]["matrix"]
        canvas = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        xf = (vm["sx"], 0.0, 0.0, vm["sy"], ax + vm["tx"], ay + vm["ty"])
        render_symbol(
            vis_shape_id, 0, xf, canvas, symbols, shapes, bitmaps,
            unsupported, missing, 0, None, stroke_source_over=True,
        )
        bbox = canvas.getbbox()
        assert bbox is not None, f"elevator #{sym_id} rendered empty"
        fname = f"frame_{f_idx:04d}.png"
        canvas.save(args.frames_out_dir / fname)
        manifest_frames.append({"frame": f_idx, "file": fname})
        cropped = canvas.crop(bbox)
        cropped_ims[sym_id] = cropped
        report_elevators.append({
            "symbol_id": sym_id,
            "visual_shape_id": vis_shape_id,
            "pack_frame": f_idx,
            "cropped_size": [cropped.width, cropped.height],
            "cropped_anchor": [ax - bbox[0], ay - bbox[1]],
            "hitbox_local_bounds": [round(v, 5) for v in hitbox_bounds[sym_id]],
            "sha256": sha256_bytes(cropped.tobytes()),
        })

    assert not unsupported, f"unsupported SWF fills: {sorted(unsupported)}"
    assert not missing, f"missing SWF bitmaps: {sorted(missing)}"

    manifest = {
        "batch": "B23",
        "frame_count": len(manifest_frames),
        "anchor_in_bitmap": [ax, ay],
        "frames": manifest_frames,
    }
    (args.frames_out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    # Side-by-side contact sheet for visual QA inspection
    pad = 24
    sheet_w = sum(im.width for im in cropped_ims.values()) + pad * 3
    sheet_h = max(im.height for im in cropped_ims.values()) + 40
    sheet = Image.new("RGBA", (sheet_w, sheet_h), (24, 24, 30, 255))
    draw = ImageDraw.Draw(sheet)
    cx = pad
    for sym_id, _ in ELEVATOR_SPECS:
        im = cropped_ims[sym_id]
        sheet.alpha_composite(im, (cx, 28))
        draw.text((cx, 6), f"elevator #{sym_id} ({im.width}x{im.height})", fill=(220, 220, 220, 255))
        cx += im.width + pad
    sheet.save(args.proof_dir / "elevators_proof.png")

    report = {
        "batch": "B23",
        "frame_count": len(manifest_frames),
        "union_canvas": [w, h],
        "union_anchor": [ax, ay],
        "elevators": report_elevators,
    }
    (args.proof_dir / "labs-elevators-report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"B23 baked 2 elevator symbols (#1328/#1330, union {w}x{h}) into {args.frames_out_dir}")


if __name__ == "__main__":
    main()
