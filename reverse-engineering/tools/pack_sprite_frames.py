#!/usr/bin/env python3
"""Pack baked per-frame PNGs into one self-contained Rust include asset.

BCBFRM02 adds an explicit logical-pixel scale to the header so the runtime can
distinguish legacy 1x packs from future HQ 3x packs without magic knowledge of
how a particular asset was baked.

The runtime still accepts BCBFRM01 for migration/backward compatibility.

Frame payloads are unchanged between v1 and v2:
    u16 width
    u16 height
    f32 cropped_anchor_x
    f32 cropped_anchor_y
    u32 png_length
    png bytes
"""
from __future__ import annotations

from pathlib import Path
import argparse
import io
import json
import math
import struct

from PIL import Image


MAGIC_V1 = b"BCBFRM01"
MAGIC_V2 = b"BCBFRM02"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "frames",
        type=Path,
        help="directory written by bake_sprite_frames.py --all-frames",
    )
    parser.add_argument(
        "output",
        type=Path,
        help="packed binary output, normally assets/batman_frames.bin",
    )
    parser.add_argument(
        "--logical-pixel-scale",
        type=float,
        default=1.0,
        help=(
            "number of packed image pixels per original 600x400 logical stage "
            "pixel (1.0 for current packs, 3.0 for future HQ packs)"
        ),
    )
    parser.add_argument(
        "--legacy-v1",
        action="store_true",
        help="emit the old BCBFRM01 header for compatibility testing only",
    )
    args = parser.parse_args()

    scale = float(args.logical_pixel_scale)
    if not math.isfinite(scale) or scale <= 0.0:
        raise ValueError(f"logical pixel scale must be finite and > 0, got {scale!r}")
    if args.legacy_v1 and scale != 1.0:
        raise ValueError("BCBFRM01 has no scale field and can only represent scale 1.0")

    manifest = json.loads((args.frames / "manifest.json").read_text(encoding="utf-8"))
    anchor_x, anchor_y = manifest["anchor_in_bitmap"]
    frame_records = []

    magic = MAGIC_V1 if args.legacy_v1 else MAGIC_V2
    payload = bytearray(magic + struct.pack("<I", manifest["frame_count"]))
    if magic == MAGIC_V2:
        payload.extend(struct.pack("<f", scale))

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

        payload.extend(
            struct.pack(
                "<HHffI",
                frame.width,
                frame.height,
                *cropped_anchor,
                len(png),
            )
        )
        payload.extend(png)
        frame_records.append(
            {
                **entry,
                "size": [frame.width, frame.height],
                "anchor_in_cropped_image": list(cropped_anchor),
                "png_bytes": len(png),
            }
        )

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

    sidecar.write_text(
        json.dumps(
            {
                "format": magic.decode("ascii"),
                "frame_count": len(frame_records),
                "logical_pixel_scale": scale,
                "header_bytes": 12 if magic == MAGIC_V1 else 16,
                "animations": animations,
                "frames": frame_records,
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    print(
        json.dumps(
            {
                "format": magic.decode("ascii"),
                "logical_pixel_scale": scale,
                "frames": len(frame_records),
                "packed_bytes": len(payload),
                "sidecar": str(sidecar),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
