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
    tiles = sorted(manifest["tiles"], key=lambda item: (item["world_y"], item["world_x"]))
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