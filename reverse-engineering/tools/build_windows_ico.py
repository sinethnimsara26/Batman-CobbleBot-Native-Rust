#!/usr/bin/env python3
"""Batch07: package Batch06's verified PNG family as a real multi-size Windows ICO.

The source HUD and visual design remain owned by bake_windows_icon_art.py.
The standard ICO directory supports up to 256x256; the 512 PNG is retained
in the independent artwork package rather than silently truncated here.
No PNG resampling is performed by this step.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import struct
from pathlib import Path
from PIL import Image

ICON_SIZES = (16, 24, 32, 48, 64, 128, 256)


def pack_icon(folder: Path, output: Path) -> dict:
    entries = []
    for size in ICON_SIZES:
        png = folder / f"batman-cobblebot-{size}.png"
        payload = png.read_bytes()
        if not payload.startswith(b"\x89PNG\r\n\x1a\n"):
            raise ValueError(f"Not an embedded PNG: {png}")
        with Image.open(png) as img:
            if img.size != (size, size) or img.mode != "RGBA":
                raise ValueError(f"Expected {size}x{size} RGBA PNG: {png}")
            corners = ((0, 0), (0, size - 1), (size - 1, 0), (size - 1, size - 1))
            if any(img.getpixel(pt)[3] != 0 for pt in corners):
                raise ValueError(f"Nontransparent icon corner: {png}")
        entries.append((size, payload))
    header = struct.pack("<HHH", 0, 1, len(entries))
    offset = 6 + 16 * len(entries)
    directory = bytearray()
    images = bytearray()
    for size, payload in entries:
        directory += struct.pack("<BBBBHHII", 0 if size == 256 else size,
                                 0 if size == 256 else size,
                                 0, 0, 1, 32, len(payload), offset)
        images += payload
        offset += len(payload)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(header + directory + images)
    verify_icon(output)
    return {"format": "Windows ICO with PNG-compressed RT_ICON images",
            "embedded_sizes": list(ICON_SIZES),
            "source_png_files": [f"batman-cobblebot-{s}.png" for s in ICON_SIZES],
            "file_sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
            "bytes": output.stat().st_size}


def verify_icon(path: Path) -> None:
    raw = path.read_bytes()
    reserved, kind, count = struct.unpack_from("<HHH", raw)
    if (reserved, kind, count) != (0, 1, len(ICON_SIZES)):
        raise ValueError("Invalid ICO header/directory entry count")
    spans = []
    for index, expected in enumerate(ICON_SIZES):
        width, height, _, _, planes, bpp, length, offset = struct.unpack_from(
            "<BBBBHHII", raw, 6 + index * 16)
        if (width or 256, height or 256) != (expected, expected):
            raise ValueError(f"ICO size {index} mismatch")
        if planes != 1 or bpp != 32 or length <= 64:
            raise ValueError(f"ICO pixel metadata invalid at {index}")
        if offset + length > len(raw):
            raise ValueError(f"ICO payload outside file at {index}")
        spans.append((offset, offset + length))
        if not raw[offset:offset+length].startswith(b"\x89PNG\r\n\x1a\n"):
            raise ValueError(f"ICO item {index} is not PNG-compressed")
    if spans[0][0] != 6 + count * 16 or any(
        spans[i][1] != spans[i+1][0] for i in range(count-1)
    ) or spans[-1][1] != len(raw):
        raise ValueError("ICO directory offsets are not contiguous")
    with Image.open(path) as image:
        if image.format != "ICO" or image.size != (256, 256):
            raise ValueError(f"Pillow could not read multi-size ICO: {path}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--art", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    result = pack_icon(args.art, args.out)
    report = args.art / "ico-resource-report.json"
    report.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print("ICON-EXE-07 ICO packaging PASS", json.dumps(result))


if __name__ == "__main__":
    main()
