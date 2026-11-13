#!/usr/bin/env python3
"""B20: original source Level 2A Labs (#1317/#1316/#1402) scenery, mask & object separator.

Development-only offline source extraction & verification from canonical cobblebot.swf:
- Validates root frame 23 (`#14` solid black root backdrop, `#1402` game clip, `#1317`
  level placement `(1210.1, 238.25)`, `player` spawn `(227.2, 115.4)` facing `-1`).
- Validates `#1316` collision mask (`8781x10060`, origin `(-1542, -2307)`, `19,922,753` px).
- Validates the full 177-tile 1x `assets/level2a_tiles.bin` / `manifest.json` bake
  (`#1317` excl. `#1316`, `--include-source-static-text`) with zero unsupported fills/missing glyphs.
- Renders sampled HQ3x (6x supersampled Lanczos -> 3x) vs 1x comparisons at 4 multi-floor
  vertical locations (`(0,0)`, `(1800,2400)`, `(4800,3600)`, `(4200,6800)`) over `#14` black.
- Separates `#1402` dynamic & foreground objects (`18` doors, `3` elevators, `9` goons,
  `1` eagle, `4` checkpoints, `10` cameras, `1` health, `9` ammo, `18` front door jambs)
  and emits typed `assets/labs2a_objects.rs` + `labs2a-scenery-report.json`.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import struct
from pathlib import Path
from PIL import Image

from bake_level_tiles import load_bitmaps
from bake_sprite_frames import load_shapes, load_text_defs, render_symbol
from swf_re import read_rect, tags

CANONICAL_SWF_SHA256 = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
SOURCE_ROOT_FRAME = 23
SOURCE_BACKDROP_ID = 14
SOURCE_GAME_ID = 1402
SOURCE_LEVEL_ID = 1317
SOURCE_GROUND_ID = 1316
LEVEL_ORIGIN = (1210.1, 238.25)
EXPECTED_PLAYER_SPAWN = (227.2, 115.4)
EXPECTED_GROUND_ORIGIN = [-1542, -2307]
EXPECTED_GROUND_SIZE = [8781, 10060]
EXPECTED_GROUND_PIXELS = 19922753
EXPECTED_1X_TILES = 177

SAMPLE_COORDS = (
    ("rooftop", 0, 0),
    ("upper_lab", 1800, 2400),
    ("mid_lab", 4800, 3600),
    ("tv_exit", 4200, 6800),
)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_root_backdrop_black(swf: Path, symbols: dict, shapes: dict, bitmaps: dict) -> dict:
    # Verify root frame 23 places #14 at depth 1 (no #134 cityscape) and renders solid opaque black.
    im = Image.new("RGBA", (600, 400), (255, 0, 255, 255))
    unsup: set[str] = set()
    missing: set[int] = set()
    m14 = (0.9918975830078125, 0.0, 0.0, 3.38983154296875, 300.0, 200.05)
    render_symbol(SOURCE_BACKDROP_ID, 0, m14, im, symbols, shapes, bitmaps, unsup, missing)
    assert not unsup and not missing, (unsup, missing)
    for pt in ((0, 0), (300, 200), (599, 399)):
        px = im.getpixel(pt)
        assert px == (0, 0, 0, 255), f"expected #14 solid black at {pt}, got {px}"
    return {
        "symbol_id": SOURCE_BACKDROP_ID,
        "matrix": [round(v, 6) for v in m14],
        "verified_opaque_black": True,
    }


def verify_ground_mask(manifest_path: Path, mask_path: Path) -> dict:
    records = json.loads(manifest_path.read_text(encoding="utf-8"))
    rec = [r for r in records if r.get("level") == "level2a"]
    assert len(rec) == 1, f"expected one level2a ground record, got {rec}"
    r = rec[0]
    assert r.get("ground_character_id") == SOURCE_GROUND_ID, r
    assert r.get("origin_level_px") == EXPECTED_GROUND_ORIGIN, r
    assert r.get("size_px") == EXPECTED_GROUND_SIZE, r
    assert r.get("occupied_pixels") == EXPECTED_GROUND_PIXELS, r
    with Image.open(mask_path) as src:
        assert src.size == tuple(EXPECTED_GROUND_SIZE), src.size
        assert src.mode == "1", src.mode
        pix = src.convert("L").load()
    ox, oy = EXPECTED_GROUND_ORIGIN
    w, h = EXPECTED_GROUND_SIZE
    def solid(gx: float, gy: float) -> bool:
        lx = int(gx - LEVEL_ORIGIN[0]) - ox
        ly = int(gy - LEVEL_ORIGIN[1]) - oy
        return 0 <= lx < w and 0 <= ly < h and pix[lx, ly] >= 128
    # Spawn in elevator shaft at (227.2, 115.4): shaft interior open, left/right walls solid.
    assert not solid(227.2, 115.4), "player spawn must be open inside shaft"
    assert solid(160.0, 115.4), "left shaft wall must be solid"
    assert solid(510.0, 115.4), "right shaft wall must be solid"
    return {
        "ground_character_id": SOURCE_GROUND_ID,
        "origin_level_px": EXPECTED_GROUND_ORIGIN,
        "size_px": EXPECTED_GROUND_SIZE,
        "occupied_pixels": EXPECTED_GROUND_PIXELS,
        "mask_png_sha256": sha256_bytes(mask_path.read_bytes()),
    }


def render_hq3x_comparisons(
    swf: Path,
    symbols: dict,
    shapes: dict,
    bitmaps: dict,
    tiles_dir: Path,
    tiles_bin: Path,
    outdir: Path,
) -> list[dict]:
    manifest = json.loads((tiles_dir / "manifest.json").read_text(encoding="utf-8"))
    assert manifest.get("symbol") == SOURCE_LEVEL_ID, manifest
    assert len(manifest.get("tiles", [])) == EXPECTED_1X_TILES, len(manifest.get("tiles", []))
    assert not manifest.get("unsupported_fills"), manifest
    assert not manifest.get("missing_symbols"), manifest
    raw_bin = tiles_bin.read_bytes()
    assert raw_bin[:8] == b"BCLVT001", raw_bin[:8]
    assert struct.unpack_from("<I", raw_bin, 8)[0] == EXPECTED_1X_TILES

    by_coord = {(t["world_x"], t["world_y"]): t for t in manifest["tiles"]}
    results = []
    sheet_1x_panels = []
    sheet_hq_panels = []
    unsup: set[str] = set()
    missing: set[int] = set()
    excluded = {SOURCE_GROUND_ID}

    for label, wx, wy in SAMPLE_COORDS:
        t = by_coord[(wx, wy)]
        with Image.open(tiles_dir / t["file"]) as t_img:
            tile_1x = t_img.convert("RGBA")
        # Composite over #14 opaque black background
        comp_1x = Image.new("RGBA", (600, 400), (0, 0, 0, 255))
        comp_1x.alpha_composite(tile_1x)
        comp_1x.save(outdir / f"labs2a_{label}_1x.png")

        # HQ3x vector render: 6x supersample (with 2px logical overscan), premul Lanczos -> 3x (1800x1200)
        render_scale = 6
        margin = 2
        lw, lh = 600 + margin * 2, 400 + margin * 2
        super_canvas = Image.new("RGBA", (lw * render_scale, lh * render_scale), (0, 0, 0, 0))
        mat = (
            float(render_scale), 0.0, 0.0, float(render_scale),
            (LEVEL_ORIGIN[0] - (wx - margin)) * render_scale,
            (LEVEL_ORIGIN[1] - (wy - margin)) * render_scale,
        )
        render_symbol(
            SOURCE_LEVEL_ID, 0, mat, super_canvas, symbols, shapes, bitmaps,
            unsup, missing, excluded_symbols=excluded,
        )
        assert not unsup and not missing, (unsup, missing)
        cropped = super_canvas.crop((
            margin * render_scale, margin * render_scale,
            (margin + 600) * render_scale, (margin + 400) * render_scale,
        ))
        hq_fg = cropped.convert("RGBa").resize((1800, 1200), Image.Resampling.LANCZOS).convert("RGBA")
        comp_hq = Image.new("RGBA", (1800, 1200), (0, 0, 0, 255))
        comp_hq.alpha_composite(hq_fg)
        comp_hq.save(outdir / f"labs2a_{label}_hq3x.png")

        # Verify non-black room content (no black placeholders)
        px_iter = comp_1x.get_flattened_data() if hasattr(comp_1x, "get_flattened_data") else comp_1x.getdata()
        non_black = sum(1 for r, g, b, _ in px_iter if (r | g | b) > 0)
        assert non_black > 600 * 400 // 20, f"sample {label} unexpectedly black: {non_black}"
        results.append({
            "label": label,
            "world_origin": [wx, wy],
            "non_black_pixels_1x": non_black,
            "sha256_1x_rgba": sha256_bytes(comp_1x.tobytes()),
            "sha256_hq3x_rgba": sha256_bytes(comp_hq.tobytes()),
        })
        sheet_1x_panels.append(comp_1x)
        sheet_hq_panels.append(comp_hq.resize((600, 400), Image.Resampling.LANCZOS))

    sheet = Image.new("RGBA", (1200, 400 * len(SAMPLE_COORDS)), (0, 0, 0, 255))
    for i, (p1, phq) in enumerate(zip(sheet_1x_panels, sheet_hq_panels)):
        sheet.paste(p1, (0, i * 400))
        sheet.paste(phq, (600, i * 400))
    sheet.save(outdir / "labs2a_multifloor_1x_vs_hq3x.png")
    return results


def separate_game_objects(symbols: dict) -> dict:
    placements = symbols[str(SOURCE_GAME_ID)]["placements"]
    doors = []
    elevators = []
    checkpoints = []
    cameras = []
    goons = []
    eagles = []
    health_pickups = []
    batarang_pickups = []
    door_jambs = []
    static_accents = []
    player = None
    halo = None

    for p in placements:
        cid = p.get("character_id")
        depth = p.get("depth", 0)
        name = p.get("name") or ""
        m = p.get("matrix", {})
        tx = float(m.get("tx", 0.0))
        ty = float(m.get("ty", 0.0))
        sx = float(m.get("sx", 1.0))
        sy = float(m.get("sy", 1.0))
        entry = {"name": name, "symbol_id": cid, "depth": depth, "x": tx, "y": ty, "sx": sx, "sy": sy}
        if cid == SOURCE_LEVEL_ID or cid in (694, 1009, 1074):
            continue
        if name == "player" or cid == 691:
            player = entry
        elif name == "halo" or cid == 1399:
            halo = entry
        elif name.startswith("door"):
            doors.append(entry)
        elif name.startswith("elevator"):
            entry["limit_y"] = int(name[12:])
            elevators.append(entry)
        elif name.startswith("checkpoint_2a"):
            checkpoints.append(entry)
        elif name.startswith("camera_"):
            cameras.append(entry)
        elif name.startswith("goon"):
            goons.append(entry)
        elif name.startswith("eagle"):
            eagles.append(entry)
        elif name.startswith("health"):
            health_pickups.append(entry)
        elif name.startswith("item_br"):
            batarang_pickups.append(entry)
        elif cid in (1394, 1395, 1396):
            door_jambs.append(entry)
        else:
            static_accents.append(entry)

    for grp in (doors, elevators, checkpoints, cameras, goons, eagles, health_pickups, batarang_pickups, door_jambs, static_accents):
        grp.sort(key=lambda item: (item["depth"], item["name"]))

    assert len(doors) == 18, len(doors)
    assert len(elevators) == 3, len(elevators)
    assert len(checkpoints) == 4, len(checkpoints)
    assert len(cameras) == 10, len(cameras)
    assert len(goons) == 9, len(goons)
    assert len(eagles) == 1, len(eagles)
    assert len(health_pickups) == 1, len(health_pickups)
    assert len(batarang_pickups) == 9, len(batarang_pickups)
    assert len(door_jambs) == 18, len(door_jambs)
    assert player and (round(player["x"], 2), round(player["y"], 2)) == EXPECTED_PLAYER_SPAWN, player
    assert halo and halo["symbol_id"] == 1399, halo

    return {
        "player": player,
        "halo": halo,
        "doors": doors,
        "elevators": elevators,
        "checkpoints": checkpoints,
        "cameras": cameras,
        "goons": goons,
        "eagles": eagles,
        "health_pickups": health_pickups,
        "batarang_pickups": batarang_pickups,
        "door_jambs": door_jambs,
        "static_accents": static_accents,
    }


def emit_rust_objects(objs: dict, rust_out: Path) -> None:
    lines = ["// Auto-generated by b20_bake_labs2a_scenery.py from canonical #1402 (Level 2A).", ""]

    def emit_list(name: str, ty: str, items: list[dict], kind: str) -> None:
        lines.append(f"pub const {name}: [{ty}; {len(items)}] = [")
        for it in items:
            if kind == "door":
                lines.append(f"    {ty} {{ name: {it['name']!r}, symbol_id: {it['symbol_id']}, depth: {it['depth']}, x: {it['x']:.4f}, y: {it['y']:.4f}, sx: {it['sx']:.6f}, sy: {it['sy']:.6f} }},")
            elif kind == "elevator":
                lines.append(f"    {ty} {{ name: {it['name']!r}, symbol_id: {it['symbol_id']}, depth: {it['depth']}, x: {it['x']:.4f}, y: {it['y']:.4f}, sx: {it['sx']:.6f}, limit_y: {it['limit_y']} }},")
            elif kind == "trigger":
                lines.append(f"    {ty} {{ name: {it['name']!r}, symbol_id: {it['symbol_id']}, depth: {it['depth']}, x: {it['x']:.4f}, y: {it['y']:.4f}, sx: {it['sx']:.6f}, sy: {it['sy']:.6f} }},")
            elif kind in ("enemy", "pickup"):
                lines.append(f"    {ty} {{ name: {it['name']!r}, symbol_id: {it['symbol_id']}, depth: {it['depth']}, x: {it['x']:.4f}, y: {it['y']:.4f}, sx: {it['sx']:.6f} }},")
        lines.append("];")
        lines.append("")

    emit_list("DOORS", "LabsDoorPlacement", objs["doors"], "door")
    emit_list("ELEVATORS", "LabsElevatorPlacement", objs["elevators"], "elevator")
    emit_list("CHECKPOINTS", "LabsTriggerPlacement", objs["checkpoints"], "trigger")
    emit_list("CAMERAS", "LabsTriggerPlacement", objs["cameras"], "trigger")
    emit_list("GOONS", "LabsEnemyPlacement", objs["goons"], "enemy")
    emit_list("EAGLES", "LabsEnemyPlacement", objs["eagles"], "enemy")
    emit_list("HEALTH_PICKUPS", "LabsPickupPlacement", objs["health_pickups"], "pickup")
    emit_list("BATARANG_PICKUPS", "LabsPickupPlacement", objs["batarang_pickups"], "pickup")
    emit_list("DOOR_JAMBS", "LabsDoorPlacement", objs["door_jambs"], "door")

    rust_out.write_text("\n".join(lines).replace("'", '"'), encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("symbols_json", type=Path)
    ap.add_argument("bitmaps_dir", type=Path)
    ap.add_argument("mask_manifest", type=Path)
    ap.add_argument("mask_png", type=Path)
    ap.add_argument("tiles_dir", type=Path)
    ap.add_argument("tiles_bin", type=Path)
    ap.add_argument("outdir", type=Path)
    ap.add_argument("--emit-rust-objects", type=Path, required=True)
    args = ap.parse_args()

    swf_sha = sha256_bytes(args.swf.read_bytes())
    assert swf_sha == CANONICAL_SWF_SHA256, swf_sha
    symbols = json.loads(args.symbols_json.read_text(encoding="utf-8"))
    unsup: set[str] = set()
    shapes = load_shapes(args.swf, unsup)
    assert not unsup, unsup
    load_text_defs(args.swf)
    bitmaps = load_bitmaps(args.bitmaps_dir)

    args.outdir.mkdir(parents=True, exist_ok=True)
    backdrop_info = verify_root_backdrop_black(args.swf, symbols, shapes, bitmaps)
    ground_info = verify_ground_mask(args.mask_manifest, args.mask_png)
    multifloor = render_hq3x_comparisons(
        args.swf, symbols, shapes, bitmaps, args.tiles_dir, args.tiles_bin, args.outdir,
    )
    objs = separate_game_objects(symbols)
    emit_rust_objects(objs, args.emit_rust_objects)

    report = {
        "batch": "B20",
        "source_swf_sha256": swf_sha,
        "root_frame": SOURCE_ROOT_FRAME,
        "game_symbol": SOURCE_GAME_ID,
        "level_symbol": SOURCE_LEVEL_ID,
        "level_origin": list(LEVEL_ORIGIN),
        "backdrop": backdrop_info,
        "ground": ground_info,
        "tile_pack": {
            "count": EXPECTED_1X_TILES,
            "bytes": args.tiles_bin.stat().st_size,
            "sha256": sha256_bytes(args.tiles_bin.read_bytes()),
        },
        "multifloor_1x_vs_hq3x": multifloor,
        "separated_object_counts": {k: len(v) for k, v in objs.items() if isinstance(v, list)},
        "separated_objects": objs,
    }
    (args.outdir / "labs2a-scenery-report.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8",
    )
    print(
        "B20 Labs 2A (#1317/#1316/#1402) verified:",
        f"tiles={EXPECTED_1X_TILES}, ground={EXPECTED_GROUND_SIZE[0]}x{EXPECTED_GROUND_SIZE[1]},",
        f"doors={len(objs['doors'])}, elevators={len(objs['elevators'])},",
        f"goons={len(objs['goons'])}, eagle={len(objs['eagles'])},",
        f"cameras={len(objs['cameras'])}, checkpoints={len(objs['checkpoints'])},",
        f"pickups=1+{len(objs['batarang_pickups'])}",
    )


if __name__ == "__main__":
    main()
