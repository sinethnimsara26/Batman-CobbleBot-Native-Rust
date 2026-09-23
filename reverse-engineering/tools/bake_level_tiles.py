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
    parser.add_argument("--output-scale", type=int, default=1)
    parser.add_argument("--supersample", type=int, default=1)
    parser.add_argument("--downsample", choices=("none","lanczos"), default="none")
    args = parser.parse_args()
    if args.output_scale < 1:
        raise ValueError("--output-scale must be >= 1")
    if args.supersample < 1:
        raise ValueError("--supersample must be >= 1")
    if args.downsample == "lanczos" and args.supersample == 1:
        raise ValueError("Lanczos finalization requires --supersample > 1")
    if args.downsample == "none" and args.supersample != 1:
        raise ValueError("--supersample > 1 requires --downsample lanczos")
    render_scale = args.output_scale * args.supersample

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
            # HQ tiles render with a two-logical-pixel overscan. Lanczos needs
            # source pixels beyond the final tile boundary; without overscan,
            # independently filtered neighboring tiles can develop hairline
            # seams even when the original vectors are continuous.
            margin = 2 if render_scale > 1 else 0
            logical_w = args.tile_width + margin * 2
            logical_h = args.tile_height + margin * 2
            canvas = Image.new(
                "RGBA",
                (logical_w * render_scale, logical_h * render_scale),
                (0, 0, 0, 0),
            )
            transform = (
                float(render_scale), 0.0, 0.0, float(render_scale),
                (args.place_x - (x - margin)) * render_scale,
                (args.place_y - (y - margin)) * render_scale,
            )
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
            if args.downsample == "lanczos":
                final_size = (
                    logical_w * args.output_scale,
                    logical_h * args.output_scale,
                )
                canvas = (
                    canvas.convert("RGBa")
                    .resize(final_size, Image.Resampling.LANCZOS)
                    .convert("RGBA")
                )
            if margin:
                m = margin * args.output_scale
                canvas = canvas.crop((
                    m, m,
                    m + args.tile_width * args.output_scale,
                    m + args.tile_height * args.output_scale,
                ))
            if canvas.getchannel("A").getbbox() is None:
                continue
            filename = f"x{x:+07d}_y{y:+07d}.png"
            canvas.save(args.output / filename, optimize=True)
            records.append({"world_x": x, "world_y": y, "file": filename})

    manifest = {
        "symbol": args.symbol,
        "excluded_symbols": sorted(excluded_symbols),
        "placement": [args.place_x, args.place_y],
        "tile_size": [args.tile_width * args.output_scale, args.tile_height * args.output_scale],
        "logical_tile_size": [args.tile_width, args.tile_height],
        "logical_pixel_scale": float(args.output_scale),
        "supersample": args.supersample,
        "downsample": args.downsample,
        "alpha_filtering": "premultiplied" if args.downsample == "lanczos" else "none",
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
        "logical_tile_size": manifest["logical_tile_size"],
        "tile_size": manifest["tile_size"],
        "logical_pixel_scale": manifest["logical_pixel_scale"],
        "supersample": manifest["supersample"],
        "downsample": manifest["downsample"],
        "unsupported_fills": manifest["unsupported_fills"],
        "missing_ids": manifest["missing_ids"],
    }, indent=2))


if __name__ == "__main__":
    main()