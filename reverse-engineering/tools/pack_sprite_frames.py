#!/usr/bin/env python3
"""Pack baked per-frame PNGs into one self-contained Rust include asset.

The runtime decodes each embedded PNG with the pure-Rust `png` crate. Cropping
transparent margins keeps the decoded sprite cache compact; each frame record
stores the original player anchor relative to its cropped image.
"""
from __future__ import annotations

from pathlib import Path
import argparse
import io
import json
import struct

from PIL import Image


MAGIC_V1 = b"BCBFRM01"
MAGIC_V2 = b"BCBFRM02"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("frames", type=Path, help="directory written by bake_sprite_frames.py --all-frames")
    parser.add_argument("output", type=Path, help="packed binary output, normally assets/batman_frames.bin")
    parser.add_argument("--format-version", choices=(1,2), type=int, default=2,
                        help="BCBFRM container version (default: 2)")
    parser.add_argument("--logical-pixel-scale", type=float, default=1.0,
                        help="presentation pixels represented by one logical stage pixel (v2 only)")
    args = parser.parse_args()
    if not (args.logical_pixel_scale > 0.0):
        raise ValueError("--logical-pixel-scale must be > 0")
    if args.format_version == 1 and args.logical_pixel_scale != 1.0:
        raise ValueError("BCBFRM01 has implicit logical pixel scale 1.0")

    manifest = json.loads((args.frames / "manifest.json").read_text(encoding="utf-8"))
    anchor_x, anchor_y = manifest["anchor_in_bitmap"]
    frame_records = []
    magic = MAGIC_V2 if args.format_version == 2 else MAGIC_V1
    payload = bytearray(magic + struct.pack("<I", manifest["frame_count"]))
    if args.format_version == 2:
        payload.extend(struct.pack("<f", args.logical_pixel_scale))
    for entry in manifest["frames"]:
        with Image.open(args.frames / entry["file"]) as opened:
            frame = opened.convert("RGBA")
        bounds = frame.getchannel("A").getbbox()
        if bounds is None:
            bounds = (0, 0, 1, 1)
            frame = Image.new("RGBA", (1, 1), (0, 0, 0, 0))
        else:
            frame = frame.crop(bounds)
        cropped_anchor = (anchor_x - bounds[0], anchor_y - bounds[1])
        encoded = io.BytesIO()
        frame.save(encoded, format="PNG", optimize=True)
        png = encoded.getvalue()
        if frame.width > 0xFFFF or frame.height > 0xFFFF:
            raise ValueError(f"frame {entry['frame']} exceeds the packed dimension limit")
        payload.extend(struct.pack("<HHffI", frame.width, frame.height, *cropped_anchor, len(png)))
        payload.extend(png)
        frame_records.append({
            **entry,
            "size": [frame.width, frame.height],
            "anchor_in_cropped_image": list(cropped_anchor),
            "png_bytes": len(png),
        })

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(payload)
    sidecar = args.output.with_name(args.output.stem + ".json")
    animations = []
    previous = None
    for frame in manifest["frames"]:
        if frame.get("animation") != previous:
            previous = frame.get("animation")
            if previous is not None:
                animations.append({"name": previous, "frame": frame["frame"]})
    sidecar.write_text(json.dumps({
        "format": magic.decode("ascii"),
        "frame_count": len(frame_records),
        "logical_pixel_scale": 1.0 if args.format_version == 1 else args.logical_pixel_scale,
        "animations": animations,
        "frames": frame_records,
    }, indent=2), encoding="utf-8")
    print(json.dumps({"format": magic.decode("ascii"), "frames": len(frame_records), "logical_pixel_scale": 1.0 if args.format_version == 1 else args.logical_pixel_scale, "packed_bytes": len(payload), "sidecar": str(sidecar)}, indent=2))


if __name__ == "__main__":
    main()