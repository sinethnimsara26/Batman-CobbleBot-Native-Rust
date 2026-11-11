#!/usr/bin/env python3
"""B03: source-backed, bounded preview baking for the *unimplemented* stages.

Inputs are extracted from the canonical SHA-verified original SWF. Do not
ship these partial proofs as a native complete level or call them live Flash
screenshots. Full-world scene tiles and gameplay animation are later batches.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys

from PIL import Image, ImageDraw
from bake_sprite_frames import display_list, load_shapes, symbol_bounds
from render_collision_masks import affine

SOURCE_SHA = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
STAGES = ("level1b", "level1c_penguin", "level2a", "level2b")
# A visible source-library sprite, NEVER a fabricated still of the gameplay.
NAMED_SPRITE_PROOFS = {
    "level1b": "goon1",
    "level1c_penguin": "boss",
    "level2a": "door_01",
    "level2b": "door_01",
}
LOGIC_PREFIXES = (
    "checkpoint", "camera_", "respawn", "pit", "ground",
    "goon", "biker", "boss", "eagle", "player", "item_br",
    "batarang", "bam", "combo", "door_", "elevator_",
    "lava", "health", "halo", "clock", "can_", "trash_", "mailbox",
)
TILE_W, TILE_H = 600, 400


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def aligned(x, size):
    return math.floor(x / size) * size


def expect_original_source(swf: Path):
    digest = sha256(swf)
    if digest != SOURCE_SHA or swf.read_bytes()[:3] != b"FWS":
        raise ValueError(f"Wrong original FWS source: {digest}")
    if swf.stat().st_size != 2819100:
        raise ValueError("Wrong original SWF length")
    return digest


def static_filter(stage):
    """Only omit script-owned or collision objects, not authored scenery."""
    excludes = {
        p["character_id"]
        for p in stage["named_level_instances"]
        if p["name"].lower().startswith(LOGIC_PREFIXES)
    }
    # All game-level actor/prop instances are separate from "level" scenery.
    # Exclude these IDs defensively if the same symbols appear nested in level.
    excludes.update(
        p["character_id"]
        for p in stage["named_game_instances"]
        if p["name"].lower().startswith(LOGIC_PREFIXES)
    )
    # Source checkpoint symbol 153 is invisible logic even when unnamed.
    excludes.add(153)
    excludes.discard(stage["level_symbol_id"])
    return sorted(excludes)


def validate_stage(name, spec, stage, symbols):
    assert spec["root_frame"] == stage["root_frame"]
    assert spec["game_symbol"] == stage["game_symbol_id"]
    assert spec["level_symbol"] == stage["level_symbol_id"]
    assert spec["ground_symbol"] == stage["ground"]["character_id"]
    level_placements = [
        p for p in stage["named_game_instances"] if p["name"] == "level"
    ]
    if len(level_placements) != 1:
        raise AssertionError(f"{name}: expected exactly one named source level placement")
    level = level_placements[0]
    assert level["character_id"] == stage["level_symbol_id"]
    m = level["matrix"]
    if not (math.isclose(m["sx"], 1.0) and math.isclose(m["sy"], 1.0)
            and abs(m["r0"]) < 1e-9 and abs(m["r1"]) < 1e-9):
        raise AssertionError(f"{name}: unsupported non-translation level transform")
    if str(stage["level_symbol_id"]) not in symbols:
        raise AssertionError(f"{name}: original level symbol undefined")
    if str(stage["game_symbol_id"]) not in symbols:
        raise AssertionError(f"{name}: original game symbol undefined")
    ground = [p for p in stage["named_level_instances"] if p["name"] == "ground"]
    if len(ground) != 1 or ground[0]["character_id"] != spec["ground_symbol"]:
        raise AssertionError(f"{name}: original collision ground not proved")
    return (float(m["tx"]), float(m["ty"]))


def point_candidates(stage, origin, symbols, shapes, excludes):
    """Real, bounds-derived source child geometry, not guessed world coordinates."""
    scene = symbols[str(stage["level_symbol_id"])]
    child_list = list(display_list(scene, 0).values())
    valid_bounds = []
    for p in child_list:
        cid = p.get("character_id")
        if cid is None or cid in excludes:
            continue
        box = symbol_bounds(cid, 0, affine(p.get("matrix", {})),
                            symbols, shapes, excluded_symbols=set(excludes))
        if all(math.isfinite(z) for z in box):
            valid_bounds.append((cid, box))
    all_box = symbol_bounds(
        stage["level_symbol_id"], 0, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        symbols, shapes, excluded_symbols=set(excludes),
    )
    if not all(math.isfinite(z) for z in all_box):
        raise AssertionError("Source 'level' has no visible non-logic geometry")
    minx, miny, maxx, maxy = all_box
    points = []
    # Pick concrete visible child bounds first; avoid assuming global bounding
    # box midpoint contains art when the original map has empty space.
    for cid, box in valid_bounds:
        a, b, c, d = box
        points.append(((a+c)/2 + origin[0], (b+d)/2 + origin[1], cid))
    points += [
        (origin[0] + minx, origin[1] + miny, None),
        (origin[0] + (minx+maxx)/2, origin[1] + (miny+maxy)/2, None),
    ]
    player = stage["player"]["matrix"]
    px, py = float(player["tx"]), float(player["ty"])
    points.sort(key=lambda item: (item[0]-px)**2 + (item[1]-py)**2)
    selected = []
    for x, y, source_id in points:
        tile = (aligned(x, TILE_W), aligned(y, TILE_H))
        if tile not in selected:
            selected.append(tile)
    if not selected:
        raise AssertionError("No source-derived tile locations")
    return selected, [minx, miny, maxx, maxy], valid_bounds


def run(*args):
    command = [str(x) for x in args]
    print("B03_EXEC", " ".join(command), flush=True)
    subprocess.run(command, check=True)


def bake_one_stage(name, swf, symbols_path, bitmaps, out, spec, stage,
                   all_symbols, all_shapes):
    origin = validate_stage(name, spec, stage, all_symbols)
    exclusions = static_filter(stage)
    locations, bounds, children = point_candidates(
        stage, origin, all_symbols, all_shapes, exclusions)
    base = out / name
    base.mkdir(parents=True, exist_ok=True)
    scene_symbol = stage["level_symbol_id"]
    baker = Path(__file__).with_name("bake_level_tiles.py")

    # Bounded original source scene samples; never scan the whole Labs grid.
    # More than two requested tiles would duplicate the future B05/B06 work.
    candidates = locations[:2]
    def bake_tiles(dir_name, scale):
        dest = base / dir_name
        opts = [sys.executable, baker, swf, symbols_path, dest,
                "--bitmaps", bitmaps, "--symbol", scene_symbol,
                "--place-x", origin[0], "--place-y", origin[1]]
        if scale == 3:
            opts += ["--output-scale", 3, "--supersample", 2, "--downsample", "lanczos"]
        # Ground/pit, checkpoint/camera triggers and independent game-level
        # actors MUST stay out of the source-level static scenery preview.
        for source_id in exclusions:
            opts += ["--exclude-symbol", source_id]
        for x, y in candidates:
            opts += ["--only-tile", x, y]
        run(*opts)
        manifest = read_json(dest / "manifest.json")
        if not manifest.get("partial_proof_only"):
            raise AssertionError("B03 proof tiles were not marked partial")
        if manifest.get("never_embed_as_complete_level") is not True:
            raise AssertionError("Incomplete stage tiles could be mistaken for runtime pack")
        if manifest["symbol"] != scene_symbol:
            raise AssertionError("Wrong source sprite baked")
        if manifest["placement"] != [*origin]:
            raise AssertionError("Source placement mismatch")
        if manifest["excluded_symbols"] != exclusions:
            raise AssertionError("Ground/logic exclusion mismatch")
        if manifest["requested_tile_origins"] != [
            list(xy) for xy in sorted(candidates, key=lambda xy: (xy[1], xy[0]))]:
            raise AssertionError("Output provenance differs from requested tiles")
        for t in manifest["tiles"]:
            with Image.open(dest / t["file"]) as image:
                assert image.size == (TILE_W*scale, TILE_H*scale)
                if image.getchannel("A").getbbox() is None:
                    raise AssertionError("Transparent tile saved erroneously")
        return manifest

    one = bake_tiles("source_1x", 1)
    hq = bake_tiles("source_3x", 3)
    if {(x["world_x"], x["world_y"]) for x in one["tiles"]} != {
       (x["world_x"], x["world_y"]) for x in hq["tiles"]}:
        raise AssertionError(f"{name}: 1x/HQ preview sample positions differ")
    if not one["tiles"]:
        raise AssertionError(f"{name}: no visible original scene tiles from source children")

    # This is a separate SOURCE LIBRARY character/prop proof, never composited
    # as if the real Flash game had been played at this camera position.
    chosen = [p for p in stage["named_game_instances"]
              if p["name"] == NAMED_SPRITE_PROOFS[name]]
    if len(chosen) != 1:
        raise AssertionError(f"{name}: named actor not uniquely source-backed")
    source_sprite = chosen[0]["character_id"]
    actor = base / "original_source_actor_frame0.png"
    run(sys.executable, Path(__file__).with_name("bake_sprite_frames.py"),
        swf, symbols_path, actor, "--bitmaps", bitmaps,
        "--symbol", source_sprite, "--frame", 0, "--scale", 0.5,
        "--width", 600, "--height", 400)
    with Image.open(actor) as image:
        if image.size != (600, 400) or image.getchannel("A").getbbox() is None:
            raise AssertionError(f"{name}: original source sprite proof was transparent")

    provenance = {
        "stage_manifest_key": name,
        "root_frame_parser_zero_based": stage["root_frame"],
        "game_symbol_id": stage["game_symbol_id"],
        "level_symbol_id": scene_symbol,
        "ground_source_symbol_id_excluded_from_visible_layer": stage["ground"]["character_id"],
        "level_origin_in_game_local": list(origin),
        "visible_level_child_count": len(children),
        "non_visible_or_separately_owned_source_symbol_exclusions": exclusions,
        "excluded_source_named_level_instances": stage["named_level_instances"],
        "named_original_actor_proof": chosen[0]["name"],
        "source_actor_symbol": source_sprite,
        "actor_png_sha256": sha256(actor),
        "available_source_child_tile_candidates": len(locations),
        "sampled_coordinates": [list(xy) for xy in candidates],
        "rendered_1x": len(one["tiles"]),
        "rendered_3x": len(hq["tiles"]),
        "source_1x_tile_sha256": {p["file"]: sha256(base / "source_1x" / p["file"])
                                  for p in one["tiles"]},
        "source_3x_tile_sha256": {p["file"]: sha256(base / "source_3x" / p["file"])
                                  for p in hq["tiles"]},
        "static_scene_does_not_include_game_level_actor_instances": True,
        "partial_preview_only_not_full_world_not_runtime_asset": True,
        "source_scene_timeline_frame_zero_based": 0,
        "classification": "static original SWF source display-list proof, NOT live gameplay",
    }
    (base / "provenance.json").write_text(
        json.dumps(provenance, indent=2) + "\n", encoding="utf-8")
    return provenance


def make_contact_sheet(out, stages):
    # Captioned composite is only a review aid; raw original source images
    # remain separately included with hashes and original sample positions.
    width, height = 1240, len(stages) * 500
    canvas = Image.new("RGB", (width, height), "#252a35")
    draw = ImageDraw.Draw(canvas)
    for row, stage in enumerate(stages):
        name = stage["stage_manifest_key"]
        dir = out / name
        source = read_json(dir / "source_1x" / "manifest.json")
        chosen = source["tiles"][0]
        with Image.open(dir / "source_1x" / chosen["file"]) as im:
            base = Image.new("RGBA", im.size, "#414957")
            base.alpha_composite(im.convert("RGBA"))
            canvas.paste(base.convert("RGB"), (0, row * 500 + 37))
        with Image.open(dir / "original_source_actor_frame0.png") as im:
            background = Image.new("RGBA", im.size, "#414957")
            background.alpha_composite(im.convert("RGBA"))
            canvas.paste(background.convert("RGB"), (620, row * 500 + 37))
        draw.text((8, row * 500 + 9),
                  f"{name}: original static level sample, tile {chosen['world_x']},{chosen['world_y']}",
                  fill="#ffffff")
        draw.text((628, row * 500 + 9),
                  f"{name}: original source sprite #{stage['source_actor_symbol']}, frame 0",
                  fill="#ffffff")
        draw.text((8, row * 500 + 449),
                  "B03 development-only SWF source library art; NOT actual Flash gameplay",
                  fill="#ffca77")
    dest = out / "B03_STATIC_SOURCE_PROOF_CONTACT_SHEET.png"
    canvas.save(dest, optimize=True)
    return {"file": dest.name, "sha256": sha256(dest), "size": [width, height]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--swf", type=Path, required=True)
    ap.add_argument("--symbols", type=Path, required=True)
    ap.add_argument("--bitmaps", type=Path, required=True)
    ap.add_argument("--instances", type=Path, required=True)
    ap.add_argument("--spec", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()

    verified_sha = expect_original_source(args.swf)
    all_symbols = read_json(args.symbols)
    levels = read_json(args.instances)
    level_spec = read_json(args.spec)
    unsupported = set()
    shapes = load_shapes(args.swf, unsupported)
    args.out.mkdir(parents=True, exist_ok=True)
    stages = []
    for stage in STAGES:
        if stage not in levels or stage not in level_spec:
            raise AssertionError(f"Missing canonical stage {stage}")
        result = bake_one_stage(
            stage, args.swf, args.symbols, args.bitmaps, args.out,
            level_spec[stage], levels[stage], all_symbols, shapes)
        stages.append(result)
        print("B03_SOURCE_STAGE_PROOF_PASS", stage,
              "1x", result["rendered_1x"],
              "3x", result["rendered_3x"],
              "actor", result["source_actor_symbol"], flush=True)
    # Explicit negative integration test: B03 preview manifests MUST NOT
    # produce a valid-looking native asset pack. Future B05/B06 full packs
    # will have no "partial_proof_only" marker and retain the old behavior.
    attempted_pack = args.out / "DANGER_INCOMPLETE_LEVEL_MUST_NOT_EXIST.bin"
    reject = subprocess.run(
        [sys.executable, str(Path(__file__).with_name("pack_level_tiles.py")),
         str(args.out / STAGES[0] / "source_1x"), str(attempted_pack)],
        text=True, capture_output=True, check=False,
    )
    if (reject.returncode == 0 or attempted_pack.exists()
            or "partial B03 source proof tiles cannot be packed" not in reject.stderr):
        raise AssertionError("B03 preview unexpectedly packable as a complete native level")
    print("B03_PREVIEW_PACK_GUARD_PASS", flush=True)

    sheet = make_contact_sheet(args.out, stages)
    summary = {
        "batch": "LEVEL2-B03",
        "original_fws_sha256_verified_in_this_run": verified_sha,
        "stage_frame_indexing": "zero-based source parser",
        "scenes": stages,
        "review_sheet": sheet,
        "source_parser_unsupported_shapes": sorted(unsupported),
        "proprietary_swf_not_uploaded": True,
        "runtime_level1a_assets_not_modified": True,
        "not_actual_source_gameplay_screenshot": True,
        "partial_preview_pack_is_rejected_by_native_packer": True,
        "all_stage_tiles_are_partial_proofs_not_complete_native_packs": True,
    }
    (args.out / "B03_STAGE_SOURCE_ASSET_REPORT.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8")
    if len(stages) != 4 or any(z["rendered_1x"] < 1 for z in stages):
        raise AssertionError("Not all four future stage proofs were rendered")
    print("B03_SOURCE_ASSET_GATE_PASS", verified_sha, sheet, flush=True)


if __name__ == "__main__":
    main()
