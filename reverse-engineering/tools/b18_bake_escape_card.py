#!/usr/bin/env python3
"""Bake SWF root frame 22's original `card:escape` composition (225 frames, 600x400).

Uses canonical cobblebot.swf depths, placement matrices, and child timelines:
- #1254 (depth 1): 162 frames with terminal AVM1 Stop at frame 161, held through frame 224.
- #1255 (depth 20): static separator/backdrop bar.
- #1267 (depth 21): 225 frames with terminal frame 224 DoAction
  `stop(); _root.gotoNext = "level2a"; _root.fadeout.play();`.
- #1268 (depth 245) + #24 (depth 247): original SKIP button UP-state and label;
  release-inside executes `_root.gotoNext = "level2a"; _root.fadeout.play();`.
- Excludes script-controlled fadeout #34 (depth 248), composited natively at runtime.
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageStat

from bake_root_intro import recover_button_up_state
from bake_sprite_frames import display_list, load_shapes, load_text_defs, render_symbol, symbol_bounds
from inspect_intro_behavior import collect_movie_sprites, scan_timeline
from render_collision_masks import affine

ROOT_ESCAPE_CARD_FRAME = 22
ESCAPE_CARD_FRAMES = 225
STAGE_SIZE = (600, 400)
GALLERY_TICKS = (0, 25, 50, 75, 100, 125, 150, 175, 200, 224)


def verify_terminal_stop_and_route(raw: bytes, symbols: dict) -> dict:
    """Verify #1254 terminal Stop (f=161) and #1267 terminal end-to-2A action (f=224)."""
    sprites = collect_movie_sprites(raw)

    src_1254 = sprites.get(1254)
    if src_1254 is None or int(symbols["1254"]["frames"]) != 162:
        raise ValueError("Escape card sprite #1254 source duration mismatch")
    actions_1254 = scan_timeline(src_1254, 4, "sprite:1254")["scripts"]
    if not any(b["frame"] == 161 and any(a["op"] == "Stop" for a in b["actions"]) for b in actions_1254):
        raise ValueError("Original sprite #1254 terminal Stop at frame 161 not verified")
    symbols["1254"]["frames"] = ESCAPE_CARD_FRAMES
    if display_list(symbols["1254"], 161) != display_list(symbols["1254"], 162):
        raise AssertionError("Stopped sprite #1254 visual display list must hold after frame 161")

    src_1267 = sprites.get(1267)
    if src_1267 is None or int(symbols["1267"]["frames"]) != ESCAPE_CARD_FRAMES:
        raise ValueError("Escape card sprite #1267 source duration mismatch")
    actions_1267 = scan_timeline(src_1267, 4, "sprite:1267")["scripts"]
    if not any(b["frame"] == 224 and any(a["op"] == "Stop" for a in b["actions"]) for b in actions_1267):
        raise ValueError("Original sprite #1267 terminal Stop at frame 224 not verified")

    return {
        "held_stopped_sprites": [{
            "symbol": 1254,
            "source_frame_count": 162,
            "stop_frame_zero_based": 161,
            "baked_frame_count": ESCAPE_CARD_FRAMES,
        }],
        "card_terminal_scripts_1267": actions_1267,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("symbols_json", type=Path)
    ap.add_argument("root_placements", type=Path)
    ap.add_argument("bitmaps", type=Path)
    ap.add_argument("out", type=Path)
    ns = ap.parse_args()

    raw = ns.swf.read_bytes()
    symbols = json.loads(ns.symbols_json.read_text(encoding="utf-8"))
    placements = json.loads(ns.root_placements.read_text(encoding="utf-8"))
    source_scene = [p for p in placements if p.get("frame") == ROOT_ESCAPE_CARD_FRAME]

    expected = {1254, 1255, 1267, 1268, 24}
    found = {p.get("character_id") for p in source_scene}
    if not expected.issubset(found):
        raise ValueError(f"Missing root22 characters: {sorted(expected - found)}")

    button_up, button_report = recover_button_up_state(ns.swf, 1268)
    symbols["1268"] = {"frames": 1, "placements": button_up, "removals": []}
    script_report = verify_terminal_stop_and_route(raw, symbols)

    script_controlled_transitions = {34}
    scene = [
        dict(p, frame=0) for p in source_scene
        if p.get("character_id") not in script_controlled_transitions
    ]
    synthetic_id = -225
    symbols[str(synthetic_id)] = {
        "frames": ESCAPE_CARD_FRAMES,
        "placements": scene,
        "removals": [],
    }

    unsupported: set[str] = set()
    shapes = load_shapes(ns.swf, unsupported)
    load_text_defs(ns.swf)

    button_placement = next(p for p in source_scene if p.get("character_id") == 1268)
    bounds = symbol_bounds(1268, 0, affine(button_placement.get("matrix", {})), symbols, shapes)
    if not all(math.isfinite(v) for v in bounds):
        raise ValueError("Recovered SKIP button #1268 has no visible geometry")
    click_rect = [
        max(0, math.floor(bounds[0])),
        max(0, math.floor(bounds[1])),
        min(600, math.ceil(bounds[2])),
        min(400, math.ceil(bounds[3])),
    ]
    if not (click_rect[0] < click_rect[2] and click_rect[1] < click_rect[3]):
        raise ValueError(f"Invalid SKIP bounds for #1268: {click_rect}")

    ns.out.mkdir(parents=True, exist_ok=True)
    (ns.out / "escape_card_button_bounds.rs").write_text(
        "const ESCAPE_CARD_SKIP_RECT:(i32,i32,i32,i32)=(" + ",".join(map(str, click_rect)) + ");\n",
        encoding="utf-8",
    )

    bitmap_manifest = json.loads((ns.bitmaps / "manifest.json").read_text(encoding="utf-8"))
    bitmaps = {}
    for item in bitmap_manifest:
        if "file" not in item:
            continue
        with Image.open(ns.bitmaps / item["file"]) as image:
            bitmaps[item["id"]] = image.convert("RGBA")

    manifest_frames = []
    frame_diagnostics = []
    previous_rgb = None
    missing: set[int] = set()
    gallery = Image.new("RGB", (300 * 5, 220 * 2), (18, 18, 22))
    gallery_draw = ImageDraw.Draw(gallery)

    for tick in range(ESCAPE_CARD_FRAMES):
        stage = Image.new("RGBA", STAGE_SIZE, (0, 0, 0, 255))
        frame_missing: set[int] = set()
        render_symbol(
            synthetic_id, tick, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0), stage,
            symbols, shapes, bitmaps, unsupported, frame_missing,
        )
        missing.update(frame_missing)

        rgb = stage.convert("RGB")
        luma = rgb.convert("L")
        histogram = luma.histogram()
        changed_fraction = 0.0
        if previous_rgb is not None:
            diff = ImageChops.difference(previous_rgb, rgb).convert("L")
            changed_fraction = sum(diff.histogram()[24:]) / (STAGE_SIZE[0] * STAGE_SIZE[1])
        frame_diagnostics.append({
            "frame": tick,
            "mean_luma": round(ImageStat.Stat(luma).mean[0], 3),
            "near_black_fraction": round(sum(histogram[:9]) / (STAGE_SIZE[0] * STAGE_SIZE[1]), 6),
            "changed_pixel_fraction_from_previous": round(changed_fraction, 6),
        })
        previous_rgb = rgb

        filename = f"{tick:03}.png"
        stage.save(ns.out / filename, format="PNG", optimize=True)
        manifest_frames.append({"frame": tick, "animation": "root-escape-card", "file": filename})
        if tick in GALLERY_TICKS:
            idx = GALLERY_TICKS.index(tick)
            x = (idx % 5) * 300
            y = (idx // 5) * 220
            gallery.paste(rgb.resize((300, 200), Image.Resampling.BILINEAR), (x, y))
            gallery_draw.text((x + 6, y + 202), f"card:escape f={tick:03} / 225", fill=(245, 240, 220))

    (ns.out / "escape-card-frame-diagnostics.json").write_text(
        json.dumps({
            "source_root_frame": ROOT_ESCAPE_CARD_FRAME,
            "fps": 25,
            "frame_count": ESCAPE_CARD_FRAMES,
            "frames": frame_diagnostics,
        }, indent=2),
        encoding="utf-8",
    )
    manifest = {
        "symbol": synthetic_id,
        "source_root_frame": ROOT_ESCAPE_CARD_FRAME,
        "source_card_symbol": 1267,
        "source_sound_id": 130,
        "frame_count": ESCAPE_CARD_FRAMES,
        "terminal_frame_zero_based": 224,
        "terminal_destination_label": "level2a",
        "size": list(STAGE_SIZE),
        "scale": [1.0, 1.0],
        "source_bounds": [0, 0, 600, 400],
        "anchor_in_bitmap": [0.0, 0.0],
        "unsupported_fills": sorted(unsupported),
        "missing_bitmap_ids": sorted(missing),
        "root_placements": source_scene,
        "recovered_button_up_state": button_report,
        "skip_button_stage_bounds": click_rect,
        "deferred_script_controlled_transition_ids": sorted(script_controlled_transitions),
        **script_report,
        "frames": manifest_frames,
    }
    (ns.out / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    gallery.save(ns.out / "escape-card-contact-sheet.png", format="PNG", optimize=True)
    print(
        "Escape card source bake:",
        {k: manifest[k] for k in ("source_root_frame", "source_card_symbol", "frame_count", "size", "unsupported_fills", "missing_bitmap_ids")},
    )
    if unsupported or missing:
        raise SystemExit("Original card:escape assets incomplete; do not accept partial frames")


if __name__ == "__main__":
    main()
