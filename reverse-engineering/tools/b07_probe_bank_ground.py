#!/usr/bin/env python3
"""B07 source-authentic Level1B #782 collision mask preflight (OFFLINE/CI).

Input is the existing SWF shape/winding rasterizer's level-space 1-bit PNG,
not a substitute gameplay screenshot. Fails closed on wrong placement,
dimensions, source-ID, empty streets or malformed mask. Emits source evidence
only; never creates a complete Level1B art pack.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import shutil
from pathlib import Path
from PIL import Image

ORIGIN = (-11196, -886)
SIZE = (12102, 1413)
GROUND = 782
SAMPLE_X = [-506, -400, -600, -1000, -3000, -6000]

def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("manifest", type=Path)
    ap.add_argument("mask", type=Path)
    ap.add_argument("outdir", type=Path)
    ns = ap.parse_args()
    records = json.loads(ns.manifest.read_text(encoding="utf-8"))
    bank = [r for r in records if r.get("level") == "level1b"]
    assert len(bank) == 1, "one exact source Level1B record required"
    record = bank[0]
    assert record.get("ground_character_id") == GROUND, record
    assert tuple(record.get("origin_level_px", [])) == ORIGIN, record
    assert tuple(record.get("size_px", [])) == SIZE, record
    assert record.get("error") is None, record
    with Image.open(ns.mask) as source:
        assert source.size == SIZE, source.size
        assert source.mode == "1", ("source mask must be binary", source.mode)
        mask = source.convert("L")
    hist = mask.histogram()
    occupied = hist[255]
    assert 0 < occupied < SIZE[0] * SIZE[1], occupied
    assert occupied == record["occupied_pixels"], (occupied, record)
    pix = mask.load()
    def solid(x: int, y: int) -> bool:
        ix, iy = x - ORIGIN[0], y - ORIGIN[1]
        return 0 <= ix < SIZE[0] and 0 <= iy < SIZE[1] and pix[ix, iy] >= 128

    floors = []
    for x in SAMPLE_X:
        for y in range(-700, 490):
            if (solid(x, y) and solid(x, y + 12)
                    and not solid(x, y - 1) and not solid(x, y - 90)):
                floors.append({"level_x": x, "surface_y": y})
                break
    assert floors, "none of the original Bank Street sample columns has exposed ground"
    report = {
        "classification": "SOURCE_GROUND_PROOF_ONLY_NOT_PLAYABLE",
        "source_ground_symbol": GROUND,
        "source_root_frame_zero_based": 20,
        "source_level_symbol": 783,
        "source_spawn_facing": -1,
        "source_player_spawn_game_local": [528.15, -18.75],
        "spawn_level_local": [-506.5, -258.0],
        "source_camera": {"min_x": -6220, "max_x": 600, "height": 125, "jump": 3},
        "origin_level_px": list(ORIGIN),
        "size_px": list(SIZE),
        "occupied_pixels": occupied,
        "fill_groups": record.get("fill_groups"),
        "contours": record.get("contours"),
        "sample_floor_probes": floors,
        "source_mask_png_sha256": hashlib.sha256(ns.mask.read_bytes()).hexdigest(),
        "partial_proof_only": True,
        "never_embed_as_complete_level": True,
    }
    ns.outdir.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ns.mask, ns.outdir / "level1b_ground_source_only.png")
    (ns.outdir / "source-ground-report.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8"
    )
    print("B07 original source Bank Street #782 mask OK:",
          f"{SIZE[0]}x{SIZE[1]}, origin={ORIGIN}, filled={occupied}",
          f"floor probes={len(floors)}", flush=True)

if __name__ == "__main__":
    main()
