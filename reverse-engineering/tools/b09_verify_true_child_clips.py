#!/usr/bin/env python3
"""Offline original SWF child-clip proof: no original AVM1 gameplay claims."""
import argparse
import json
from pathlib import Path
from PIL import Image
from bake_sprite_frames import display_list

EXPECTED = {
    880: [("stand",799,1),("walk",824,10),("retreat",834,10),("attack",847,10),
          ("hurt",854,6),("fall",873,8),("die",879,8),("idle",799,1),("intro",799,1)],
    964: [("stand",883,1),("walk",915,11),("retreat",923,11),("attack",939,10),
          ("hurt",944,6),("fall",959,8),("die",963,8),("idle",883,1),("intro",939,10)],
}

def verify(symbols_path: Path, root: Path) -> None:
    symbols = json.loads(symbols_path.read_text(encoding="utf-8"))
    for sid, expected in EXPECTED.items():
        source = symbols[str(sid)]
        assert source["frames"] == 89
        labels = sorted(source["labels"], key=lambda item: item["frame"])
        folder = root / str(sid)
        baked = json.loads((folder / "manifest.json").read_text(encoding="utf-8"))
        assert baked["format"] == "source-true-child-timelines-v1"
        assert baked["outer_symbol"] == sid
        assert (baked["base_scale"],baked["output_scale"],baked["supersample"]) == (1,1,1)
        assert not baked["unsupported_fills"] and not baked["missing_bitmap_ids"]
        cursor = 0
        for label, (name, child, count), state in zip(labels, expected, baked["states"], strict=True):
            assert label["label"] == name
            active = display_list(source, label["frame"]).get(1)
            assert active and active["character_id"] == child
            assert symbols[str(child)]["frames"] == count
            assert (state["name"], state["parent_frame"], state["child_symbol"]) == (name,label["frame"],child)
            assert (state["start"],state["end"],state["frame_count"]) == (cursor,cursor+count,count)
            cursor += count
        assert cursor == (55 if sid == 880 else 66)
        assert baked["frame_count"] == cursor
        assert len(baked["frames"]) == len(list(folder.glob("*.png"))) == cursor
        for i, frame in enumerate(baked["frames"]):
            assert frame["frame"] == i and frame["file"] == f"{i:03}.png"
            assert 0 <= frame["local_frame"] < symbols[str(frame["child_symbol"])]["frames"]
            with Image.open(folder / frame["file"]) as png:
                assert png.size == tuple(baked["size"])
                assert png.getchannel("A").getbbox() is not None, (sid,i)
        if sid == 964:
            for label in ("walk","retreat"):
                assert any(x["animation"] == label and x["local_frame"] == 10 for x in baked["frames"])
        print("B09_TRUE_CHILD_CLIPS_PASS",sid,cursor,"source frames",flush=True)
    print("B09_TRUE_CHILD_SOURCE_GATE_PASS 121 independently baked 1x child frames",flush=True)

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--symbols",type=Path,required=True)
    p.add_argument("--root",type=Path,required=True)
    args = p.parse_args()
    verify(args.symbols,args.root)
