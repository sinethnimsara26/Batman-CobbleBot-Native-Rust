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


MAGIC = b"BCBFRM01"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("frames", type=Path, help="directory written by bake_sprite_frames.py --all-frames")
    parser.add_argument("output", type=Path, help="packed binary output, normally assets/batman_frames.bin")
    args = parser.parse_args()

    manifest = json.loads((args.frames / "manifest.json").read_text(encoding="utf-8"))
    anchor_x, anchor_y = manifest["anchor_in_bitmap"]
    frame_records = []
    payload = bytearray(MAGIC + struct.pack("<I", manifest["frame_count"]))
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
        "format": MAGIC.decode("ascii"),
        "frame_count": len(frame_records),
        "animations": animations,
        "frames": frame_records,
    }, indent=2), encoding="utf-8")
    print(json.dumps({"frames": len(frame_records), "packed_bytes": len(payload), "sidecar": str(sidecar)}, indent=2))


if __name__ == "__main__":
    main()