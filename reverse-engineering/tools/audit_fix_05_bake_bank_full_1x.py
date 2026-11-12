#!/usr/bin/env python3
"""AUDIT-FIX-05: canonical complete Bank Street #783 source 1x coverage and pack gate.

Only development-time source artifact. No Level1B runtime asset or gameplay
activation; original frame zero (zero-based) and historical B05 1x pixels.
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

from PIL import Image

from b03_bake_stage_asset_proofs import expect_original_source, static_filter, validate_stage
from b05_bake_bank_street_1x import source_depth_audit, source_definition_tags
from b06_bake_bank_street_hq3x import B05_RGBA
from pack_level_tiles import MAGIC_V1


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(*parts) -> None:
    print("AF05_EXEC", *(str(x) for x in parts), flush=True)
    subprocess.run([str(x) for x in parts], check=True)


def reject_pack(manifest: dict, target: Path, label: str) -> None:
    folder = target.parent / ("invalid_" + label)
    folder.mkdir(parents=True, exist_ok=True)
    (folder / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    if target.exists():
        target.unlink()
    attempt = subprocess.run(
        [sys.executable, str(Path(__file__).with_name("pack_level_tiles.py")),
         str(folder), str(target)],
        capture_output=True, text=True, check=False,
    )
    if attempt.returncode == 0 or target.exists() or not (
        "Bank Street" in attempt.stderr or "partial B03" in attempt.stderr
    ):
        raise AssertionError(f"unsafe incomplete Bank packing accepted: {label} / {attempt.stderr}")
    print("AF05_NEGATIVE_PACK_GUARD_PASS", label, flush=True)


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__)
    for key in ("swf", "symbols", "bitmaps", "instances", "spec", "out"):
        p.add_argument("--" + key, required=True, type=Path)
    args = p.parse_args()
    digest = expect_original_source(args.swf)
    symbols = json.loads(args.symbols.read_text(encoding="utf-8"))
    stage = json.loads(args.instances.read_text(encoding="utf-8"))["level1b"]
    spec = json.loads(args.spec.read_text(encoding="utf-8"))["level1b"]
    if (stage["root_frame"], stage["game_symbol_id"], stage["level_symbol_id"],
            stage["ground"]["character_id"]) != (20, 1075, 783, 782):
        raise AssertionError("original zero-based root20 / GAME#1075 / street#783 / ground#782 changed")
    origin = validate_stage("level1b", spec, stage, symbols)
    if origin != (1034.65, 239.25):
        raise AssertionError(f"Bank source placement drift: {origin}")
    excludes = sorted(set(static_filter(stage)) | {758, 771})
    if not {153, 782, 758, 771}.issubset(excludes):
        raise AssertionError("Bank static scene still includes logic, ground or interactive buttons")
    depth = source_depth_audit(stage, symbols, set(excludes))
    if source_definition_tags(args.swf, {758, 771, 774}) != {758: 34, 771: 34, 774: 11}:
        raise AssertionError("source DefineButton2/DefineText identity changed")

    tilesdir = args.out / "source_1x_complete"
    cmd = [
        sys.executable, Path(__file__).with_name("bake_level_tiles.py"),
        args.swf, args.symbols, tilesdir, "--bitmaps", args.bitmaps,
        "--symbol", 783, "--place-x", origin[0], "--place-y", origin[1],
        "--include-source-static-text", "--complete-bank-street-1x",
    ]
    for sid in excludes:
        cmd.extend(("--exclude-symbol", sid))
    run(*cmd)
    manifest = json.loads((tilesdir / "manifest.json").read_text(encoding="utf-8"))
    if (manifest.get("symbol") != 783 or
            manifest.get("placement") != [1034.65, 239.25] or
            manifest.get("tile_size") != [600, 400] or
            manifest.get("logical_pixel_scale") != 1.0 or
            manifest.get("source_static_text_enabled") is not True or
            manifest.get("source_over_strokes") is not False or
            manifest.get("excluded_symbols") != excludes or
            manifest.get("complete_bank_street_1x") is not True or
            manifest.get("partial_proof_only") or
            manifest.get("never_embed_as_complete_level") or
            manifest.get("missing_ids") or manifest.get("unsupported_fills") or
            len(manifest["tiles"]) != 75):
        raise AssertionError("full Bank manifest is incomplete, noncanonical or not safe to pack")

    records = {}
    for item in manifest["tiles"]:
        xy = (item["world_x"], item["world_y"])
        if xy in records or xy[0] % 600 or xy[1] % 400:
            raise AssertionError(f"duplicate or unaligned Bank source tile {xy}")
        path = tilesdir / item["file"]
        encoded = path.read_bytes()
        with Image.open(io.BytesIO(encoded)) as im:
            image = im.convert("RGBA")
            if image.size != (600, 400) or image.getchannel("A").getbbox() is None:
                raise AssertionError(f"blank/wrong-sized full Bank tile {xy}")
            rgba = sha(image.tobytes())
        records[xy] = {
            "world_x": xy[0], "world_y": xy[1], "file": item["file"],
            "file_sha256": sha(encoded), "decoded_rgba_sha256": rgba,
        }

    if len(records) != 75 or not set(B05_RGBA).issubset(records):
        raise AssertionError("full original Bank street tile set misses a baseline source view")
    for xy, expected in B05_RGBA.items():
        if records[xy]["decoded_rgba_sha256"] != expected:
            raise AssertionError(f"historical B05 decoded 1x RGBA drift at {xy}")
    print("AF05_B05_NINE_OF_NINE_RGBA_BASELINES_PASS", flush=True)

    # Independent FULL-GRID location check: tile origins must match raster
    # world bounds, not merely a list claimed complete by the manifest.
    import math
    x0, y0, x1, y1 = manifest["world_bounds"]
    candidates = {(x, y)
                  for y in range(math.floor(y0 / 400) * 400, math.floor(y1 / 400) * 400 + 1, 400)
                  for x in range(math.floor(x0 / 600) * 600, math.floor(x1 / 600) * 600 + 1, 600)}
    if (len(candidates) != manifest["full_grid_candidate_count"] or
            not set(records).issubset(candidates) or len(candidates) != 76):
        raise AssertionError("Bank full-grid scan does not match original geometry-derived bounds")

    packed = args.out / "bank_street_complete_1x.BCLVT001.bin"
    run(sys.executable, Path(__file__).with_name("pack_level_tiles.py"), tilesdir, packed)
    content = packed.read_bytes()
    if content[:8] != MAGIC_V1 or struct.unpack_from("<I", content, 8)[0] != 75:
        raise AssertionError("BCLVT001 header/count do not describe 75 source tiles")
    pos = 12
    packed_positions = set()
    for _ in range(75):
        x, y, size = struct.unpack_from("<iiI", content, pos)
        pos += 12
        payload = content[pos:pos + size]
        pos += size
        xy = (x, y)
        if xy not in records or xy in packed_positions or size == 0:
            raise AssertionError(f"missing/duplicate source tile in BCLVT001 at {xy}")
        if sha(payload) != records[xy]["file_sha256"]:
            raise AssertionError(f"packed source tile corrupted at {xy}")
        packed_positions.add(xy)
    if pos != len(content) or packed_positions != set(records):
        raise AssertionError("BCLVT001 contains unexpected trailing bytes or missing tiles")

    # Malicious/inadvertent marker deletion and shrinking to B05's nine tiles
    # must fail before producing an apparently valid bank pack.
    reject_pack({**manifest, "complete_bank_street_1x": False},
                args.out / "DANGER_BANK_UNVERIFIED.bin", "missing_provenance")
    reject_pack({**manifest, "tiles": manifest["tiles"][:9]},
                args.out / "DANGER_BANK_NINE.bin", "truncated_nine")

    report = {
        "batch": "AUDIT-FIX-05", "source_fws_sha256": digest,
        "source_frame_zero_based": {"root": 20, "street": 0},
        "source_symbols": {"game": 1075, "street": 783, "ground_excluded": 782},
        "level_origin_game_local": list(origin),
        "depth_ownership": depth,
        "excluded_non_static_symbols": excludes,
        "scanned_grid_origins": len(candidates),
        "nonempty_source_1x_tiles": len(records),
        "historic_B05_1x_rgba_hashes_checked": len(B05_RGBA),
        "all_tile_hashes": [records[x] for x in sorted(records, key=lambda p: (p[1],p[0]))],
        "packed_file": packed.name, "packed_sha256": sha(content),
        "packed_bytes": len(content), "pack_format": "BCLVT001",
        "negative_pack_contracts_checked": ["missing_provenance", "truncated_nine"],
        "development_artifact_only": True, "not_embedded_as_runtime_stage": True,
        "no_gameplay_or_original_live_flash_equivalence_claim": True,
        "hq_3x_complete_tiles": False,
    }
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "AUDIT_FIX_05_BANK_FULL_1X_REPORT.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("AF05_BANK_FULL_1X_GATE_PASS", len(records),
          "tiles; 9/9 historic hashes; full BCLVT001 round-trip; zero shipped game assets", flush=True)


if __name__ == "__main__":
    main()
