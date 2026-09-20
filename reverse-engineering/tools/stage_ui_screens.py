#!/usr/bin/env python3
"""Stage the original title and controls bitmap art for Rust compile-time embedding."""
from __future__ import annotations

from pathlib import Path
import argparse
import json

from PIL import Image


SCREEN_IDS = {22: "title_screen.png", 37: "instructions_screen.png"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("bitmaps", type=Path, help="directory produced by extract_bitmaps.py")
    parser.add_argument("output", type=Path, help="project assets directory")
    args = parser.parse_args()

    manifest = json.loads((args.bitmaps / "manifest.json").read_text(encoding="utf-8"))
    entries = {entry["id"]: entry for entry in manifest if "file" in entry}
    args.output.mkdir(parents=True, exist_ok=True)
    for bitmap_id, filename in SCREEN_IDS.items():
        entry = entries.get(bitmap_id)
        if entry is None:
            raise ValueError(f"original SWF bitmap {bitmap_id} is missing")
        with Image.open(args.bitmaps / entry["file"]) as source:
            image = source.convert("RGBA")
        if image.size != (600, 400):
            raise ValueError(f"bitmap {bitmap_id} expected 600x400, got {image.size}")
        image.save(args.output / filename, format="PNG", optimize=True)
        print(f"staged bitmap {bitmap_id} ({image.width}x{image.height}) as {filename}")


if __name__ == "__main__":
    main()