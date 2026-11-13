#!/usr/bin/env python3
"""Bake SWF root frame 24's original `card:tv` composition (185 frames, 600x400).

Uses canonical cobblebot.swf depths, placement matrices, and child timelines:
- #1479 (depth 1): 185 frames; frame 151 triggers `_root.gotoNext = "level2b"; _root.fadeout.play();`
  and frame 184 executes terminal AVM1 `Stop()`.
- #1415 (child of #1417): loops frames 0..4 via frame-4 `GotoFrame(0); Play()`.
- #1480 (depth 251) + #24 (depth 253): original SKIP button UP-state and label;
  release-inside executes `_root.gotoNext = "level2b"; _root.fadeout.play();`.
- Excludes script-controlled fadeout #34 (depth 254), composited natively at runtime.
"""
from __future__ import annotations

import argparse
from pathlib import Path

from story_card_baker import StoryCardSpec, bake_story_card, collect_sprite_actions

TV_CARD_SPEC = StoryCardSpec(
    root_frame=24,
    card_symbol=1479,
    skip_button_id=1480,
    expected_root_chars=frozenset({1479, 1480, 24, 34}),
    frame_count=185,
    auto_fade_frame_zero_based=151,
    terminal_frame_zero_based=184,
    terminal_destination_label="level2b",
    synthetic_id=-1479,
    animation_name="root-tv-card",
    bounds_const_name="TV_CARD_SKIP_RECT",
    bounds_filename="tv_card_button_bounds.rs",
    report_prefix="tv-card",
    label_prefix="card:tv",
    gallery_ticks=(0, 20, 40, 60, 86, 110, 130, 151, 168, 184),
    sound_id=130,
)


def audit_tv_card_scripts(_raw: bytes, symbols: dict, sprites: dict) -> dict:
    """Verify #1479 auto-fade (f=151) / Stop (f=184) and #1415 5-frame rain/noise loop (f=4)."""
    src_1479 = sprites.get(1479)
    if src_1479 is None or int(symbols["1479"]["frames"]) != 185:
        raise ValueError("TV card sprite #1479 source duration mismatch")
    blocks_1479 = collect_sprite_actions(src_1479)
    has_151 = any(
        b["frame"] == 151
        and "_root" in b["strings"]
        and "gotoNext" in b["strings"]
        and "level2b" in b["strings"]
        and "fadeout" in b["strings"]
        and "play" in b["strings"]
        for b in blocks_1479
    )
    has_184_stop = any(b["frame"] == 184 and "Stop" in b["ops"] for b in blocks_1479)
    if not has_151 or not has_184_stop:
        raise ValueError(f"TV card sprite #1479 expected scripts f=151 & f=184 missing: {blocks_1479}")

    src_1415 = sprites.get(1415)
    if src_1415 is None or int(symbols["1415"]["frames"]) != 25:
        raise ValueError("TV card child sprite #1415 source duration mismatch")
    blocks_1415 = collect_sprite_actions(src_1415)
    loop_1415 = any(
        b["frame"] == 4
        and any(a.get("op") == "GotoFrame" and a.get("frame") == 0 for a in b["actions"])
        and "Play" in b["ops"]
        for b in blocks_1415
    )
    if not loop_1415:
        raise ValueError(f"TV card child sprite #1415 expected frame-4 loop missing: {blocks_1415}")
    symbols["1415"]["frames"] = 5

    return {
        "loop_trimmed_sprites": [{
            "symbol": 1415,
            "source_frame_count": 25,
            "loop_action_frame_zero_based": 4,
            "effective_loop_frames": 5,
        }],
        "card_scripts_1479": [
            {"frame": b["frame"], "ops": b["ops"], "strings": b["strings"]}
            for b in blocks_1479
        ],
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("symbols_json", type=Path)
    ap.add_argument("root_placements", type=Path)
    ap.add_argument("bitmaps", type=Path)
    ap.add_argument("out", type=Path)
    ns = ap.parse_args()

    manifest = bake_story_card(
        ns.swf,
        ns.symbols_json,
        ns.root_placements,
        ns.bitmaps,
        ns.out,
        TV_CARD_SPEC,
        audit_tv_card_scripts,
    )
    print(
        "TV card source bake:",
        {
            k: manifest[k]
            for k in (
                "source_root_frame",
                "source_card_symbol",
                "frame_count",
                "auto_fade_frame_zero_based",
                "size",
                "unsupported_fills",
                "missing_bitmap_ids",
            )
        },
    )


if __name__ == "__main__":
    main()
