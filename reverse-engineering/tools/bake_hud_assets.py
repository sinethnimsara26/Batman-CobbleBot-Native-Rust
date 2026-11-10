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

from PIL import Image, ImageChops, ImageDraw, ImageFont

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


def draw_field(image,info,transform,anchor,font_path,style="source",hq_layout=None):
    box=field_box(info,transform,anchor)
    sx=math.hypot(transform[0],transform[2])
    sy=math.hypot(transform[1],transform[3])
    scale=(sx+sy)/2.0
    size=max(1,round(info.get("font_height",12.0)*scale))
    font=ImageFont.truetype(str(font_path),size=size)
    text=info.get("initial_text","").replace("\r","").replace("\n","")
    draw=ImageDraw.Draw(image)
    color=info.get("color",(255,255,255,255))
    centered=info.get("align")==2
    xy=((box[0]+box[2])/2.0,(box[1]+box[3])/2.0) if centered else (box[0],box[1])
    anchor_type="mm" if centered else "lt"
    if style=="themed":
        # Player-requested design candidate: follow the tutorial's condensed,
        # heavy yellow action lettering (Impact is the closest installed face).
        # Do not claim Impact is the original Flash embedded tutorial font.
        # The white _sans legacy rendering above remains the source oracle.
        text=text.upper()
        # Respect original SWF field rectangles while gaining visual weight.
        # Especially important near the health icon / lives digit.
        max_width=max(1.0,box[2]-box[0]-2.0*scale)
        max_height=max(1.0,box[3]-box[1]-1.0*scale)
        # Single HQ layout manifest also generates the native Rust counters.
        dx,dy=hq_layout["static_offsets_logical"][str(info["id"])]
        xy=(xy[0]+dx*scale,xy[1]+dy*scale)
        if info["id"]==715:
            # Draw the caption alone. Measure its previous full "LIVES X"
            # length so this label does not grow or move when X is promoted.
            text="LIVES"
            fit_text="LIVES X"
        else:
            fit_text=text
        size=max(1,round(info.get("font_height",12.0)*scale*
                         (0.73 if info["id"]==715 else 0.94)))
        while size>1:
            font=ImageFont.truetype(str(font_path),size=size)
            tb=draw.textbbox((0,0),fit_text,font=font,anchor=anchor_type)
            if tb[2]-tb[0]<=max_width and tb[3]-tb[1]<=max_height:
                break
            size-=1
        color=(255,235,80,255) if info["id"]!=715 else (255,248,205,255)
        outline=max(2,round(0.90*scale))
        # HUD-REAL-05B: slightly wider near-black edge and darker offset
        # shadow improve contrast on bright scenery without changing the
        # stage geometry, font face, or legacy source-rendered captions.
        shadow=(xy[0]+0.62*scale,xy[1]+0.62*scale)
        draw.text(shadow,text,font=font,fill=(5,5,8,255),anchor=anchor_type,
                  stroke_width=outline+max(1,round(0.24*scale)),
                  stroke_fill=(5,5,8,255))
        draw.text(xy,text,font=font,fill=color,anchor=anchor_type,
                  stroke_width=outline,stroke_fill=(2,2,5,255))
    else:
        draw.text(xy,text,font=font,fill=color,anchor=anchor_type)
    return box

def resize_premultiplied(image:Image.Image,size:tuple[int,int],method:str):
    if image.size==size:
        return image.copy()
    if method!="lanczos":
        raise ValueError("HQ HUD downsample currently supports only lanczos")
    # Pillow's RGBa mode keeps RGB premultiplied while filtering transparent
    # edges, preventing dark/bright fringes around text and vector artwork.
    return image.convert("RGBa").resize(size,Image.Resampling.LANCZOS).convert("RGBA")


def digit_atlas(font_path:Path,base_size:int,pixel_scale:float,style="source",vertical_percent:int=100,extra_glyph=None):
    # Legacy atlases remain unchanged. The HQ atlas uses the exact same
    # condensed typeface as the HQ labels, with distinct number sizes.
    size=max(1,round(base_size*pixel_scale*
                     ((0.80 if base_size>16 else 0.76) if style=="themed" else 1.0)))
    font=ImageFont.truetype(str(font_path),size=size)
    probe=Image.new("RGBA",(8,8),(0,0,0,0))
    draw=ImageDraw.Draw(probe)
    bounds=[draw.textbbox((0,0),str(i),font=font,anchor="lt") for i in range(10)]
    cell_w=max(b[2]-b[0] for b in bounds)+max(1,round(6*pixel_scale))
    cell_h=max(b[3]-b[1] for b in bounds)+max(1,round(8*pixel_scale))
    if style=="themed":
        # The revised, smaller Impact glyphs must retain the source-sized
        # presentation CELLS; shrinking the whole 3x atlas breaks the HUD
        # asset contract and changes every number's placement/spacing.
        # Legacy source atlases have 15x20 (small) and 24x31 (large)
        # cells. Keep at least 92% of those *at the requested raster scale*
        # while drawing smaller centered glyphs inside their empty padding.
        # This preserves >2.5x legacy physical atlas dimensions after the
        # 6x -> 3x Lanczos step, without enlarging the actual type.
        source_cell=(15,20) if base_size==16 else (24,31)
        cell_w=max(cell_w,math.ceil(source_cell[0]*pixel_scale*0.92))
        cell_h=max(cell_h,math.ceil(source_cell[1]*pixel_scale*0.92))
    glyphs=[str(i) for i in range(10)] + ([extra_glyph] if extra_glyph else [])
    atlas=Image.new("RGBA",(cell_w*len(glyphs),cell_h),(0,0,0,0))
    draw=ImageDraw.Draw(atlas)
    for i,glyph in enumerate(glyphs):
        xy=(i*cell_w+cell_w/2.0,cell_h/2.0)
        if style=="themed":
            color=(255,241,153,255) if base_size>16 else (255,250,226,255)
            outline=max(2,round(0.89*pixel_scale))
            draw.text((xy[0]+0.60*pixel_scale,xy[1]+0.60*pixel_scale),
                      glyph,font=font,fill=(5,5,8,255),anchor="mm",
                      stroke_width=outline+1,stroke_fill=(5,5,8,255))
            draw.text(xy,glyph,font=font,fill=color,anchor="mm",
                      stroke_width=outline,stroke_fill=(2,2,5,255))
        else:
            draw.text(xy,glyph,font=font,fill=(255,255,255,255),anchor="mm")
    if style=="themed" and base_size==16 and vertical_percent>100:
        # Enlarge ONLY the visible small-digit ink in the offline 6x bake.
        # Keep atlas cell dimensions and the revised HQ palette intact.
        # The native game still blits already-baked 3x pixels at exact 1:1.
        stretched_atlas=Image.new("RGBA",atlas.size,(0,0,0,0))
        margin=max(1,round(0.5*pixel_scale))
        for i in range(len(glyphs)):
            cell=atlas.crop((i*cell_w,0,(i+1)*cell_w,cell_h))
            bounds=cell.getbbox()
            if bounds is None:
                raise ValueError(f"Small HQ digit {i} has no visible ink")
            glyph=cell.crop(bounds)
            new_height=min(cell_h-2*margin,round(glyph.height*vertical_percent/100))
            if new_height<=glyph.height:
                raise ValueError(f"Small HQ digit {i} lacks headroom to stretch")
            taller=resize_premultiplied(
                glyph,(glyph.width,new_height),"lanczos"
            )
            stretched_atlas.alpha_composite(
                taller,(i*cell_w+bounds[0],(cell_h-new_height)//2)
            )
        atlas=stretched_atlas
    meta={"cell_w":cell_w,"cell_h":cell_h,"font_px":size}
    if style=="themed" and base_size==16:
        meta["vertical_percent"]=vertical_percent
    return atlas,meta

def scale_box(box,factor):
    return [value/factor for value in box]


def read_hq_layout(path:Path):
    cfg=json.loads(path.read_text(encoding="utf-8"))
    if cfg.get("schema")!=1 or cfg.get("hud_sprite_id")!=716 or cfg.get("logical_stage")!=[600,400]:
        raise ValueError("Invalid HQ HUD schema, Flash symbol or stage")
    fields=cfg.get("static_offsets_logical",{})
    if set(fields)!={"700","710","711","715"}:
        raise ValueError("HQ label fields mismatch original sprite")
    for cid,pair in fields.items():
        if not isinstance(pair,list) or len(pair)!=2 or any(
            type(v) is not int or abs(v)>20 for v in pair
        ):
            raise ValueError(f"Invalid HQ label offset for {cid}: {pair}")
    ctr=cfg.get("counters",{})
    if set(ctr)!={"batarangs","score","lives"}:
        raise ValueError("HQ counter roles mismatch original HUD")
    if [ctr[k]["field_id"] for k in ("batarangs","score","lives")]!=[702,713,714]:
        raise ValueError("HQ counters no longer match source fields")
    for role in ("batarangs","score"):
        c=ctr[role]
        x,y=c["center"]
        if not (type(x) is int and type(y) is int and 0<x<600 and 0<y<400 and
                type(c["max_width"]) is int and 20<=c["max_width"]<=150):
            raise ValueError(f"Invalid HQ {role} position/width")
    if not (type(ctr["batarangs"].get("digit_advance")) is int and
            6<=ctr["batarangs"]["digit_advance"]<=13):
        raise ValueError("Batarang digit pitch must fit visible ink and cell")
    if not (type(ctr["lives"].get("digit_advance")) is int and
            6<=ctr["lives"]["digit_advance"]<=13):
        raise ValueError("Lives digit pitch must fit visible ink and cell")
    large_advance=ctr["score"].get("large_digit_advance")
    if type(large_advance) is not int or not 14<=large_advance<=22:
        raise ValueError("HQ large SCORE digit advance must be 14..22 logical pixels")
    percent=cfg.get("small_counter_height_percent")
    if type(percent) is not int or not 100<=percent<=125:
        raise ValueError("Small digit vertical stretch must be 100..125 percent")
    bat_height=cfg.get("batarang_ink_height_percent")
    if type(bat_height) is not int or not percent<bat_height<=125:
        raise ValueError("Dedicated Batarang digit ink must be taller but fit in the same small cell")
    l=ctr["lives"]
    if not (type(l["left"]) is int and type(l["center_y"]) is int and
            0<l["left"]<600 and 0<l["center_y"]<400 and
            type(l["max_width"]) is int and 20<=l["max_width"]<=150 and
            l.get("multiplier_center")==[l["left"]-7,l["center_y"]]):
        raise ValueError("Invalid HQ lives anchor or width")
    return cfg


def generate_rust_hq_layout(path:Path,cfg:dict):
    b=cfg["counters"]["batarangs"]
    s=cfg["counters"]["score"]
    l=cfg["counters"]["lives"]
    lines=[
        "// GENERATED by bake_hud_assets.py from hud-hq-layout.json; do not edit.",
        "// Original 600x400 logical stage, compiled into native Rust HUD.",
        f"pub const BATARANG_CENTER:(i32,i32)=({b['center'][0]},{b['center'][1]});",
        f"pub const BATARANG_MAX_WIDTH:i32={b['max_width']};",
        f"pub const BATARANG_DIGIT_ADVANCE:i32={b['digit_advance']};",
        f"pub const SCORE_CENTER:(i32,i32)=({s['center'][0]},{s['center'][1]});",
        f"pub const SCORE_MAX_WIDTH:i32={s['max_width']};",
        f"pub const SCORE_LARGE_DIGIT_ADVANCE:i32={s['large_digit_advance']};",
        f"pub const LIVES_VALUE_LEFT:i32={l['left']};",
        f"pub const LIVES_CENTER_Y:i32={l['center_y']};",
        f"pub const LIVES_MAX_WIDTH:i32={l['max_width']};",
        f"pub const LIVES_DIGIT_ADVANCE:i32={l['digit_advance']};",
    ]
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text("\n".join(lines)+"\n",encoding="utf-8")


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
    ap.add_argument("--style",choices=("source","themed"),default="source",help="source device-font oracle or alternate Batman-style HQ labels")
    ap.add_argument("--hq-layout",type=Path,help="themed label and Rust counter stage coordinates")
    ap.add_argument("--emit-rust-layout",type=Path,help="generated native Rust HUD constants")
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

    if ns.style=="themed":
        if ns.hq_layout is None or ns.emit_rust_layout is None:
            raise ValueError("Themed HQ HUD needs --hq-layout and --emit-rust-layout")
        hq_layout=read_hq_layout(ns.hq_layout)
        generate_rust_hq_layout(ns.emit_rust_layout,hq_layout)
    else:
        if ns.hq_layout is not None or ns.emit_rust_layout is not None:
            raise ValueError("Source-style legacy HUD cannot use custom HQ coordinates")
        hq_layout=None

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
    # Independent mask of just the baked static captions, no HUD vector art.
    static_ink=Image.new("RGBA",image.size,(0,0,0,0)) if ns.style=="themed" else None
    x_shell_contact_rows=0
    temp_fields={}

    for cid in sorted(HUD_IDS):
        info=edit[cid]
        if cid in STATIC_IDS:
            temp_fields[str(cid)]=draw_field(image,info,found[cid],temp_anchor,ns.font,ns.style,hq_layout)
            if static_ink is not None:
                draw_field(static_ink,info,found[cid],temp_anchor,ns.font,ns.style,hq_layout)
        else:
            temp_fields[str(cid)]=field_box(info,found[cid],temp_anchor)

    temp_scale=ns.output_scale*ns.supersample
    small_image,small_meta=digit_atlas(
        ns.font,16,temp_scale,ns.style,
        hq_layout["small_counter_height_percent"] if hq_layout is not None else 100,
        extra_glyph="X" if ns.style=="themed" else None
    )
    # Build X from exactly the same rasterized numeric atlas, using the
    # existing 115%-stretched Impact glyph appearance. The ten number cells
    # are cropped back before output, leaving the numeric runtime unmodified.
    if ns.style=="themed":
        cell_w=small_meta["cell_w"]
        assert small_image.width==11*cell_w
        x_glyph=small_image.crop((10*cell_w,0,11*cell_w,small_image.height))
        small_image=small_image.crop((0,0,10*cell_w,small_image.height))
        mx,my=hq_layout["counters"]["lives"]["multiplier_center"]
        # HUD sprite 716 is placed at source root stage (159,-11).
        x=round((mx-159)*temp_scale+temp_anchor[0]-x_glyph.width/2)
        y=round((my+11)*temp_scale+temp_anchor[1]-x_glyph.height/2)
        # An atlas CELL contains intentional transparent padding. Its bottom
        # can extend a few 6x pixels past sprite 716's canvas even when every
        # actual anti-aliased X pixel fits. Crop ONLY transparent margins;
        # never clip visible ink or enlarge/reposition the source HUD shell.
        ink_box=x_glyph.getchannel("A").getbbox()
        if ink_box is None:
            raise ValueError("Lives X glyph unexpectedly has no visible pixels")
        x+=ink_box[0]
        y+=ink_box[1]
        x_glyph=x_glyph.crop(ink_box)
        if x<0 or y<0 or x+x_glyph.width>image.width or y+x_glyph.height>image.height:
            raise ValueError("Visible Lives X ink is outside the original HUD shell canvas")
        old_alpha=image.crop((x,y,x+x_glyph.width,y+x_glyph.height)).getchannel("A")
        old_opaque=old_alpha.point(lambda v:255 if v>=208 else 0)
        x_ink=x_glyph.getchannel("A").point(lambda v:255 if v>=32 else 0)
        caption_alpha=static_ink.crop((x,y,x+x_glyph.width,y+x_glyph.height)).getchannel("A")
        caption_ink=caption_alpha.point(lambda v:255 if v>=32 else 0)
        caption_collision=ImageChops.multiply(caption_ink,x_ink)
        if caption_collision.getbbox() is not None:
            raise ValueError(f"Lives X overlaps independently rasterized static caption ink: {caption_collision.getbbox()}")
        # Permit at most ONE 6x source-art edge row of contact; wider
        # opaque-shell or any static-caption overlap remains a hard failure.
        source_collision=ImageChops.multiply(old_opaque,x_ink).getbbox()
        x_shell_contact_rows=0 if source_collision is None else source_collision[3]-source_collision[1]
        if x_shell_contact_rows>1:
            raise ValueError(f"Lives X has multirow opaque-shell collision: {source_collision} glyph_origin=({x},{y})")
        print("HUD-LIVES-X:",{"static_caption_overlap_px":0,"source_opaque_contact_rows_6x":x_shell_contact_rows,"source_bbox":source_collision})
        image.alpha_composite(x_glyph,(x,y))
    large_image,large_meta=digit_atlas(ns.font,32,temp_scale,ns.style)
    # HUD-REAL-06B: separate, taller Batarang-only HQ atlas. Health/Lives
    # and any narrow-score fallback keep the same original 115% small atlas.
    batarang_image=batarang_meta=None
    if ns.style=="themed":
        batarang_image,batarang_meta=digit_atlas(
            ns.font,16,temp_scale,ns.style,
            hq_layout["batarang_ink_height_percent"]
        )
        if batarang_meta["cell_w"]!=small_meta["cell_w"] or batarang_meta["cell_h"]!=small_meta["cell_h"]:
            raise ValueError("Dedicated Batarang atlas must retain small digit geometry")

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
                **({"vertical_percent":meta["vertical_percent"]}
                   if "vertical_percent" in meta else {}),
            }

        small_image,small_meta=finish_atlas(small_image,small_meta)
        large_image,large_meta=finish_atlas(large_image,large_meta)
        if batarang_image is not None:
            batarang_image,batarang_meta=finish_atlas(batarang_image,batarang_meta)

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
    if batarang_image is not None:
        batarang_image.save(ns.outdir/f"hud_digits_batarang{ns.suffix}.png")

    meta={
        "hud_size":list(image.size),
        "style":ns.style,
        "font_filename":ns.font.name,
        "hq_layout_schema":hq_layout["schema"] if hq_layout is not None else None,
        "anchor_in_bitmap":final_anchor,
        "fields":fields,
        "small_digits":small_meta,
        "large_digits":large_meta,
        "batarang_digits":batarang_meta,
        "hq_lives_multiplier":({"text":"X","logical_center":hq_layout["counters"]["lives"]["multiplier_center"],"atlas":"small","static_caption_ink_overlap_px":0,"source_opaque_contact_rows_6x":x_shell_contact_rows} if hq_layout is not None else None),
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
