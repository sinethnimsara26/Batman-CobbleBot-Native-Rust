#!/usr/bin/env python3
"""AUDIT-FIX-06: complete original Bank #783 HQ3x full-grid and source-depth packs.

OFFLINE ONLY. Does not install assets or activate Level1B. Root frame20 and
street frame0 are both zero-based. No claim about live Flash pixel equivalence.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path
import struct
import subprocess
import sys

from PIL import Image, ImageChops

from b03_bake_stage_asset_proofs import expect_original_source, static_filter, validate_stage
from b05_bake_bank_street_1x import source_depth_audit, source_definition_tags
from b06_bake_bank_street_hq3x import B05_RGBA, source_depth_split, run_bake
from pack_level_tiles import MAGIC_V2


def digest(data):
    return hashlib.sha256(data).hexdigest()


def execute(*args):
    print("AF06_EXEC", *(str(x) for x in args), flush=True)
    subprocess.run([str(x) for x in args], check=True)


def read_manifest(folder):
    return json.loads((folder / "manifest.json").read_text(encoding="utf-8"))


def index(folder, dimensions):
    manifest = read_manifest(folder)
    results = {}
    for item in manifest["tiles"]:
        xy = (item["world_x"], item["world_y"])
        if xy in results:
            raise AssertionError(f"duplicate Bank tile {xy}")
        encoded = (folder / item["file"]).read_bytes()
        with Image.open(io.BytesIO(encoded)) as opened:
            image = opened.convert("RGBA")
        if image.size != dimensions or image.getchannel("A").getbbox() is None:
            raise AssertionError(f"invalid source tile {xy}: {image.size}")
        results[xy] = (item["file"], digest(encoded), digest(image.tobytes()))
    return manifest, results


def reject(manifest, parent, label):
    directory = parent / ("invalid_" + label)
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    dest = parent / ("DANGER_" + label + ".bin")
    test = subprocess.run(
        [sys.executable, str(Path(__file__).with_name("pack_level_tiles.py")),
         directory, dest, "--format-version", "2", "--logical-pixel-scale", "3"],
        text=True, capture_output=True, check=False)
    if test.returncode == 0 or dest.exists() or "Bank Street" not in test.stderr:
        raise AssertionError(f"fail-closed HQ guard did not reject {label}: {test.stderr}")
    print("AF06_HQ_NEGATIVE_PACK_GUARD_PASS", label, flush=True)


def pack_verify(folder, output, expected):
    execute(sys.executable, Path(__file__).with_name("pack_level_tiles.py"),
            folder, output, "--format-version", "2", "--logical-pixel-scale", "3")
    payload = output.read_bytes()
    if payload[:8] != MAGIC_V2 or struct.unpack_from("<I", payload, 8)[0] != len(expected):
        raise AssertionError("invalid BCLVT002 Bank HQ header")
    if struct.unpack_from("<f", payload, 12)[0] != 3.0:
        raise AssertionError("incorrect Bank HQ logical scale")
    offset, seen = 16, set()
    for _ in range(len(expected)):
        x, y, crop_x, crop_y, size = struct.unpack_from("<iiiiI", payload, offset)
        offset += 20
        data = payload[offset:offset + size]
        offset += size
        xy = (x, y)
        if xy in seen or xy not in expected or (crop_x, crop_y) != (0, 0):
            raise AssertionError(f"bad Bank HQ packed source tile {xy}")
        if digest(data) != expected[xy][1]:
            raise AssertionError(f"Bank HQ packed source bytes changed at {xy}")
        seen.add(xy)
    if offset != len(payload) or seen != set(expected):
        raise AssertionError("Bank HQ pack missing tiles or has trailing data")
    return {"name": output.name, "sha256": digest(payload),
            "bytes": len(payload), "tiles": len(seen)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ("swf", "symbols", "bitmaps", "instances", "spec", "out"):
        parser.add_argument("--" + key, required=True, type=Path)
    args = parser.parse_args()
    sha = expect_original_source(args.swf)
    symbols = json.loads(args.symbols.read_text(encoding="utf-8"))
    stage = json.loads(args.instances.read_text(encoding="utf-8"))["level1b"]
    spec = json.loads(args.spec.read_text(encoding="utf-8"))["level1b"]
    if (stage["root_frame"], stage["game_symbol_id"], stage["level_symbol_id"],
            stage["ground"]["character_id"]) != (20, 1075, 783, 782):
        raise AssertionError("canonical root20/GAME#1075/Bank#783/ground#782 changed")
    origin = validate_stage("level1b", spec, stage, symbols)
    if origin != (1034.65, 239.25):
        raise AssertionError("original street placement changed")
    excluded = sorted(set(static_filter(stage)) | {758, 771})
    if not {153, 782, 758, 771}.issubset(excluded):
        raise AssertionError("nonstatic game logic or collision source leaked")
    source_depth_audit(stage, symbols, set(excluded))
    if source_definition_tags(args.swf, {758, 771, 774}) != {758:34, 771:34, 774:11}:
        raise AssertionError("original Button2/Text tags changed")
    split = source_depth_split(stage, symbols, set(excluded))
    if split["original_direct_display_depth_cut"] != 626:
        raise AssertionError("original Bank authored-depth cut changed")
    args.out.mkdir(parents=True, exist_ok=True)

    # First re-run the previously independently audited full 1x contract.
    baseline_folder = args.out / "baseline"
    execute(sys.executable, Path(__file__).with_name("audit_fix_05_bake_bank_full_1x.py"),
            "--swf", args.swf, "--symbols", args.symbols, "--bitmaps", args.bitmaps,
            "--instances", args.instances, "--spec", args.spec, "--out", baseline_folder)
    base_manifest, base = index(baseline_folder / "source_1x_complete", (600, 400))
    if len(base) != 75 or set(B05_RGBA) - set(base):
        raise AssertionError("incomplete 75-tile 1x source reference")
    for xy, hash_value in B05_RGBA.items():
        if base[xy][2] != hash_value:
            raise AssertionError(f"historic original B05 1x hash drift at {xy}")

    roles = {"flat": None,
             "lower": tuple(split["lower_band"]),
             "upper": tuple(split["upper_band"])}
    tile_paths, manifests, indexes, packed = {}, {}, {}, []
    for role, band in roles.items():
        folder = args.out / ("hq3x_" + role)
        command = [sys.executable, Path(__file__).with_name("bake_level_tiles.py"),
                   args.swf, args.symbols, folder, "--bitmaps", args.bitmaps,
                   "--symbol", 783, "--place-x", origin[0], "--place-y", origin[1],
                   "--include-source-static-text", "--source-over-strokes",
                   "--complete-bank-street-hq3x", "--bank-depth-band", role,
                   "--output-scale", 3, "--supersample", 2, "--downsample", "lanczos"]
        if band is not None:
            command.extend(("--root-depth-range", *band))
        for sid in excluded:
            command.extend(("--exclude-symbol", sid))
        execute(*command)
        manifest, records = index(folder, (1800, 1200))
        if (manifest["complete_bank_street_hq3x"] is not True
                or manifest["complete_bank_street_1x"] is not False
                or manifest["canonical_bank_fws_sha256"] != sha
                or manifest["full_grid_candidate_count"] != 76
                or manifest["bank_depth_band"] != role
                or manifest["source_root_depth_range"] != (list(band) if band else None)
                or manifest["excluded_symbols"] != excluded
                or manifest["placement"] != list(origin)
                or manifest["logical_pixel_scale"] != 3.0
                or manifest["supersample"] != 2 or manifest["downsample"] != "lanczos"
                or manifest["source_over_strokes"] is not True
                or manifest.get("partial_proof_only") or manifest.get("never_embed_as_complete_level")
                or manifest["missing_ids"] or manifest["unsupported_fills"]):
            raise AssertionError(f"unsafe or incomplete full Bank HQ manifest: {role}")
        if not set(records).issubset(set(base)):
            raise AssertionError(f"HQ original-depth band {role} has source positions outside complete 1x")
        if role == "flat" and set(records) != set(base):
            raise AssertionError("full HQ3x Bank flat layer does not cover all 75 source positions")
        if role != "flat" and not records:
            raise AssertionError(f"empty authored source-depth band {role}")
        manifests[role], indexes[role], tile_paths[role] = manifest, records, folder
        target = args.out / ("BankStreet_" + role + "_HQ3x.BCLVT002.bin")
        packed.append(pack_verify(folder, target, records))
        print("AF06_FULL_HQ_BAND_VERIFIED", role, len(records), flush=True)
    if set(indexes["lower"]) | set(indexes["upper"]) != set(base):
        raise AssertionError("original authored lower+upper depth bands omit some original source cells")

    # True 3x must be sourced from original vectors, not a 1x bilinear upscale.
    independent = 0
    for xy in B05_RGBA:
        with Image.open((baseline_folder/"source_1x_complete")/base[xy][0]) as original:
            upscaled = original.convert("RGBA").resize((1800, 1200), Image.Resampling.BILINEAR)
        with Image.open(tile_paths["flat"]/indexes["flat"][xy][0]) as rendered:
            if upscaled.tobytes() != rendered.convert("RGBA").tobytes():
                independent += 1
    if independent != 9:
        raise AssertionError(f"source HQ3x failed independent raster proof: {independent}/9")

    # Separate no-postfilter source-over partition identity at two fixed
    # source sample positions. Filtering individually composited layers can
    # produce small differences, so do NOT falsely assert filtered pixel
    # equality (nor pretend these are live Flash gameplay captures).
    args.excludes = excluded
    control_points = ((-4200, 0), (0, 0))
    comparisons = []
    controls = {}
    for role, band in roles.items():
        path = args.out/"control_depth_3x"/role
        m, _ = run_bake(args, path, control_points, scale=3,
                        depth_range=band, source_over_strokes=True)
        entries = {(t["world_x"],t["world_y"]): path/t["file"] for t in m["tiles"]}
        controls[role] = entries
    for xy in control_points:
        if xy not in controls["flat"]:
            raise AssertionError("original control point missing flat layer")
        layers = []
        for role in ("lower", "upper"):
            path = controls[role].get(xy)
            if path:
                with Image.open(path) as image:
                    layers.append(image.convert("RGBA"))
            else:
                layers.append(Image.new("RGBA", (1800,1200), (0,0,0,0)))
        combo = layers[0].copy()
        combo.alpha_composite(layers[1])
        with Image.open(controls["flat"][xy]) as opened:
            flat = opened.convert("RGBA")
        bbox = ImageChops.difference(flat, combo).getbbox()
        if bbox is not None:
            raise AssertionError(f"source-authored depth recomposition differs at {xy}: {bbox}")
        comparisons.append({"world_origin": list(xy), "different_rgba_pixels": 0})
    print("AF06_AUTHOR_DEPTH_CONTROL_RECOMPOSITION_PASS", comparisons, flush=True)

    reject({**manifests["flat"], "complete_bank_street_hq3x": False}, args.out, "missing_hq_certificate")
    reject({**manifests["flat"], "tiles": manifests["flat"]["tiles"][:9]}, args.out, "truncated_hq_flat")
    reject({**manifests["upper"], "source_root_depth_range": [1, 626]}, args.out, "fake_depth_band")
    report = {
        "batch": "AUDIT-FIX-06", "canonical_fws_sha256": sha,
        "zero_based_source_frames": {"root": 20, "bank_street": 0},
        "symbols": {"GAME":1075, "Bank_Street":783, "ground_excluded":782,
                    "buttons_excluded":[758,771], "original_text":774},
        "source_origin": list(origin), "filtered_static_depth_partition": split,
        "full_original_grid_candidates": 76, "verified_1x_reference_tiles": 75,
        "hq_true_vector_6x_to_3x_bilinear_distinction": independent,
        "full_hq3x_depth_ownership_union_cells": len(set(indexes["lower"])|set(indexes["upper"])),
        "layer_tile_counts": {role:len(indexes[role]) for role in roles},
        "packed_BCLVT002": packed,
        "sha256_of_each_source_hq_png_and_rgba": {
            role:[{"x":xy[0], "y":xy[1], "file":item[0], "png_sha256":item[1], "decoded_rgba_sha256":item[2]}
                  for xy,item in sorted(indexes[role].items(), key=lambda i:(i[0][1],i[0][0]))]
            for role in roles},
        "unfiltered_source_depth_recomposition_checks":comparisons,
        "negative_pack_contracts":["missing_hq_certificate","truncated_hq_flat","fake_depth_band"],
        "dev_artifact_not_native_assets":True, "no_level1b_activation":True,
        "live_original_flash_pixels_unverified":True, "physical_audio_unverified":True,
    }
    (args.out/"AUDIT_FIX_06_BANK_FULL_HQ3X_REPORT.json").write_text(
        json.dumps(report,indent=2)+"\n",encoding="utf-8")
    print("AF06_BANK_FULL_HQ3X_GATE_PASS", len(indexes["flat"]),
          "flat tiles, fully scanned two depth bands, BCLVT002 x3, zero activated game assets", flush=True)


if __name__ == "__main__":
    main()
