#!/usr/bin/env python3
"""B14: original source Level 1C Penguin Arena (#1095) & Ground (#1094) proof baker.

Development-only offline source extraction & verification from canonical cobblebot.swf:
- Validates root21 placements (#134 moon, #1251 game, #716 smaller HUD scale, #1250 boss, #691 player).
- Validates collision mask #1094 (`level1c_penguin`) dimensions, occupied pixel count, floor, and walls.
- Bakes all 60 non-empty 1x tiles of arena #1095 (excluding #1094) and packs a dev-only BCLVT001 check file.
- Renders #1078 distant background and sampled HQ3x arena tiles + contact sheet for visual verification.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

from b03_bake_stage_asset_proofs import expect_original_source
from bake_level_tiles import load_bitmaps
from bake_sprite_frames import display_list, load_shapes, load_text_defs, render_symbol, symbol_bounds
from swf_re import cstr, parse_place2, read_rect, tags

SOURCE_ROOT_FRAME = 21
SOURCE_GAME_ID = 1251
SOURCE_LEVEL_ID = 1095
SOURCE_GROUND_ID = 1094
SOURCE_BG_ID = 1078
SOURCE_HUD_ID = 716
SOURCE_BOSS_ID = 1250
SOURCE_PLAYER_ID = 691
SOURCE_MOON_ID = 134

LEVEL_ORIGIN = (1485.65, 239.25)
EXPECTED_GROUND_SIZE = [2813, 926]
EXPECTED_GROUND_ORIGIN = [-1731, -549]
EXPECTED_GROUND_PIXELS = 1502432

EXPECTED_PICKUPS = {
    "item_br01": (-647.1, 153.0),
    "item_br02": (394.4, 201.8),
    "item_br03": (722.65, 212.4),
    "item_br04": (1239.65, 196.95),
}

EXPECTED_PENGUIN_CLIPS = [
    ("stand", 0, 9, 1103, 21),
    ("walk", 9, 10, 1117, 8),
    ("jump", 19, 10, 1135, 9),
    ("dodge", 29, 10, 1149, 10),
    ("block", 39, 10, 1156, 14),
    ("attack", 49, 10, 1172, 10),
    ("attack2", 59, 10, 1205, 23),
    ("hurt", 69, 9, 1207, 6),
    ("fall", 78, 10, 1224, 21),
    ("die", 88, 11, 1249, 40),
]

HQ_SAMPLE_TILES = ((0, 0), (600, 0), (1200, 0), (1800, 0))


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def png_and_rgba_hashes(path: Path) -> tuple[str, str, list[int]]:
    png_sha = sha256_bytes(path.read_bytes())
    with Image.open(path) as img:
        rgba = img.convert("RGBA")
        if rgba.getchannel("A").getbbox() is None:
            raise AssertionError(f"Empty transparent raster: {path}")
        return png_sha, sha256_bytes(rgba.tobytes()), list(rgba.size)


def assert_close(actual: float, expected: float, tol: float = 1e-5, label: str = "") -> None:
    if abs(actual - expected) > tol:
        raise AssertionError(f"{label}: expected {expected}, got {actual}")


def extract_root_timeline(swf_path: Path) -> dict:
    raw = swf_path.read_bytes()
    _, pos = read_rect(raw, 8)
    pos += 2
    frame_count = struct.unpack_from("<H", raw, pos)[0]
    pos += 2
    frame = 0
    places = []
    removals = []
    labels = []
    for code, pl, toff, _ in tags(raw, pos, len(raw)):
        if code == 1:
            frame += 1
        elif code == 43:
            lab, _ = cstr(pl, 0)
            labels.append({"frame": frame, "label": lab})
        elif code == 26:
            entry = parse_place2(pl)
            if entry:
                entry["frame"] = frame
                entry["tag_offset"] = toff
                places.append(entry)
        elif code == 28 and len(pl) >= 2:
            removals.append({
                "depth": struct.unpack_from("<H", pl, 0)[0],
                "frame": frame,
                "tag_offset": toff,
            })
    return {"frames": frame_count, "placements": places, "removals": removals, "labels": labels}


def verify_root21_and_stage(swf_path: Path, instances: dict, symbols: dict) -> dict:
    stage = instances["level1c_penguin"]
    if (stage["root_frame"], stage["game_symbol_id"], stage["level_symbol_id"]) != (
        SOURCE_ROOT_FRAME,
        SOURCE_GAME_ID,
        SOURCE_LEVEL_ID,
    ):
        raise AssertionError(f"Unexpected level1c_penguin IDs: {stage}")
    if stage["ground"]["character_id"] != SOURCE_GROUND_ID:
        raise AssertionError(f"Unexpected ground symbol: {stage['ground']}")

    game_dl = display_list(symbols[str(SOURCE_GAME_ID)], 0)
    level_entry = next(
        p for p in stage["named_game_instances"] if p.get("name") == "level"
    )
    assert_close(level_entry["matrix"]["tx"], LEVEL_ORIGIN[0], 1e-4, "level.tx")
    assert_close(level_entry["matrix"]["ty"], LEVEL_ORIGIN[1], 1e-4, "level.ty")

    # Root placements: verify persisted sky #131 (depth 1 from frame 19), moon #134, and smaller HUD scale (~0.8636x).
    root_sym = extract_root_timeline(swf_path)
    root_dl = display_list(root_sym, SOURCE_ROOT_FRAME)
    root_chars = {p.get("character_id") for p in root_dl.values()}
    if 131 not in root_chars:
        raise AssertionError("Expected sky backdrop #131 persisted at depth 1 from root frame 19 into frame 21")
    moon = next((p for p in root_dl.values() if p.get("character_id") == SOURCE_MOON_ID), None)
    hud = next((p for p in root_dl.values() if p.get("character_id") == SOURCE_HUD_ID), None)
    game_root = next((p for p in root_dl.values() if p.get("character_id") == SOURCE_GAME_ID), None)
    if moon is None or hud is None or game_root is None:
        raise AssertionError("Missing moon/HUD/game placement on root21")

    assert_close(hud["matrix"]["sx"], 0.8636322021484375, 1e-6, "HUD.sx")
    assert_close(hud["matrix"]["sy"], 0.863800048828125, 1e-6, "HUD.sy")
    assert_close(hud["matrix"]["tx"], 159.0, 1e-4, "HUD.tx")
    assert_close(hud["matrix"]["ty"], -11.0, 1e-4, "HUD.ty")

    assert_close(game_root["matrix"]["tx"], 0.2, 1e-4, "root_game.tx")
    assert_close(game_root["matrix"]["ty"], 15.95, 1e-4, "root_game.ty")

    # Verify bg #1078, player #691, boss #1250, and pickups inside game #1251.
    bg = next((p for p in game_dl.values() if p.get("character_id") == SOURCE_BG_ID), None)
    boss = next((p for p in game_dl.values() if p.get("character_id") == SOURCE_BOSS_ID), None)
    player = next((p for p in game_dl.values() if p.get("character_id") == SOURCE_PLAYER_ID), None)
    if bg is None or boss is None or player is None:
        raise AssertionError("Missing bg/boss/player in game #1251")
    assert_close(bg["matrix"]["sx"], 1.029632568359375, 1e-6, "bg.sx")
    assert_close(bg["matrix"]["ty"], -13.1, 1e-4, "bg.ty")
    assert_close(boss["matrix"]["tx"], 1035.65, 1e-4, "boss.tx")
    assert_close(boss["matrix"]["ty"], 316.2, 1e-4, "boss.ty")
    assert_close(boss["matrix"]["sx"], -0.752166748046875, 1e-6, "boss.sx")
    assert_close(player["matrix"]["tx"], 122.05, 1e-4, "player.tx")
    assert_close(player["matrix"]["ty"], 281.25, 1e-4, "player.ty")

    pickups = {
        p["name"]: (p["matrix"]["tx"], p["matrix"]["ty"])
        for p in stage["named_game_instances"]
        if p.get("name", "").startswith("item_br")
    }
    if set(pickups) != set(EXPECTED_PICKUPS):
        raise AssertionError(f"Unexpected pickups on level1c_penguin: {pickups}")
    for name, (ex, ey) in EXPECTED_PICKUPS.items():
        ax, ay = pickups[name]
        assert_close(ax, ex, 1e-4, f"{name}.tx")
        assert_close(ay, ey, 1e-4, f"{name}.ty")

    # Verify Penguin #1250 timeline labels & direct-child MovieClips.
    boss_sym = symbols[str(SOURCE_BOSS_ID)]
    if boss_sym["frames"] != 99:
        raise AssertionError(f"Expected 99 parent frames on #1250, got {boss_sym['frames']}")
    boss_labels = {x["label"]: x["frame"] for x in boss_sym.get("labels", [])}
    verified_clips = []
    for label, start_frame, span_frames, child_cid, expected_child_frames in EXPECTED_PENGUIN_CLIPS:
        if boss_labels.get(label) != start_frame:
            raise AssertionError(f"Label mismatch on #1250: {label} -> {boss_labels.get(label)}")
        dl = display_list(boss_sym, start_frame)
        child = next((p for p in dl.values() if p.get("character_id") == child_cid), None)
        if child is None:
            raise AssertionError(f"Missing child clip #{child_cid} on #1250 frame {start_frame}")
        child_frames = symbols[str(child_cid)]["frames"]
        if child_frames != expected_child_frames:
            raise AssertionError(f"Child #{child_cid} frames {child_frames} != {expected_child_frames}")
        verified_clips.append({
            "label": label,
            "start_frame": start_frame,
            "span_frames": span_frames,
            "child_symbol_id": child_cid,
            "child_frames": child_frames,
            "instance_name": child.get("name"),
        })

    return {
        "hud_matrix": hud["matrix"],
        "moon_matrix": moon["matrix"],
        "root_game_matrix": game_root["matrix"],
        "bg_matrix": bg["matrix"],
        "boss_matrix": boss["matrix"],
        "player_matrix": player["matrix"],
        "pickups": pickups,
        "penguin_clips": verified_clips,
        "total_child_frames": sum(c["child_frames"] for c in verified_clips),
    }


def verify_ground_mask(mask_manifest_path: Path, mask_png_path: Path) -> dict:
    manifest = read_json(mask_manifest_path)
    entries = manifest if isinstance(manifest, list) else manifest["levels"]
    entry = next(item for item in entries if item.get("level") == "level1c_penguin")
    if entry["ground_character_id"] != SOURCE_GROUND_ID:
        raise AssertionError(f"Mask character mismatch: {entry}")
    if entry["size_px"] != EXPECTED_GROUND_SIZE or entry["origin_level_px"] != EXPECTED_GROUND_ORIGIN:
        raise AssertionError(f"Mask geometry mismatch: {entry}")
    if entry["occupied_pixels"] != EXPECTED_GROUND_PIXELS:
        raise AssertionError(f"Mask occupied pixels mismatch: {entry['occupied_pixels']}")

    mask_sha256 = sha256_bytes(mask_png_path.read_bytes())
    with Image.open(mask_png_path) as img:
        luma = img.convert("L")
        w, h = luma.size
        ox, oy = entry["origin_level_px"]

        def solid(gx: float, gy: float) -> bool:
            lx = round(gx - LEVEL_ORIGIN[0]) - ox
            ly = round(gy - LEVEL_ORIGIN[1]) - oy
            if lx < 0 or ly < 0 or lx >= w or ly >= h:
                return False
            return luma.getpixel((lx, ly)) >= 128

        # Spawn and boss sit above floor (level_y=124 -> game_y=363.25) and land solidly.
        for gx in (50.0, 122.0, 500.0, 1035.0, 1500.0, 2250.0):
            if solid(gx, 360.0):
                raise AssertionError(f"Unexpected solid above arena floor at gx={gx}")
            if not solid(gx, 365.0):
                raise AssertionError(f"Expected solid arena floor at gx={gx}")
        if not solid(-10.0, 300.0) or not solid(2320.0, 300.0):
            raise AssertionError("Expected solid left/right arena boundary walls")

    return {
        "mask_sha256": mask_sha256,
        "size": entry["size_px"],
        "origin": entry["origin_level_px"],
        "occupied_pixels": entry["occupied_pixels"],
        "contours": entry["contours"],
        "fill_groups": entry["fill_groups"],
    }


def render_bg_cutout(
    symbols: dict,
    shapes: dict,
    bitmaps: dict,
    out_dir: Path,
) -> dict:
    bounds = symbol_bounds(SOURCE_BG_ID, 0, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0), symbols, shapes)
    left = math.floor(bounds[0]) - 2
    top = math.floor(bounds[1]) - 2
    right = math.ceil(bounds[2]) + 2
    bottom = math.ceil(bounds[3]) + 2
    w, h = right - left, bottom - top
    canvas = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    bg_unsupported: set[str] = set()
    bg_missing: set[int] = set()
    render_symbol(
        SOURCE_BG_ID,
        0,
        (1.0, 0.0, 0.0, 1.0, -left, -top),
        canvas,
        symbols,
        shapes,
        bitmaps,
        bg_unsupported,
        bg_missing,
    )
    if bg_unsupported or bg_missing:
        raise AssertionError(f"BG #1078 render issues: unsupported={bg_unsupported} missing={bg_missing}")
    out_path = out_dir / "level1c_bg_1x.png"
    canvas.save(out_path, optimize=True)
    png_sha, rgba_sha, size = png_and_rgba_hashes(out_path)
    return {
        "file": out_path.name,
        "bounds": [left, top, right, bottom],
        "size": size,
        "png_sha256": png_sha,
        "rgba_sha256": rgba_sha,
    }


def build_contact_sheet(hq_dir: Path, out_dir: Path, sample_records: list[dict]) -> dict:
    sheet = Image.new("RGB", (1236, 876), "#1c222e")
    draw = ImageDraw.Draw(sheet)
    for idx, rec in enumerate(sample_records):
        col = idx % 2
        row = idx // 2
        x0 = col * 612 + 6
        y0 = row * 432 + 6
        draw.text((x0, y0), f"1C Arena #1095 HQ3x world ({rec['world_x']},{rec['world_y']})", fill="white")
        with Image.open(hq_dir / rec["file"]) as tile:
            thumb = tile.convert("RGBA").resize((600, 400), Image.Resampling.LANCZOS)
            bg = Image.new("RGBA", (600, 400), "#252b38")
            bg.alpha_composite(thumb)
            sheet.paste(bg.convert("RGB"), (x0, y0 + 20))
    draw.text((8, 858), "B14 Level 1C Penguin Arena (#1095/#1094) source proof", fill="#ffcb81")
    out_path = out_dir / "B14_PENGUIN_ARENA_CONTACT_SHEET.png"
    sheet.save(out_path, optimize=True)
    return {"file": out_path.name, "sha256": sha256_bytes(out_path.read_bytes()), "size": list(sheet.size)}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--swf", required=True, type=Path)
    parser.add_argument("--symbols", required=True, type=Path)
    parser.add_argument("--bitmaps", required=True, type=Path)
    parser.add_argument("--instances", required=True, type=Path)
    parser.add_argument("--mask-manifest", required=True, type=Path)
    parser.add_argument("--mask-png", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()

    args.out.mkdir(parents=True, exist_ok=True)
    source_sha = expect_original_source(args.swf)
    symbols = read_json(args.symbols)
    instances = read_json(args.instances)

    stage_audit = verify_root21_and_stage(args.swf, instances, symbols)
    ground_audit = verify_ground_mask(args.mask_manifest, args.mask_png)

    unsupported: set[str] = set()
    shapes = load_shapes(args.swf, unsupported)
    bitmaps = load_bitmaps(args.bitmaps)
    bg_cutout = render_bg_cutout(symbols, shapes, bitmaps, args.out)

    baker = str(Path(__file__).with_name("bake_level_tiles.py"))
    packer = str(Path(__file__).with_name("pack_level_tiles.py"))

    # 1. Full 1x arena bake of #1095 (excluding ground #1094) + pack verification.
    arena_1x_dir = args.out / "arena_1x"
    subprocess.run(
        [
            sys.executable,
            baker,
            str(args.swf),
            str(args.symbols),
            str(arena_1x_dir),
            "--bitmaps",
            str(args.bitmaps),
            "--symbol",
            str(SOURCE_LEVEL_ID),
            "--place-x",
            str(LEVEL_ORIGIN[0]),
            "--place-y",
            str(LEVEL_ORIGIN[1]),
            "--exclude-symbol",
            str(SOURCE_GROUND_ID),
            "--include-source-static-text",
        ],
        check=True,
    )
    arena_1x_manifest = read_json(arena_1x_dir / "manifest.json")
    if arena_1x_manifest.get("unsupported_fills") or arena_1x_manifest.get("missing_ids"):
        raise AssertionError(
            f"Unexpected missing/unsupported fills on #1095 1x bake: {arena_1x_manifest}"
        )
    if len(arena_1x_manifest["tiles"]) != 60:
        raise AssertionError(f"Expected 60 tiles on #1095 1x bake, got {len(arena_1x_manifest['tiles'])}")

    packed_1x = args.out / "level1c_arena_1x_dev.bin"
    subprocess.run(
        [sys.executable, packer, str(arena_1x_dir), str(packed_1x)],
        check=True,
    )
    raw_pack = packed_1x.read_bytes()
    if raw_pack[:8] != b"BCLVT001" or struct.unpack_from("<I", raw_pack, 8)[0] != 60:
        raise AssertionError("Packed level1c_arena_1x_dev.bin header mismatch")

    # 2. HQ3x sampled camera-band proofs covering player spawn, center arena, and Penguin spawn.
    arena_hq3x_dir = args.out / "arena_hq3x_samples"
    hq_cmd = [
        sys.executable,
        baker,
        str(args.swf),
        str(args.symbols),
        str(arena_hq3x_dir),
        "--bitmaps",
        str(args.bitmaps),
        "--symbol",
        str(SOURCE_LEVEL_ID),
        "--place-x",
        str(LEVEL_ORIGIN[0]),
        "--place-y",
        str(LEVEL_ORIGIN[1]),
        "--exclude-symbol",
        str(SOURCE_GROUND_ID),
        "--include-source-static-text",
        "--source-over-strokes",
        "--output-scale",
        "3",
        "--supersample",
        "2",
        "--downsample",
        "lanczos",
    ]
    for wx, wy in HQ_SAMPLE_TILES:
        hq_cmd.extend(("--only-tile", str(wx), str(wy)))
    subprocess.run(hq_cmd, check=True)

    hq_manifest = read_json(arena_hq3x_dir / "manifest.json")
    hq_samples = []
    for tile in hq_manifest["tiles"]:
        png_sha, rgba_sha, size = png_and_rgba_hashes(arena_hq3x_dir / tile["file"])
        hq_samples.append({
            "world_x": tile["world_x"],
            "world_y": tile["world_y"],
            "file": tile["file"],
            "size": size,
            "png_sha256": png_sha,
            "rgba_sha256": rgba_sha,
        })
    if len(hq_samples) != len(HQ_SAMPLE_TILES):
        raise AssertionError(f"Expected {len(HQ_SAMPLE_TILES)} HQ3x tiles, got {len(hq_samples)}")

    sheet = build_contact_sheet(arena_hq3x_dir, args.out, hq_samples)

    report = {
        "batch": "LEVEL2-B14",
        "canonical_source_fws_sha256": source_sha,
        "source_root_zero_based": SOURCE_ROOT_FRAME,
        "source_game_id": SOURCE_GAME_ID,
        "source_level_id": SOURCE_LEVEL_ID,
        "source_ground_id": SOURCE_GROUND_ID,
        "source_bg_id": SOURCE_BG_ID,
        "source_boss_id": SOURCE_BOSS_ID,
        "level_origin_game_local": list(LEVEL_ORIGIN),
        "stage_audit": stage_audit,
        "ground_mask_audit": ground_audit,
        "bg_cutout": bg_cutout,
        "arena_1x_world_bounds": arena_1x_manifest["world_bounds"],
        "arena_1x_rendered_tiles": len(arena_1x_manifest["tiles"]),
        "arena_1x_pack_bytes": len(raw_pack),
        "arena_1x_pack_sha256": sha256_bytes(raw_pack),
        "arena_hq3x_samples": hq_samples,
        "contact_sheet": sheet,
        "dormant_source_metadata_only": True,
        "native_level1a_and_1b_goldens_unchanged": True,
    }
    report_path = args.out / "B14_PENGUIN_ARENA_SOURCE_REPORT.json"
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(
        "B14_PENGUIN_ARENA_GATE_PASS",
        source_sha,
        f"tiles_1x={len(arena_1x_manifest['tiles'])}",
        f"pack_bytes={len(raw_pack)}",
        f"hq_samples={len(hq_samples)}",
        f"penguin_child_frames={stage_audit['total_child_frames']}",
        flush=True,
    )


if __name__ == "__main__":
    main()
