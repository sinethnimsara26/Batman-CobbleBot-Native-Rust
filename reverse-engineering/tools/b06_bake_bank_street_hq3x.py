#!/usr/bin/env python3
"""B06: original source Bank Street #783 3x HQ stills and authored-depth proofs.

Development-only source extraction, not playable native Level1B gameplay.
The 1x artifact is re-baked and checked against ALL nine B05 original pixels.
Every 3x image is re-rasterized from canonical FWS at 6x before Lanczos to 3x.
Static depth bands use authored source placement.depth, not invented parallax.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys
import time

from PIL import Image, ImageChops, ImageDraw

from b03_bake_stage_asset_proofs import (
    TILE_H, TILE_W, expect_original_source, point_candidates, static_filter,
    validate_stage,
)
from b05_bake_bank_street_1x import (
    read_json, source_depth_audit, select_source_views, source_definition_tags,
)
from bake_sprite_frames import display_list, load_shapes

# Independently reviewed B05 Windows artifact #13858908870, original
# decoded RGBA bytes, not PNG encoder-dependent file checks.
B05_RGBA = {
    (-9000, -400): "47cde9bb7eed1040078dc98db5383808da613f89bf4bc596c9961247f0d5c7d2",
    (-7200, 0): "eeafa403fdf44df0623568c9f3cf14e20e915416b03f3b5205be423a92595386",
    (-5400, 0): "3739d7cdb6f4771f72d5e6500cb330914f311ba12d20a7175e639afa76965410",
    (-4200, 0): "dc64aa08b35f4b9bd774ca85274b0de19970ab596ca8762e02bd660fdfa8375c",
    (-2400, 0): "2fab5d45ac3b42dd6ea304864c5774fee8740d47b04b5de2b298dcfa6f05ec8a",
    (-600, 0): "c24285e35970d640ddb3bdeaa4f0906d8a3760996d86714b8eb94144653b7617",
    (0, 0): "896f7373e114393a233ecd279dd8b143955f920913c8dde26013ebee32e59a6c",
    (600, 0): "2381074e5468f5c0e7fd8326898fb2d8bb907981467433936046e47649e8c8c6",
    (1200, 0): "7b092ab245686020dab3e574596bb16588389a04400eab5cc737f22d26f74914",
}
VIEWS = tuple(sorted(B05_RGBA, key=lambda p: (p[1], p[0])))
DEPTH_VIEWPOINTS = ((-4200, 0), (0, 0))
SOURCE_SHA = "3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"

def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def image_file_sha(path: Path) -> str:
    return digest(path.read_bytes())

def raster_hash(path: Path, size: tuple[int, int]) -> str:
    with Image.open(path) as source:
        rgba = source.convert("RGBA")
        if rgba.size != size:
            raise AssertionError(f"Unexpected original-source raster size {path}: {rgba.size} != {size}")
        if rgba.getchannel("A").getbbox() is None:
            raise AssertionError(f"Unexpected empty original-source tile {path}")
        return digest(rgba.tobytes())

def source_depth_split(stage, symbols, excluded):
    # Layer membership is from raw SWF depth, never heuristically guessed.
    display = sorted(display_list(symbols[str(stage["level_symbol_id"])], 0).values(),
                     key=lambda p: p.get("depth", 0))
    kept = [p for p in display if p.get("character_id") is not None
            and p["character_id"] not in excluded]
    depths = sorted({int(p.get("depth", 0)) for p in kept})
    if len(depths) < 4:
        raise AssertionError("Original static source has insufficient distinct depth bands")
    clipped = [(int(p.get("depth", 0)), int(p["clip_depth"]))
               for p in display if p.get("clip_depth") is not None]
    splits = [d for d in depths[1:-1]
              if all(not (start <= d < end) for start, end in clipped)]
    if not splits:
        raise AssertionError(f"No safe original display-list depth split: {clipped}")
    center = len(kept) // 2
    cut = min(splits, key=lambda d: (abs(sum(p["depth"] <= d for p in kept)-center), d))
    low = [p for p in kept if p["depth"] <= cut]
    high = [p for p in kept if p["depth"] > cut]
    if not low or not high or any(start <= cut < end for start, end in clipped):
        raise AssertionError("Unsafe original depth split, crossed clipping mask")
    # Metadata records actual source IDs, no story/actor behind-layer guess.
    return {
        "original_direct_display_depth_cut": cut,
        "lower_band": [min(depths), cut],
        "upper_band": [cut+1, max(depths)],
        "original_direct_depths": depths,
        "source_clip_depth_spans": [list(x) for x in clipped],
        "lower_direct_source_placements": len(low),
        "upper_direct_source_placements": len(high),
        "lower_character_ids": sorted({p["character_id"] for p in low}),
        "upper_character_ids": sorted({p["character_id"] for p in high}),
        "selection_basis": "SWF #783 frame0 authored direct placement depths; child timelines preserved",
    }

def run_bake(args, out: Path, origins, *, scale=1, hq=False, depth_range=None):
    tool = Path(__file__).with_name("bake_level_tiles.py")
    command = [sys.executable, str(tool), str(args.swf), str(args.symbols),
               str(out), "--bitmaps", str(args.bitmaps), "--symbol", "783",
               "--place-x", "1034.65", "--place-y", "239.25",
               "--output-scale", str(scale), "--include-source-static-text"]
    if hq:
        command.extend(("--supersample", "2", "--downsample", "lanczos"))
    if depth_range is not None:
        command.extend(("--root-depth-range", *(str(d) for d in depth_range)))
    for cid in args.excludes:
        command.extend(("--exclude-symbol", str(cid)))
    for x, y in origins:
        command.extend(("--only-tile", str(x), str(y)))
    print("B06_SOURCE_EXEC", " ".join(command), flush=True)
    started = time.perf_counter()
    subprocess.run(command, check=True)
    elapsed = time.perf_counter() - started
    manifest = read_json(out / "manifest.json")
    if (manifest["symbol"] != 783 or manifest["placement"] != [1034.65, 239.25]
            or manifest["excluded_symbols"] != args.excludes
            or manifest["source_static_text_enabled"] is not True
            or manifest.get("partial_proof_only") is not True
            or manifest.get("never_embed_as_complete_level") is not True):
        raise AssertionError("Original Bank source/partial-pack manifest contract violated")
    if manifest["requested_tile_origins"] != [list(x) for x in sorted(origins, key=lambda p:(p[1],p[0]))]:
        raise AssertionError("Original fixed world positions shifted")
    if (manifest["logical_pixel_scale"] != scale or
            manifest["tile_size"] != [TILE_W*scale, TILE_H*scale] or
            manifest["source_root_depth_range"] != (list(depth_range) if depth_range else None)):
        raise AssertionError("Source 1x/HQ/depth scale metadata mismatch")
    if manifest["missing_ids"] or manifest["unsupported_fills"]:
        raise AssertionError(f"Missing/unsupported canonical source art: {manifest['missing_ids']} / {manifest['unsupported_fills']}")
    return manifest, elapsed

def index_tiles(manifest, out, size):
    result = {}
    for t in manifest["tiles"]:
        pos = (t["world_x"], t["world_y"])
        if pos in result:
            raise AssertionError(f"Duplicate original source tile coordinate {pos}")
        path = out / t["file"]
        result[pos] = {
            "world_x": pos[0], "world_y": pos[1],
            "file": t["file"], "png_sha256": image_file_sha(path),
            "rgba_sha256": raster_hash(path, size),
            "size": list(size),
        }
    return result

def ensure_not_packable(out, attempt):
    target = out.parent / attempt
    call = subprocess.run(
        [sys.executable, str(Path(__file__).with_name("pack_level_tiles.py")),
         str(out), str(target)],
        capture_output=True, text=True, check=False)
    if (call.returncode == 0 or target.exists() or
            "partial B03 source proof tiles cannot be packed" not in call.stderr):
        raise AssertionError(f"Incomplete B06 tiles wrongly packable: {out}")
    print("B06_PARTIAL_STAGE_PACK_GUARD_PASS", out, flush=True)

def contact_sheet(out, reference, hq, samples):
    w, rh = 1240, 456
    sheet = Image.new("RGB", (w, len(VIEWS)*rh+32), "#242a37")
    draw = ImageDraw.Draw(sheet)
    for i, origin in enumerate(VIEWS):
        y = i*rh
        source1 = reference / samples[origin]["file"]
        target = hq / samples[origin]["hq_file"]
        with Image.open(source1) as source, Image.open(target) as high:
            orig = Image.new("RGBA", (TILE_W,TILE_H), "#424853")
            orig.alpha_composite(source.convert("RGBA"))
            resized = high.convert("RGBA").resize((TILE_W,TILE_H), Image.Resampling.LANCZOS)
            rev = Image.new("RGBA", (TILE_W,TILE_H), "#424853")
            rev.alpha_composite(resized)
        sheet.paste(orig.convert("RGB"),(6,y+32))
        sheet.paste(rev.convert("RGB"),(626,y+32))
        draw.text((8,y+8),f"Original Flash #783 1x | {origin}",fill="white")
        draw.text((628,y+8),"SOURCE vector 6x -> true 3x Lanczos (preview at 1x)",fill="white")
        draw.text((8,y+438),"DEVELOPMENT SOURCE ART - NOT a played original Flash or native Level1B",fill="#ffd180")
    draw.text((8,len(VIEWS)*rh+5),"Original source stills ONLY | Level1A/gameplay unchanged | partial NON-PACKABLE",fill="#ffd180")
    name = "B06_ORIGINAL_BANK_STREET_HQ3X_SOURCE_CONTACT_SHEET.png"
    sheet.save(out/name,optimize=True)
    return {"file":name,"sha256":image_file_sha(out/name),"size":list(sheet.size)}

def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ("swf","symbols","bitmaps","instances","spec","out"):
        p.add_argument("--"+name,required=True,type=Path)
    args = p.parse_args()
    sha = expect_original_source(args.swf)
    if sha != SOURCE_SHA:
        raise AssertionError("Noncanonical FWS")
    symbols = read_json(args.symbols)
    stage = read_json(args.instances)["level1b"]
    spec = read_json(args.spec)["level1b"]
    if (stage["root_frame"],stage["game_symbol_id"],stage["level_symbol_id"],stage["ground"]["character_id"]) != (20,1075,783,782):
        raise AssertionError("Wrong original Bank source root20 game#1075 street#783 ground#782")
    if validate_stage("level1b",spec,stage,symbols) != (1034.65,239.25):
        raise AssertionError("Original source level placement changed")
    source_excludes = sorted(set(static_filter(stage)) | {758,771})
    if any(x not in source_excludes for x in (153,782,758,771)):
        raise AssertionError("Source collision, checkpoint or button leaked into street")
    args.excludes = source_excludes
    ownership = source_depth_audit(stage,symbols,set(source_excludes))
    original_tags=source_definition_tags(args.swf,{758,771,774})
    if original_tags!={758:34,771:34,774:11}:
        raise AssertionError("Raw Flash buttons and text changed definition identity")
    shapes = load_shapes(args.swf,set())
    locations, bounds, _ = point_candidates(stage,(1034.65,239.25),symbols,shapes,source_excludes)
    original_views, x_columns = select_source_views(locations)
    if sorted(original_views,key=lambda p:(p[1],p[0]))!=list(VIEWS):
        raise AssertionError(f"Source geometry-derived views no longer match B05: {original_views}")
    split=source_depth_split(stage,symbols,set(source_excludes))
    args.out.mkdir(parents=True,exist_ok=True)

    # 1x is a HARD BASELINE: all nine B05 original decoded pixels must match.
    ref_dir = args.out/"source_1x_baseline_recheck"
    m1,t1=run_bake(args,ref_dir,VIEWS,scale=1)
    reference=index_tiles(m1,ref_dir,(TILE_W,TILE_H))
    if set(reference)!=set(B05_RGBA):
        raise AssertionError("Lost original B05 source views in B06")
    for xy,expected in B05_RGBA.items():
        if reference[xy]["rgba_sha256"]!=expected:
            raise AssertionError(f"B05 source original 1x pixel drift at {xy}")
    print("B06_B05_ORIGINAL_1X_9_OF_9_PIXEL_HASHES_PASS",flush=True)
    ensure_not_packable(ref_dir,"DANGER_B06_1X_MUST_NOT_PACK.bin")

    # True 3x original source rerasterization with 6x intermediates.
    hq_dir = args.out/"source_hq3x"
    m3,t3=run_bake(args,hq_dir,VIEWS,scale=3,hq=True)
    hq=index_tiles(m3,hq_dir,(TILE_W*3,TILE_H*3))
    if set(hq)!=set(reference):
        raise AssertionError("HQ source tile origins do not match original 1x")
    if (m3["supersample"],m3["downsample"],m3["alpha_filtering"])!=(2,"lanczos","premultiplied"):
        raise AssertionError("HQ source proof not independently rasterized using premultiplied overscan")
    ensure_not_packable(hq_dir,"DANGER_B06_HQ_MUST_NOT_PACK.bin")
    independent_hq=0
    samples={}
    for xy in VIEWS:
        old=ref_dir/reference[xy]["file"]
        new=hq_dir/hq[xy]["file"]
        with Image.open(old) as a, Image.open(new) as b:
            bilinear = a.convert("RGBA").resize((1800,1200),Image.Resampling.BILINEAR)
            if bilinear.tobytes()!=b.convert("RGBA").tobytes():
                independent_hq+=1
        samples[xy]={**reference[xy],
                     "hq_file":hq[xy]["file"],
                     "hq_png_sha256":hq[xy]["png_sha256"],
                     "hq_rgba_sha256":hq[xy]["rgba_sha256"]}
    if independent_hq<1:
        raise AssertionError("HQ images appear to be mere bilinear upscale of source 1x")
    print("B06_HQ_TRUE_SOURCE_RERASTER_PASS",len(hq),"distinct from bilinear",independent_hq,flush=True)

    # Source-authoritative direct depth bands, only for bounded proof views.
    # Render at 3x WITHOUT post-filter for a faithful compositing experiment;
    # it is independent of the nine HQ final Lanczos tiles.
    parts=[]
    layer_dir=args.out/"authored_depth_proof"
    low=tuple(split["lower_band"]); high=tuple(split["upper_band"])
    for label,band in (("lower",low),("upper",high)):
        sub=layer_dir/label
        manifest,elapsed=run_bake(args,sub,DEPTH_VIEWPOINTS,scale=3,depth_range=band)
        ensure_not_packable(sub,"DANGER_B06_"+label+"_MUST_NOT_PACK.bin")
        entries=index_tiles(manifest,sub,(1800,1200))
        parts.append((label,entries,elapsed))
    plain_dir=layer_dir/"flat_3x_comparison"
    flat_manifest,elapsed_flat=run_bake(args,plain_dir,DEPTH_VIEWPOINTS,scale=3)
    ensure_not_packable(plain_dir,"DANGER_B06_FLAT_MUST_NOT_PACK.bin")
    flat=index_tiles(flat_manifest,plain_dir,(1800,1200))
    comparisons=[]
    part_by_name={name:(info,elapsed) for name,info,elapsed in parts}
    visible_by_layer={label:0 for label,_ in part_by_name.items()}
    for xy in DEPTH_VIEWPOINTS:
        layers=[]
        for label in ("lower","upper"):
            entry=part_by_name[label][0].get(xy)
            if entry:
                visible_by_layer[label]+=1
                with Image.open(layer_dir/label/entry["file"]) as im:
                    layers.append(im.convert("RGBA"))
            else:
                layers.append(Image.new("RGBA",(1800,1200),(0,0,0,0)))
        with Image.open(plain_dir/flat[xy]["file"]) as opened:
            expected=opened.convert("RGBA")
        # The existing source renderer overwrites direct ImageDraw shape pixels
        # (even semi-transparent RGBA) rather than alpha-compositing them.
        # CI artifact #13876348846 showed 22,382 false over-blend differences
        # at -4200; binary nontransparent replacement recovers ZERO difference.
        # This tests deterministic *source rasterizer* parity, not GPU
        # alpha behavior during actual original Flash gameplay.
        combined=layers[0].copy()
        binary_mask=layers[1].getchannel("A").point(
            lambda alpha: 255 if alpha else 0
        )
        combined.paste(layers[1],(0,0),binary_mask)
        diff=ImageChops.difference(expected,combined)
        bbox=diff.getbbox()
        raw=diff.tobytes() if bbox else b""
        pixels=sum(any(raw[i:i+4]) for i in range(0,len(raw),4))
        fraction=pixels/(1800*1200)
        if pixels != 0:
            raise AssertionError(f"Original source rasterizer depth bands did not reproduce flat source pixels at {xy}: {pixels} mismatches")
        comparisons.append({"world_origin":list(xy),"different_pixels":pixels,
                            "different_fraction":fraction,
                            "rgba_recomposition_exact":pixels==0,
                            "difference_bbox":list(bbox) if bbox else None})
    if any(n==0 for n in visible_by_layer.values()):
        raise AssertionError(f"No visible original authored depth geometry for one band: {visible_by_layer}")
    print("B06_SOURCE_AUTHORED_DEPTH_RECOMPOSITION_PASS",comparisons,flush=True)

    sheet=contact_sheet(args.out,ref_dir,hq_dir,samples)
    report={
        "batch":"LEVEL2-B06", "canonical_fws_sha256":sha,
        "source_root_zero_based":20, "source_sprite_frame_zero_based":0,
        "game_symbol_id":1075, "static_level_symbol_id":783,
        "collision_ground_source_id_excluded":782,
        "original_source_level_origin":[1034.65,239.25],
        "raw_define_button2_exclusions":[758,771],
        "raw_define_text_original_glyph_source_id":774,
        "source_depth_ownership":ownership,
        "original_frame0_authored_depth_partition":split,
        "source_level_local_bounds":bounds,
        "source_geometry_derived_candidate_x_columns":len(x_columns),
        "b05_1x_pixel_hashes_identical_all_nine":True,
        "one_x_sha256_inputs":[{"world_x":xy[0],"world_y":xy[1],**samples[xy]} for xy in VIEWS],
        "hq_true_original_source_rerasterized_not_bilinear_count":independent_hq,
        "hq_source_render_scale":6,
        "hq_output_scale":3,
        "hq_filter":"premultiplied Lanczos with 2px source overscan",
        "hq_3x_source_tiles_count":len(hq),
        "measured_offline_source_bake_seconds":{"one_x_baseline":round(t1,3),"hq_6x_to_3x":round(t3,3),
             "depth_flat_plain3x":round(elapsed_flat,3),
             **{label:round(seconds,3) for label,(_,seconds) in part_by_name.items()}},
        "depth_recomposition_semantics":"source rasterizer direct-shape nonzero-alpha pixel replacement; not proven Flash live GPU blending",
        "depth_recomposition_comparison":comparisons,
        "depth_layer_visible_view_counts":visible_by_layer,
        "source_hq3x_png_and_rgba_hashes":{str(xy):{"png":hq[xy]["png_sha256"],"rgba":hq[xy]["rgba_sha256"]} for xy in VIEWS},
        "contact_sheet":sheet,
        "partial_manifests_and_negative_pack_guard":True,
        "source_library_art_not_live_flash_gameplay":True,
        "no_level1b_native_gameplay_or_assets":True,
        "level1a_1x_hq_goldens_unmodified":True,
    }
    path=args.out/"B06_SOURCE_BANK_STREET_HQ3X_REPORT.json"
    path.write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
    print("B06_BANK_STREET_HQ3X_GATE_PASS",sha,
          "original 1x",len(reference),"true HQ",len(hq),
          "direct source layer comparisons",len(comparisons),
          "offline hq seconds",round(t3,3),flush=True)

if __name__=="__main__":
    main()
