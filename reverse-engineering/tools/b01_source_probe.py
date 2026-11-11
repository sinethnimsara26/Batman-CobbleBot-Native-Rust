#!/usr/bin/env python3
"""B01: evidence-only source AVM1/timeline probe for the canonical CobbleBot FWS.

Inputs must be SHA-verified before use. This never modifies game code or
asserts that static clip timelines equal actual interactive playback.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

from swf_re import read_rect
from inspect_intro_behavior import collect_movie_sprites, scan_timeline

EXPECTED_SHA256 = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
STAGES = {
    "level1b": (20, 1075, 783, 782),
    "level1c_penguin": (21, 1251, 1095, 1094),
    "level2a": (23, 1402, 1317, 1316),
    "level2b": (25, 1496, 1485, 1484),
}
CARD_SPRITES = {"escape": 1267, "tv": 1479, "kabuki": 1569}
FUNCTIONS = (
    "respawn", "respawn2", "goonAI", "bikerAI", "penguinAI",
    "birdAI", "cameraLogic", "cameraLogic2", "doorOpen",
    "elevatorLogic", "clockTimer", "pitLogic", "playerLogic",
)
SOURCE_SPRITES = {
    "level1b": (1075, 783, 880, 964, 1005, 716, 737),
    "level1c_penguin": (1251, 1095, 1250, 716, 737),
    "level2a": (1402, 1317, 1325, 1328, 1330, 1365, 1393),
    "level2b": (1496, 1485, 1325, 1328, 1330, 1486),
    "story_cards": (1267, 1479, 1569),
}


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def push_values(action):
    return [v.get("resolved") if v.get("resolved") is not None else v.get("value")
            for v in action.get("values", [])]


def slim(action):
    result = {"op": action.get("op"), "offset": action.get("offset")}
    for key in ("branch", "frame", "string", "function", "code_size", "body_offset"):
        if key in action:
            result[key] = action[key]
    if action.get("op") == "Push":
        result["push"] = push_values(action)
    return result


def flatten(ops, limit=150000):
    """Flatten disassembly in source order; preserve function nesting separately."""
    result = []
    def visit(items, depth):
        for item in items:
            if len(result) >= limit:
                raise ValueError("source AVM1 trace exceeded conservative action bound")
            result.append((item, depth))
            if item.get("body"):
                visit(item["body"], depth + 1)
    visit(ops, 0)
    return result


def windowed_conditions(ops):
    """Evidence for each original IF/JUMP, NOT semantic decompilation."""
    flat = flatten(ops)
    details = []
    for i, (a, depth) in enumerate(flat):
        if a.get("op") not in ("If", "Jump", "GotoFrame", "GotoFrame2", "GoToLabel"):
            continue
        lo = max(0, i - 13)
        hi = min(len(flat), i + 5)
        details.append({
            "operator": a["op"], "offset": a.get("offset"), "depth": depth,
            "branch": a.get("branch"),
            "nearby_avm1": [slim(z) for z, _ in flat[lo:hi]],
            "qualification": "Nearby stack-machine operations only; no proven logical predicate."
        })
    return details


def symbol_counts(instances):
    names = [p.get("name", "") for p in instances]
    return {
        "goons": sum(x.startswith("goon") for x in names),
        "bikers": sum(x.startswith("biker") for x in names),
        "doors": sum(x.startswith("door_") for x in names),
        "elevators": sum(x.startswith("elevator_") for x in names),
        "cameras": sum(x.startswith("camera_") for x in names),
        "checkpoints": sum(x.startswith("checkpoint_") for x in names),
        "ammo_pickups": sum(x.startswith("item_br") for x in names),
        "eagles": sum(x.startswith("eagle") for x in names),
        "lava": sum(x.startswith("lava") for x in names),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--swf", type=Path, required=True)
    parser.add_argument("--reports", type=Path, required=True)
    parser.add_argument("--generated-manifests", type=Path, required=True)
    parser.add_argument("--tracked-manifests", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)

    source = args.swf.read_bytes()
    digest = hashlib.sha256(source).hexdigest()
    if source[:3] != b"FWS" or digest != EXPECTED_SHA256:
        raise ValueError(f"Canonical FWS mismatch: signature={source[:3]!r}, SHA={digest}")
    if len(source) != 2819100:
        raise ValueError(f"Canonical FWS length changed: {len(source)}")

    actions = read_json(args.reports / "actions.json")
    symbols = read_json(args.reports / "symbols.json")
    root_placements = read_json(args.reports / "root_placements.json")
    regen_instances = read_json(args.generated_manifests / "level-instances.json")
    regen_routes = read_json(args.generated_manifests / "routing-evidence.json")
    parity = {}
    for name in ("level-instances", "level-constants-raw", "routing-evidence",
                 "function-index", "animation-labels", "movie"):
        regen = read_json(args.generated_manifests / f"{name}.json")
        tracked = read_json(args.tracked_manifests / f"{name}.json")
        parity[name] = {"equal": regen == tracked, "tracked_file": f"manifests/{name}.json"}
        if regen != tracked:
            raise AssertionError(f"Regenerated source differs from preserved {name}.json")

    # Global functions are defined in original source's root context, parser frame 15.
    original_global = [b for b in actions if b["context"] == "root" and b["frame"] == 15]
    found = {}
    for block in original_global:
        operations = block["actions"]
        for i, op in enumerate(operations):
            if op.get("op") not in ("DefineFunction", "DefineFunction2"):
                continue
            name = op.get("function", "")
            if not name and i and operations[i - 1].get("op") == "Push":
                vals = push_values(operations[i - 1])
                name = vals[-1] if vals and isinstance(vals[-1], str) else ""
            if name in FUNCTIONS:
                found[name] = {
                    "name": name,
                    "function": op.get("op"),
                    "body_offset": op.get("body_offset"),
                    "code_size": op.get("code_size"),
                    "params": op.get("params", []),
                    "registers": op.get("registers"),
                    "tag_offset": block["tag_offset"],
                    "actions": op.get("body", []),
                }
    missing = set(FUNCTIONS) - set(found)
    if missing:
        raise AssertionError(f"Original source global functions missing: {sorted(missing)}")

    # Full raw disassemblies and source jump sites are kept as reusable CI evidence.
    function_trace = {}
    for name, entry in found.items():
        trace = flatten(entry["actions"])
        function_trace[name] = {
            **entry,
            "total_flat_actions": len(trace),
            "op_frequency": dict(Counter(z.get("op", "?") for z, _ in trace)),
            "source_control_branches": windowed_conditions(entry["actions"]),
        }
    write_json(args.out / "b01-original-avm1-functions.json", function_trace)

    root_trace = {}
    level_report = {}
    for name, (frame, game_id, level_id, ground_id) in STAGES.items():
        stage = regen_instances[name]
        assert stage["root_frame"] == frame
        assert stage["game_symbol_id"] == game_id
        assert stage["level_symbol_id"] == level_id
        assert stage["ground"]["character_id"] == ground_id
        root_game = [x for x in root_placements
                     if x.get("frame") == frame and x.get("name") == "game"]
        if not any(x.get("character_id") == game_id for x in root_game):
            raise AssertionError(f"Root frame {frame} game placement mismatch")
        blocks = [x for x in actions if x["context"] == "root" and x["frame"] == frame]
        if not blocks:
            raise AssertionError(f"No original root AVM1 initialization at frame {frame}")
        root_trace[name] = {
            "root_parser_frame_zero_based": frame,
            "root_tag_offsets_absolute_FWS": [x["tag_offset"] for x in blocks],
            "blocks": [
                {"tag_offset_absolute_FWS": x["tag_offset"],
                 "source_actions": x["actions"],
                 "branch_windows": windowed_conditions(x["actions"])}
                for x in blocks
            ],
        }
        placements = stage["named_game_instances"]
        level_report[name] = {
            "root_parser_frame_zero_based": frame,
            "source_game_sprite": game_id,
            "source_level_sprite": level_id,
            "source_ground_sprite": ground_id,
            "player_placement": stage.get("player"),
            "ground_placement": stage.get("ground"),
            "named_source_placements": placements,
            "object_census_from_names": symbol_counts(placements),
            "root_AVM1_blocks": len(blocks),
            "root_AVM1_tag_offsets_absolute_FWS": [x["tag_offset"] for x in blocks],
            "root_movieclip_placements": root_game,
        }

    write_json(args.out / "b01-original-root-stage-actions.json", root_trace)

    route_details = [r for r in regen_routes if (
        (r.get("context") == "root" and r.get("frame") in (19, 20, 21, 22, 23, 24, 25, 26))
        or r.get("context") in ("sprite:1267", "sprite:1479", "sprite:1569")
    )]
    sprites = collect_movie_sprites(source)
    stage_audio = {}
    for name, ids in SOURCE_SPRITES.items():
        cue_sources = {}
        for sid in ids:
            if sid not in sprites:
                # HUD and known sprite IDs should always be defined.
                raise AssertionError(f"Source sprite {sid} absent")
            raw = scan_timeline(sprites[sid], 4, f"sprite:{sid}")
            cue_sources[str(sid)] = {
                "sprite_frames": symbols[str(sid)]["frames"],
                "direct_start_sounds": raw["sounds"],
                "significant_timeline_script_controls": raw["scripts"],
                "note": "Sprite-local timeline; not equivalent to actually executed audio/screen time.",
            }
        stage_audio[name] = cue_sources
    for name, (frame, _, _, _) in STAGES.items():
        _, start = read_rect(source, 8)
        raw = scan_timeline(source, start + 4, "root", root_frame=frame)
        stage_audio[name]["root"] = {
            "direct_start_sounds": raw["sounds"],
            "significant_timeline_script_controls": raw["scripts"],
        }
    write_json(args.out / "b01-source-timeline-and-sound-cues.json", stage_audio)

    # A readable summary deliberately does not pretend stack operations are
    # fully decompiled predicates or local clip frames prove real Flash playback.
    summary = {
        "batch": "LEVEL2-B01",
        "source": {
            "source_format": "canonical uncompressed FWS7",
            "sha256_verified_this_run": digest,
            "bytes": len(source),
            "stage_frames_are": "0-based indices from swf_re.py ShowFrame parser",
            "timeline_sound_cues_are": "source-local, NOT observed runtime playback",
        },
        "manifest_reproducibility": parity,
        "stages": level_report,
        "source_routing_pushes": route_details,
        "global_functions": {
            n: {"body_offset": f["body_offset"], "code_size": f["code_size"],
                "flat_actions": f["total_flat_actions"],
                "branch_sites": len(f["source_control_branches"])}
            for n, f in function_trace.items()
        },
        "source_timelines": {
            "identified_source_sprite_count": len(sprites),
            "selected_sprite_ids": SOURCE_SPRITES,
            "story_card_end_routes": {
                "escape": {"sprite": 1267, "zero_based_frame": 224, "next": "level2a"},
                "tv": {"sprite": 1479, "zero_based_frame": 151, "next": "level2b"},
                "kabuki": {"sprite": 1569, "zero_based_frame": 159, "next": "level3a"},
            },
        },
        "limitations": [
            "Bytecode source control branches are recorded in disassembly but not yet a verified high-level execution trace.",
            "Source placements and SoundStart tags do not prove frame-perfect original playback or enemy activation.",
            "Preview art is source symbol rasterization, not live game engine screenshot or camera replay.",
            "A final source-vs-Rust comparison requires exercising native gameplay after later batches."
        ],
    }
    write_json(args.out / "B01_SOURCE_EVIDENCE.json", summary)
    print("B01_SOURCE_FWS_SHA_VERIFIED", digest, "bytes", len(source))
    print("B01_MANIFEST_PARITY_PASS", ",".join(sorted(parity)))
    for name, info in level_report.items():
        print("B01_STAGE", name, "root_index", info["root_parser_frame_zero_based"],
              "census", json.dumps(info["object_census_from_names"], sort_keys=True),
              "root_DoAction", info["root_AVM1_blocks"])
    for name, f in summary["global_functions"].items():
        print("B01_SOURCE_FUNCTION", name, "body_offset", f["body_offset"],
              "size", f["code_size"], "conditional_jumps", f["branch_sites"])
    for name, parts in stage_audio.items():
        cues = [(key, item["frame"], item["sound_id"])
                for key, sprite in parts.items()
                for item in sprite.get("direct_start_sounds", [])]
        print("B01_TIMELINE_SOUND_CUES", name, json.dumps(cues[:30]),
              "total", len(cues), "SOURCE_LOCAL_NOT_PLAYBACK")
    print("B01_SOURCE_PROBE_PASS", args.out)


if __name__ == "__main__":
    main()
