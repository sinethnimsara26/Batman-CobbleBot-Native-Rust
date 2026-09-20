#!/usr/bin/env python3
"""Pack baked level PNG tiles for lazy decoding inside the native executable."""
from __future__ import annotations

from pathlib import Path
import argparse
import json
import struct


MAGIC = b"BCLVT001"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("tiles", type=Path, help="directory written by bake_level_tiles.py")
    parser.add_argument("output", type=Path, help="packed binary output")
    args = parser.parse_args()

    manifest = json.loads((args.tiles / "manifest.json").read_text(encoding="utf-8"))
    tiles = sorted(manifest["tiles"], key=lambda item: (item["world_y"], item["world_x"]))
    payload = bytearray(MAGIC + struct.pack("<I", len(tiles)))
    records = []
    for tile in tiles:
        png = (args.tiles / tile["file"]).read_bytes()
        payload.extend(struct.pack("<iiI", tile["world_x"], tile["world_y"], len(png)))
        payload.extend(png)
        records.append({**tile, "png_bytes": len(png)})

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(payload)
    sidecar = args.output.with_name(args.output.stem + ".json")
    sidecar.write_text(json.dumps({
        "format": MAGIC.decode("ascii"),
        "tile_size": manifest["tile_size"],
        "world_bounds": manifest["world_bounds"],
        "tile_count": len(records),
        "unsupported_fills": manifest["unsupported_fills"],
        "tiles": records,
    }, indent=2), encoding="utf-8")
    print(json.dumps({"tiles": len(records), "packed_bytes": len(payload), "sidecar": str(sidecar)}, indent=2))


if __name__ == "__main__":
    main()