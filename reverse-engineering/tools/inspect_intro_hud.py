#!/usr/bin/env python3
"""Report original SWF intro/HUD evidence without guessing scene timing.

This diagnostic is build-time only and ships no proprietary source assets.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from bake_hud_assets import parse_edit_texts


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("symbols", type=Path)
    ap.add_argument("root_placements", type=Path)
    ap.add_argument("out", type=Path)
    ns = ap.parse_args()

    symbols = json.loads(ns.symbols.read_text(encoding="utf-8"))
    root = json.loads(ns.root_placements.read_text(encoding="utf-8"))
    edit = parse_edit_texts(ns.swf)
    scene = [p for p in root if p.get("frame") == 18]
    summary = {}
    for placement in scene:
        sid = placement.get("character_id")
        if sid is None:
            continue
        sprite = symbols.get(str(sid), {})
        summary[str(sid)] = {
            "root_depth": placement.get("depth"),
            "root_matrix": placement.get("matrix"),
            "frames": sprite.get("frames"),
            "labels": sprite.get("labels", []),
            "initial_placements": [p for p in sprite.get("placements", []) if p.get("frame") == 0][:60],
            "placed_symbols": sorted(set(p.get("character_id") for p in sprite.get("placements", []) if isinstance(p.get("character_id"),int)))[:120],
        }
    report = {
        "root_intro_frame": 18,
        "root_intro_placements": scene,
        "intro_symbols": summary,
        "hud_fields": {str(cid):edit[cid] for cid in (700,702,710,711,713,714,715)},
        "note": "Root frame 18 exists, but which nested actions/sounds stop or advance the story must be verified before final timing is claimed.",
    }
    ns.out.parent.mkdir(parents=True, exist_ok=True)
    ns.out.write_text(json.dumps(report,indent=2),encoding="utf-8")
    print("Intro symbols and durations:", {k:v["frames"] for k,v in summary.items()})
    print("HUD Flash field geometry:", json.dumps(report["hud_fields"],indent=2))


if __name__ == "__main__":
    main()
