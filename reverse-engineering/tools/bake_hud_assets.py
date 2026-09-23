#!/usr/bin/env python3
"""Bake the original CobbleBot HUD shell plus device-font digit atlases.

The HUD is character 716. Static vector art is rendered by bake_sprite_frames.py.
Static DefineEditText labels (_sans) are recreated with Windows Arial, which is
what Flash's _sans device-font alias resolves to on the original target platform.
Dynamic fields remain Rust-owned at runtime.

Round 6 makes this tool scale-aware. HQ mode may receive a supersampled HUD
render (for example 6x), rasterize Arial at that same temporary scale, then
premultiplied-alpha Lanczos downsample the complete shell and digit atlases to
the final 3x presentation scale. The optional pack directory emits a one-frame
manifest so pack_sprite_frames.py can preserve the exact final HUD anchor.
"""
from __future__ import annotations

from pathlib import Path
import argparse
import json
import math
import struct

from PIL import Image, ImageDraw, ImageFont

from swf_re import Bits, read_rect, tags
from render_collision_masks import affine, compose, point
from bake_sprite_frames import display_list


STATIC_IDS={700,710,711,715}
DYNAMIC_IDS={702,713,714}
HUD_IDS=STATIC_IDS|DYNAMIC_IDS


def cstr(data:bytes,pos:int):
    end=data.index(0,pos)
    return data[pos:end].decode("utf-8","replace"),end+1


def parse_edit_texts(swf:Path):
    data=swf.read_bytes()
    _,pos=read_rect(data,8)
    pos+=4
    out={}
    names=[
        "has_text","word_wrap","multiline","password","readonly",
        "has_text_color","has_max_length","has_font","has_font_class",
        "autosize","has_layout","no_select","border","was_static","html","use_outlines",
    ]
    for code,payload,_,_ in tags(data,pos,len(data)):
        if code!=37:
            continue
        cid=struct.unpack_from("<H",payload,0)[0]
        bounds,p=read_rect(payload,2)
        bits=Bits(payload,p)
        flags={name:bool(bits.u(1)) for name in names}
        bits.align()
        p=bits.pos
        info={"id":cid,"bounds":[v/20.0 for v in bounds],**flags}
        if flags["has_font"]:
            info["font_id"]=struct.unpack_from("<H",payload,p)[0]
            p+=2
        if flags["has_font_class"]:
            info["font_class"],p=cstr(payload,p)
        if flags["has_font"]:
            info["font_height"]=struct.unpack_from("<H",payload,p)[0]/20.0
            p+=2
        if flags["has_text_color"]:
            info["color"]=tuple(payload[p:p+4])
            p+=4
        if flags["has_max_length"]:
            info["max_length"]=struct.unpack_from("<H",payload,p)[0]
            p+=2
        if flags["has_layout"]:
            info["align"]=payload[p]
            p+=1
            info["left_margin"]=struct.unpack_from("<H",payload,p)[0]/20.0; p+=2
            info["right_margin"]=struct.unpack_from("<H",payload,p)[0]/20.0; p+=2
            info["indent"]=struct.unpack_from("<H",payload,p)[0]/20.0; p+=2
            info["leading"]=struct.unpack_from("<h",payload,p)[0]/20.0; p+=2
        info["variable"],p=cstr(payload,p)
        if flags["has_text"]:
            info["initial_text"],p=cstr(payload,p)
        out[cid]=info
    return out


def find_edit_placements(symbol_id,frame,transform,symbols,found,depth=0):
    if depth>24:
        raise RuntimeError("HUD placement recursion exceeded")
    symbol=symbols.get(str(symbol_id))
    if symbol is None:
        return
    for placement in display_list(symbol,frame).values():
        cid=placement.get("character_id")
        if cid is None:
            continue
        child_transform=compose(transform,affine(placement.get("matrix",{})))
        if cid in HUD_IDS:
            found[cid]=child_transform
        elif str(cid) in symbols:
            child_frame=max(0,frame-placement.get("born",0))
            find_edit_placements(cid,child_frame,child_transform,symbols,found,depth+1)


def field_box(info,transform,anchor):
    x0,x1,y0,y1=info["bounds"]
    corners=[
        point(transform,(x0,y0)),point(transform,(x1,y0)),
        point(transform,(x0,y1)),point(transform,(x1,y1)),
    ]
    return [
        min(p[0] for p in corners)+anchor[0],
        min(p[1] for p in corners)+anchor[1],
        max(p[0] for p in corners)+anchor[0],
        max(p[1] for p in corners)+anchor[1],
    ]


def draw_field(image,info,transform,anchor,font_path):
    box=field_box(info,transform,anchor)
    sx=math.hypot(transform[0],transform[2])
    sy=math.hypot(transform[1],transform[3])
    scale=(sx+sy)/2.0
    size=max(1,round(info.get("font_height",12.0)*scale))
    font=ImageFont.truetype(str(font_path),size=size)
    text=info.get("initial_text","").replace("\r","").replace("\n","")
    draw=ImageDraw.Draw(image)
    color=info.get("color",(255,255,255,255))
    if info.get("align")==2:
        draw.text(((box[0]+box[2])/2.0,(box[1]+box[3])/2.0),text,font=font,fill=color,anchor="mm")
    else:
        draw.text((box[0],box[1]),text,font=font,fill=color,anchor="lt")
    return box


def resize_premultiplied(image:Image.Image,size:tuple[int,int],method:str):
    if image.size==size:
        return image.copy()
    if method!="lanczos":
        raise ValueError("HQ HUD downsample currently supports only lanczos")
    # Pillow's RGBa mode keeps RGB premultiplied while filtering transparent
    # edges, preventing dark/bright fringes around text and vector artwork.
    return image.convert("RGBa").resize(size,Image.Resampling.LANCZOS).convert("RGBA")


def digit_atlas(font_path:Path,base_size:int,pixel_scale:float):
    size=max(1,round(base_size*pixel_scale))
    font=ImageFont.truetype(str(font_path),size=size)
    probe=Image.new("RGBA",(8,8),(0,0,0,0))
    draw=ImageDraw.Draw(probe)
    bounds=[draw.textbbox((0,0),str(i),font=font,anchor="lt") for i in range(10)]
    cell_w=max(b[2]-b[0] for b in bounds)+max(1,round(6*pixel_scale))
    cell_h=max(b[3]-b[1] for b in bounds)+max(1,round(8*pixel_scale))
    atlas=Image.new("RGBA",(cell_w*10,cell_h),(0,0,0,0))
    draw=ImageDraw.Draw(atlas)
    for i in range(10):
        draw.text((i*cell_w+cell_w/2.0,cell_h/2.0),str(i),font=font,fill=(255,255,255,255),anchor="mm")
    return atlas,{"cell_w":cell_w,"cell_h":cell_h,"font_px":size}


def scale_box(box,factor):
    return [value/factor for value in box]


def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("swf",type=Path)
    ap.add_argument("symbols_json",type=Path)
    ap.add_argument("hud_frames",type=Path,help="symbol 716 --all-frames output")
    ap.add_argument("outdir",type=Path)
    ap.add_argument("--font",type=Path,required=True,help="Arial-compatible TTF for Flash _sans")
    ap.add_argument("--output-scale",type=float,default=1.0,
                    help="final presentation pixels per logical stage pixel")
    ap.add_argument("--supersample",type=int,default=1,
                    help="temporary raster scale multiplier before final downsample")
    ap.add_argument("--downsample",choices=("none","lanczos"),default="none")
    ap.add_argument("--suffix",default="",help="output suffix, e.g. _hq")
    ap.add_argument("--pack-dir",type=Path,
                    help="optional one-frame pack-ready directory for the final HUD shell")
    ns=ap.parse_args()

    if ns.output_scale<=0:
        raise ValueError("--output-scale must be positive")
    if ns.supersample<1:
        raise ValueError("--supersample must be >= 1")
    if ns.supersample>1 and ns.downsample=="none":
        raise ValueError("supersampled HUD output requires --downsample lanczos")
    if ns.supersample==1 and ns.downsample!="none":
        raise ValueError("--downsample is only meaningful with --supersample > 1")

    symbols=json.loads(ns.symbols_json.read_text(encoding="utf-8"))
    edit=parse_edit_texts(ns.swf)
    missing=HUD_IDS-set(edit)
    if missing:
        raise ValueError(f"HUD DefineEditText IDs missing: {sorted(missing)}")

    manifest=json.loads((ns.hud_frames/"manifest.json").read_text(encoding="utf-8"))
    input_scale=float(manifest["scale"][0])
    if abs(float(manifest["scale"][1])-input_scale)>1e-9:
        raise ValueError("HUD source must use uniform scale")
    expected_input_scale=ns.output_scale*ns.supersample
    if abs(input_scale-expected_input_scale)>1e-6:
        raise ValueError(
            f"HUD source scale mismatch: expected {expected_input_scale}, got {input_scale}"
        )

    # EditText placement coordinates must live in the same pixel space as the
    # supersampled/vector HUD shell.
    found={}
    root_transform=(input_scale,0.0,0.0,input_scale,0.0,0.0)
    find_edit_placements(716,0,root_transform,symbols,found)
    if set(found)!=HUD_IDS:
        raise ValueError(f"HUD edit placements mismatch: {sorted(found)}")

    temp_anchor=[float(v) for v in manifest["anchor_in_bitmap"]]
    image=Image.open(ns.hud_frames/"000.png").convert("RGBA")
    temp_fields={}

    for cid in sorted(HUD_IDS):
        info=edit[cid]
        if cid in STATIC_IDS:
            temp_fields[str(cid)]=draw_field(image,info,found[cid],temp_anchor,ns.font)
        else:
            temp_fields[str(cid)]=field_box(info,found[cid],temp_anchor)

    temp_scale=ns.output_scale*ns.supersample
    small_image,small_meta=digit_atlas(ns.font,16,temp_scale)
    large_image,large_meta=digit_atlas(ns.font,32,temp_scale)

    factor=float(ns.supersample)
    if ns.supersample>1:
        final_size=(
            max(1,round(image.width/factor)),
            max(1,round(image.height/factor)),
        )
        image=resize_premultiplied(image,final_size,ns.downsample)

        def finish_atlas(atlas,meta):
            final_cell_w=max(1,round(meta["cell_w"]/factor))
            final_cell_h=max(1,round(meta["cell_h"]/factor))
            final=resize_premultiplied(
                atlas,(final_cell_w*10,final_cell_h),ns.downsample
            )
            return final,{
                "cell_w":final_cell_w,
                "cell_h":final_cell_h,
                "font_px":round(meta["font_px"]/factor),
            }

        small_image,small_meta=finish_atlas(small_image,small_meta)
        large_image,large_meta=finish_atlas(large_image,large_meta)

    final_anchor=[value/factor for value in temp_anchor]
    fields={key:scale_box(box,factor) for key,box in temp_fields.items()}

    ns.outdir.mkdir(parents=True,exist_ok=True)
    base_name=f"hud_base{ns.suffix}.png"
    small_name=f"hud_digits_small{ns.suffix}.png"
    large_name=f"hud_digits_large{ns.suffix}.png"
    json_name=f"hud{ns.suffix}.json"
    image.save(ns.outdir/base_name)
    small_image.save(ns.outdir/small_name)
    large_image.save(ns.outdir/large_name)

    meta={
        "hud_size":list(image.size),
        "anchor_in_bitmap":final_anchor,
        "fields":fields,
        "small_digits":small_meta,
        "large_digits":large_meta,
        "variables":{str(cid):edit[cid].get("variable","") for cid in sorted(DYNAMIC_IDS)},
        "static_text":{str(cid):edit[cid].get("initial_text","").replace("\r","") for cid in sorted(STATIC_IDS)},
        "output_scale":ns.output_scale,
        "supersample":ns.supersample,
        "temporary_scale":temp_scale,
        "downsample":ns.downsample,
    }
    (ns.outdir/json_name).write_text(json.dumps(meta,indent=2),encoding="utf-8")

    if ns.pack_dir is not None:
        ns.pack_dir.mkdir(parents=True,exist_ok=True)
        image.save(ns.pack_dir/"000.png")
        pack_manifest={
            "symbol":716,
            "frame_count":1,
            "size":list(image.size),
            "scale":[ns.output_scale,ns.output_scale],
            "source_bounds":manifest.get("source_bounds"),
            "anchor_in_bitmap":final_anchor,
            "unsupported_fills":manifest.get("unsupported_fills",[]),
            "missing_bitmap_ids":manifest.get("missing_bitmap_ids",[]),
            "frames":[{"frame":0,"animation":"hud","file":"000.png"}],
        }
        (ns.pack_dir/"manifest.json").write_text(
            json.dumps(pack_manifest,indent=2),encoding="utf-8"
        )

    print(json.dumps(meta,indent=2))


if __name__=="__main__":
    main()
