#!/usr/bin/env python3
"""Shared SWF story-card baker helper for root interstitial frames (e.g., card:tv, card:kabuki)."""
from __future__ import annotations

import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from PIL import Image, ImageChops, ImageDraw, ImageStat

from bake_root_intro import recover_button_up_state
from bake_sprite_frames import load_shapes, load_text_defs, render_symbol, symbol_bounds
from inspect_intro_behavior import collect_movie_sprites, inspect_skip_button, scan_timeline
from render_collision_masks import affine
from swf_re import disasm_actions, read_rect, tags

STAGE_SIZE = (600, 400)


@dataclass(frozen=True)
class StoryCardSpec:
    """SWF placement and timeline metadata for a root-frame story interstitial."""

    root_frame: int
    card_symbol: int
    skip_button_id: int
    expected_root_chars: frozenset[int]
    frame_count: int
    auto_fade_frame_zero_based: int
    terminal_frame_zero_based: int
    terminal_destination_label: str
    synthetic_id: int
    animation_name: str
    bounds_const_name: str
    bounds_filename: str
    report_prefix: str
    label_prefix: str
    gallery_ticks: tuple[int, ...]
    sound_id: int = 130


def collect_sprite_actions(sprite_payload: bytes) -> list[dict]:
    """Disassemble every DoAction block inside a DefineSprite payload."""
    frame = 0
    blocks = []
    for code, blob, tag_offset, payload_offset in tags(sprite_payload, 4, len(sprite_payload)):
        if code == 1:
            frame += 1
        elif code == 12:
            acts = disasm_actions(blob, base=payload_offset)
            strings = []
            for act in acts:
                for val in act.get("values", ()):
                    if isinstance(val, dict) and isinstance(val.get("resolved"), str):
                        strings.append(val["resolved"])
            blocks.append({
                "frame": frame,
                "tag_offset": tag_offset,
                "ops": [a.get("op") for a in acts],
                "actions": acts,
                "strings": strings,
            })
    return blocks


def bake_story_card(
    swf_path: Path,
    symbols_json: Path,
    root_placements_json: Path,
    bitmaps_dir: Path,
    out_dir: Path,
    spec: StoryCardSpec,
    script_auditor: Callable[[bytes, dict, dict], dict],
) -> dict:
    """Bake a SWF root-frame story card composition with full script and button verification."""
    raw = swf_path.read_bytes()
    symbols = json.loads(symbols_json.read_text(encoding="utf-8"))
    placements = json.loads(root_placements_json.read_text(encoding="utf-8"))
    source_scene = [p for p in placements if p.get("frame") == spec.root_frame]

    found_chars = {p.get("character_id") for p in source_scene}
    if not spec.expected_root_chars.issubset(found_chars):
        raise ValueError(
            f"Missing root{spec.root_frame} characters: {sorted(spec.expected_root_chars - found_chars)}"
        )

    _, root_pos = read_rect(raw, 8)
    root_pos += 4
    root_timeline = scan_timeline(raw, root_pos, f"root:{spec.root_frame}", root_frame=spec.root_frame)
    if not any(s.get("sound_id") == spec.sound_id for s in root_timeline["sounds"]):
        raise ValueError(f"Root frame {spec.root_frame} missing StartSound {spec.sound_id}")
    if not any(any(a.get("op") == "Stop" for a in b.get("actions", ())) for b in root_timeline["scripts"]):
        raise ValueError(f"Root frame {spec.root_frame} missing Stop()")

    btn_info = inspect_skip_button(raw, root_pos, button_id=spec.skip_button_id)
    if not any(
        "over_down_to_over_up" in h.get("events", ())
        and spec.terminal_destination_label in h.get("strings", ())
        and "fadeout" in h.get("strings", ())
        for h in btn_info.get("handlers", ())
    ):
        raise ValueError(
            f"SKIP button #{spec.skip_button_id} handler mismatch for {spec.terminal_destination_label}"
        )

    button_up, button_report = recover_button_up_state(swf_path, spec.skip_button_id)
    symbols[str(spec.skip_button_id)] = {"frames": 1, "placements": button_up, "removals": []}

    sprites = collect_movie_sprites(raw)
    script_report = script_auditor(raw, symbols, sprites)

    script_controlled_transitions = {34}
    scene = [
        dict(p, frame=0)
        for p in source_scene
        if p.get("character_id") not in script_controlled_transitions
    ]
    symbols[str(spec.synthetic_id)] = {
        "frames": spec.frame_count,
        "placements": scene,
        "removals": [],
    }

    unsupported: set[str] = set()
    shapes = load_shapes(swf_path, unsupported)
    load_text_defs(swf_path)

    button_placement = next(p for p in source_scene if p.get("character_id") == spec.skip_button_id)
    bounds = symbol_bounds(spec.skip_button_id, 0, affine(button_placement.get("matrix", {})), symbols, shapes)
    if not all(math.isfinite(v) for v in bounds):
        raise ValueError(f"Recovered SKIP button #{spec.skip_button_id} has no visible geometry")
    click_rect = [
        max(0, math.floor(bounds[0])),
        max(0, math.floor(bounds[1])),
        min(STAGE_SIZE[0], math.ceil(bounds[2])),
        min(STAGE_SIZE[1], math.ceil(bounds[3])),
    ]
    if not (click_rect[0] < click_rect[2] and click_rect[1] < click_rect[3]):
        raise ValueError(f"Invalid SKIP bounds for #{spec.skip_button_id}: {click_rect}")

    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / spec.bounds_filename).write_text(
        f"const {spec.bounds_const_name}:(i32,i32,i32,i32)=(" + ",".join(map(str, click_rect)) + ");\n",
        encoding="utf-8",
    )

    bitmap_manifest = json.loads((bitmaps_dir / "manifest.json").read_text(encoding="utf-8"))
    bitmaps = {}
    for item in bitmap_manifest:
        if "file" not in item:
            continue
        with Image.open(bitmaps_dir / item["file"]) as image:
            bitmaps[item["id"]] = image.convert("RGBA")

    manifest_frames = []
    frame_diagnostics = []
    previous_rgb = None
    missing: set[int] = set()
    cols = 5
    rows = max(1, math.ceil(len(spec.gallery_ticks) / cols))
    gallery = Image.new("RGB", (300 * cols, 220 * rows), (18, 18, 22))
    gallery_draw = ImageDraw.Draw(gallery)

    for tick in range(spec.frame_count):
        stage = Image.new("RGBA", STAGE_SIZE, (0, 0, 0, 255))
        frame_missing: set[int] = set()
        render_symbol(
            spec.synthetic_id,
            tick,
            (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            stage,
            symbols,
            shapes,
            bitmaps,
            unsupported,
            frame_missing,
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
        stage.save(out_dir / filename, format="PNG", optimize=True)
        manifest_frames.append({"frame": tick, "animation": spec.animation_name, "file": filename})
        if tick in spec.gallery_ticks:
            idx = spec.gallery_ticks.index(tick)
            x = (idx % cols) * 300
            y = (idx // cols) * 220
            gallery.paste(rgb.resize((300, 200), Image.Resampling.BILINEAR), (x, y))
            gallery_draw.text((x + 6, y + 202), f"{spec.label_prefix} f={tick:03} / {spec.frame_count}", fill=(245, 240, 220))

    (out_dir / f"{spec.report_prefix}-frame-diagnostics.json").write_text(
        json.dumps({
            "source_root_frame": spec.root_frame,
            "fps": 25,
            "frame_count": spec.frame_count,
            "frames": frame_diagnostics,
        }, indent=2),
        encoding="utf-8",
    )
    manifest = {
        "symbol": spec.synthetic_id,
        "source_root_frame": spec.root_frame,
        "source_card_symbol": spec.card_symbol,
        "source_sound_id": spec.sound_id,
        "frame_count": spec.frame_count,
        "auto_fade_frame_zero_based": spec.auto_fade_frame_zero_based,
        "terminal_frame_zero_based": spec.terminal_frame_zero_based,
        "terminal_destination_label": spec.terminal_destination_label,
        "size": list(STAGE_SIZE),
        "scale": [1.0, 1.0],
        "source_bounds": [0, 0, STAGE_SIZE[0], STAGE_SIZE[1]],
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
    (out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    gallery.save(out_dir / f"{spec.report_prefix}-contact-sheet.png", format="PNG", optimize=True)
    if unsupported or missing:
        raise SystemExit(
            f"Original {spec.label_prefix} assets incomplete (unsupported={sorted(unsupported)}, missing={sorted(missing)})"
        )
    return manifest
