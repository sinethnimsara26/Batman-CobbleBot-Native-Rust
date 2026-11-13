#!/usr/bin/env python3
"""B25: Verify & bake Labs 2A pickups (#1401 health01), spotlight halo (#1399),
and LEVEL 2 title banner (#1408), plus checkpoint/audio source verification.

Offline build/CI verifier & asset generator from canonical cobblebot.swf:
- Validates canonical SWF SHA-256.
- Validates Level 2A (#1402) health01 (#1401), halo (#1399), 9 item_br* (#695),
  checkpoint_2a_01/02/tv/end, overlay (#737) labels, AVM1 followPlayer/lifeLogic/itemLogic,
  and root frame 23/25 sound IDs (#1409/#1504) + combat MP3 IDs.
- Bakes 1.0x frames into:
  - pickups_dir: frame 0 = health01 (#1401 @ 0.876053), frame 1 = halo (#1399 @ 1.0)
  - title_dir: 100 frames of #1408 (Level 2A levelTitle @ 1.0x)
- Generates visual contact sheet + JSON report.
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

HEALTH_SYMBOL_ID = 1401
HEALTH_SCALE = 0.8760528564453125
HALO_SYMBOL_ID = 1399
TITLE_SYMBOL_ID = 1408

EXPECTED_2A_BATARANG_NAMES = {
    "item_br01",
    "item_br02",
    "item_br03",
    "item_br04",
    "item_br05",
    "item_br06",
    "item_br07",
    "item_br08",
    "item_br10",
}

EXPECTED_2A_CHECKPOINTS = {
    "checkpoint_2a_01",
    "checkpoint_2a_02",
    "checkpoint_2a_end",
    "checkpoint_2a_tv",
}

EXPECTED_MP3_SOUNDS = {
    1: "swish",
    2: "punch2",
    4: "item",
    9: "bam",
    11: "explode",
    12: "punch",
    13: "metal1",
    130: "sound_0130",
    751: "sound_0751",
    1076: "sound_1076",
    1252: "sound_1252",
    1323: "sound_1323",
    1409: "sound_1409",
    1504: "sound_1504",
}


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_source_contracts(
    symbols: dict, actions: list, sounds_manifest: list[dict]
) -> dict:
    s1402 = symbols["1402"]
    by_name = {p.get("name"): p for p in s1402["placements"] if p.get("name")}

    halo = by_name["halo"]
    assert halo["character_id"] == HALO_SYMBOL_ID, halo
    assert halo["depth"] == 413, halo
    assert abs(halo["matrix"]["tx"] - 3140.0) < 0.01, halo
    assert abs(halo["matrix"]["ty"] - 658.7) < 0.01, halo

    health = by_name["health01"]
    assert health["character_id"] == HEALTH_SYMBOL_ID, health
    assert health["depth"] == 415, health
    assert abs(health["matrix"]["sx"] - HEALTH_SCALE) < 1e-4, health

    br_names = {
        p["name"]
        for p in s1402["placements"]
        if p.get("name", "").startswith("item_br")
    }
    assert br_names == EXPECTED_2A_BATARANG_NAMES, br_names

    cp_names = {
        p["name"]
        for p in s1402["placements"]
        if p.get("name", "").startswith("checkpoint_2a_")
    }
    assert cp_names == EXPECTED_2A_CHECKPOINTS, cp_names

    # Verify overlay #737 label frames used in Level 2A (findLab=66, glidedown=39)
    labels_737 = {l["label"]: l["frame"] for l in symbols["737"]["labels"]}
    assert labels_737["findLab"] == 66, labels_737
    assert labels_737["glidedown"] == 39, labels_737
    assert symbols[str(TITLE_SYMBOL_ID)]["frames"] == 100

    # Verify AVM1 pools
    f23_pools = [
        op.get("pool", [])
        for b in actions
        if b.get("context") == "root" and b.get("frame") == 23
        for op in b.get("actions", [])
        if op.get("op") == "ConstantPool"
    ]
    flat_f23 = {s for pool in f23_pools for s in pool}
    assert {"followPlayer", "halo", "itemLogic", "lifeLogic"} <= flat_f23
    assert {"checkpoint_2a_01", "findLab", "checkpoint_2a_02", "glidedown"} <= flat_f23

    # Verify extracted sound manifest IDs & MP3 format
    by_sound_id = {x["id"]: x for x in sounds_manifest}
    for sid, expected_name in EXPECTED_MP3_SOUNDS.items():
        rec = by_sound_id[sid]
        assert rec["export_name"] == expected_name, rec
        assert rec["format_name"] == "mp3", rec

    return {
        "halo_depth": halo["depth"],
        "health_depth": health["depth"],
        "item_br_count": len(br_names),
        "checkpoints": sorted(cp_names),
        "find_lab_frame": labels_737["findLab"],
        "glidedown_frame": labels_737["glidedown"],
        "title_2a_frames": symbols[str(TITLE_SYMBOL_ID)]["frames"],
    }


def render_single_frame(
    symbol_id: int,
    frame: int,
    scale: float,
    bounds: list[float],
    symbols: dict,
    shapes: dict,
    bitmaps: dict,
    unsupported: set,
    missing: set,
) -> tuple[Image.Image, float, float]:
    min_x, min_y, max_x, max_y = bounds
    x0 = math.floor(min_x) - 2
    y0 = math.floor(min_y) - 2
    x1 = math.ceil(max_x) + 2
    y1 = math.ceil(max_y) + 2
    w = max(1, x1 - x0)
    h = max(1, y1 - y0)
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    render_symbol(
        symbol_id,
        frame,
        (scale, 0.0, 0.0, scale, float(-x0), float(-y0)),
        im,
        symbols,
        shapes,
        bitmaps,
        unsupported,
        missing,
        0,
    )
    return im, float(-x0), float(-y0)


def write_pack_dir(
    out_dir: Path,
    anchor: tuple[float, float],
    frames_info: list[tuple[int, str, int, int]],
) -> None:
    manifest = {
        "source": "cobblebot.swf",
        "frame_count": len(frames_info),
        "anchor_in_bitmap": [anchor[0], anchor[1]],
        "logical_pixel_scale": 1.0,
        "frames": [
            {
                "frame": idx,
                "file": fname,
                "width": w,
                "height": h,
            }
            for idx, fname, w, h in frames_info
        ],
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2), encoding="utf-8"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("swf", type=Path)
    parser.add_argument("symbols_json", type=Path)
    parser.add_argument("actions_json", type=Path)
    parser.add_argument("sounds_manifest", type=Path)
    parser.add_argument("pickups_dir", type=Path)
    parser.add_argument("title_dir", type=Path)
    parser.add_argument("--bitmaps", type=Path, required=True)
    parser.add_argument("--proof", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()

    expect_original_source(args.swf)
    assert sha256_bytes(args.swf.read_bytes()) == SOURCE_SHA, "SWF SHA-256 mismatch"
    symbols = json.loads(args.symbols_json.read_text(encoding="utf-8"))
    actions = json.loads(args.actions_json.read_text(encoding="utf-8"))
    sounds_manifest = json.loads(args.sounds_manifest.read_text(encoding="utf-8"))

    verified = verify_source_contracts(symbols, actions, sounds_manifest)

    unsupported: set[str] = set()
    missing: set[int] = set()
    shapes = load_shapes(args.swf, unsupported)
    bitmaps = load_bitmaps(args.bitmaps)

    args.pickups_dir.mkdir(parents=True, exist_ok=True)
    args.title_dir.mkdir(parents=True, exist_ok=True)
    args.proof.parent.mkdir(parents=True, exist_ok=True)
    args.report.parent.mkdir(parents=True, exist_ok=True)

    health_b = symbol_bounds(
        HEALTH_SYMBOL_ID, 0, (HEALTH_SCALE, 0.0, 0.0, HEALTH_SCALE, 0.0, 0.0), symbols, shapes, 0, None
    )
    halo_b = symbol_bounds(
        HALO_SYMBOL_ID, 0, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0), symbols, shapes, 0, None
    )
    pickup_union = [
        min(health_b[0], halo_b[0]),
        min(health_b[1], halo_b[1]),
        max(health_b[2], halo_b[2]),
        max(health_b[3], halo_b[3]),
    ]

    # Frame 0: health01 (#1401 @ 0.876053)
    health_im, health_ax, health_ay = render_single_frame(
        HEALTH_SYMBOL_ID, 0, HEALTH_SCALE, pickup_union, symbols, shapes, bitmaps, unsupported, missing
    )
    health_name = "frame_0000.png"
    health_im.save(args.pickups_dir / health_name)

    # Frame 1: halo (#1399 @ 1.0)
    halo_im, halo_ax, halo_ay = render_single_frame(
        HALO_SYMBOL_ID, 0, 1.0, pickup_union, symbols, shapes, bitmaps, unsupported, missing
    )
    halo_name = "frame_0001.png"
    halo_im.save(args.pickups_dir / halo_name)

    assert halo_im.getpixel((round(halo_ax), round(halo_ay)))[3] == 0, "halo center must be transparent"
    assert halo_im.getpixel((round(halo_ax) + 600, round(halo_ay))) == (0, 0, 0, 255), "halo outer ring must be opaque black"

    write_pack_dir(
        args.pickups_dir,
        (halo_ax, halo_ay),
        [
            (0, health_name, health_im.width, health_im.height),
            (1, halo_name, halo_im.width, halo_im.height),
        ],
    )

    # Level 2A levelTitle (#1408, 100 frames at 1.0x)
    title_frames_count = symbols[str(TITLE_SYMBOL_ID)]["frames"]
    union = [math.inf, math.inf, -math.inf, -math.inf]
    for f in range(title_frames_count):
        b = symbol_bounds(TITLE_SYMBOL_ID, f, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0), symbols, shapes, 0, None)
        union[0] = min(union[0], b[0])
        union[1] = min(union[1], b[1])
        union[2] = max(union[2], b[2])
        union[3] = max(union[3], b[3])
    title_manifest = []
    title_preview = None
    title_ax, title_ay = 0.0, 0.0
    for f in range(title_frames_count):
        im, ax, ay = render_single_frame(
            TITLE_SYMBOL_ID, f, 1.0, union, symbols, shapes, bitmaps, unsupported, missing
        )
        title_ax, title_ay = ax, ay
        fname = f"frame_{f:04d}.png"
        im.save(args.title_dir / fname)
        if f == 15:
            title_preview = im.copy()
        title_manifest.append((f, fname, im.width, im.height))
    write_pack_dir(args.title_dir, (title_ax, title_ay), title_manifest)

    assert not unsupported, f"Unsupported fills encountered: {sorted(unsupported)}"
    assert not missing, f"Missing bitmaps encountered: {sorted(missing)}"

    # Build compact proof contact sheet (health01, halo 600x400 viewport crop over grid, LEVEL 2 title)
    health_cropped = health_im.crop(health_im.getchannel("A").getbbox())
    proof = Image.new("RGBA", (960, 460), (24, 24, 30, 255))
    draw = ImageDraw.Draw(proof)
    draw.text((16, 12), "B25 Labs 2A: health01 (#1401 @ 0.876x)", fill=(235, 235, 240, 255))
    proof.alpha_composite(health_cropped.resize((health_cropped.width * 2, health_cropped.height * 2), Image.Resampling.NEAREST), (40, 56))

    draw.text((16, 200), "LEVEL 2 Title (#1408 f15)", fill=(235, 235, 240, 255))
    if title_preview is not None:
        proof.alpha_composite(title_preview, (16, 240))

    draw.text((340, 12), "halo (#1399) 600x400 crop over grid (center transparent -> outer black)", fill=(235, 235, 240, 255))
    crop_w, crop_h = 600, 400
    bg = Image.new("RGBA", (crop_w, crop_h), (80, 110, 145, 255))
    bdraw = ImageDraw.Draw(bg)
    for gx in range(0, crop_w, 40):
        bdraw.line([(gx, 0), (gx, crop_h)], fill=(120, 160, 200, 255))
    for gy in range(0, crop_h, 40):
        bdraw.line([(0, gy), (crop_w, gy)], fill=(120, 160, 200, 255))
    cx0 = round(halo_ax) - crop_w // 2
    cy0 = round(halo_ay) - crop_h // 2
    halo_crop = halo_im.crop((cx0, cy0, cx0 + crop_w, cy0 + crop_h))
    bg.alpha_composite(halo_crop)
    proof.alpha_composite(bg, (340, 40))

    proof.save(args.proof)

    report = {
        "batch": "B25",
        "verified": verified,
        "health_1401": {
            "size": [health_cropped.width, health_cropped.height],
            "anchor": [health_ax, health_ay],
        },
        "halo_1399": {
            "size": [halo_im.width, halo_im.height],
            "anchor": [halo_ax, halo_ay],
        },
        "title_1408": {
            "frames": title_frames_count,
            "size": [title_manifest[0][2], title_manifest[0][3]],
        },
        "proof_sha256": sha256_bytes(args.proof.read_bytes()),
    }
    args.report.write_text(json.dumps(report, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
