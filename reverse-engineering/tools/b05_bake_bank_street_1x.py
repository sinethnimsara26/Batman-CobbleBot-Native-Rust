#!/usr/bin/env python3
"""B05: development-only source-authentic 1x Bank street (#783) frame-0 tiles.

These are original SWF *static source library* proofs, not live gameplay or
a stage to be packed/loaded by the native game. All original media remain in
CI .generated/ and artifacts; never commit the SWF or scene PNG files.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import struct
from pathlib import Path
import subprocess
import sys

from PIL import Image, ImageDraw

from b03_bake_stage_asset_proofs import (
    LOGIC_PREFIXES, TILE_H, TILE_W, expect_original_source,
    point_candidates, static_filter, validate_stage,
)
from bake_sprite_frames import display_list, load_shapes
from swf_re import read_rect, tags

# Independently downloaded and decoded B03 Windows artifact #13843344043.
# Pixel-level RGBA SHA256, deliberately independent of PNG encoder metadata.
B03_1X_RGBA = {
    (0, 0): "896f7373e114393a233ecd279dd8b143955f920913c8dde26013ebee32e59a6c",
    (600, 0): "2381074e5468f5c0e7fd8326898fb2d8bb907981467433936046e47649e8c8c6",
}

def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def rgba_sha(path):
    with Image.open(path) as image:
        rgba = image.convert("RGBA")
        if rgba.size != (TILE_W, TILE_H):
            raise AssertionError(f"Not original 600x400 pixel resolution: {path}")
        if rgba.getchannel("A").getbbox() is None:
            raise AssertionError(f"Blank original 1x scenery tile: {path}")
        return hashlib.sha256(rgba.tobytes()).hexdigest()

def select_source_views(locations):
    if not set(B03_1X_RGBA).issubset(set(locations)):
        raise AssertionError("Prior B03 source viewpoints are no longer geometry-derived")
    by_x = {}
    for x, y in locations:
        old = by_x.get(x)
        if old is None or (abs(y), y) < (abs(old), old):
            by_x[x] = y
    x_columns = sorted(by_x)
    if len(x_columns) < 7 or x_columns[-1] - x_columns[0] < 5 * TILE_W:
        raise AssertionError("Source Bank street has insufficient distinct geometry viewpoints")
    chosen = list(B03_1X_RGBA)
    for i in range(7):
        idx = round(i * (len(x_columns) - 1) / 6)
        t = (x_columns[idx], by_x[x_columns[idx]])
        if t not in chosen:
            chosen.append(t)
    result = sorted(chosen, key=lambda p: (p[1], p[0]))
    if len(result) < 7:
        raise AssertionError("Cannot reconstruct seven distinct original Bank street views")
    return result, x_columns

def source_definition_tags(swf, ids):
    raw = swf.read_bytes()
    _, start = read_rect(raw, 8)
    start += 4
    matched = {}
    for code, payload, _, _ in tags(raw, start, len(raw)):
        if code not in (2, 6, 11, 21, 22, 32, 35, 36, 37, 39, 48, 83) or len(payload) < 2:
            continue
        symbol = struct.unpack_from("<H", payload, 0)[0]
        if symbol in ids:
            matched[symbol] = code
    return matched


def source_depth_audit(stage, symbols, excluded):
    children = list(display_list(symbols[str(stage["level_symbol_id"])], 0).values())
    ids = {p.get("character_id") for p in children}
    ground_id = stage["ground"]["character_id"]
    if ground_id != 782 or ground_id not in ids or ground_id not in excluded:
        raise AssertionError("Original level-local collision ground #782 missing/not excluded")
    if 153 not in excluded or 783 in excluded:
        raise AssertionError("Source checkpoint #153 or backdrop #783 exclusion error")
    if [(p["name"], p["character_id"]) for p in stage["named_level_instances"]] != [("ground", 782)]:
        raise AssertionError("Unexpected source-named level object; static ownership needs new audit")
    game_dynamic = {
        p["character_id"] for p in stage["named_game_instances"]
        if p["name"] != "level" and p["name"].lower().startswith(LOGIC_PREFIXES)
    }
    overlapping = sorted(ids.intersection(game_dynamic) - {ground_id})
    if overlapping:
        raise AssertionError(f"Dynamic actor/prop symbol reused at level depth: {overlapping}")
    return {
        "level_frame_zero_based": 0,
        "direct_source_display_children": len(children),
        "direct_source_symbol_ids": sorted(x for x in ids if x is not None),
        "named_level_instances": stage["named_level_instances"],
        "game_dynamic_source_ids": sorted(game_dynamic),
        "reused_dynamic_source_ids_under_level": overlapping,
        "excluded_direct_source_ids": sorted(ids.intersection(excluded)),
    }

def contact_sheet(out, samples):
    images = [r for r in samples if r["file"]]
    rows = (len(images) + 2) // 3
    sheet = Image.new("RGB", (1836, rows * 438 + 24), "#262c38")
    draw = ImageDraw.Draw(sheet)
    for index, row in enumerate(images):
        x, y = (index % 3) * 612, (index // 3) * 438
        draw.text((x + 8, y + 5),
                  f"Original Bank #783 static 1x; world {row['world_x']},{row['world_y']}",
                  fill="white")
        with Image.open(out / "source_1x" / row["file"]) as im:
            base = Image.new("RGBA", (TILE_W, TILE_H), "#434953")
            base.alpha_composite(im.convert("RGBA"))
        sheet.paste(base.convert("RGB"), (x + 6, y + 24))
        draw.text((x + 6, y + 427), "SOURCE ART ONLY - no player/actors/collision", fill="#ffcb81")
    draw.text((6, rows * 438 + 6),
              "B05 source library proof - NOT live Flash gameplay, NOT a complete native stage",
              fill="#ffcb81")
    name = "B05_BANK_STREET_SOURCE_1X_CONTACT_SHEET.png"
    sheet.save(out / name, optimize=True)
    return {"file": name, "sha256": sha(out / name), "size": list(sheet.size)}

def main():
    parser = argparse.ArgumentParser()
    for arg in ("swf", "symbols", "bitmaps", "instances", "spec", "out"):
        parser.add_argument("--" + arg, required=True, type=Path)
    args = parser.parse_args()
    source_sha = expect_original_source(args.swf)
    symbols = read_json(args.symbols)
    stage = read_json(args.instances)["level1b"]
    spec = read_json(args.spec)["level1b"]
    if (stage["root_frame"], stage["game_symbol_id"], stage["level_symbol_id"]) != (20, 1075, 783):
        raise AssertionError("Source root20 Bank #1075/#783 mismatches verified original")
    if stage["ground"]["character_id"] != spec["ground_symbol"] or spec["ground_symbol"] != 782:
        raise AssertionError("Source ground #782 verification failed")
    origin = validate_stage("level1b", spec, stage, symbols)
    if origin != (1034.65, 239.25):
        raise AssertionError("Original source level placement changed")
    excluded = static_filter(stage)
    audit = source_depth_audit(stage, symbols, set(excluded))
    unsupported = set()
    shapes = load_shapes(args.swf, unsupported)
    locations, bounds, visible_children = point_candidates(stage, origin, symbols, shapes, excluded)
    chosen, x_columns = select_source_views(locations)

    source_tool = str(Path(__file__).with_name("bake_level_tiles.py"))
    def bake(dest, samples, source_text):
        command = [sys.executable, source_tool,
                   str(args.swf), str(args.symbols), str(dest),
                   "--bitmaps", str(args.bitmaps), "--symbol", "783",
                   "--place-x", str(origin[0]), "--place-y", str(origin[1])]
        if source_text:
            command.append("--include-source-static-text")
        for cid in excluded:
            command.extend(("--exclude-symbol", str(cid)))
        for x, y in samples:
            command.extend(("--only-tile", str(x), str(y)))
        print("B05_EXEC", " ".join(command), flush=True)
        subprocess.run(command, check=True)
        return read_json(dest / "manifest.json")

    # B03 baseline lacked source DefineText. Reproduce it independently
    # WITHOUT changing its historical 1x pixel evidence or native 1A art.
    legacy_dest = args.out / "b03_original_source_1x_recheck"
    legacy = bake(legacy_dest, sorted(B03_1X_RGBA), source_text=False)
    old_views = {(p["world_x"], p["world_y"]): p["file"] for p in legacy["tiles"]}
    for coordinate, digest in B03_1X_RGBA.items():
        if coordinate not in old_views or rgba_sha(legacy_dest / old_views[coordinate]) != digest:
            raise AssertionError(f"Prior B03 decoded-pixel regression at {coordinate}")
        print("B05_B03_1X_RGBA_IDENTITY_PASS", coordinate, digest, flush=True)
    absent = legacy.get("missing_ids", [])
    original_types = source_definition_tags(args.swf, set(absent))
    print("B05_ORIGINAL_SOURCE_DEFINITION_TYPES", original_types, flush=True)
    if sorted(absent) != [758, 771, 774] or original_types != {758: 11, 771: 11, 774: 11}:
        raise AssertionError(f"Previously missing Bank source IDs are not solely original DefineText: {absent}, {original_types}")

    # B05 improves only its source-art proof: load exact original DefineText
    # fonts and glyphs. No change to default Level1A bake semantics.
    dest = args.out / "source_1x"
    manifest = bake(dest, chosen, source_text=True)
    if (manifest["symbol"], manifest["placement"], manifest["tile_size"],
        manifest["logical_pixel_scale"]) != (783, list(origin), [600, 400], 1.0):
        raise AssertionError("Original source 1x level manifest contract changed")
    if manifest["excluded_symbols"] != excluded:
        raise AssertionError("Source actor/logic exclusion not forwarded to rasterizer")
    if manifest.get("partial_proof_only") is not True or manifest.get("never_embed_as_complete_level") is not True:
        raise AssertionError("B05 sampled art must not be packable as a complete level")
    if manifest["requested_tile_origins"] != [list(xy) for xy in chosen]:
        raise AssertionError("Rendered sample locations mismatch source-derived selection")
    if manifest.get("missing_ids"):
        raise AssertionError(f"Missing original art even after source DefineText reconstruction: {manifest['missing_ids']}")
    if manifest.get("source_static_text_enabled") is not True:
        raise AssertionError("B05 original DefineText glyph recovery was not enabled")
    rendered = {(p["world_x"], p["world_y"]): p["file"] for p in manifest["tiles"]}
    if len(rendered) < 7:
        raise AssertionError(f"Only {len(rendered)} visible 1x Bank street samples")
    for coordinate in B03_1X_RGBA:
        if coordinate not in rendered:
            raise AssertionError(f"Missing prior Bank sample coordinate {coordinate}")

    samples = []
    for x, y in chosen:
        filename = rendered.get((x, y))
        samples.append({
            "world_x": x, "world_y": y, "file": filename,
            "png_sha256": sha(dest / filename) if filename else None,
            "rgba_sha256": rgba_sha(dest / filename) if filename else None,
        })
    # B05 art is deliberately incomplete: packer MUST reject it even if a
    # future native-game developer accidentally passes the proof folder.
    forbidden = args.out / "DANGER_B05_PARTIAL_NATIVE_LEVEL.bin"
    reject = subprocess.run(
        [sys.executable, str(Path(__file__).with_name("pack_level_tiles.py")),
         str(dest), str(forbidden)], capture_output=True, text=True,
    )
    if reject.returncode == 0 or forbidden.exists() or "partial B03 source proof tiles cannot be packed" not in reject.stderr:
        raise AssertionError("Partial B05 original scene unexpectedly became packable")
    print("B05_INCOMPLETE_STAGE_PACK_GUARD_PASS", flush=True)
    sheet = contact_sheet(args.out, samples)
    report = {
        "batch": "LEVEL2-B05", "canonical_source_fws_sha256": source_sha,
        "source_root_zero_based": 20, "source_sprite_frame_zero_based": 0,
        "source_game_id": 1075, "source_static_level_id": 783,
        "source_collision_ground_excluded": 782, "level_origin_game_local": list(origin),
        "source_ownership": audit,
        "visible_level_source_child_geometry_count": len(visible_children),
        "source_local_static_bounds": bounds,
        "source_candidate_tile_count": len(locations),
        "distinct_source_derived_x_columns": len(x_columns),
        "source_1x_requested_views": [list(xy) for xy in chosen],
        "source_1x_rendered_nonempty_views": len(rendered),
        "excluded_source_logic_actor_prop_symbols": excluded,
        "source_1x_view_hashes": samples, "contact_sheet": sheet,
        "source_unsupported_fill_types": manifest["unsupported_fills"],
        "b03_original_1x_pixel_hashes_rechecked_on_unmodified_default_baker": True,
        "previous_b03_missing_ids": absent,
        "original_missing_source_id_definition_types": {str(k):v for k,v in original_types.items()},
        "b05_exact_original_define_text_glyphs_enabled": True,
        "b05_missing_original_definitions_after_fix": manifest["missing_ids"],
        "complete_level_pack_refusal_verified": True,
        "all_art_partial_proof_only_never_embeddable": True,
        "original_source_static_art_not_live_gameplay": True,
        "native_level1a_runtime_assets_and_all_goldens_unchanged": True,
        "no_playable_level1b_or_original_gameplay_parity_claim": True,
    }
    (args.out / "B05_BANK_STREET_SOURCE_1X_REPORT.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("B05_BANK_STREET_1X_GATE_PASS", source_sha,
          "source viewpoints", len(chosen), "visible", len(rendered),
          "child bounds", len(visible_children), flush=True)

if __name__ == "__main__":
    main()
