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
INTRO_SYMBOLS = (122, 123, 126, 35)  # Also inspect the root's 15-frame transition sprite.


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
    for code, blob, tag_offset, payload_offset in tags(payload, start, len(payload)):
        if code == 1:
            frame += 1
        elif code == 15 and len(blob) >= 3:
            sounds.append({
                "frame": frame,
                "sound_id": struct.unpack_from("<H", blob, 0)[0],
                "sound_flags": blob[2],
                "context": context,
                "tag_offset": tag_offset,
            })
        elif code == 12:
            from swf_re import disasm_actions
            acts = disasm_actions(blob, base=payload_offset)
            significant = []
            for a in acts:
                if a.get("op") in (
                    "Stop", "Play", "StopSounds", "GotoFrame",
                    "GotoFrame2", "GoToLabel", "SetTarget", "SetTarget2",
                    "NextFrame", "PreviousFrame",
                ):
                    significant.append({
                        key:a[key] for key in ("op", "offset", "frame", "string", "values")
                        if key in a
                    })
            if significant:
                scripts.append({"frame": frame, "tag_offset": tag_offset, "actions": significant})
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



def inspect_skip_button(data: bytes, root_tag_start: int, button_id: int = 129) -> dict:
    """Read original DefineButton2 BUTTONCONDACTION handlers, not just UP art.

    SWF v19: ActionOffset is measured from its UI16 field at payload byte 3.
    Each nonfinal CondActionSize measures from its own UI16 field to the next.
    This only DISASSEMBLES source events; it never simulates a click/branch.
    """
    from swf_re import disasm_actions, collect_strings_actions

    events = (
        (15, "idle_to_over_down"),
        (14, "out_down_to_idle"),
        (13, "out_down_to_over_down"),
        (12, "over_down_to_out_down"),
        (11, "over_down_to_over_up"),
        (10, "over_up_to_over_down"),
        (9, "over_up_to_idle"),
        (8, "idle_to_over_up"),
        (0, "over_down_to_idle"),
    )
    for code, blob, tag_offset, payload_offset in tags(data, root_tag_start, len(data)):
        if code != 34 or len(blob) < 5:
            continue
        if struct.unpack_from("<H", blob, 0)[0] != button_id:
            continue
        source_offset = struct.unpack_from("<H", blob, 3)[0]
        report = {
            "button_id": button_id,
            "tag_offset": tag_offset,
            "action_offset": source_offset,
            "track_as_menu": bool(blob[2] & 1),
            "handlers": [],
            "note": "Byte offsets are absolute FWS offsets; event names are SWF BUTTONCONDACTION bits.",
        }
        if source_offset == 0:
            return report
        pos = 3 + source_offset
        if not (5 <= pos and pos + 4 <= len(blob)):
            raise ValueError(f"DefineButton2 {button_id} ActionOffset outside source tag")
        for _ in range(64):
            # CondActionSize is LE UI16; the adjacent UB[16] condition
            # flags are MSB-first. Previously both were read LE, which
            # misclassified the source mouse-release flag as End-key.
            cond_size = struct.unpack_from("<H", blob, pos)[0]
            flags = int.from_bytes(blob[pos+2:pos+4], "big")
            end = pos + cond_size if cond_size else len(blob)
            if end < pos + 5 or end > len(blob):
                raise ValueError(f"DefineButton2 {button_id} invalid CondActionSize {cond_size}")
            raw_actions = blob[pos + 4:end]
            if raw_actions[-1] != 0:
                raise ValueError("BUTTONCONDACTION does not terminate with ActionEndFlag")
            actions = disasm_actions(raw_actions, base=payload_offset + pos + 4)
            report["handlers"].append({
                "condition_offset": payload_offset + pos,
                "condition_size": cond_size,
                "conditions_hex": f"0x{flags:04x}",
                "events": [name for bit, name in events if flags & (1 << bit)],
                "key_code": (flags >> 1) & 127,
                "actions": [
                    {k: a[k] for k in ("op", "offset", "frame", "string", "values", "branch", "function")
                     if k in a}
                    for a in actions
                ],
                "strings": collect_strings_actions(actions),
            })
            if cond_size == 0:
                break
            pos = end
        else:
            raise ValueError(f"DefineButton2 {button_id} contains too many event handlers")
        return report
    raise ValueError(f"DefineButton2 {button_id} not found in source SWF")


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
        "skip_button_actions": inspect_skip_button(data, start),
        "offset_coordinates": "Root tag/action offsets are absolute FWS bytes; sprite tag/action offsets are relative to DefineSprite payload.",
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
        "skip_button_handlers":len(result["skip_button_actions"]["handlers"]),
    },indent=2))


if __name__ == "__main__":
    main()
