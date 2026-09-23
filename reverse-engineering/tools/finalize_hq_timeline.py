#!/usr/bin/env python3
"""Finalize a supersampled bake_sprite_frames.py timeline at a lower scale.

This is intentionally separate from the SWF renderer: bake_sprite_frames.py
first resolves display lists, embedded DefineText glyphs, masks and placement
CXFORMWITHALPHA at the temporary supersample resolution. Only then do we
premultiplied-alpha Lanczos downsample the completed RGBA frames.

That ordering is required for HQ UI/transitions so transformed alpha edges are
part of the antialias filter instead of being applied after resampling.
"""
from __future__ import annotations

from pathlib import Path
import argparse
import json

from PIL import Image


def resize_premultiplied(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    if image.size == size:
        return image.copy()
    return (
        image.convert("RGBa")
        .resize(size, Image.Resampling.LANCZOS)
        .convert("RGBA")
    )


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("source", type=Path, help="supersampled all-frames directory")
    ap.add_argument("output", type=Path, help="final all-frames directory")
    ap.add_argument("--output-scale", type=float, required=True)
    ap.add_argument("--supersample", type=int, required=True)
    args = ap.parse_args()

    if args.output_scale <= 0:
        raise ValueError("--output-scale must be positive")
    if args.supersample < 1:
        raise ValueError("--supersample must be >= 1")

    manifest = json.loads((args.source / "manifest.json").read_text(encoding="utf-8"))
    sx, sy = [float(v) for v in manifest["scale"]]
    if abs(sx - sy) > 1e-9:
        raise ValueError("source timeline must use uniform scale")
    expected = args.output_scale * args.supersample
    if abs(sx - expected) > 1e-6:
        raise ValueError(f"source scale mismatch: expected {expected}, got {sx}")

    factor = float(args.supersample)
    args.output.mkdir(parents=True, exist_ok=True)

    final_frames = []
    expected_size = None
    for rec in manifest["frames"]:
        src_path = args.source / rec["file"]
        with Image.open(src_path) as source:
            rgba = source.convert("RGBA")
            final_size = (
                max(1, round(rgba.width / factor)),
                max(1, round(rgba.height / factor)),
            )
            final = resize_premultiplied(rgba, final_size)
        if expected_size is None:
            expected_size = list(final.size)
        elif list(final.size) != expected_size:
            raise ValueError("all timeline frames must share one packed canvas size")
        out_name = rec["file"]
        final.save(args.output / out_name, optimize=True)
        final_frames.append(dict(rec))

    final_manifest = dict(manifest)
    final_manifest["size"] = expected_size
    final_manifest["scale"] = [args.output_scale, args.output_scale]
    final_manifest["anchor_in_bitmap"] = [
        float(v) / factor for v in manifest["anchor_in_bitmap"]
    ]
    final_manifest["frames"] = final_frames
    final_manifest["hq_finalize"] = {
        "source_scale": sx,
        "output_scale": args.output_scale,
        "supersample": args.supersample,
        "downsample": "lanczos",
        "alpha_filtering": "premultiplied",
    }
    (args.output / "manifest.json").write_text(
        json.dumps(final_manifest, indent=2), encoding="utf-8"
    )

    print(json.dumps({
        "symbol": final_manifest["symbol"],
        "frame_count": final_manifest["frame_count"],
        "size": final_manifest["size"],
        "scale": final_manifest["scale"],
        "anchor_in_bitmap": final_manifest["anchor_in_bitmap"],
        "hq_finalize": final_manifest["hq_finalize"],
    }, indent=2))


if __name__ == "__main__":
    main()
