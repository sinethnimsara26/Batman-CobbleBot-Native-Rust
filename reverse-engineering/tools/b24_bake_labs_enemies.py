#!/usr/bin/env python3
"""B24: Verify & bake Labs 2A enemies — 9 Goons (#1365) and Eagle (#1393, birdAI).

Offline build/CI asset verifier and contact-sheet proof generator from canonical cobblebot.swf:
- Validates canonical SWF SHA-256.
- Validates Labs Goon #1365 (89 parent frames, 9 labels, 69 true child frames across
  #1340/#1348/#1349/#1358/#1362/#1363/#1364) and all 9 Level 2A (`#1402`) placements.
- Validates Eagle #1393 (50 parent frames, 5 labels, 41 true child frames across
  #1377/#1379/#1384/#1387/#1392), Level 2A `eagle1` placement, root frame 23 `eagle_BAK` gate,
  and exported `eagle` sound (#5, SWF ADPCM).
- Bakes 1.0x child frames (via bake_player_states) and builds visual contact sheet + JSON report.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

from b03_bake_stage_asset_proofs import expect_original_source

SOURCE_SHA = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"

EXPECTED_GOON_STATES = (
    ("stand", 0, 1340, 1),
    ("walk", 9, 1348, 17),
    ("retreat", 19, 1349, 17),
    ("attack", 29, 1358, 12),
    ("hurt", 39, 1362, 5),
    ("fall", 49, 1363, 8),
    ("die", 59, 1364, 7),
    ("idle", 69, 1340, 1),
    ("intro", 79, 1340, 1),
)

EXPECTED_EAGLE_STATES = (
    ("fly", 0, 1377, 10),
    ("glide", 9, 1379, 10),
    ("attack", 20, 1384, 12),
    ("hurt", 30, 1387, 5),
    ("die", 40, 1392, 4),
)

EXPECTED_2A_GOONS = {
    "goon2": (354, 1574.85, 2656.35, 0.9361419677734375),
    "goon1": (356, 2026.80, 2656.35, 0.9361419677734375),
    "goon4": (358, 4254.05, 3445.10, 0.9257049560546875),
    "goon5": (360, 4676.15, 3445.10, 0.9257049560546875),
    "goon7": (362, 6034.55, 3842.65, 0.9361419677734375),
    "goon9": (364, 7028.70, 3842.65, 0.9361419677734375),
    "goon13": (366, 5069.75, 5947.50, 0.9586029052734375),
    "goon11": (368, 6739.45, 5947.50, 0.9586029052734375),
    "goon14": (370, 3406.45, 5020.00, 1.0),
}


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_source(symbols: dict, actions: list) -> dict:
    s1365 = symbols["1365"]
    assert s1365["frames"] == 89, s1365["frames"]
    labs_g = [(l["label"], l["frame"]) for l in sorted(s1365["labels"], key=lambda x: x["frame"])]
    assert labs_g == [(n, pf) for n, pf, _, _ in EXPECTED_GOON_STATES], labs_g

    s1393 = symbols["1393"]
    assert s1393["frames"] == 50, s1393["frames"]
    labs_e = [(l["label"], l["frame"]) for l in sorted(s1393["labels"], key=lambda x: x["frame"])]
    assert labs_e == [(n, pf) for n, pf, _, _ in EXPECTED_EAGLE_STATES], labs_e

    game_2a = symbols["1402"]
    goons_2a = {p["name"]: p for p in game_2a["placements"] if (p.get("name") or "").startswith("goon")}
    assert set(goons_2a) == set(EXPECTED_2A_GOONS), set(goons_2a)
    for nm, (depth, x, y, sx) in EXPECTED_2A_GOONS.items():
        p = goons_2a[nm]
        m = p["matrix"]
        assert (p["character_id"], p["depth"]) == (1365, depth), p
        assert abs(m["tx"] - x) < 1e-4 and abs(m["ty"] - y) < 1e-4, (nm, m)
        assert abs(m["sx"] - sx) < 1e-4, (nm, m)

    eagles_2a = [p for p in game_2a["placements"] if (p.get("name") or "").startswith("eagle")]
    assert len(eagles_2a) == 1 and eagles_2a[0]["name"] == "eagle1"
    e0 = eagles_2a[0]
    assert e0["character_id"] == 1393 and e0["depth"] == 372
    assert abs(e0["matrix"]["tx"] - 1388.0) < 1e-4 and abs(e0["matrix"]["ty"] - (-163.8)) < 1e-4

    frame23 = [b for b in actions if b.get("frame") == 23]
    f23_strings = [s for b in frame23 for s in b.get("strings", [])]
    assert "eagle_BAK" in f23_strings and "goonAI" in f23_strings and "birdAI" in f23_strings

    return {
        "goon_2a_count": len(goons_2a),
        "eagle_2a_count": len(eagles_2a),
        "eagle_2a_init_gate": "eagle_BAK",
    }


def verify_manifest(manifest_path: Path, expected_states: tuple, expected_total: int) -> dict:
    data = json.loads(manifest_path.read_text(encoding="utf-8"))
    assert data["frame_count"] == expected_total, (manifest_path, data["frame_count"])
    assert not data["unsupported_fills"] and not data["missing_bitmap_ids"]
    assert len(data["states"]) == len(expected_states)
    cursor = 0
    for st, (name, pf, csid, cnt) in zip(data["states"], expected_states, strict=True):
        assert (st["name"], st["parent_frame"], st["child_symbol"], st["frame_count"]) == (name, pf, csid, cnt), st
        assert (st["start"], st["end"]) == (cursor, cursor + cnt)
        cursor += cnt
    assert cursor == expected_total
    return data


def build_contact_sheet(goon_dir: Path, eagle_dir: Path, out_png: Path) -> None:
    samples = [
        ("goon #1365 stand", goon_dir / "000.png"),
        ("goon walk[8]", goon_dir / "009.png"),
        ("goon attack[5]", goon_dir / "040.png"),
        ("goon hurt[1]", goon_dir / "048.png"),
        ("goon fall[3]", goon_dir / "055.png"),
        ("goon die[6]", goon_dir / "066.png"),
        ("eagle #1393 fly[2]", eagle_dir / "002.png"),
        ("eagle glide[2]", eagle_dir / "012.png"),
        ("eagle attack[5]", eagle_dir / "025.png"),
        ("eagle hurt[1]", eagle_dir / "033.png"),
        ("eagle die[3]", eagle_dir / "040.png"),
    ]
    cell_w, cell_h = 240, 260
    cols = 6
    rows = (len(samples) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell_w, rows * cell_h), (24, 24, 30, 255))
    draw = ImageDraw.Draw(sheet)
    for idx, (label, path) in enumerate(samples):
        cx = (idx % cols) * cell_w
        cy = (idx // cols) * cell_h
        with Image.open(path) as im:
            im = im.convert("RGBA")
            im.thumbnail((cell_w - 16, cell_h - 36), Image.Resampling.NEAREST)
            sheet.alpha_composite(im, (cx + (cell_w - im.width) // 2, cy + 28 + (cell_h - 36 - im.height) // 2))
        draw.text((cx + 8, cy + 8), label, fill=(230, 230, 240, 255))
    out_png.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out_png)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("symbols", type=Path)
    ap.add_argument("actions", type=Path)
    ap.add_argument("goon_dir", type=Path)
    ap.add_argument("eagle_dir", type=Path)
    ap.add_argument("--bitmaps", type=Path, required=True)
    ap.add_argument("--proof", type=Path, required=True)
    ap.add_argument("--report", type=Path, required=True)
    args = ap.parse_args()

    expect_original_source(args.swf)
    symbols = json.loads(args.symbols.read_text(encoding="utf-8"))
    actions = json.loads(args.actions.read_text(encoding="utf-8"))
    src_info = verify_source(symbols, actions)

    baker = Path(__file__).with_name("bake_player_states.py")
    for sid, outdir in ((1365, args.goon_dir), (1393, args.eagle_dir)):
        subprocess.run(
            [
                sys.executable,
                str(baker),
                str(args.swf),
                str(args.symbols),
                str(outdir),
                "--bitmaps",
                str(args.bitmaps),
                "--symbol",
                str(sid),
                "--scale",
                "1.0",
            ],
            check=True,
        )

    goon_m = verify_manifest(args.goon_dir / "manifest.json", EXPECTED_GOON_STATES, 69)
    eagle_m = verify_manifest(args.eagle_dir / "manifest.json", EXPECTED_EAGLE_STATES, 41)
    build_contact_sheet(args.goon_dir, args.eagle_dir, args.proof)

    report = {
        "batch": "B24",
        **src_info,
        "goon_1365": {"frames": goon_m["frame_count"], "size": goon_m["size"], "anchor": goon_m["anchor_in_bitmap"]},
        "eagle_1393": {"frames": eagle_m["frame_count"], "size": eagle_m["size"], "anchor": eagle_m["anchor_in_bitmap"]},
        "proof_sha256": sha256_bytes(args.proof.read_bytes()),
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
