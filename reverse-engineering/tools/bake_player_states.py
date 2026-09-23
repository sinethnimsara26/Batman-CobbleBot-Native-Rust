#!/usr/bin/env python3
"""Bake Batman's actual child animation timelines into a native frame source.

The outer Batman sprite (691) selects states. Each state places a child movie
clip whose true frame count differs from the outer label span. Baking the outer
271 frames truncates several animations. This tool resolves each label to its
depth-1 child, renders every real child frame through the exact parent placement
matrix, and keeps one common registration anchor at the outer Batman origin.
"""
from __future__ import annotations

from pathlib import Path
import argparse, json, math
from PIL import Image

from bake_sprite_frames import load_shapes, display_list, symbol_bounds, render_symbol
from render_collision_masks import affine, compose


def load_bitmaps(bitmap_dir: Path):
    manifest=json.loads((bitmap_dir/"manifest.json").read_text(encoding="utf-8"))
    out={}
    for item in manifest:
        if "file" not in item:
            continue
        with Image.open(bitmap_dir/item["file"]) as src:
            out[item["id"]]=src.convert("RGBA")
    return out


def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("swf",type=Path)
    ap.add_argument("symbols_json",type=Path)
    ap.add_argument("output",type=Path)
    ap.add_argument("--bitmaps",type=Path,required=True)
    ap.add_argument("--symbol",type=int,default=691)
    ap.add_argument("--scale",type=float,default=0.163742,help="recovered logical player placement scale")
    ap.add_argument("--output-scale",type=int,default=1,help="final raster pixels per legacy output pixel")
    ap.add_argument("--supersample",type=int,default=1,help="temporary raster multiplier before final downsampling")
    ap.add_argument("--downsample",choices=("lanczos","bicubic","bilinear"),default="lanczos")
    args=ap.parse_args()
    if args.output_scale < 1:
        raise SystemExit("--output-scale must be >= 1")
    if args.supersample < 1:
        raise SystemExit("--supersample must be >= 1")

    symbols=json.loads(args.symbols_json.read_text(encoding="utf-8"))
    outer=symbols.get(str(args.symbol))
    if outer is None:
        raise SystemExit(f"missing outer player symbol {args.symbol}")
    labels=sorted(outer.get("labels",[]),key=lambda x:x["frame"])
    if not labels:
        raise SystemExit("player symbol has no labels")

    unsupported=set()
    shapes=load_shapes(args.swf,unsupported)
    bitmaps=load_bitmaps(args.bitmaps)

    states=[]
    all_bounds=[]
    for label in labels:
        parent_frame=label["frame"]
        placement=display_list(outer,parent_frame).get(1)
        if not placement or "character_id" not in placement:
            raise SystemExit(f"label {label['label']} frame {parent_frame}: no depth-1 child")
        child=placement["character_id"]
        child_symbol=symbols.get(str(child))
        if child_symbol is None:
            raise SystemExit(f"label {label['label']}: child {child} is not a sprite")
        count=int(child_symbol.get("frames",0))
        if count<=0:
            raise SystemExit(f"label {label['label']}: child {child} has no frames")
        matrix=affine(placement.get("matrix",{}))
        states.append({
            "name":label["label"],"parent_frame":parent_frame,
            "child_symbol":child,"frame_count":count,"matrix":matrix,
        })
        for frame in range(count):
            bounds=symbol_bounds(child,frame,matrix,symbols,shapes)
            if all(math.isfinite(v) for v in bounds):
                all_bounds.append(bounds)

    if not all_bounds:
        raise SystemExit("no renderable Batman state bounds")

    min_x=min(b[0] for b in all_bounds); min_y=min(b[1] for b in all_bounds)
    max_x=max(b[2] for b in all_bounds); max_y=max(b[3] for b in all_bounds)
    # Keep legacy registration mathematically identical at output-scale 1,
    # while making HQ output explicit instead of hiding a multiplier in CI.
    final_scale=args.scale*args.output_scale
    final_pad=2*args.output_scale
    final_tx=final_pad-min_x*final_scale; final_ty=final_pad-min_y*final_scale
    final_width=math.ceil((max_x-min_x)*final_scale)+final_pad*2
    final_height=math.ceil((max_y-min_y)*final_scale)+final_pad*2

    raster_scale=final_scale*args.supersample
    raster_tx=final_tx*args.supersample; raster_ty=final_ty*args.supersample
    raster_width=final_width*args.supersample
    raster_height=final_height*args.supersample
    root=(raster_scale,0.0,0.0,raster_scale,raster_tx,raster_ty)

    resampling={
        "lanczos":Image.Resampling.LANCZOS,
        "bicubic":Image.Resampling.BICUBIC,
        "bilinear":Image.Resampling.BILINEAR,
    }[args.downsample]

    def finish_frame(canvas):
        if args.supersample==1:
            return canvas
        # Pillow's RGBa mode is premultiplied-alpha RGBA. Filtering in this
        # mode prevents transparent RGB from creating dark/colored edge halos.
        return canvas.convert("RGBa").resize(
            (final_width,final_height),resampling
        ).convert("RGBA")

    args.output.mkdir(parents=True,exist_ok=True)
    frames=[]; ranges=[]; cursor=0; missing=set()
    for state in states:
        start=cursor
        transform=compose(root,state["matrix"])
        for local in range(state["frame_count"]):
            canvas=Image.new("RGBA",(raster_width,raster_height),(0,0,0,0))
            render_symbol(
                state["child_symbol"],local,transform,canvas,
                symbols,shapes,bitmaps,unsupported,missing
            )
            canvas=finish_frame(canvas)
            filename=f"{cursor:03}.png"
            canvas.save(args.output/filename)
            frames.append({
                "frame":cursor,"animation":state["name"],
                "local_frame":local,"child_symbol":state["child_symbol"],
                "file":filename,
            })
            cursor+=1
        ranges.append({
            "name":state["name"],"start":start,"end":cursor,
            "frame_count":state["frame_count"],
            "child_symbol":state["child_symbol"],
            "parent_frame":state["parent_frame"],
        })

    manifest={
        "format":"batman-true-child-timelines-v1",
        "outer_symbol":args.symbol,
        "frame_count":cursor,
        "size":[final_width,final_height],
        "scale":[final_scale,final_scale],
        "base_scale":args.scale,
        "output_scale":args.output_scale,
        "supersample":args.supersample,
        "downsample":args.downsample if args.supersample>1 else "none",
        "raster_scale":[raster_scale,raster_scale],
        "source_bounds":[min_x,min_y,max_x,max_y],
        "anchor_in_bitmap":[final_tx,final_ty],
        "states":ranges,
        "unsupported_fills":sorted(unsupported),
        "missing_bitmap_ids":sorted(missing),
        "frames":frames,
    }
    (args.output/"manifest.json").write_text(
        json.dumps(manifest,indent=2),encoding="utf-8"
    )
    print(json.dumps({
        "outer_symbol":args.symbol,
        "frame_count":cursor,
        "size":[final_width,final_height],
        "anchor_in_bitmap":[final_tx,final_ty],
        "base_scale":args.scale,
        "output_scale":args.output_scale,
        "supersample":args.supersample,
        "downsample":args.downsample if args.supersample>1 else "none",
        "states":ranges,
        "unsupported_fills":sorted(unsupported),
        "missing_bitmap_ids":sorted(missing),
    },indent=2))


if __name__=="__main__":
    main()
