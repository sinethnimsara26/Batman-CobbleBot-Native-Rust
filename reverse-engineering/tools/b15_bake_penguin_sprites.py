#!/usr/bin/env python3
"""B15: source-backed Level 1C Penguin (#1250 `boss`) timeline, child clip, and FX baker.

Development-only offline source extraction & verification from canonical cobblebot.swf:
- Validates canonical FWS SHA-256 and #1250 (`boss`) 99-frame outer timeline + 10 labels.
- Validates all 10 depth-1 child clips (#1103..#1249, totaling 162 true child frames),
  exact child placement matrices, DoAction stop/loop/state scripts, and grandchild FX
  (`#695` deflected batarang, `#1184..#1192` laser charge/beam/alpha-0 hitBox, `#1103`
  white-flash cxform in `fall`, and `#1240..#1248` die umbrella explosion).
- Renders all 99 1x outer parent frames and all 10 true 6x->HQ3x label-start views.
- Bakes all 162 1x true nested child frames onto the unified `[938, 410]` canvas via
  `bake_player_states.py` and builds a contact sheet + JSON audit report.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

from b03_bake_stage_asset_proofs import expect_original_source
from b05_bake_bank_street_1x import source_definition_tags
from bake_level_tiles import load_bitmaps
from bake_sprite_frames import display_list, load_shapes, render_symbol, symbol_bounds
from swf_re import read_rect, tags

SOURCE_SHA = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
BOSS_SYMBOL_ID = 1250
EXPECTED_PARENT_FRAMES = 99
EXPECTED_TOTAL_CHILD_FRAMES = 162

# (label, outer_start, outer_span, child_symbol_id, child_frames, instance_name,
#  expected_sx, expected_sy, expected_tx, expected_ty)
EXPECTED_SPANS = (
    ("stand", 0, 9, 1103, 21, None, 1.4289703369140625, 1.4289703369140625, 17.55, -76.95, ()),
    ("walk", 9, 10, 1117, 8, None, 1.45574951171875, 1.45574951171875, 16.3, -78.5, ()),
    ("jump", 19, 10, 1135, 9, None, 1.46575927734375, 1.46575927734375, 11.15, -79.0, (8,)),
    ("dodge", 29, 10, 1149, 10, None, 1.46575927734375, 1.46575927734375, 11.15, -79.0, (9,)),
    ("block", 39, 10, 1156, 14, None, 1.472930908203125, 1.472930908203125, -62.1, -80.45, (13,)),
    ("attack", 49, 10, 1172, 10, None, 1.4600067138671875, 1.4600067138671875, 15.8, -78.9, (9,)),
    ("attack2", 59, 10, 1205, 23, "laser", 1.4437255859375, 1.4437255859375, 13.35, -77.5, (10, 22)),
    ("hurt", 69, 9, 1207, 6, None, 1.458709716796875, 1.458709716796875, -60.05, -79.5, (5,)),
    ("fall", 78, 10, 1224, 21, None, 1.4470672607421875, 1.4470672607421875, -58.6, -77.1, (20,)),
    ("die", 88, 11, 1249, 40, None, 1.5249481201171875, 1.5249481201171875, 28.05, -81.05, (39,)),
)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def assert_close(actual: float, expected: float, tol: float = 1e-6, label: str = "") -> None:
    if abs(actual - expected) > tol:
        raise AssertionError(f"{label}: expected {expected}, got {actual}")


def extract_sprite_action_frames(swf_path: Path, sprite_ids: set[int]) -> dict[int, list[int]]:
    raw = swf_path.read_bytes()
    _, pos = read_rect(raw, 8)
    pos += 4
    result: dict[int, list[int]] = {}
    for code, pl, _toff, _ in tags(raw, pos, len(raw)):
        if code == 39 and len(pl) >= 4:
            sid = struct.unpack_from("<H", pl, 0)[0]
            if sid in sprite_ids:
                action_frames: list[int] = []
                frame = 0
                for sub_code, _sub_pl, _, _ in tags(pl, 4, len(pl)):
                    if sub_code == 1:
                        frame += 1
                    elif sub_code == 12:
                        action_frames.append(frame)
                result[sid] = action_frames
    missing = sprite_ids - set(result)
    if missing:
        raise AssertionError(f"DefineSprite IDs not found in SWF: {sorted(missing)}")
    return result


def verify_source_timeline_and_fx(swf_path: Path, symbols: dict, shapes: dict) -> dict:
    defs = source_definition_tags(swf_path, {BOSS_SYMBOL_ID})
    assert defs == {BOSS_SYMBOL_ID: 39}, defs

    boss = symbols[str(BOSS_SYMBOL_ID)]
    assert boss["frames"] == EXPECTED_PARENT_FRAMES, boss["frames"]
    labels = sorted(boss["labels"], key=lambda x: x["frame"])
    assert [(l["label"], l["frame"]) for l in labels] == [
        (s[0], s[1]) for s in EXPECTED_SPANS
    ], labels

    all_sids = {BOSS_SYMBOL_ID} | {s[3] for s in EXPECTED_SPANS}
    actions_by_sid = extract_sprite_action_frames(swf_path, all_sids)
    assert actions_by_sid[BOSS_SYMBOL_ID] == [], actions_by_sid[BOSS_SYMBOL_ID]

    span_audit = []
    baked_cursor = 0
    for i, (
        label,
        start,
        span_len,
        cid,
        exp_child_frames,
        exp_name,
        esx,
        esy,
        etx,
        ety,
        exp_actions,
    ) in enumerate(EXPECTED_SPANS):
        end = EXPECTED_SPANS[i + 1][1] if i + 1 < len(EXPECTED_SPANS) else EXPECTED_PARENT_FRAMES
        assert end - start == span_len, (label, start, end, span_len)

        dl = display_list(boss, start)
        assert 1 in dl, (label, "missing depth-1 placement")
        p = dl[1]
        assert p.get("character_id") == cid, (label, p)
        assert p.get("name") == exp_name, (label, p)
        m = p["matrix"]
        assert_close(m["sx"], esx, 1e-6, f"{label}.sx")
        assert_close(m["sy"], esy, 1e-6, f"{label}.sy")
        assert_close(m["tx"], etx, 1e-4, f"{label}.tx")
        assert_close(m["ty"], ety, 1e-4, f"{label}.ty")

        csym = symbols[str(cid)]
        cframes = int(csym["frames"])
        assert cframes == exp_child_frames, (label, cid, cframes, exp_child_frames)
        assert tuple(actions_by_sid[cid]) == exp_actions, (label, cid, actions_by_sid[cid], exp_actions)

        span_audit.append(
            {
                "label": label,
                "parent_start_frame": start,
                "parent_span_frames": span_len,
                "child_symbol_id": cid,
                "child_frames": cframes,
                "child_doaction_frames": list(exp_actions),
                "baked_start_frame": baked_cursor,
                "baked_end_frame": baked_cursor + cframes,
                "instance_name": exp_name,
                "child_matrix": {k: m[k] for k in ("sx", "sy", "r0", "r1", "tx", "ty")},
            }
        )
        baked_cursor += cframes

    assert baked_cursor == EXPECTED_TOTAL_CHILD_FRAMES

    # Verify grandchild FX placements inside #1156, #1205, #1224, and #1249.
    s1156 = symbols["1156"]["placements"]
    assert any(
        p["frame"] == 1 and p["depth"] == 10 and p.get("character_id") == 695 and p.get("name") == "item_br03"
        for p in s1156
    ), "Missing #695 deflected batarang in #1156 frame 1"
    assert any(
        p["frame"] == 9 and p["depth"] == 1 and p.get("character_id") == 1103 for p in s1156
    ), "Missing #1103 recovery placement in #1156 frame 9"

    s1205 = symbols["1205"]
    s1205_places = s1205["placements"]
    for req_frame, req_depth, req_cid in (
        (7, 2, 1184),
        (7, 4, 1185),
        (8, 4, 1187),
        (8, 5, 1189),
        (9, 4, 1190),
    ):
        assert any(
            p["frame"] == req_frame and p["depth"] == req_depth and p.get("character_id") == req_cid
            for p in s1205_places
        ), f"Missing laser FX ({req_frame}, depth={req_depth}, cid={req_cid}) in #1205"

    hitbox_places = [
        p
        for p in s1205_places
        if p["frame"] == 9 and p["depth"] == 10 and p.get("character_id") == 1192 and p.get("name") == "hitBox"
    ]
    assert len(hitbox_places) == 1, hitbox_places
    hb = hitbox_places[0]
    assert hb["cxform"]["mult"] == [256, 256, 256, 0], hb["cxform"]
    assert any(
        r["frame"] == 11 and r["depth"] == 10 for r in s1205.get("removals", [])
    ), "Missing RemoveObject2(depth=10) for #1192 hitBox at frame 11"

    hb_raw = symbol_bounds(1192, 0, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0), symbols, shapes)
    hm = hb["matrix"]
    hb_in_1205 = [
        hb_raw[0] * hm["sx"] + hm["tx"],
        hb_raw[1] * hm["sy"] + hm["ty"],
        hb_raw[2] * hm["sx"] + hm["tx"],
        hb_raw[3] * hm["sy"] + hm["ty"],
    ]
    a2_m = span_audit[6]["child_matrix"]
    hb_in_1250 = [
        hb_in_1205[0] * a2_m["sx"] + a2_m["tx"],
        hb_in_1205[1] * a2_m["sy"] + a2_m["ty"],
        hb_in_1205[2] * a2_m["sx"] + a2_m["tx"],
        hb_in_1205[3] * a2_m["sy"] + a2_m["ty"],
    ]

    s1224 = symbols["1224"]["placements"]
    assert any(
        p["frame"] == 0
        and p["depth"] == 1
        and p.get("character_id") == 1103
        and p.get("cxform") == {"mult": [0, 0, 0, 256], "add": [255, 255, 255, 0]}
        for p in s1224
    ), "Missing white-flash cxform on #1224 frame 0"

    s1249 = symbols["1249"]["placements"]
    for req_depth, req_cid in ((1, 1240), (3, 1242), (5, 1244), (7, 1246), (9, 1248)):
        assert any(
            p["frame"] == 13 and p["depth"] == req_depth and p.get("character_id") == req_cid
            for p in s1249
        ), f"Missing explosion debris #{req_cid} at #1249 frame 13 depth {req_depth}"

    return {
        "symbol_id": BOSS_SYMBOL_ID,
        "definition_tag_code": defs[BOSS_SYMBOL_ID],
        "parent_frames": EXPECTED_PARENT_FRAMES,
        "parent_doaction_frames": actions_by_sid[BOSS_SYMBOL_ID],
        "total_child_frames": EXPECTED_TOTAL_CHILD_FRAMES,
        "spans": span_audit,
        "laser_hitbox": {
            "symbol_id": 1192,
            "parent_child_symbol": 1205,
            "depth": 10,
            "active_local_frames": [9, 10],
            "removed_on_local_frame": 11,
            "cxform": hb["cxform"],
            "matrix_in_1205": hm,
            "raw_bounds": hb_raw,
            "aabb_in_1205": hb_in_1205,
            "unflipped_aabb_in_1250": hb_in_1250,
        },
        "verified_grandchild_fx": {
            "block_1156_deflect_batarang": {"frame": 1, "depth": 10, "symbol_id": 695, "name": "item_br03"},
            "attack2_1205_charge_muzzle_beam": [
                {"frame": 7, "depth": 2, "symbol_id": 1184},
                {"frame": 7, "depth": 4, "symbol_id": 1185},
                {"frame": 8, "depth": 4, "symbol_id": 1187},
                {"frame": 8, "depth": 5, "symbol_id": 1189},
                {"frame": 9, "depth": 4, "symbol_id": 1190},
                {"frame": 9, "depth": 10, "symbol_id": 1192, "name": "hitBox"},
            ],
            "fall_1224_white_flash": {"frame": 0, "depth": 1, "symbol_id": 1103, "cxform_add_rgb": [255, 255, 255]},
            "die_1249_explosion_debris": [
                {"frame": 13, "depth": 1, "symbol_id": 1240},
                {"frame": 13, "depth": 3, "symbol_id": 1242},
                {"frame": 13, "depth": 5, "symbol_id": 1244},
                {"frame": 13, "depth": 7, "symbol_id": 1246},
                {"frame": 13, "depth": 9, "symbol_id": 1248},
            ],
        },
    }


def render_outer_1x_and_hq3x(symbols: dict, shapes: dict, bitmaps: dict, out_dir: Path) -> dict:
    raw = [symbol_bounds(BOSS_SYMBOL_ID, f, (1, 0, 0, 1, 0, 0), symbols, shapes) for f in range(EXPECTED_PARENT_FRAMES)]
    good = [box for box in raw if all(math.isfinite(n) for n in box)]
    assert len(good) == EXPECTED_PARENT_FRAMES, (len(good), EXPECTED_PARENT_FRAMES)
    left = math.floor(min(box[0] for box in good)) - 3
    top = math.floor(min(box[1] for box in good)) - 3
    right = math.ceil(max(box[2] for box in good)) + 3
    bottom = math.ceil(max(box[3] for box in good)) + 3
    w, h = right - left, bottom - top
    assert 50 < w <= 1200 and 50 < h <= 800, (w, h)

    outer_1x_dir = out_dir / "outer_1x"
    outer_hq3x_dir = out_dir / "outer_hq3x"
    outer_1x_dir.mkdir(parents=True, exist_ok=True)
    outer_hq3x_dir.mkdir(parents=True, exist_ok=True)

    label_starts = {s[1]: s[0] for s in EXPECTED_SPANS}
    frames_1x = []
    frames_hq3x = []
    non_empty_1x = 0
    non_empty_hq3x = 0

    for frame in range(EXPECTED_PARENT_FRAMES):
        canvas = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        missing: set[int] = set()
        unsupported: set[str] = set()
        render_symbol(
            BOSS_SYMBOL_ID,
            frame,
            (1.0, 0.0, 0.0, 1.0, -float(left), -float(top)),
            canvas,
            symbols,
            shapes,
            bitmaps,
            unsupported,
            missing,
        )
        assert not missing and not unsupported, (frame, missing, unsupported)
        png_1x = outer_1x_dir / f"{frame:03}.png"
        canvas.save(png_1x, optimize=True)
        bbox = canvas.getchannel("A").getbbox()
        if bbox is None:
            raise AssertionError(f"Empty outer 1x frame {frame}")
        non_empty_1x += 1
        label = next(lbl for lbl, start, *_ in reversed(EXPECTED_SPANS) if frame >= start)
        frames_1x.append(
            {
                "frame": frame,
                "label": label,
                "file": str(png_1x.relative_to(out_dir)).replace("\\", "/"),
                "bbox": list(bbox),
                "png_sha256": sha256_bytes(png_1x.read_bytes()),
                "rgba_sha256": sha256_bytes(canvas.tobytes()),
            }
        )

        if frame in label_starts:
            margin = 2
            hi = Image.new("RGBA", ((w + margin * 2) * 6, (h + margin * 2) * 6), (0, 0, 0, 0))
            render_symbol(
                BOSS_SYMBOL_ID,
                frame,
                (6.0, 0.0, 0.0, 6.0, -float(left - margin) * 6, -float(top - margin) * 6),
                hi,
                symbols,
                shapes,
                bitmaps,
                unsupported,
                missing,
            )
            assert not missing and not unsupported, (frame, "HQ3x", missing, unsupported)
            hi = (
                hi.convert("RGBa")
                .resize(((w + margin * 2) * 3, (h + margin * 2) * 3), Image.Resampling.LANCZOS)
                .convert("RGBA")
            )
            px = margin * 3
            hi = hi.crop((px, px, px + w * 3, px + h * 3))
            assert hi.size == (w * 3, h * 3)
            hq_bbox = hi.getchannel("A").getbbox()
            if hq_bbox is None:
                raise AssertionError(f"Empty outer HQ3x frame {frame}")
            non_empty_hq3x += 1
            png_hq = outer_hq3x_dir / f"{frame:03}.png"
            hi.save(png_hq, optimize=True)
            frames_hq3x.append(
                {
                    "frame": frame,
                    "label": label_starts[frame],
                    "file": str(png_hq.relative_to(out_dir)).replace("\\", "/"),
                    "size": list(hi.size),
                    "bbox": list(hq_bbox),
                    "png_sha256": sha256_bytes(png_hq.read_bytes()),
                    "rgba_sha256": sha256_bytes(hi.tobytes()),
                }
            )

    return {
        "outer_1x": {
            "frame_count": len(frames_1x),
            "non_empty_frames": non_empty_1x,
            "canvas_size": [w, h],
            "anchor_in_bitmap": [-float(left), -float(top)],
            "raw_source_bounds_union": [left, top, right, bottom],
            "frames": frames_1x,
        },
        "outer_hq3x": {
            "frame_count": len(frames_hq3x),
            "non_empty_frames": non_empty_hq3x,
            "canvas_size": [w * 3, h * 3],
            "frames": frames_hq3x,
        },
    }


def bake_true_child_sequences(swf_path: Path, symbols_path: Path, bitmaps_dir: Path, out_dir: Path) -> dict:
    child_dir = out_dir / "true_child_1x"
    cmd = [
        sys.executable,
        str(Path(__file__).resolve().parent / "bake_player_states.py"),
        str(swf_path),
        str(symbols_path),
        str(child_dir),
        "--bitmaps",
        str(bitmaps_dir),
        "--symbol",
        str(BOSS_SYMBOL_ID),
        "--scale",
        "1",
        "--output-scale",
        "1",
    ]
    subprocess.run(cmd, check=True)
    manifest = json.loads((child_dir / "manifest.json").read_text(encoding="utf-8"))
    assert manifest["outer_symbol"] == BOSS_SYMBOL_ID
    assert manifest["frame_count"] == EXPECTED_TOTAL_CHILD_FRAMES
    assert manifest["size"] == [938, 410], manifest["size"]
    assert not manifest["unsupported_fills"], manifest["unsupported_fills"]
    assert not manifest["missing_bitmap_ids"], manifest["missing_bitmap_ids"]

    non_empty_count = 0
    frame_records = []
    for entry in manifest["frames"]:
        idx = entry["frame"]
        fpath = child_dir / entry["file"]
        with Image.open(fpath) as img:
            rgba = img.convert("RGBA")
            bbox = rgba.getchannel("A").getbbox()
            non_empty = bbox is not None
            if non_empty:
                non_empty_count += 1
            elif not (entry["animation"] == "die" and entry["local_frame"] >= 38):
                # Only #1249 fade-out frames 38..39 have cxform alpha=0.
                raise AssertionError(f"Unexpected transparent true-child frame: {entry}")
            frame_records.append(
                {
                    "frame": idx,
                    "animation": entry["animation"],
                    "local_frame": entry["local_frame"],
                    "child_symbol": entry["child_symbol"],
                    "file": f"true_child_1x/{entry['file']}",
                    "non_empty": non_empty,
                    "bbox": list(bbox) if bbox else None,
                    "png_sha256": sha256_bytes(fpath.read_bytes()),
                    "rgba_sha256": sha256_bytes(rgba.tobytes()),
                }
            )

    return {
        "frame_count": manifest["frame_count"],
        "non_empty_frames": non_empty_count,
        "size": manifest["size"],
        "anchor_in_bitmap": manifest["anchor_in_bitmap"],
        "source_bounds": manifest["source_bounds"],
        "states": manifest["states"],
        "unsupported_fills": manifest["unsupported_fills"],
        "missing_bitmap_ids": manifest["missing_bitmap_ids"],
        "manifest_sha256": sha256_bytes((child_dir / "manifest.json").read_bytes()),
        "frames": frame_records,
    }


def build_contact_sheet(out_dir: Path, outer_hq3x: dict, child_1x: dict) -> dict:
    # Row 0 & Row 1 (5 columns x 2 rows): the 10 HQ3x label-start views.
    # Row 2 (5 columns): 5 key nested FX frames from true_child_1x.
    cols = 5
    rows = 3
    cell_w = 340
    cell_h = 200
    pad = 12
    sheet = Image.new("RGB", (cols * cell_w, rows * cell_h + 36), "#1d2430")
    draw = ImageDraw.Draw(sheet)

    for i, item in enumerate(outer_hq3x["frames"]):
        r = i // cols
        c = i % cols
        with Image.open(out_dir / item["file"]) as src:
            thumb = src.convert("RGBA")
        thumb.thumbnail((cell_w - 20, cell_h - 38), Image.Resampling.LANCZOS)
        x0 = c * cell_w + (cell_w - thumb.width) // 2
        y0 = r * cell_h + 26 + (cell_h - 38 - thumb.height) // 2
        sheet.paste(thumb, (x0, y0), thumb)
        draw.text(
            (c * cell_w + pad, r * cell_h + 6),
            f"HQ3x {item['label']} (p{item['frame']})",
            fill="#ffffff",
        )

    fx_picks = (
        (49, "FX: block f1 item_br03 #695"),
        (67, "FX: attack f5 peak #1172"),
        (81, "FX: attack2 f9 laser #1205+#1192"),
        (101, "FX: fall f0 white-flash #1224"),
        (135, "FX: die f13 explosion #1249"),
    )
    for c, (baked_idx, title) in enumerate(fx_picks):
        f_info = child_1x["frames"][baked_idx]
        with Image.open(out_dir / f_info["file"]) as src:
            thumb = src.convert("RGBA")
        thumb.thumbnail((cell_w - 20, cell_h - 38), Image.Resampling.LANCZOS)
        x0 = c * cell_w + (cell_w - thumb.width) // 2
        y0 = 2 * cell_h + 26 + (cell_h - 38 - thumb.height) // 2
        sheet.paste(thumb, (x0, y0), thumb)
        draw.text((c * cell_w + pad, 2 * cell_h + 6), title, fill="#7ee0ff")

    draw.text(
        (pad, rows * cell_h + 10),
        "B15 PENGUIN #1250 SOURCE PROOF: 99 outer frames + 10 HQ3x label starts + 162 true child frames & attack FX",
        fill="#ffd166",
    )
    sheet_path = out_dir / "B15_PENGUIN_SPRITES_CONTACT_SHEET.png"
    sheet.save(sheet_path, optimize=True)
    return {
        "file": sheet_path.name,
        "size": list(sheet.size),
        "png_sha256": sha256_bytes(sheet_path.read_bytes()),
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--swf", type=Path, required=True)
    ap.add_argument("--symbols", type=Path, required=True)
    ap.add_argument("--bitmaps", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()

    args.out.mkdir(parents=True, exist_ok=True)
    source_sha = expect_original_source(args.swf)
    if source_sha != SOURCE_SHA:
        raise AssertionError(f"Canonical FWS SHA mismatch: {source_sha} != {SOURCE_SHA}")

    symbols = json.loads(args.symbols.read_text(encoding="utf-8"))
    unsupported_shapes: set[str] = set()
    shapes = load_shapes(args.swf, unsupported_shapes)
    if unsupported_shapes:
        raise AssertionError(f"Unsupported shape fill kinds: {sorted(unsupported_shapes)}")
    bitmaps = load_bitmaps(args.bitmaps)

    source_audit = verify_source_timeline_and_fx(args.swf, symbols, shapes)
    outer_result = render_outer_1x_and_hq3x(symbols, shapes, bitmaps, args.out)
    child_result = bake_true_child_sequences(args.swf, args.symbols, args.bitmaps, args.out)
    sheet = build_contact_sheet(args.out, outer_result["outer_hq3x"], child_result)

    report = {
        "batch": "LEVEL2-B15",
        "canonical_source_fws_sha256": source_sha,
        "source_audit": source_audit,
        "outer_1x": outer_result["outer_1x"],
        "outer_hq3x": outer_result["outer_hq3x"],
        "true_child_1x": child_result,
        "contact_sheet": sheet,
        "dormant_source_metadata_only": True,
        "native_level1a_and_1b_goldens_unchanged": True,
    }
    report_path = args.out / "B15_PENGUIN_SPRITES_SOURCE_REPORT.json"
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(
        "B15_PENGUIN_SPRITES_GATE_PASS",
        source_sha,
        f"outer_1x={outer_result['outer_1x']['non_empty_frames']}/{outer_result['outer_1x']['frame_count']}",
        f"outer_hq3x={outer_result['outer_hq3x']['non_empty_frames']}/{outer_result['outer_hq3x']['frame_count']}",
        f"true_child_1x={child_result['non_empty_frames']}/{child_result['frame_count']}",
        f"child_canvas={child_result['size']}",
        flush=True,
    )


if __name__ == "__main__":
    main()
