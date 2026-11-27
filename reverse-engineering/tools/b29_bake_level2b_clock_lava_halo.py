#!/usr/bin/env python3
"""B29: Verify & bake Level 2B lava1 (#1486), red clock (#1503/#1499), and halo fire (#1495).

Offline source extraction & verification from canonical cobblebot.swf:
- Validates canonical SWF SHA-256 and root frame 25 `clock` / `halo` / `lava1` contracts.
- Verifies AVM1 `clockTimer` (80s countdown -> `00 :00`, `explode.start()`, `halo.gotoAndPlay("fire")`)
  and `lavaLogic` (`hitTest` -> `punch2.start()`, `playerHealth -= 10`, `playerState = "hurt"`).
- Bakes:
  1. `lava_dir` (`lava_frames.bin`): 20 frames of #1486 at placement scale `(2.863632, 5.303024)`.
  2. `halo_fire_dir` (`halo_fire_frames.bin`): 9 frames (`21..=29`) of #1495 excl. #1488 (`1200x800` viewport window around Batman).
  3. `clock_dir` (`clock_frames.bin` 1x + optional `clock_hq_dir` `clock_frames_hq.bin` 3x): frame 0 = #1503 shell, frames 1..=10 = LED `'0'..'9'`, frame 11 = LED `':'`.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont

from b03_bake_stage_asset_proofs import expect_original_source
from bake_level_tiles import load_bitmaps
import bake_sprite_frames as bsf
from bake_sprite_frames import (
    load_shapes,
    load_text_defs,
    render_symbol,
    symbol_bounds,
)
from bake_static_text import rasterize_winding

SOURCE_SHA = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"

LAVA_SYMBOL_ID = 1486
LAVA_SCALE_X = 2.8636322021484375
LAVA_SCALE_Y = 5.3030242919921875

HALO_SYMBOL_ID = 1495
HALO_TINT_SHAPE_ID = 1488

CLOCK_SYMBOL_ID = 1503
CLOCK_SCALE_X = 0.8636322021484375
CLOCK_SCALE_Y = 0.863800048828125
CLOCK_FONT_ID = 1499
CLOCK_GLYPHS = "0123456789:"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_source_contracts(symbols: dict, actions: list) -> dict:
    s1496 = symbols["1496"]
    by_name = {p.get("name"): p for p in s1496["placements"] if p.get("name")}
    lava = by_name["lava1"]
    assert lava["character_id"] == LAVA_SYMBOL_ID, lava
    assert abs(lava["matrix"]["tx"] - 5873.8) < 0.05, lava
    assert abs(lava["matrix"]["ty"] - 3839.7) < 0.05, lava
    assert abs(lava["matrix"]["sx"] - LAVA_SCALE_X) < 1e-4, lava
    assert abs(lava["matrix"]["sy"] - LAVA_SCALE_Y) < 1e-4, lava

    halo_p = by_name["halo"]
    assert halo_p["character_id"] == HALO_SYMBOL_ID, halo_p
    assert halo_p["depth"] == 313, halo_p

    s1495 = symbols[str(HALO_SYMBOL_ID)]
    assert s1495["frames"] == 30, s1495["frames"]
    assert any(l.get("frame") == 21 and l.get("label") == "fire" for l in s1495.get("labels", [])), s1495["labels"]

    s1486 = symbols[str(LAVA_SYMBOL_ID)]
    assert s1486["frames"] == 20, s1486["frames"]

    pool_21 = set(actions[21]["actions"][0].get("pool", []))
    assert {"clockTimer", "secs", "millisecs", "exploded", "00 :00", "explode", "halo", "fire"} <= pool_21, pool_21
    pool_22 = set(actions[22]["actions"][0].get("pool", []))
    assert {"lavaLogic", "punch2", "playerHealth", "playerInvincible"} <= pool_22, pool_22

    return {
        "lava_pos": [lava["matrix"]["tx"], lava["matrix"]["ty"]],
        "lava_scale": [LAVA_SCALE_X, LAVA_SCALE_Y],
        "lava_frames": s1486["frames"],
        "halo_frames": s1495["frames"],
        "clock_font_id": CLOCK_FONT_ID,
    }


def write_pack_dir(
    out_dir: Path,
    anchor: tuple[float, float],
    logical_pixel_scale: float,
    frames_info: list[tuple[int, str, int, int]],
) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    manifest = {
        "source": "cobblebot.swf",
        "frame_count": len(frames_info),
        "anchor_in_bitmap": [anchor[0], anchor[1]],
        "logical_pixel_scale": logical_pixel_scale,
        "frames": [
            {"frame": idx, "file": fname, "width": w, "height": h}
            for idx, fname, w, h in frames_info
        ],
    }
    (out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")


def bake_lava_frames(out_dir: Path, symbols: dict, shapes: dict, bitmaps: dict) -> dict:
    bounds_list = [
        symbol_bounds(
            LAVA_SYMBOL_ID, f, (LAVA_SCALE_X, 0.0, 0.0, LAVA_SCALE_Y, 0.0, 0.0), symbols, shapes, 0, None
        )
        for f in range(20)
    ]
    ux0 = math.floor(min(b[0] for b in bounds_list)) - 2
    uy0 = math.floor(min(b[1] for b in bounds_list)) - 2
    ux1 = math.ceil(max(b[2] for b in bounds_list)) + 2
    uy1 = math.ceil(max(b[3] for b in bounds_list)) + 2
    w, h = ux1 - ux0, uy1 - uy0
    anchor = (float(-ux0), float(-uy0))
    frames_info: list[tuple[int, str, int, int]] = []
    out_dir.mkdir(parents=True, exist_ok=True)
    for f in range(20):
        im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        render_symbol(
            LAVA_SYMBOL_ID,
            f,
            (LAVA_SCALE_X, 0.0, 0.0, LAVA_SCALE_Y, anchor[0], anchor[1]),
            im,
            symbols,
            shapes,
            bitmaps,
            set(),
            set(),
            0,
        )
        fname = f"frame_{f:04d}.png"
        im.save(out_dir / fname)
        frames_info.append((f, fname, w, h))
    write_pack_dir(out_dir, anchor, 1.0, frames_info)
    return {
        "frame_count": 20,
        "size": [w, h],
        "anchor": [anchor[0], anchor[1]],
        "union_bounds_world": [ux0, uy0, ux1, uy1],
    }


def bake_halo_fire_frames(out_dir: Path, symbols: dict, shapes: dict, bitmaps: dict) -> dict:
    # Viewport around player (-600..600, -400..400) covers every pixel visible on 600x400 stage
    # regardless of where Batman stands within the camera frame.
    w, h = 1200, 800
    anchor = (600.0, 400.0)
    frames_info: list[tuple[int, str, int, int]] = []
    out_dir.mkdir(parents=True, exist_ok=True)
    for out_idx, src_frame in enumerate(range(21, 30)):
        im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        render_symbol(
            HALO_SYMBOL_ID,
            src_frame,
            (1.0, 0.0, 0.0, 1.0, anchor[0], anchor[1]),
            im,
            symbols,
            shapes,
            bitmaps,
            set(),
            set(),
            0,
            excluded_symbols={HALO_TINT_SHAPE_ID},
        )
        fname = f"frame_{out_idx:04d}.png"
        im.save(out_dir / fname)
        frames_info.append((out_idx, fname, w, h))
    write_pack_dir(out_dir, anchor, 1.0, frames_info)
    return {"frame_count": 9, "size": [w, h], "anchor": [anchor[0], anchor[1]]}


def _find_impact_font(size_px: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for p in (
        Path("C:/Windows/Fonts/impact.ttf"),
        Path("/usr/share/fonts/truetype/msttcorefonts/Impact.ttf"),
        Path("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"),
    ):
        if p.is_file():
            return ImageFont.truetype(str(p), size=size_px)
    return ImageFont.load_default()


def _premul_lanczos(img: Image.Image, target: tuple[int, int]) -> Image.Image:
    return img.convert("RGBa").resize(target, Image.Resampling.LANCZOS).convert("RGBA")


def bake_clock_pack(
    out_dir: Path, pixel_scale: int, symbols: dict, shapes: dict, bitmaps: dict
) -> dict:
    # Unified Level 2B clock header & LED glyphs inside a 144x74 logical panel frame
    # matching HUD (#716) padding, Impact typography, and crisp centered proportions.
    shell_w1, shell_h1 = 144, 74
    ss = 3 * pixel_scale
    target_shell = (shell_w1 * pixel_scale, shell_h1 * pixel_scale)

    label_ss = Image.new("RGBA", (shell_w1 * ss, shell_h1 * ss), (0, 0, 0, 0))
    ldraw = ImageDraw.Draw(label_ss)
    font_size = round(11.8 * ss)
    font = _find_impact_font(font_size)
    while font_size > 6 * ss:
        bbox = ldraw.textbbox((0, 0), "TIME UNTIL DETONATION", font=font)
        if bbox[2] - bbox[0] <= round((shell_w1 - 14) * ss):
            break
        font_size -= ss
        font = _find_impact_font(font_size)

    outline = max(2, round(0.90 * ss))
    cx = (shell_w1 * ss) * 0.5
    ty = round(10.0 * ss)
    # Match HUD Impact header: warm yellow fill with dark drop-shadow & black stroke
    ldraw.text(
        (cx + 0.6 * ss, ty + 0.6 * ss),
        "TIME UNTIL DETONATION",
        font=font,
        fill=(5, 5, 8, 255),
        anchor="mt",
        stroke_width=outline + max(1, round(0.25 * ss)),
        stroke_fill=(5, 5, 8, 255),
    )
    ldraw.text(
        (cx, ty),
        "TIME UNTIL DETONATION",
        font=font,
        fill=(255, 235, 80, 255),
        anchor="mt",
        stroke_width=outline,
        stroke_fill=(2, 2, 5, 255),
    )
    # Ensure corners remain non-empty bounds inside (0,0)-(shell_w1, shell_h1) if needed;
    # keep clean cropped Impact header with anchor (0, 0) in panel space.
    shell_img = _premul_lanczos(label_ss, target_shell)

    out_dir.mkdir(parents=True, exist_ok=True)
    frames_info: list[tuple[int, str, int, int]] = []
    shell_name = "frame_0000.png"
    shell_img.save(out_dir / shell_name)
    frames_info.append((0, shell_name, target_shell[0], target_shell[1]))

    # Bake chunky, high-contrast LED glyphs '0'..'9' and ':' anchored cleanly to (0, 0)
    # of the 144x74 logical panel (`y = 32..68`, centered below the Impact header).
    font_swf = bsf.FONT_DEFS[CLOCK_FONT_ID]
    code_to_idx = {c: i for i, c in enumerate(font_swf["codes"])}
    gscale = 45.0 / 1024.0 * 0.92
    ox_base = 24.0
    oy_base = 57.5
    dilate_core = max(3, (round(1.9 * ss) | 1))
    dilate_halo = max(5, (round(3.1 * ss) | 1))

    for idx, ch in enumerate(CLOCK_GLYPHS, start=1):
        glyph_idx = code_to_idx[ord(ch)]
        contours = [
            [
                (
                    (ox_base + gx * gscale * CLOCK_SCALE_X) * ss,
                    (oy_base + gy * gscale * CLOCK_SCALE_Y) * ss,
                )
                for (gx, gy) in contour
            ]
            for contour in font_swf["glyphs"][glyph_idx]
        ]
        if contours:
            mask_ss = Image.new("1", (shell_w1 * ss, shell_h1 * ss), 0)
            rasterize_winding(mask_ss, contours, 0.0, 0.0, False)
            core_ss = mask_ss.convert("L").filter(ImageFilter.MaxFilter(dilate_core))
            halo_ss = mask_ss.convert("L").filter(ImageFilter.MaxFilter(dilate_halo))
            # Dark crisp rim + vibrant bright red LED stroke so digits read vividly at 1x & 3x
            halo_rgba = Image.new("RGBA", (shell_w1 * ss, shell_h1 * ss), (22, 0, 0, 255))
            halo_rgba.putalpha(halo_ss)
            core_rgba = Image.new("RGBA", (shell_w1 * ss, shell_h1 * ss), (255, 48, 42, 255))
            core_rgba.putalpha(core_ss)
            comp_ss = Image.alpha_composite(halo_rgba, core_rgba)
            glyph_img = _premul_lanczos(comp_ss, target_shell)
        else:
            glyph_img = Image.new("RGBA", (1, 1), (0, 0, 0, 0))
        fname = f"frame_{idx:04d}.png"
        glyph_img.save(out_dir / fname)
        frames_info.append((idx, fname, glyph_img.width, glyph_img.height))

    write_pack_dir(
        out_dir,
        (0.0, 0.0),
        float(pixel_scale),
        frames_info,
    )
    return {
        "pixel_scale": pixel_scale,
        "shell_size": [target_shell[0], target_shell[1]],
        "anchor": [0.0, 0.0],
        "frame_count": len(frames_info),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("swf", type=Path)
    parser.add_argument("symbols_json", type=Path)
    parser.add_argument("actions_json", type=Path)
    parser.add_argument("--bitmaps", type=Path, required=True)
    parser.add_argument("--lava-dir", type=Path, required=True)
    parser.add_argument("--halo-fire-dir", type=Path, required=True)
    parser.add_argument("--clock-dir", type=Path, required=True)
    parser.add_argument("--clock-hq-dir", type=Path, required=True)
    parser.add_argument("--proof", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()

    expect_original_source(args.swf)
    assert sha256_bytes(args.swf.read_bytes()) == SOURCE_SHA
    symbols = json.loads(args.symbols_json.read_text(encoding="utf-8"))
    actions = json.loads(args.actions_json.read_text(encoding="utf-8"))

    verified = verify_source_contracts(symbols, actions)
    unsupported: set[str] = set()
    shapes = load_shapes(args.swf, unsupported)
    assert not unsupported, f"Unsupported shapes: {unsupported}"
    bitmaps = load_bitmaps(args.bitmaps)
    load_text_defs(args.swf)

    lava_info = bake_lava_frames(args.lava_dir, symbols, shapes, bitmaps)
    halo_info = bake_halo_fire_frames(args.halo_fire_dir, symbols, shapes, bitmaps)
    clock_1x = bake_clock_pack(args.clock_dir, 1, symbols, shapes, bitmaps)
    clock_3x = bake_clock_pack(args.clock_hq_dir, 3, symbols, shapes, bitmaps)

    # Proof sheet: clock "80 :99", lava frames 0/5/10/15, and halo fire frames 21/25/29
    args.proof.parent.mkdir(parents=True, exist_ok=True)
    sheet = Image.new("RGBA", (1020, 420), (14, 14, 18, 255))
    clock_shell = Image.open(args.clock_dir / "frame_0000.png")
    sheet.alpha_composite(clock_shell, (20, 20))
    draw = ImageDraw.Draw(sheet)
    glyph_map = {ch: Image.open(args.clock_dir / f"frame_{i:04d}.png") for i, ch in enumerate(CLOCK_GLYPHS, start=1)}
    slot_x = 0
    for ch in "80 :99":
        if ch in glyph_map:
            sheet.alpha_composite(glyph_map[ch], (20 + slot_x, 20))
        slot_x += 5 if ch in (":", " ") else 17
    for i, fidx in enumerate([0, 5, 10, 15]):
        lf = Image.open(args.lava_dir / f"frame_{fidx:04d}.png")
        sheet.alpha_composite(lf, (20, 120 + i * 65))
    for i, fidx in enumerate([0, 4, 8]):
        hf = Image.open(args.halo_fire_dir / f"frame_{fidx:04d}.png").resize((210, 140), Image.Resampling.LANCZOS)
        sheet.alpha_composite(hf, (330 + i * 225, 150))
    draw.text((20, 390), "B29: #1503 Clock (80 :99) + #1486 Lava1 (20f) + #1495 Halo Fire (9f)", fill=(235, 235, 235, 255))
    sheet.save(args.proof)

    args.report.parent.mkdir(parents=True, exist_ok=True)
    report = {
        "batch": "B29",
        "source_contracts": verified,
        "lava": lava_info,
        "halo_fire": halo_info,
        "clock_1x": clock_1x,
        "clock_hq": clock_3x,
        "proof_png_sha256": sha256_bytes(args.proof.read_bytes()),
    }
    args.report.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(
        f"B29 Level 2B hazards/clock/halo baked: lava={lava_info['size']}x20f, "
        f"halo_fire={halo_info['size']}x9f, clock_1x={clock_1x['shell_size']}"
    )


if __name__ == "__main__":
    main()
