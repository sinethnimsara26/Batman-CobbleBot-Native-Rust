#!/usr/bin/env python3
"""Bake one SWF level's static display list into viewport-sized world tiles."""
from __future__ import annotations

from pathlib import Path
import argparse
import json
import math

from PIL import Image

from bake_sprite_frames import load_shapes, render_symbol, symbol_bounds


def load_bitmaps(bitmap_dir: Path):
    manifest = json.loads((bitmap_dir / "manifest.json").read_text(encoding="utf-8"))
    images = {}
    for item in manifest:
        if "file" not in item:
            continue
        with Image.open(bitmap_dir / item["file"]) as source:
            images[item["id"]] = source.convert("RGBA")
    return images


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("swf", type=Path)
    parser.add_argument("symbols_json", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--bitmaps", type=Path, help="directory produced by extract_bitmaps.py")
    parser.add_argument("--symbol", type=int, default=152)
    parser.add_argument(
        "--exclude-symbol",
        type=int,
        action="append",
        default=[],
        help="child character IDs to omit from visible tiles (repeatable)",
    )
    parser.add_argument("--place-x", type=float, default=2467.65)
    parser.add_argument("--place-y", type=float, default=239.25)
    parser.add_argument("--tile-width", type=int, default=600)
    parser.add_argument("--tile-height", type=int, default=400)
    args = parser.parse_args()

    symbols = json.loads(args.symbols_json.read_text(encoding="utf-8"))
    if str(args.symbol) not in symbols:
        raise ValueError(f"symbol {args.symbol} is not present in symbols.json")
    unsupported: set[str] = set()
    shapes = load_shapes(args.swf, unsupported)
    bitmap_dir = args.bitmaps or args.swf.resolve().parent.parent / "bitmaps"
    bitmaps = load_bitmaps(bitmap_dir)
    excluded_symbols = set(args.exclude_symbol)

    local_bounds = symbol_bounds(
        args.symbol,
        0,
        (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        symbols,
        shapes,
        excluded_symbols=excluded_symbols,
    )
    if not all(math.isfinite(value) for value in local_bounds):
        raise ValueError(f"symbol {args.symbol} has no supported renderable geometry")
    min_x = math.floor(local_bounds[0] + args.place_x)
    min_y = math.floor(local_bounds[1] + args.place_y)
    max_x = math.ceil(local_bounds[2] + args.place_x)
    max_y = math.ceil(local_bounds[3] + args.place_y)
    first_x = math.floor(min_x / args.tile_width) * args.tile_width
    first_y = math.floor(min_y / args.tile_height) * args.tile_height
    last_x = math.floor(max_x / args.tile_width) * args.tile_width
    last_y = math.floor(max_y / args.tile_height) * args.tile_height

    args.output.mkdir(parents=True, exist_ok=True)
    missing: set[int] = set()
    records = []
    for y in range(first_y, last_y + 1, args.tile_height):
        for x in range(first_x, last_x + 1, args.tile_width):
            canvas = Image.new("RGBA", (args.tile_width, args.tile_height), (0, 0, 0, 0))
            transform = (1.0, 0.0, 0.0, 1.0, args.place_x - x, args.place_y - y)
            render_symbol(
                args.symbol,
                0,
                transform,
                canvas,
                symbols,
                shapes,
                bitmaps,
                unsupported,
                missing,
                excluded_symbols=excluded_symbols,
            )
            if canvas.getchannel("A").getbbox() is None:
                continue
            filename = f"x{x:+07d}_y{y:+07d}.png"
            canvas.save(args.output / filename, optimize=True)
            records.append({"world_x": x, "world_y": y, "file": filename})

    manifest = {
        "symbol": args.symbol,
        "excluded_symbols": sorted(excluded_symbols),
        "placement": [args.place_x, args.place_y],
        "tile_size": [args.tile_width, args.tile_height],
        "local_bounds": local_bounds,
        "world_bounds": [min_x, min_y, max_x, max_y],
        "unsupported_fills": sorted(unsupported),
        "missing_ids": sorted(missing),
        "tiles": records,
    }
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(json.dumps({
        "symbol": args.symbol,
        "world_bounds": manifest["world_bounds"],
        "tile_grid": [first_x, first_y, last_x, last_y],
        "rendered_tiles": len(records),
        "unsupported_fills": manifest["unsupported_fills"],
        "missing_ids": manifest["missing_ids"],
    }, indent=2))


if __name__ == "__main__":
    main()