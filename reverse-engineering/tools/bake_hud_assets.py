#!/usr/bin/env python3
"""Bake the original CobbleBot HUD shell plus device-font digit atlases.

The HUD is character 716. Static vector art is rendered by bake_sprite_frames.py.
Static DefineEditText labels (_sans) are recreated with Windows Arial, which is
what Flash's _sans device-font alias resolves to on the original target platform.
Dynamic fields remain Rust-owned at runtime.
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


def digit_atlas(font_path:Path,size:int,path:Path):
    font=ImageFont.truetype(str(font_path),size=size)
    probe=Image.new("RGBA",(8,8),(0,0,0,0))
    draw=ImageDraw.Draw(probe)
    bounds=[draw.textbbox((0,0),str(i),font=font,anchor="lt") for i in range(10)]
    cell_w=max(b[2]-b[0] for b in bounds)+6
    cell_h=max(b[3]-b[1] for b in bounds)+8
    atlas=Image.new("RGBA",(cell_w*10,cell_h),(0,0,0,0))
    draw=ImageDraw.Draw(atlas)
    for i in range(10):
        draw.text((i*cell_w+cell_w/2.0,cell_h/2.0),str(i),font=font,fill=(255,255,255,255),anchor="mm")
    atlas.save(path)
    return {"cell_w":cell_w,"cell_h":cell_h,"font_px":size}


def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("swf",type=Path)
    ap.add_argument("symbols_json",type=Path)
    ap.add_argument("hud_frames",type=Path,help="symbol 716 --all-frames output")
    ap.add_argument("outdir",type=Path)
    ap.add_argument("--font",type=Path,required=True,help="Arial-compatible TTF for Flash _sans")
    ns=ap.parse_args()

    symbols=json.loads(ns.symbols_json.read_text(encoding="utf-8"))
    edit=parse_edit_texts(ns.swf)
    missing=HUD_IDS-set(edit)
    if missing:
        raise ValueError(f"HUD DefineEditText IDs missing: {sorted(missing)}")

    found={}
    find_edit_placements(716,0,(1.0,0.0,0.0,1.0,0.0,0.0),symbols,found)
    if set(found)!=HUD_IDS:
        raise ValueError(f"HUD edit placements mismatch: {sorted(found)}")

    manifest=json.loads((ns.hud_frames/"manifest.json").read_text(encoding="utf-8"))
    anchor=manifest["anchor_in_bitmap"]
    image=Image.open(ns.hud_frames/"000.png").convert("RGBA")
    fields={}

    for cid in sorted(HUD_IDS):
        info=edit[cid]
        if cid in STATIC_IDS:
            fields[str(cid)]=draw_field(image,info,found[cid],anchor,ns.font)
        else:
            fields[str(cid)]=field_box(info,found[cid],anchor)

    ns.outdir.mkdir(parents=True,exist_ok=True)
    image.save(ns.outdir/"hud_base.png")
    small=digit_atlas(ns.font,16,ns.outdir/"hud_digits_small.png")
    large=digit_atlas(ns.font,32,ns.outdir/"hud_digits_large.png")
    meta={
        "hud_size":list(image.size),
        "anchor_in_bitmap":anchor,
        "fields":fields,
        "small_digits":small,
        "large_digits":large,
        "variables":{str(cid):edit[cid].get("variable","") for cid in sorted(DYNAMIC_IDS)},
        "static_text":{str(cid):edit[cid].get("initial_text","").replace("\r","") for cid in sorted(STATIC_IDS)},
    }
    (ns.outdir/"hud.json").write_text(json.dumps(meta,indent=2),encoding="utf-8")
    print(json.dumps(meta,indent=2))


if __name__=="__main__":
    main()
