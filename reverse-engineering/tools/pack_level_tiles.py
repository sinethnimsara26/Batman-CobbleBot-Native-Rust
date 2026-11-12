#!/usr/bin/env python3
"""Pack baked level PNG tiles for lazy decoding inside the native executable."""
from __future__ import annotations

from pathlib import Path
import argparse
import io
import json
import struct

from PIL import Image


MAGIC_V1 = b"BCLVT001"
MAGIC_V2 = b"BCLVT002"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("tiles", type=Path, help="directory written by bake_level_tiles.py")
    parser.add_argument("output", type=Path, help="packed binary output")
    parser.add_argument("--format-version", type=int, choices=(1,2), default=1)
    parser.add_argument("--logical-pixel-scale", type=float, default=1.0)
    parser.add_argument("--crop-transparent", action="store_true")
    args = parser.parse_args()
    if args.logical_pixel_scale <= 0:
        raise ValueError("--logical-pixel-scale must be > 0")
    if args.format_version == 1 and args.logical_pixel_scale != 1.0:
        raise ValueError("BCLVT001 has implicit logical pixel scale 1.0")
    if args.format_version == 1 and args.crop_transparent:
        raise ValueError("transparent tile cropping requires BCLVT002")

    manifest = json.loads((args.tiles / "manifest.json").read_text(encoding="utf-8"))
    # B03 evidence packs sample only a tiny fraction of the original world.
    # Refuse to create a valid-looking runtime BCLVT container from them.
    if manifest.get("partial_proof_only") or manifest.get("never_embed_as_complete_level"):
        raise ValueError("partial B03 source proof tiles cannot be packed as a complete native level")
    manifest_scale = float(manifest.get("logical_pixel_scale", 1.0))
    if abs(manifest_scale - args.logical_pixel_scale) > 1e-6:
        raise ValueError(
            f"manifest logical_pixel_scale {manifest_scale} does not match "
            f"--logical-pixel-scale {args.logical_pixel_scale}"
        )
    tiles = sorted(manifest["tiles"], key=lambda item: (item["world_y"], item["world_x"]))
    if manifest.get("symbol") == 783:
        # A sparse B05/B06 preview must never become a seemingly valid native
        # Level1B scenery pack. Full coverage is explicitly certified by the
        # canonical frame-zero source baker, never inferred from a filename.
        if manifest.get("complete_bank_street_hq3x"):
            if (manifest.get("complete_bank_street_1x")
                    or manifest.get("canonical_bank_fws_sha256") != "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
                    or manifest.get("placement") != [1034.65, 239.25]
                    or manifest.get("tile_size") != [1800, 1200]
                    or manifest.get("logical_tile_size") != [600, 400]
                    or manifest.get("full_grid_candidate_count") != 76
                    or not {153, 782, 758, 771}.issubset(set(manifest.get("excluded_symbols", [])))
                    or manifest.get("source_static_text_enabled") is not True
                    or manifest.get("source_over_strokes") is not True
                    or manifest.get("supersample") != 2 or manifest.get("downsample") != "lanczos"
                    or manifest.get("alpha_filtering") != "premultiplied"
                    or manifest_scale != 3.0 or args.format_version != 2
                    or args.crop_transparent):
                raise ValueError("Bank Street HQ3x pack source certificate/format mismatch")
            band = manifest.get("bank_depth_band")
            depths = manifest.get("source_root_depth_range")
            if (band == "flat" and (depths is not None or len(tiles) != 75)):
                raise ValueError("Bank Street flat HQ3x must contain all 75 source tiles")
            if (band == "lower" and not (
                    isinstance(depths, list) and len(depths) == 2
                    and isinstance(depths[0], int) and depths[0] < 626 and depths[1] == 626)):
                raise ValueError("Bank Street lower HQ3x original depth certificate mismatch")
            if (band == "upper" and not (
                    isinstance(depths, list) and len(depths) == 2
                    and depths[0] == 627 and isinstance(depths[1], int) and depths[1] >= 627)):
                raise ValueError("Bank Street upper HQ3x original depth certificate mismatch")
            if band not in ("flat", "lower", "upper") or not tiles or len(tiles) > 75:
                raise ValueError("Bank Street HQ3x missing authored-band or tile data")
            coords = [(item["world_x"], item["world_y"]) for item in tiles]
            # A complete authored band can contain fewer nonempty tiles; it
            # must still be a unique subset of the canonical 76-cell grid.
            allowed = {(x, y) for y in range(-400, 801, 400)
                       for x in range(-9000, 1801, 600)}
            if len(set(coords)) != len(coords) or not set(coords).issubset(allowed):
                raise ValueError("Bank Street HQ3x has duplicate/off-grid tile coordinates")
        else:
            if not manifest.get("complete_bank_street_1x"):
                raise ValueError("Bank Street #783 requires verified full-stage 1x provenance before packing")
            if (len(tiles) != 75 or manifest.get("full_grid_candidate_count", 0) != 76
                    or manifest.get("source_root_depth_range") is not None
                    or manifest.get("source_static_text_enabled") is not True
                    or manifest.get("source_over_strokes") is not False
                    or manifest_scale != 1.0 or args.format_version != 1):
                raise ValueError("Bank Street #783 full-stage 1x pack source contract failed")
            coords = [(item["world_x"], item["world_y"]) for item in tiles]
            if len(set(coords)) != 75:
                raise ValueError("Bank Street #783 contains duplicate tile origins")
    magic = MAGIC_V2 if args.format_version == 2 else MAGIC_V1
    payload = bytearray(magic + struct.pack("<I", len(tiles)))
    if args.format_version == 2:
        payload.extend(struct.pack("<f", args.logical_pixel_scale))
    records = []
    for tile in tiles:
        crop_x = crop_y = 0
        if args.crop_transparent:
            with Image.open(args.tiles / tile["file"]) as opened:
                image = opened.convert("RGBA")
            bounds = image.getchannel("A").getbbox()
            if bounds is None:
                continue
            crop_x, crop_y = bounds[0], bounds[1]
            image = image.crop(bounds)
            encoded = io.BytesIO()
            image.save(encoded, format="PNG", optimize=True)
            png = encoded.getvalue()
            size = [image.width, image.height]
        else:
            png = (args.tiles / tile["file"]).read_bytes()
            with Image.open(args.tiles / tile["file"]) as opened:
                size = [opened.width, opened.height]
        if args.format_version == 2:
            payload.extend(struct.pack(
                "<iiiiI",
                tile["world_x"], tile["world_y"],
                crop_x, crop_y, len(png)
            ))
        else:
            payload.extend(struct.pack("<iiI", tile["world_x"], tile["world_y"], len(png)))
        payload.extend(png)
        records.append({
            **tile,
            "crop_offset": [crop_x, crop_y],
            "size": size,
            "png_bytes": len(png),
        })

    # Empty transparent tiles are already omitted by bake_level_tiles.py, so
    # V2 cropping must not change the key set/count.
    if len(records) != len(tiles):
        raise ValueError("packing unexpectedly removed a tile")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(payload)
    sidecar = args.output.with_name(args.output.stem + ".json")
    sidecar.write_text(json.dumps({
        "format": magic.decode("ascii"),
        "logical_pixel_scale": 1.0 if args.format_version == 1 else args.logical_pixel_scale,
        "tile_size": manifest["tile_size"],
        "logical_tile_size": manifest.get("logical_tile_size", manifest["tile_size"]),
        "world_bounds": manifest["world_bounds"],
        "tile_count": len(records),
        "unsupported_fills": manifest["unsupported_fills"],
        "tiles": records,
    }, indent=2), encoding="utf-8")
    print(json.dumps({
        "format": magic.decode("ascii"),
        "logical_pixel_scale": 1.0 if args.format_version == 1 else args.logical_pixel_scale,
        "tiles": len(records),
        "packed_bytes": len(payload),
        "sidecar": str(sidecar),
    }, indent=2))


if __name__ == "__main__":
    main()