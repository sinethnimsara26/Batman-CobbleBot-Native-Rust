#!/usr/bin/env python3
"""Audit the actual AVM1 frame actions and StartSound cues in CobbleBot's intro.

This is an evidence-only tool. It reads a verified SWF in CI, records direct
and reachable nested timeline cues, and never guesses the original audio
sequence or pretends script-controlled clips always play automatically.
"""
from __future__ import annotations

import argparse
import json
import struct
from pathlib import Path

from swf_re import read_rect, tags

ROOT_INTRO_FRAME = 18
INTRO_SYMBOLS = (122, 123, 126)


def collect_movie_sprites(data: bytes) -> dict[int, bytes]:
    result = {}
    _, pos = read_rect(data, 8)
    pos += 4
    def walk(blob: bytes, start: int):
        for code, payload, _, _ in tags(blob, start, len(blob)):
            if code == 39 and len(payload) >= 4:
                sid = struct.unpack_from("<H", payload, 0)[0]
                result[sid] = payload
                walk(payload, 4)
    walk(data, pos)
    return result


def scan_timeline(payload: bytes, start: int, context: str, root_frame: int | None = None):
    frame = 0
    sounds = []
    scripts = []
    placements = []
    for code, blob, _, _ in tags(payload, start, len(payload)):
        if code == 1:
            frame += 1
        elif code == 15 and len(blob) >= 3:
            sounds.append({
                "frame": frame,
                "sound_id": struct.unpack_from("<H", blob, 0)[0],
                "sound_flags": blob[2],
                "context": context,
            })
        elif code == 12:
            from swf_re import disasm_actions
            acts = disasm_actions(blob)
            significant = []
            for a in acts:
                if a.get("op") in (
                    "Stop", "Play", "StopSounds", "GotoFrame",
                    "GotoFrame2", "GoToLabel", "SetTarget", "SetTarget2",
                    "NextFrame", "PreviousFrame",
                ):
                    significant.append({
                        key:a[key] for key in ("op", "frame", "string", "values")
                        if key in a
                    })
            if significant:
                scripts.append({"frame": frame, "actions": significant})
        elif code == 26 and len(blob) >= 3:
            from swf_re import parse_place2
            obj = parse_place2(blob)
            if obj and obj.get("character_id") is not None:
                placements.append({
                    "frame": frame,
                    "depth": obj.get("depth"),
                    "symbol": obj["character_id"],
                    "name": obj.get("name"),
                })
    if root_frame is not None:
        sounds = [a for a in sounds if a["frame"] == root_frame]
        scripts = [a for a in scripts if a["frame"] == root_frame]
        placements = [a for a in placements if a["frame"] == root_frame]
    return {"sounds": sounds, "scripts": scripts, "placements": placements}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("sound_manifest", type=Path)
    ap.add_argument("out", type=Path)
    ns = ap.parse_args()
    data = ns.swf.read_bytes()
    if data[:3] != b"FWS":
        raise ValueError("Expected hash-verified, uncompressed FWS")
    sounds = json.loads(ns.sound_manifest.read_text(encoding="utf-8"))
    sound_by_id = {s["id"]: s for s in sounds}
    sprites = collect_movie_sprites(data)
    _, start = read_rect(data, 8)
    start += 4

    source = {"root": scan_timeline(data, start, "root", ROOT_INTRO_FRAME)}
    visited = set()

    def visit(symbol: int, depth: int):
        if symbol in visited or symbol not in sprites or depth > 7:
            return
        visited.add(symbol)
        payload = sprites[symbol]
        direct = scan_timeline(payload, 4, f"sprite:{symbol}")
        source[f"sprite:{symbol}"] = {
            "frame_count": struct.unpack_from("<H", payload, 2)[0],
            **direct,
        }
        for obj in direct["placements"]:
            visit(obj["symbol"], depth+1)

    for symbol in INTRO_SYMBOLS:
        visit(symbol, 0)

    # Clip 34 is a script-driven fade and must not be treated as an
    # auto-playing timeline, but its actions/sounds can be reviewed separately.
    if 34 in sprites:
        payload = sprites[34]
        source["scripted_fade:34"] = scan_timeline(payload, 4, "sprite:34")
    events = []
    for context, entry in source.items():
        for cue in entry["sounds"]:
            sid = cue["sound_id"]
            meta = sound_by_id.get(sid, {})
            events.append({
                **cue,
                "sound_name": meta.get("export_name", f"sound_{sid}"),
                "codec": meta.get("format_name", "unknown"),
                "file": meta.get("file"),
                "note": "Nested clip timing depends on parent placement and ActionScript stop/start",
            })

    result = {
        "source_intro_frame": ROOT_INTRO_FRAME,
        "direct_root_sound_cues": source["root"]["sounds"],
        "intro_symbol_ids": list(INTRO_SYMBOLS),
        "nested_sprite_count": len(visited),
        "timeline_sound_cues": events,
        "original_timeline_actions": {
            k:v["scripts"] for k,v in source.items() if v.get("scripts")
        },
        "note": (
            "Cues are not automatically a stage-clock schedule: nested movie clips "
            "can pause, start later, or loop. Verify each cue before playback."
        ),
    }
    ns.out.parent.mkdir(parents=True, exist_ok=True)
    ns.out.write_text(json.dumps(result,indent=2),encoding="utf-8")
    print("INTRO AVM1 SOUND AUDIT:",json.dumps({
        "nested_sprites":len(visited),
        "sound_events":len(events),
        "clip_actions":sum(len(v) for v in result["original_timeline_actions"].values()),
        "cues":events[:35],
    },indent=2))


if __name__ == "__main__":
    main()
