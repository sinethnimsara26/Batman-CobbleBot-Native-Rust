#!/usr/bin/env python3
"""Bake SWF root frame 26's original `card:kabuki` composition (166 active frames, 600x400).

Uses canonical cobblebot.swf depths, placement matrices, and child timelines:
- #1255 (depth 1): static widescreen backdrop bar.
- #1569 (depth 2): 184-frame timeline (`0..=165` active before `Stop()` at frame 165,
  `166..=183` removed/empty); frame 159 runs `_root.gotoNext = "level3a"; _root.fadeout.play();`.
- #1254 (depth 165): 162 frames with terminal AVM1 `Stop()` at frame 161, held through frame 165.
- #1570 (depth 184) + #24 (depth 186): original SKIP button UP-state and label;
  release-inside executes `_root.gotoNext = "level3a"; _root.fadeout.play();`.
- Excludes script-controlled fadeout #34 (depth 187), composited natively at runtime.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from bake_sprite_frames import display_list
from story_card_baker import StoryCardSpec, bake_story_card, collect_sprite_actions

KABUKI_CARD_FRAME_COUNT = 166

KABUKI_CARD_SPEC = StoryCardSpec(
    root_frame=26,
    card_symbol=1569,
    skip_button_id=1570,
    expected_root_chars=frozenset({1255, 1569, 1254, 1570, 24, 34}),
    frame_count=KABUKI_CARD_FRAME_COUNT,
    auto_fade_frame_zero_based=159,
    terminal_frame_zero_based=165,
    terminal_destination_label="level3a",
    synthetic_id=-1569,
    animation_name="root-kabuki-card",
    bounds_const_name="KABUKI_CARD_SKIP_RECT",
    bounds_filename="kabuki_card_button_bounds.rs",
    report_prefix="kabuki-card",
    label_prefix="card:kabuki",
    gallery_ticks=(0, 18, 36, 54, 72, 90, 110, 135, 159, 165),
    sound_id=130,
)


def audit_kabuki_card_scripts(raw: bytes, symbols: dict, sprites: dict) -> dict:
    """Verify #1254/#1262/#1564/#1569 AVM1 scripts and hold stopped sprite #1254 through frame 165."""
    src_1254 = sprites.get(1254)
    if src_1254 is None or int(symbols["1254"]["frames"]) != 162:
        raise ValueError("Kabuki card sprite #1254 source duration mismatch")
    actions_1254 = collect_sprite_actions(src_1254)
    if not any(b["frame"] == 161 and "Stop" in b["ops"] for b in actions_1254):
        raise ValueError("Original sprite #1254 terminal Stop() at frame 161 not verified")
    symbols["1254"]["frames"] = KABUKI_CARD_FRAME_COUNT
    if display_list(symbols["1254"], 161) != display_list(symbols["1254"], 162):
        raise AssertionError("Stopped sprite #1254 visual display list must hold after frame 161")

    src_1262 = sprites.get(1262)
    if src_1262 is None or int(symbols["1262"]["frames"]) != 53:
        raise ValueError("Kabuki card child sprite #1262 source duration mismatch")
    actions_1262 = collect_sprite_actions(src_1262)
    if not any(b["frame"] == 52 and "Stop" in b["ops"] for b in actions_1262):
        raise ValueError("Child sprite #1262 terminal Stop() at frame 52 not verified")

    src_1564 = sprites.get(1564)
    if src_1564 is None or int(symbols["1564"]["frames"]) != 208:
        raise ValueError("Kabuki card child sprite #1564 source duration mismatch")
    actions_1564 = collect_sprite_actions(src_1564)
    if not any(b["frame"] == 165 and "Stop" in b["ops"] for b in actions_1564):
        raise ValueError("Child sprite #1564 Stop() at frame 165 not verified")

    src_1569 = sprites.get(1569)
    if src_1569 is None or int(symbols["1569"]["frames"]) != 184:
        raise ValueError("Kabuki card sprite #1569 source duration mismatch")
    actions_1569 = collect_sprite_actions(src_1569)
    if not any(
        b["frame"] == 159
        and "level3a" in b["strings"]
        and "fadeout" in b["strings"]
        and "play" in b["strings"]
        for b in actions_1569
    ):
        raise ValueError("Sprite #1569 auto-fade action at frame 159 (`level3a`) not verified")
    if not any(b["frame"] == 165 and "Stop" in b["ops"] for b in actions_1569):
        raise ValueError("Sprite #1569 terminal Stop() at frame 165 not verified")

    return {
        "held_stopped_sprites": [{
            "symbol": 1254,
            "source_frame_count": 162,
            "stop_frame_zero_based": 161,
            "baked_frame_count": KABUKI_CARD_FRAME_COUNT,
        }],
        "child_timelines_verified": {
            "1262": {"frames": 53, "stop_frame_zero_based": 52},
            "1564": {"frames": 208, "stop_frame_zero_based": 165},
            "1494": {"frames": int(symbols["1494"]["frames"])},
        },
        "card_scripts_1569": actions_1569,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("symbols_json", type=Path)
    ap.add_argument("root_placements", type=Path)
    ap.add_argument("bitmaps", type=Path)
    ap.add_argument("out", type=Path)
    ap.add_argument(
        "--fix-gradient-scale",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="divide SWF gradient fill matrices by 20.0 (pixels vs twips); enabled by default for story cards",
    )
    ns = ap.parse_args()

    manifest = bake_story_card(
        ns.swf,
        ns.symbols_json,
        ns.root_placements,
        ns.bitmaps,
        ns.out,
        KABUKI_CARD_SPEC,
        audit_kabuki_card_scripts,
        fix_gradient_scale=ns.fix_gradient_scale,
    )
    print(
        "Kabuki card source bake:",
        {
            k: manifest[k]
            for k in (
                "source_root_frame",
                "source_card_symbol",
                "frame_count",
                "auto_fade_frame_zero_based",
                "terminal_frame_zero_based",
                "size",
                "unsupported_fills",
                "missing_bitmap_ids",
                "skip_button_stage_bounds",
            )
        },
    )


if __name__ == "__main__":
    main()
