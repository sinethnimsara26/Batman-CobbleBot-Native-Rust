#!/usr/bin/env python3
"""Shared offline Labs stage scenery, mask, HQ3x proof & object-separation helpers."""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import struct
from pathlib import Path
from PIL import Image

from bake_sprite_frames import render_symbol

CANONICAL_SWF_SHA256 = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
SOURCE_BACKDROP_ID = 14


@dataclass(frozen=True)
class LabsStageSpec:
    batch: str
    level_key: str
    root_frame: int
    game_id: int
    level_id: int
    ground_id: int
    level_origin: tuple[float, float]
    expected_player_spawn: tuple[float, float]
    expected_ground_origin: tuple[int, int]
    expected_ground_size: tuple[int, int]
    expected_ground_pixels: int
    expected_1x_tiles: int
    sample_coords: tuple[tuple[str, int, int], ...]
    checkpoint_prefix: str
    expected_counts: dict[str, int]
    open_check_pts: tuple[tuple[float, float], ...]
    solid_check_pts: tuple[tuple[float, float], ...]


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_root_backdrop_black(symbols: dict, shapes: dict, bitmaps: dict) -> dict:
    """Verify #14 solid opaque black backdrop (no #134 cityscape) at 600x400."""
    im = Image.new("RGBA", (600, 400), (255, 0, 255, 255))
    unsup: set[str] = set()
    missing: set[int] = set()
    m14 = (0.9918975830078125, 0.0, 0.0, 3.38983154296875, 300.0, 200.05)
    render_symbol(SOURCE_BACKDROP_ID, 0, m14, im, symbols, shapes, bitmaps, unsup, missing)
    assert not unsup and not missing, (unsup, missing)
    for pt in ((0, 0), (300, 200), (599, 399)):
        px = im.getpixel(pt)
        assert px == (0, 0, 0, 255), f"expected #14 solid black at {pt}, got {px}"
    return {
        "symbol_id": SOURCE_BACKDROP_ID,
        "matrix": [round(v, 6) for v in m14],
        "verified_opaque_black": True,
    }


def verify_ground_mask(spec: LabsStageSpec, manifest_path: Path, mask_path: Path) -> dict:
    records = json.loads(manifest_path.read_text(encoding="utf-8"))
    rec = [r for r in records if r.get("level") == spec.level_key]
    assert len(rec) == 1, f"expected one {spec.level_key} ground record, got {rec}"
    r = rec[0]
    assert r.get("ground_character_id") == spec.ground_id, r
    assert r.get("origin_level_px") == list(spec.expected_ground_origin), r
    assert r.get("size_px") == list(spec.expected_ground_size), r
    assert r.get("occupied_pixels") == spec.expected_ground_pixels, r
    with Image.open(mask_path) as src:
        assert src.size == spec.expected_ground_size, src.size
        assert src.mode == "1", src.mode
        pix = src.convert("L").load()
    ox, oy = spec.expected_ground_origin
    w, h = spec.expected_ground_size

    def solid(gx: float, gy: float) -> bool:
        lx = int(gx - spec.level_origin[0]) - ox
        ly = int(gy - spec.level_origin[1]) - oy
        return 0 <= lx < w and 0 <= ly < h and pix[lx, ly] >= 128

    for pt in spec.open_check_pts:
        assert not solid(*pt), f"{spec.level_key}: point {pt} must be open air"
    for pt in spec.solid_check_pts:
        assert solid(*pt), f"{spec.level_key}: point {pt} must be solid"
    return {
        "ground_character_id": spec.ground_id,
        "origin_level_px": list(spec.expected_ground_origin),
        "size_px": list(spec.expected_ground_size),
        "occupied_pixels": spec.expected_ground_pixels,
        "mask_png_sha256": sha256_bytes(mask_path.read_bytes()),
    }


def render_hq3x_comparisons(
    spec: LabsStageSpec,
    symbols: dict,
    shapes: dict,
    bitmaps: dict,
    tiles_dir: Path,
    tiles_bin: Path,
    outdir: Path,
) -> list[dict]:
    manifest = json.loads((tiles_dir / "manifest.json").read_text(encoding="utf-8"))
    assert manifest.get("symbol") == spec.level_id, manifest
    assert len(manifest.get("tiles", [])) == spec.expected_1x_tiles, len(manifest.get("tiles", []))
    assert not manifest.get("unsupported_fills"), manifest
    assert not manifest.get("missing_symbols"), manifest
    raw_bin = tiles_bin.read_bytes()
    assert raw_bin[:8] == b"BCLVT001", raw_bin[:8]
    assert struct.unpack_from("<I", raw_bin, 8)[0] == spec.expected_1x_tiles

    by_coord = {(t["world_x"], t["world_y"]): t for t in manifest["tiles"]}
    results = []
    sheet_1x_panels = []
    sheet_hq_panels = []
    unsup: set[str] = set()
    missing: set[int] = set()
    excluded = {spec.ground_id}
    prefix = spec.level_key.replace("level", "labs")

    for label, wx, wy in spec.sample_coords:
        t = by_coord[(wx, wy)]
        with Image.open(tiles_dir / t["file"]) as t_img:
            tile_1x = t_img.convert("RGBA")
        comp_1x = Image.new("RGBA", (600, 400), (0, 0, 0, 255))
        comp_1x.alpha_composite(tile_1x)
        comp_1x.save(outdir / f"{prefix}_{label}_1x.png")

        render_scale = 6
        margin = 2
        lw, lh = 600 + margin * 2, 400 + margin * 2
        super_canvas = Image.new("RGBA", (lw * render_scale, lh * render_scale), (0, 0, 0, 0))
        mat = (
            float(render_scale), 0.0, 0.0, float(render_scale),
            (spec.level_origin[0] - (wx - margin)) * render_scale,
            (spec.level_origin[1] - (wy - margin)) * render_scale,
        )
        render_symbol(
            spec.level_id, 0, mat, super_canvas, symbols, shapes, bitmaps,
            unsup, missing, excluded_symbols=excluded,
        )
        assert not unsup and not missing, (unsup, missing)
        cropped = super_canvas.crop((
            margin * render_scale, margin * render_scale,
            (margin + 600) * render_scale, (margin + 400) * render_scale,
        ))
        hq_fg = cropped.convert("RGBa").resize((1800, 1200), Image.Resampling.LANCZOS).convert("RGBA")
        comp_hq = Image.new("RGBA", (1800, 1200), (0, 0, 0, 255))
        comp_hq.alpha_composite(hq_fg)
        comp_hq.save(outdir / f"{prefix}_{label}_hq3x.png")

        px_iter = comp_1x.get_flattened_data() if hasattr(comp_1x, "get_flattened_data") else comp_1x.getdata()
        non_black = sum(1 for r, g, b, _ in px_iter if (r | g | b) > 0)
        assert non_black > 600 * 400 // 20, f"sample {label} unexpectedly black: {non_black}"
        results.append({
            "label": label,
            "world_origin": [wx, wy],
            "non_black_pixels_1x": non_black,
            "sha256_1x_rgba": sha256_bytes(comp_1x.tobytes()),
            "sha256_hq3x_rgba": sha256_bytes(comp_hq.tobytes()),
        })
        sheet_1x_panels.append(comp_1x)
        sheet_hq_panels.append(comp_hq.resize((600, 400), Image.Resampling.LANCZOS))

    sheet = Image.new("RGBA", (1200, 400 * len(spec.sample_coords)), (0, 0, 0, 255))
    for i, (p1, phq) in enumerate(zip(sheet_1x_panels, sheet_hq_panels)):
        sheet.paste(p1, (0, i * 400))
        sheet.paste(phq, (600, i * 400))
    sheet.save(outdir / f"{prefix}_multifloor_1x_vs_hq3x.png")
    return results


def separate_game_objects(spec: LabsStageSpec, symbols: dict) -> dict:
    placements = symbols[str(spec.game_id)]["placements"]
    doors = []
    elevators = []
    checkpoints = []
    cameras = []
    goons = []
    eagles = []
    health_pickups = []
    batarang_pickups = []
    door_jambs = []
    lavas = []
    static_accents = []
    player = None
    halo = None

    for p in placements:
        cid = p.get("character_id")
        depth = p.get("depth", 0)
        name = p.get("name") or ""
        m = p.get("matrix", {})
        tx = float(m.get("tx", 0.0))
        ty = float(m.get("ty", 0.0))
        sx = float(m.get("sx", 1.0))
        sy = float(m.get("sy", 1.0))
        entry = {"name": name, "symbol_id": cid, "depth": depth, "x": tx, "y": ty, "sx": sx, "sy": sy}
        if cid == spec.level_id or cid in (694, 1009, 1074):
            continue
        if name == "player" or cid == 691:
            player = entry
        elif name == "halo" or cid in (1399, 1495):
            halo = entry
        elif name.startswith("door"):
            doors.append(entry)
        elif name.startswith("elevator"):
            entry["limit_y"] = int(name[12:])
            elevators.append(entry)
        elif name.startswith(spec.checkpoint_prefix):
            checkpoints.append(entry)
        elif name.startswith("camera_"):
            cameras.append(entry)
        elif name.startswith("goon"):
            goons.append(entry)
        elif name.startswith("eagle"):
            eagles.append(entry)
        elif name.startswith("health"):
            health_pickups.append(entry)
        elif name.startswith("item_br"):
            batarang_pickups.append(entry)
        elif name.startswith("lava") or cid == 1486:
            lavas.append(entry)
        elif cid in (1394, 1395, 1396):
            door_jambs.append(entry)
        else:
            static_accents.append(entry)

    groups = {
        "doors": doors,
        "elevators": elevators,
        "checkpoints": checkpoints,
        "cameras": cameras,
        "goons": goons,
        "eagles": eagles,
        "health_pickups": health_pickups,
        "batarang_pickups": batarang_pickups,
        "door_jambs": door_jambs,
        "lavas": lavas,
        "static_accents": static_accents,
    }
    for grp in groups.values():
        grp.sort(key=lambda item: (item["depth"], item["name"]))

    for k, expected in spec.expected_counts.items():
        actual = len(groups[k])
        assert actual == expected, f"{spec.level_key} {k}: expected {expected}, got {actual}"
    assert player and (round(player["x"], 2), round(player["y"], 2)) == spec.expected_player_spawn, player
    assert halo and halo["symbol_id"] in (1399, 1495), halo

    return {
        "player": player,
        "halo": halo,
        **groups,
    }


def emit_labs_rust_objects(objs: dict, header_comment: str, rust_out: Path, include_lavas: bool = False) -> None:
    lines = [f"// {header_comment}", ""]

    def emit_list(name: str, ty: str, items: list[dict], kind: str) -> None:
        lines.append(f"pub const {name}: [{ty}; {len(items)}] = [")
        for it in items:
            if kind == "door":
                lines.append(f"    {ty} {{ name: {it['name']!r}, symbol_id: {it['symbol_id']}, depth: {it['depth']}, x: {it['x']:.4f}, y: {it['y']:.4f}, sx: {it['sx']:.6f}, sy: {it['sy']:.6f} }},")
            elif kind == "elevator":
                lines.append(f"    {ty} {{ name: {it['name']!r}, symbol_id: {it['symbol_id']}, depth: {it['depth']}, x: {it['x']:.4f}, y: {it['y']:.4f}, sx: {it['sx']:.6f}, limit_y: {it['limit_y']} }},")
            elif kind in ("trigger", "lava"):
                lines.append(f"    {ty} {{ name: {it['name']!r}, symbol_id: {it['symbol_id']}, depth: {it['depth']}, x: {it['x']:.4f}, y: {it['y']:.4f}, sx: {it['sx']:.6f}, sy: {it['sy']:.6f} }},")
            elif kind in ("enemy", "pickup"):
                lines.append(f"    {ty} {{ name: {it['name']!r}, symbol_id: {it['symbol_id']}, depth: {it['depth']}, x: {it['x']:.4f}, y: {it['y']:.4f}, sx: {it['sx']:.6f} }},")
        lines.append("];")
        lines.append("")

    emit_list("DOORS", "LabsDoorPlacement", objs["doors"], "door")
    emit_list("ELEVATORS", "LabsElevatorPlacement", objs["elevators"], "elevator")
    emit_list("CHECKPOINTS", "LabsTriggerPlacement", objs["checkpoints"], "trigger")
    emit_list("CAMERAS", "LabsTriggerPlacement", objs["cameras"], "trigger")
    emit_list("GOONS", "LabsEnemyPlacement", objs["goons"], "enemy")
    emit_list("EAGLES", "LabsEnemyPlacement", objs["eagles"], "enemy")
    emit_list("HEALTH_PICKUPS", "LabsPickupPlacement", objs["health_pickups"], "pickup")
    emit_list("BATARANG_PICKUPS", "LabsPickupPlacement", objs["batarang_pickups"], "pickup")
    emit_list("DOOR_JAMBS", "LabsDoorPlacement", objs["door_jambs"], "door")
    if include_lavas:
        emit_list("LAVAS", "LabsLavaPlacement", objs["lavas"], "lava")

    rust_out.write_text("\n".join(lines).replace("'", '"'), encoding="utf-8")
